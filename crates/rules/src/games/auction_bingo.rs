//! Auction Bingo (main game, 3–8 seats). Rules page: docs/games/auction-bingo.md
//!
//! Everyone has a secret 3×3 board of circles, triangles and stars. Each round
//! you pick a cell (its shape), see how many went for each shape, then bid
//! coins; the single highest bidder per shape pays and marks the cell.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 7;
pub const COINS: i64 = 20;
pub const SHAPES: [&str; 3] = ["circle", "triangle", "star"];

/// Rows, columns and diagonals of a 3×3 board.
pub const LINES: [[usize; 3]; 8] = [[0, 1, 2], [3, 4, 5], [6, 7, 8], [0, 3, 6], [1, 4, 7], [2, 5, 8], [0, 4, 8], [2, 4, 6]];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    /// Shape index per cell (0 circle, 1 triangle, 2 star).
    pub shapes: Vec<usize>,
    pub marked: Vec<bool>,
}

impl Board {
    pub fn lines(&self) -> usize {
        LINES.iter().filter(|l| l.iter().all(|&c| self.marked[c])).count()
    }
}

pub fn board_score(b: &Board, coins: i64) -> i64 {
    5 * b.lines() as i64 + b.marked.iter().filter(|&&m| m).count() as i64 + coins / 5
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Pick,
    Bid,
}

#[derive(Clone, Debug)]
pub struct BingoState {
    pub players: Vec<String>,
    pub boards: Vec<Board>,
    pub coins: Vec<i64>,
    pub round: usize,
    pub phase: Phase,
    pub picks: Vec<Option<usize>>,
    pub bids: Vec<Option<i64>>,
    pub logs: Vec<Vec<String>>,
    pub over: bool,
    /// Replay moments: a won cell that completes a line (only the winner sees it).
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BingoMove {
    Pick { cell: usize },
    Bid { coins: i64 },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Final {
    pub boards: Vec<Board>,
    pub coins: Vec<i64>,
    pub scores: Vec<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BingoView {
    pub players: Vec<String>,
    pub me: usize,
    pub round: usize,
    pub rounds: usize,
    /// "pick" | "bid" | "over"
    pub phase: &'static str,
    pub board: Board,
    pub coins: i64,
    pub my_pick: Option<usize>,
    pub my_bid: Option<i64>,
    /// Pickers per shape, known in the bid step.
    pub counts: Option<[usize; 3]>,
    /// Who has already moved in this step.
    pub done: Vec<bool>,
    pub score: i64,
    pub log: Vec<String>,
    pub result: Option<Final>,
}

impl BingoState {
    fn counts(&self) -> [usize; 3] {
        let mut c = [0; 3];
        for (seat, p) in self.picks.iter().enumerate() {
            if let Some(cell) = p {
                c[self.boards[seat].shapes[*cell]] += 1;
            }
        }
        c
    }

    fn after_move(&mut self) {
        match self.phase {
            Phase::Pick if self.picks.iter().all(Option::is_some) => self.phase = Phase::Bid,
            Phase::Bid if self.bids.iter().all(Option::is_some) => self.resolve(),
            _ => {}
        }
    }

    fn resolve(&mut self) {
        let n = self.players.len();
        let mut public = vec![format!("Round {}:", self.round + 1)];
        for (shape, name) in SHAPES.iter().enumerate() {
            let seats: Vec<usize> = (0..n).filter(|&s| self.picks[s].is_some_and(|c| self.boards[s].shapes[c] == shape)).collect();
            let bid = |s: usize| self.bids[s].unwrap_or(0);
            let Some(top) = seats.iter().map(|&s| bid(s)).max() else {
                public.push(format!("{name}: nobody picked it."));
                continue;
            };
            let best: Vec<usize> = seats.iter().copied().filter(|&s| bid(s) == top).collect();
            if best.len() > 1 {
                public.push(format!("{name}: tie at {top}, nobody won."));
                for &s in &seats {
                    self.logs[s].push(format!("You did not win the {name}."));
                }
                continue;
            }
            let w = best[0];
            public.push(format!("{name}: won for {top} coin{}.", if top == 1 { "" } else { "s" }));
            self.coins[w] -= top;
            let cell = self.picks[w].unwrap_or(0);
            let before = self.boards[w].lines();
            self.boards[w].marked[cell] = true;
            // A replay moment: the won cell completes a line against rivals (their number is public).
            // Who won stays secret, so only the winner sees it.
            let lines = self.boards[w].lines() - before;
            let rivals = seats.len() - 1;
            if lines > 0 && rivals > 0 {
                let what = if lines == 1 { "Bingo".to_string() } else { format!("{lines} lines at once") };
                let text = format!("{what}: {rivals} rival{} outbid (+{})", if rivals == 1 { "" } else { "s" }, 5 * lines);
                let me = self.players[w].clone();
                self.moments.push_for(&me, text, &[&me]);
            }
            for &s in &seats {
                self.logs[s].push(if s == w { format!("You won the {name} for {top}.") } else { format!("You did not win the {name}.") });
            }
        }
        for log in &mut self.logs {
            // Public lines first, then the private outcome that was just pushed.
            let private = log.pop();
            log.extend(public.iter().cloned());
            log.extend(private);
        }
        self.round += 1;
        self.picks = vec![None; n];
        self.bids = vec![None; n];
        self.phase = Phase::Pick;
        if self.round >= ROUNDS || self.boards.iter().all(|b| b.marked.iter().all(|&m| m)) {
            self.over = true;
        }
    }

    fn score(&self, seat: usize) -> i64 {
        board_score(&self.boards[seat], self.coins[seat])
    }
}

/// Ranks from scores; points 1000 for first down to 100 for last.
pub fn rank_points(scores: &[(String, i64)]) -> Vec<PlayerResult> {
    let n = scores.len() as i64;
    rank_by_score(scores)
        .into_iter()
        .map(|mut r| {
            r.points = 100 + 900 * (n - i64::from(r.rank)) / (n - 1).max(1);
            r
        })
        .collect()
}

/// How much a cell helps build lines on this board.
fn cell_value(b: &Board, cell: usize) -> f64 {
    LINES
        .iter()
        .filter(|l| l.contains(&cell))
        .map(|l| {
            let have = l.iter().filter(|&&c| b.marked[c]).count() as f64;
            1.0 + have * have * 2.0
        })
        .sum()
}

pub struct Rules;

impl Game for Rules {
    type State = BingoState;
    type Move = BingoMove;
    type View = BingoView;

    fn info() -> GameInfo {
        GameInfo {
            id: "auction-bingo",
            kind: "main",
            difficulty: "medium",
            tags: &["auction-economy", "speed-risk"],
            min_players: 3,
            max_players: 8,
            name: "Auction Bingo",
            short: "A secret 3×3 board of shapes. Each round pick a shape, then bid coins for it; complete lines.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<BingoState, String> {
        if !(3..=8).contains(&players.len()) {
            return Err("Auction Bingo needs 3–8 players".into());
        }
        let n = players.len();
        let mut rng = Rng::new(seed);
        let boards = (0..n)
            .map(|_| {
                let mut shapes = vec![0, 0, 0, 1, 1, 1, 2, 2, 2];
                rng.shuffle(&mut shapes);
                Board { shapes, marked: vec![false; 9] }
            })
            .collect();
        Ok(BingoState {
            players: players.to_vec(),
            boards,
            coins: vec![COINS; n],
            round: 0,
            phase: Phase::Pick,
            picks: vec![None; n],
            bids: vec![None; n],
            logs: vec![Vec::new(); n],
            over: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut BingoState, player: &str, mv: BingoMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.players.iter().position(|p| p == player).ok_or("Not in this game")?;
        match (mv, s.phase) {
            (BingoMove::Pick { cell }, Phase::Pick) => {
                if s.picks[seat].is_some() {
                    return Err("You already picked".into());
                }
                if cell >= 9 || s.boards[seat].marked[cell] {
                    return Err("Pick an empty cell".into());
                }
                s.picks[seat] = Some(cell);
            }
            (BingoMove::Bid { coins }, Phase::Bid) => {
                if s.bids[seat].is_some() {
                    return Err("You already bid".into());
                }
                if coins < 0 || coins > s.coins[seat] {
                    return Err("You do not have that many coins".into());
                }
                s.bids[seat] = Some(coins);
            }
            _ => return Err("Not now".into()),
        }
        s.after_move();
        Ok(())
    }

    fn view_for(s: &BingoState, player: &str) -> BingoView {
        let me = s.players.iter().position(|p| p == player).unwrap_or(0);
        let log = &s.logs[me];
        BingoView {
            players: s.players.clone(),
            me,
            round: (s.round + 1).min(ROUNDS),
            rounds: ROUNDS,
            phase: if s.over {
                "over"
            } else if s.phase == Phase::Pick {
                "pick"
            } else {
                "bid"
            },
            board: s.boards[me].clone(),
            coins: s.coins[me],
            my_pick: s.picks[me],
            my_bid: s.bids[me],
            counts: (s.phase == Phase::Bid && !s.over).then(|| s.counts()),
            done: (0..s.players.len()).map(|i| if s.phase == Phase::Pick { s.picks[i].is_some() } else { s.bids[i].is_some() }).collect(),
            score: s.score(me),
            log: log[log.len().saturating_sub(16)..].to_vec(),
            result: s.over.then(|| Final {
                boards: s.boards.clone(),
                coins: s.coins.clone(),
                scores: (0..s.players.len()).map(|i| s.score(i)).collect(),
            }),
        }
    }

    fn is_over(s: &BingoState) -> bool {
        s.over
    }

    fn results(s: &BingoState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.players.iter().enumerate().map(|(i, p)| (p.clone(), s.score(i))).collect();
        rank_points(&scores)
    }

    fn to_move(s: &BingoState) -> Vec<String> {
        if s.over {
            return Vec::new();
        }
        (0..s.players.len())
            .filter(|&i| if s.phase == Phase::Pick { s.picks[i].is_none() } else { s.bids[i].is_none() })
            .map(|i| s.players[i].clone())
            .collect()
    }

    fn bot_move(s: &BingoState, player: &str, seed: u64) -> BingoMove {
        let mut rng = Rng::new(seed);
        let seat = s.players.iter().position(|p| p == player).unwrap_or(0);
        let b = &s.boards[seat];
        if s.phase == Phase::Pick {
            let cell = (0..9)
                .filter(|&c| !b.marked[c])
                .map(|c| (c, cell_value(b, c) + rng.next_f64() * 2.0))
                .max_by(|x, y| x.1.total_cmp(&y.1))
                .map_or(0, |(c, _)| c);
            return BingoMove::Pick { cell };
        }
        let coins = s.coins[seat];
        let shape = s.picks[seat].map_or(0, |c| b.shapes[c]);
        let rivals = s.counts()[shape].saturating_sub(1) as f64;
        if rivals == 0.0 {
            return BingoMove::Bid { coins: rng.int(0, 1).min(coins) };
        }
        let pace = coins as f64 / (ROUNDS - s.round) as f64;
        let want = pace * (1.0 + 0.5 * rivals) * (0.6 + 0.8 * rng.next_f64());
        BingoMove::Bid { coins: (want.round() as i64).clamp(0, coins) }
    }

    fn moments(s: &BingoState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn ids(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    /// Makes seat i pick a cell of `shape` and bid `bid`.
    fn play(s: &mut BingoState, moves: &[(usize, i64)]) {
        for (i, &(shape, _)) in moves.iter().enumerate() {
            let cell = (0..9).find(|&c| s.boards[i].shapes[c] == shape && !s.boards[i].marked[c]).unwrap();
            Rules::apply_move(s, &format!("p{}", i + 1), BingoMove::Pick { cell }).unwrap();
        }
        for (i, &(_, bid)) in moves.iter().enumerate() {
            Rules::apply_move(s, &format!("p{}", i + 1), BingoMove::Bid { coins: bid }).unwrap();
        }
    }

    #[test]
    fn boards_have_three_of_each_shape() {
        let s = Rules::setup(&ids(3), 1).unwrap();
        for b in &s.boards {
            for shape in 0..3 {
                assert_eq!(b.shapes.iter().filter(|&&x| x == shape).count(), 3);
            }
        }
    }

    #[test]
    fn highest_bidder_wins_and_pays() {
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        play(&mut s, &[(0, 5), (0, 3), (1, 0)]);
        assert_eq!(s.coins, vec![15, 20, 20]);
        assert_eq!(s.boards[0].marked.iter().filter(|&&m| m).count(), 1);
        assert_eq!(s.boards[1].marked.iter().filter(|&&m| m).count(), 0);
        assert_eq!(s.boards[2].marked.iter().filter(|&&m| m).count(), 1, "alone on triangles, wins for 0");
        assert_eq!(s.round, 1);
        let log = Rules::view_for(&s, "p2").log;
        assert!(log.iter().any(|l| l == "circle: won for 5 coins."));
        assert!(log.iter().any(|l| l == "You did not win the circle."));
        assert!(!log.iter().any(|l| l.contains("p1")), "the winner stays secret");
    }

    #[test]
    fn tie_means_nobody_wins() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        play(&mut s, &[(2, 4), (2, 4), (2, 1)]);
        assert_eq!(s.coins, vec![20, 20, 20]);
        assert!(s.boards.iter().all(|b| b.marked.iter().all(|&m| !m)));
    }

    #[test]
    fn counts_show_in_the_bid_step_only() {
        let mut s = Rules::setup(&ids(3), 4).unwrap();
        assert!(Rules::view_for(&s, "p1").counts.is_none());
        for p in ["p1", "p2", "p3"] {
            Rules::apply_move(&mut s, p, BingoMove::Pick { cell: 0 }).unwrap();
        }
        assert_eq!(s.phase, Phase::Bid);
        let c = Rules::view_for(&s, "p1").counts.unwrap();
        assert_eq!(c.iter().sum::<usize>(), 3);
        assert!(Rules::apply_move(&mut s, "p1", BingoMove::Bid { coins: 21 }).is_err());
        for p in ["p1", "p2", "p3"] {
            Rules::apply_move(&mut s, p, BingoMove::Bid { coins: 0 }).unwrap();
        }
        assert_eq!(s.round, 1);
    }

    #[test]
    fn no_clock_the_step_waits_for_everyone() {
        let mut s = Rules::setup(&ids(3), 5).unwrap();
        Rules::apply_move(&mut s, "p1", BingoMove::Pick { cell: 0 }).unwrap();
        Rules::tick(&mut s, 600_000);
        assert_eq!(s.phase, Phase::Pick);
        assert_eq!(Rules::to_move(&s), vec!["p2".to_string(), "p3".to_string()]);
    }

    #[test]
    fn score_counts_lines_cells_and_coins() {
        let b = Board { shapes: vec![0; 9], marked: vec![true, true, true, true, false, false, true, false, false] };
        // Row 0 and column 0: 2 lines; 5 cells; 12 coins = 2.
        assert_eq!(board_score(&b, 12), 10 + 5 + 2);
    }

    #[test]
    fn thousand_bot_games() {
        for n in [3, 8] {
            bot_games_with::<Rules>(500, n, 3000);
        }
    }

    #[test]
    fn a_won_cell_that_completes_a_line_is_a_private_replay_moment() {
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        // p1 has two cells of its top row marked; the third one's shape is on offer.
        let shape = s.boards[0].shapes[2];
        (s.boards[0].marked[0], s.boards[0].marked[1]) = (true, true);
        Rules::apply_move(&mut s, "p1", BingoMove::Pick { cell: 2 }).unwrap();
        for p in ["p2", "p3"] {
            let i = if p == "p2" { 1 } else { 2 };
            let cell = (0..9).find(|&c| s.boards[i].shapes[c] == shape).unwrap();
            Rules::apply_move(&mut s, p, BingoMove::Pick { cell }).unwrap();
        }
        Rules::apply_move(&mut s, "p1", BingoMove::Bid { coins: 6 }).unwrap();
        Rules::apply_move(&mut s, "p2", BingoMove::Bid { coins: 2 }).unwrap();
        Rules::apply_move(&mut s, "p3", BingoMove::Bid { coins: 1 }).unwrap();
        let m = Rules::moments(&s, "p1");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Bingo: 2 rivals outbid (+5)"));
        // Who won stays secret: the others see nothing.
        assert!(Rules::moments(&s, "p2").is_empty());
        assert!(Rules::moments(&s, "p3").is_empty());
        // A line nobody fought for is no moment.
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        let shape = s.boards[0].shapes[2];
        (s.boards[0].marked[0], s.boards[0].marked[1]) = (true, true);
        Rules::apply_move(&mut s, "p1", BingoMove::Pick { cell: 2 }).unwrap();
        for (i, p) in [(1, "p2"), (2, "p3")] {
            let cell = (0..9).find(|&c| s.boards[i].shapes[c] != shape).unwrap();
            Rules::apply_move(&mut s, p, BingoMove::Pick { cell }).unwrap();
        }
        for p in ["p1", "p2", "p3"] {
            Rules::apply_move(&mut s, p, BingoMove::Bid { coins: 1 }).unwrap();
        }
        assert!(s.boards[0].marked[2]);
        assert!(Rules::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 3_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

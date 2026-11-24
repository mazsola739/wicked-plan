//! Lay the Grass (mini game, 1–4 seats). Rules page: docs/games/lay-the-grass.md
//!
//! Everyone places the same grass piece each round, in turn, anywhere on the
//! field. Score: the largest connected patch plus a bonus for the largest
//! all-grass square.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 7;
pub const ROCK: i8 = -2;
pub const EMPTY: i8 = -1;

pub type Cell = (i32, i32);

/// The grass pieces (2–5 squares), in their base orientation.
pub fn shapes() -> Vec<Vec<Cell>> {
    vec![
        vec![(0, 0), (1, 0)],
        vec![(0, 0), (1, 0), (2, 0)],
        vec![(0, 0), (0, 1), (1, 1)],
        vec![(0, 0), (1, 0), (2, 0), (3, 0)],
        vec![(0, 0), (1, 0), (0, 1), (1, 1)],
        vec![(0, 0), (1, 0), (2, 0), (1, 1)],
        vec![(0, 0), (0, 1), (0, 2), (1, 2)],
        vec![(1, 0), (2, 0), (0, 1), (1, 1)],
        vec![(0, 0), (1, 0), (0, 1), (1, 1), (0, 2)],
        vec![(0, 0), (0, 1), (0, 2), (0, 3), (1, 3)],
        vec![(0, 0), (1, 0), (2, 0), (1, 1), (1, 2)],
        vec![(0, 0), (2, 0), (0, 1), (1, 1), (2, 1)],
        vec![(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)],
    ]
}

fn normalize(cells: &[Cell]) -> Vec<Cell> {
    let mx = cells.iter().map(|c| c.0).min().unwrap_or(0);
    let my = cells.iter().map(|c| c.1).min().unwrap_or(0);
    let mut out: Vec<Cell> = cells.iter().map(|&(x, y)| (x - mx, y - my)).collect();
    out.sort_by_key(|&(x, y)| (y, x));
    out
}

/// The distinct turns and flips of a piece, normalized.
pub fn orientations(piece: &[Cell]) -> Vec<Vec<Cell>> {
    let mut out: Vec<Vec<Cell>> = Vec::new();
    for flip in [false, true] {
        let mut cur: Vec<Cell> = piece.iter().map(|&(x, y)| if flip { (-x, y) } else { (x, y) }).collect();
        for _ in 0..4 {
            let n = normalize(&cur);
            if !out.contains(&n) {
                out.push(n);
            }
            cur = cur.iter().map(|&(x, y)| (-y, x)).collect();
        }
    }
    out
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tally {
    pub patch: usize,
    pub square: usize,
    pub score: i64,
}

#[derive(Clone, Debug)]
pub struct State {
    pub players: Vec<String>,
    pub size: i32,
    pub cells: Vec<i8>,
    pub pieces: Vec<Vec<Cell>>,
    pub round: usize,
    /// Turns taken in this round (0..players).
    pub step: usize,
    pub last: Vec<Cell>,
    pub over: bool,
    /// Replay moments: the first 3×3 square on the field, any 4×4 square.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Place { cells: Vec<Cell> },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    #[serde(flatten)]
    pub tally: Tally,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub size: i32,
    pub cells: Vec<i8>,
    pub seats: Vec<SeatView>,
    pub you: usize,
    pub round: usize,
    pub rounds: usize,
    pub piece: Option<Vec<Cell>>,
    pub next: Option<Vec<Cell>>,
    pub turn: Option<String>,
    pub last: Vec<Cell>,
    pub over: bool,
}

/// Largest connected patch and largest square of `seat`'s grass.
pub fn tally(cells: &[i8], size: i32, seat: i8) -> Tally {
    let n = size as usize;
    let mut seen = vec![false; cells.len()];
    let mut patch = 0;
    for start in 0..cells.len() {
        if cells[start] != seat || seen[start] {
            continue;
        }
        seen[start] = true;
        let mut stack = vec![start];
        let mut count = 0;
        while let Some(c) = stack.pop() {
            count += 1;
            let (x, y) = (c % n, c / n);
            let mut push = |d: usize| {
                if cells[d] == seat && !seen[d] {
                    seen[d] = true;
                    stack.push(d);
                }
            };
            if x > 0 {
                push(c - 1);
            }
            if x + 1 < n {
                push(c + 1);
            }
            if y > 0 {
                push(c - n);
            }
            if y + 1 < n {
                push(c + n);
            }
        }
        patch = patch.max(count);
    }
    let mut dp = vec![0usize; cells.len()];
    let mut square = 0;
    for c in 0..cells.len() {
        if cells[c] != seat {
            continue;
        }
        let (x, y) = (c % n, c / n);
        dp[c] = if x == 0 || y == 0 { 1 } else { 1 + dp[c - 1].min(dp[c - n]).min(dp[c - n - 1]) };
        square = square.max(dp[c]);
    }
    Tally { patch, square, score: 10 * patch as i64 + 20 * (square * square) as i64 }
}

impl State {
    fn seat_to_move(&self) -> usize {
        (self.round + self.step) % self.players.len()
    }

    fn idx(&self, (x, y): Cell) -> Option<usize> {
        (x >= 0 && y >= 0 && x < self.size && y < self.size).then(|| (y * self.size + x) as usize)
    }

    fn fits(&self, cells: &[Cell]) -> bool {
        cells.iter().all(|&c| self.idx(c).is_some_and(|i| self.cells[i] == EMPTY))
    }

    /// Every spot the current piece fits, as cell lists.
    fn placements(&self) -> Vec<Vec<Cell>> {
        let mut out = Vec::new();
        for o in orientations(&self.pieces[self.round]) {
            for y in 0..self.size {
                for x in 0..self.size {
                    let cells: Vec<Cell> = o.iter().map(|&(dx, dy)| (x + dx, y + dy)).collect();
                    if self.fits(&cells) {
                        out.push(cells);
                    }
                }
            }
        }
        out
    }

    /// Next turn; skips players whose piece fits nowhere.
    fn advance(&mut self) {
        loop {
            self.step += 1;
            if self.step >= self.players.len() {
                self.step = 0;
                self.round += 1;
                if self.round >= ROUNDS {
                    self.over = true;
                    return;
                }
            }
            if !self.placements().is_empty() {
                break;
            }
        }
    }

    fn scores(&self) -> Vec<Tally> {
        (0..self.players.len()).map(|i| tally(&self.cells, self.size, i as i8)).collect()
    }
}

pub struct LayTheGrass;

impl Game for LayTheGrass {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "lay-the-grass",
            kind: "mini-solo",
            difficulty: "medium",
            tags: &["board"],
            min_players: 1,
            max_players: 4,
            name: "Lay the Grass",
            short: "Everyone lays the same grass piece each round: grow the biggest patch and the biggest square.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if players.is_empty() || players.len() > 4 {
            return Err("Lay the Grass needs 1–4 players".into());
        }
        let mut rng = Rng::new(seed);
        let size = 6 + players.len() as i32;
        let mut cells = vec![EMPTY; (size * size) as usize];
        let mut rocks = 0;
        while rocks < players.len() + 2 {
            let i = rng.index(cells.len());
            if cells[i] == EMPTY {
                cells[i] = ROCK;
                rocks += 1;
            }
        }
        let all = shapes();
        let pieces = (0..ROUNDS).map(|_| rng.pick(&all).clone()).collect();
        Ok(State {
            players: players.to_vec(),
            size,
            cells,
            pieces,
            round: 0,
            step: 0,
            last: Vec::new(),
            over: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.seat_to_move();
        if s.players[seat] != player {
            return Err("Not your turn".into());
        }
        let Move::Place { cells } = mv;
        if !s.fits(&cells) {
            return Err("The piece must go on empty squares".into());
        }
        let mut unique = cells.clone();
        unique.sort();
        unique.dedup();
        if unique.len() != cells.len() || !orientations(&s.pieces[s.round]).contains(&normalize(&cells)) {
            return Err("That is not this round's piece".into());
        }
        let before = s.scores();
        for &c in &cells {
            let i = s.idx(c).expect("checked");
            s.cells[i] = seat as i8;
        }
        // Replay moments: a 4×4 square, or the first 3×3 square anyone lays (the field is public).
        let square = tally(&s.cells, s.size, seat as i8).square;
        if square >= 4 && before[seat].square < 4 {
            s.moments.push(player, format!("A {square}×{square} square of grass: {} bonus", 20 * square * square));
        } else if square >= 3 && before.iter().all(|t| t.square < 3) {
            s.moments.push(player, "First 3×3 square of grass on the field");
        }
        s.last = cells;
        s.advance();
        Ok(())
    }

    fn view_for(s: &State, player: &str) -> View {
        let scores = s.scores();
        View {
            size: s.size,
            cells: s.cells.clone(),
            seats: s.players.iter().zip(scores).map(|(id, tally)| SeatView { id: id.clone(), tally }).collect(),
            you: s.players.iter().position(|p| p == player).unwrap_or(0),
            round: (s.round + 1).min(ROUNDS),
            rounds: ROUNDS,
            piece: (!s.over).then(|| s.pieces[s.round].clone()),
            next: s.pieces.get(s.round + 1).filter(|_| !s.over).cloned(),
            turn: (!s.over).then(|| s.players[s.seat_to_move()].clone()),
            last: s.last.clone(),
            over: s.over,
        }
    }

    fn is_over(s: &State) -> bool {
        s.over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        let scores = s.scores();
        rank_by_score(&s.players.iter().zip(scores).map(|(id, t)| (id.clone(), t.score)).collect::<Vec<_>>())
    }

    fn to_move(s: &State) -> Vec<String> {
        if s.over { Vec::new() } else { vec![s.players[s.seat_to_move()].clone()] }
    }

    fn moments(s: &State, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &State, _player: &str, seed: u64) -> Move {
        let mut rng = Rng::new(seed);
        let seat = s.seat_to_move() as i8;
        let n = s.size;
        let own = |c: Cell, cells: &[i8]| s.idx(c).is_some_and(|i| cells[i] == seat);
        let all = s.placements();
        let touching = |p: &Vec<Cell>| -> i64 {
            p.iter().map(|&(x, y)| [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)].iter().filter(|&&c| own(c, &s.cells)).count() as i64).sum()
        };
        let has_grass = s.cells.contains(&seat);
        let mut best: Option<(i64, &Vec<Cell>)> = None;
        for p in &all {
            let touch = touching(p);
            if has_grass && touch == 0 {
                continue;
            }
            let mut cells = s.cells.clone();
            for &(x, y) in p {
                cells[(y * n + x) as usize] = seat;
            }
            let v = tally(&cells, n, seat).score * 4 + touch * 3 + rng.int(0, 6);
            if best.is_none_or(|(b, _)| v > b) {
                best = Some((v, p));
            }
        }
        let cells = match best {
            Some((_, p)) => p.clone(),
            None if !all.is_empty() => rng.pick(&all).clone(),
            None => vec![(0, 0)],
        };
        Move::Place { cells }
    }
}

pub type Rules = LayTheGrass;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn pieces_have_their_turns_and_flips() {
        assert_eq!(orientations(&[(0, 0), (1, 0)]).len(), 2);
        assert_eq!(orientations(&[(0, 0), (1, 0), (0, 1), (1, 1)]).len(), 1);
        assert_eq!(orientations(&[(0, 0), (0, 1), (0, 2), (1, 2)]).len(), 8);
        assert!(shapes().iter().all(|p| (2..=5).contains(&p.len())));
    }

    #[test]
    fn tally_counts_patch_and_square() {
        // 4×4 field: a 2×2 square plus a separate pair.
        let mut cells = vec![EMPTY; 16];
        for i in [0, 1, 4, 5, 3, 7, 11] {
            cells[i] = 0;
        }
        let t = tally(&cells, 4, 0);
        assert_eq!((t.patch, t.square), (4, 2));
        assert_eq!(t.score, 40 + 80);
    }

    #[test]
    fn placing_checks_shape_squares_and_turn() {
        let mut s = LayTheGrass::setup(&players(2), 1).unwrap();
        s.cells.iter_mut().for_each(|c| *c = EMPTY);
        s.pieces[0] = vec![(0, 0), (1, 0), (2, 0)];
        assert!(LayTheGrass::apply_move(&mut s, "p2", Move::Place { cells: vec![(0, 0), (1, 0), (2, 0)] }).is_err());
        assert!(LayTheGrass::apply_move(&mut s, "p1", Move::Place { cells: vec![(0, 0), (1, 0), (1, 1)] }).is_err());
        assert!(LayTheGrass::apply_move(&mut s, "p1", Move::Place { cells: vec![(0, 0), (0, 0), (1, 0)] }).is_err());
        // Turned upright is fine.
        LayTheGrass::apply_move(&mut s, "p1", Move::Place { cells: vec![(0, 0), (0, 1), (0, 2)] }).unwrap();
        assert!(LayTheGrass::apply_move(&mut s, "p2", Move::Place { cells: vec![(0, 2), (1, 2), (2, 2)] }).is_err());
        LayTheGrass::apply_move(&mut s, "p2", Move::Place { cells: vec![(1, 2), (2, 2), (3, 2)] }).unwrap();
        // Round 2 starts with the second player.
        assert_eq!(LayTheGrass::to_move(&s), vec!["p2".to_string()]);
    }

    #[test]
    fn a_turn_waits_for_the_move() {
        let mut s = LayTheGrass::setup(&players(3), 2).unwrap();
        LayTheGrass::tick(&mut s, 3_600_000);
        assert_eq!(LayTheGrass::to_move(&s), vec!["p1".to_string()]);
        assert_eq!((s.round, s.step), (0, 0));
    }

    #[test]
    fn bot_grows_a_patch() {
        let mut s = LayTheGrass::setup(&players(1), 3).unwrap();
        while !s.over {
            let mv = LayTheGrass::bot_move(&s, "p1", 1);
            LayTheGrass::apply_move(&mut s, "p1", mv).unwrap();
        }
        let t = tally(&s.cells, s.size, 0);
        let total = s.cells.iter().filter(|&&c| c == 0).count();
        assert_eq!(t.patch, total, "solo bot keeps one patch");
        assert!(t.square >= 2);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<LayTheGrass>(1000, 1, 200);
        bot_games_with::<LayTheGrass>(150, 4, 400);
    }

    #[test]
    fn the_first_3x3_and_any_4x4_square_are_replay_moments() {
        let mut s = LayTheGrass::setup(&players(2), 1).unwrap();
        s.cells.iter_mut().for_each(|c| *c = EMPTY);
        let mut grass = |cells: &[Cell], seat: i8| {
            for &(x, y) in cells {
                s.cells[(y * s.size + x) as usize] = seat;
            }
        };
        // p1 has a 3×2 block (columns 0–2, rows 1–2); p2 a 2×3 block (columns 6–7, rows 0–2).
        grass(&[(0, 1), (1, 1), (2, 1), (0, 2), (1, 2), (2, 2)], 0);
        grass(&[(6, 0), (7, 0), (6, 1), (7, 1), (6, 2), (7, 2)], 1);
        s.pieces[0] = vec![(0, 0), (1, 0), (2, 0)];
        s.pieces[1] = vec![(0, 0), (1, 0), (2, 0), (3, 0)];
        // p1 lays row 0 and makes the first 3×3 on the field.
        LayTheGrass::apply_move(&mut s, "p1", Move::Place { cells: vec![(0, 0), (1, 0), (2, 0)] }).unwrap();
        // p2 makes a 3×3 too: not the first one, no moment.
        LayTheGrass::apply_move(&mut s, "p2", Move::Place { cells: vec![(5, 0), (5, 1), (5, 2)] }).unwrap();
        let m = LayTheGrass::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "First 3×3 square of grass on the field"));
        // Round 2 starts with p2; then p1, with row 3 already grown, completes a 4×4 with column 3.
        LayTheGrass::apply_move(&mut s, "p2", Move::Place { cells: vec![(0, 7), (1, 7), (2, 7), (3, 7)] }).unwrap();
        for x in 0..3 {
            s.cells[(3 * s.size + x) as usize] = 0;
        }
        LayTheGrass::apply_move(&mut s, "p1", Move::Place { cells: vec![(3, 0), (3, 1), (3, 2), (3, 3)] }).unwrap();
        let m = LayTheGrass::moments(&s, "p2");
        assert_eq!(m.len(), 2);
        assert_eq!((m[1].by.as_str(), m[1].text.as_str()), ("p1", "A 4×4 square of grass: 320 bonus"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<LayTheGrass>(300, 4, 400);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 10 <= stats.games * 9, "too common: {stats:?}");
        assert!(stats.most <= 3, "too many in one game: {stats:?}");
    }
}

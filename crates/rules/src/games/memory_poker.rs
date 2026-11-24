//! Memory Poker (mini game, 1–4 seats). Rules page: docs/games/memory-poker.md
//!
//! Study 16 cards, then take two of them face down in snake order; each hand is
//! those two plus three shared cards. Taken cards stay secret until the showdown.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 3;
pub const GRID: usize = 16;
pub const SHARED: usize = 3;
pub const SHOW_MS: i64 = 6_000;
pub const BEST_BONUS: i64 = 60;
pub const HAND_POINTS: [i64; 9] = [20, 50, 90, 130, 180, 200, 250, 320, 400];
pub const HAND_NAMES: [&str; 9] =
    ["High card", "Pair", "Two pair", "Three of a kind", "Straight", "Flush", "Full house", "Four of a kind", "Straight flush"];
const BOT_MEMORY: f64 = 0.6;

/// 0..52: rank 2 + c / 4 (14 is the ace), suit c % 4.
pub type Card = u8;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Face {
    pub rank: u8,
    pub suit: u8,
}

fn face(c: Card) -> Face {
    Face { rank: 2 + c / 4, suit: c % 4 }
}

/// Category (index into `HAND_NAMES`) and tie-break ranks, comparable as a tuple.
pub fn evaluate(cards: &[Card]) -> (usize, Vec<u8>) {
    let mut ranks: Vec<u8> = cards.iter().map(|&c| 2 + c / 4).collect();
    ranks.sort_unstable_by(|a, b| b.cmp(a));
    let flush = cards.len() == 5 && cards.iter().all(|&c| c % 4 == cards[0] % 4);
    let mut groups: Vec<(usize, u8)> = Vec::new();
    for &r in &ranks {
        match groups.iter_mut().find(|g| g.1 == r) {
            Some(g) => g.0 += 1,
            None => groups.push((1, r)),
        }
    }
    groups.sort_unstable_by(|a, b| b.cmp(a));
    let distinct = groups.len() == 5;
    let straight_high = if distinct && ranks[0] - ranks[4] == 4 {
        Some(ranks[0])
    } else if distinct && ranks == [14, 5, 4, 3, 2] {
        Some(5)
    } else {
        None
    };
    let tiebreak: Vec<u8> = groups.iter().map(|g| g.1).collect();
    let counts: Vec<usize> = groups.iter().map(|g| g.0).collect();
    match (straight_high, flush, counts.as_slice()) {
        (Some(h), true, _) => (8, vec![h]),
        (_, _, [4, ..]) => (7, tiebreak),
        (_, _, [3, 2]) => (6, tiebreak),
        (_, true, _) => (5, tiebreak),
        (Some(h), _, _) => (4, vec![h]),
        (_, _, [3, ..]) => (3, tiebreak),
        (_, _, [2, 2, ..]) => (2, tiebreak),
        (_, _, [2, ..]) => (1, tiebreak),
        _ => (0, tiebreak),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Study,
    Pick,
    Show,
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    pub ready: bool,
    pub picks: Vec<usize>,
    pub points: i64,
    pub round_points: i64,
}

#[derive(Clone, Debug)]
pub struct State {
    pub seats: Vec<Seat>,
    pub seed: u64,
    pub rng: Rng,
    pub round: usize,
    pub grid: Vec<Card>,
    pub taken: Vec<Option<usize>>,
    pub shared: Vec<Card>,
    pub phase: Phase,
    /// Time left of the showdown (study and picks have no clock).
    pub time_left_ms: i64,
    /// Seat indices in pick order for this round; `pick` is the position in it.
    pub order: Vec<usize>,
    pub pick: usize,
    pub over: bool,
    /// Replay moments: a straight or better at the showdown.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Ready,
    Pick { cell: usize },
    Next,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GridCell {
    pub card: Option<Face>,
    pub taken_by: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub ready: bool,
    /// Your own cards always; everyone's at the showdown; otherwise null per card.
    pub cards: Vec<Option<Face>>,
    pub hand: Option<&'static str>,
    pub points: i64,
    pub round_points: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub phase: Phase,
    pub round: usize,
    pub rounds: usize,
    pub grid: Vec<GridCell>,
    pub shared: Vec<Face>,
    pub seats: Vec<SeatView>,
    pub you: usize,
    pub turn: Option<String>,
    pub over: bool,
}

/// A fixed hash in [0, 1): whether a bot remembers a cell this round.
fn remembers(seed: u64, round: usize, seat: usize, cell: usize) -> bool {
    let mut x = seed ^ ((round as u64) << 40) ^ ((seat as u64) << 20) ^ cell as u64;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^= x >> 31;
    (x >> 11) as f64 / (1u64 << 53) as f64 <= BOT_MEMORY
}

impl State {
    fn start_round(&mut self) {
        let mut deck: Vec<Card> = (0..52).collect();
        self.rng.shuffle(&mut deck);
        self.grid = deck[..GRID].to_vec();
        self.shared = deck[GRID..GRID + SHARED].to_vec();
        self.taken = vec![None; GRID];
        let n = self.seats.len();
        let forward: Vec<usize> = (0..n).map(|i| (self.round + i) % n).collect();
        self.order = forward.iter().chain(forward.iter().rev()).copied().collect();
        self.pick = 0;
        for s in &mut self.seats {
            s.ready = false;
            s.picks.clear();
            s.round_points = 0;
        }
        self.phase = Phase::Study;
    }

    fn hand_of(&self, seat: usize) -> Vec<Card> {
        self.seats[seat].picks.iter().map(|&c| self.grid[c]).chain(self.shared.iter().copied()).collect()
    }

    fn start_pick(&mut self) {
        self.phase = Phase::Pick;
    }

    fn take(&mut self, cell: usize) {
        let seat = self.order[self.pick];
        self.taken[cell] = Some(seat);
        self.seats[seat].picks.push(cell);
        self.pick += 1;
        if self.pick >= self.order.len() {
            self.showdown();
        }
    }

    fn showdown(&mut self) {
        let hands: Vec<(usize, Vec<u8>)> = (0..self.seats.len()).map(|i| evaluate(&self.hand_of(i))).collect();
        let best = hands.iter().max().cloned();
        for (i, h) in hands.iter().enumerate() {
            let s = &mut self.seats[i];
            s.round_points = HAND_POINTS[h.0] + if Some(h) == best.as_ref() { BEST_BONUS } else { 0 };
            s.points += s.round_points;
            // A replay moment: a straight or better (every hand is shown at the showdown).
            if h.0 >= 4 {
                self.moments.push(&s.id, format!("{} from memory (+{})", HAND_NAMES[h.0], s.round_points));
            }
        }
        self.phase = Phase::Show;
        self.time_left_ms = SHOW_MS;
    }

    fn end_show(&mut self) {
        self.round += 1;
        if self.round >= ROUNDS {
            self.over = true;
        } else {
            self.start_round();
        }
    }

    fn turn(&self) -> Option<usize> {
        (self.phase == Phase::Pick && !self.over).then(|| self.order[self.pick])
    }
}

pub struct MemoryPoker;

impl Game for MemoryPoker {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "memory-poker",
            kind: "mini-solo",
            difficulty: "hard",
            tags: &["memory-observation"],
            min_players: 1,
            max_players: 4,
            name: "Memory Poker",
            short: "Memorise 16 cards, then take two blind: the best hand with the three shared cards wins.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if players.is_empty() || players.len() > 4 {
            return Err("Memory Poker needs 1–4 players".into());
        }
        let seats = players.iter().map(|id| Seat { id: id.clone(), ready: false, picks: Vec::new(), points: 0, round_points: 0 }).collect();
        let mut s = State {
            seats,
            seed,
            rng: Rng::new(seed),
            round: 0,
            grid: Vec::new(),
            taken: Vec::new(),
            shared: Vec::new(),
            phase: Phase::Study,
            time_left_ms: 0,
            order: Vec::new(),
            pick: 0,
            over: false,
            moments: Moments::default(),
        };
        s.start_round();
        Ok(s)
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.seats.iter().position(|x| x.id == player).ok_or("Not in this game")?;
        match (mv, s.phase) {
            (Move::Ready, Phase::Study) => {
                s.seats[seat].ready = true;
                if s.seats.iter().all(|x| x.ready) {
                    s.start_pick();
                }
                Ok(())
            }
            (Move::Pick { cell }, Phase::Pick) => {
                if s.turn() != Some(seat) {
                    return Err("Not your turn".into());
                }
                if cell >= GRID || s.taken[cell].is_some() {
                    return Err("That square is empty".into());
                }
                s.take(cell);
                Ok(())
            }
            (Move::Next, Phase::Show) if seat == 0 => {
                s.end_show();
                Ok(())
            }
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut State, ms: u64) {
        if s.over || s.phase != Phase::Show {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.end_show();
        }
    }

    fn view_for(s: &State, player: &str) -> View {
        let me = s.seats.iter().position(|x| x.id == player).unwrap_or(0);
        let open = s.phase != Phase::Pick || s.over;
        View {
            phase: s.phase,
            round: (s.round + 1).min(ROUNDS),
            rounds: ROUNDS,
            grid: (0..GRID)
                .map(|c| GridCell {
                    card: (open || s.taken[c] == Some(me)).then(|| face(s.grid[c])),
                    taken_by: s.taken[c].map(|i| s.seats[i].id.clone()),
                })
                .collect(),
            shared: s.shared.iter().map(|&c| face(c)).collect(),
            seats: s
                .seats
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    let show = s.phase == Phase::Show || s.over;
                    let visible = show || i == me;
                    SeatView {
                        id: x.id.clone(),
                        ready: x.ready,
                        cards: x.picks.iter().map(|&c| visible.then(|| face(s.grid[c]))).collect(),
                        hand: (visible && x.picks.len() == 2).then(|| HAND_NAMES[evaluate(&s.hand_of(i)).0]),
                        points: x.points,
                        round_points: x.round_points,
                    }
                })
                .collect(),
            you: me,
            turn: s.turn().map(|i| s.seats[i].id.clone()),
            over: s.over,
        }
    }

    fn is_over(s: &State) -> bool {
        s.over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        rank_by_score(&s.seats.iter().map(|x| (x.id.clone(), x.points)).collect::<Vec<_>>())
    }

    fn to_move(s: &State) -> Vec<String> {
        match (s.over, s.phase) {
            (true, _) | (_, Phase::Show) => Vec::new(),
            (_, Phase::Study) => s.seats.iter().filter(|x| !x.ready).map(|x| x.id.clone()).collect(),
            (_, Phase::Pick) => vec![s.seats[s.order[s.pick]].id.clone()],
        }
    }

    fn moments(s: &State, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &State, player: &str, seed: u64) -> Move {
        let mut rng = Rng::new(seed);
        let Some(seat) = s.seats.iter().position(|x| x.id == player) else { return Move::Ready };
        match s.phase {
            Phase::Study => return Move::Ready,
            Phase::Show => return Move::Next,
            Phase::Pick => {}
        }
        let free: Vec<usize> = (0..GRID).filter(|&c| s.taken[c].is_none()).collect();
        let known: Vec<usize> = free.iter().copied().filter(|&c| remembers(s.seed, s.round, seat, c)).collect();
        let mut base: Vec<Card> = s.seats[seat].picks.iter().map(|&c| s.grid[c]).collect();
        base.extend(&s.shared);
        let value = |cards: &[Card]| {
            let (cat, tie) = evaluate(cards);
            cat * 10_000 + tie.first().copied().unwrap_or(0) as usize * 100 + tie.get(1).copied().unwrap_or(0) as usize
        };
        let mut best: Option<(usize, usize)> = None;
        for &a in &known {
            let mut with_a = base.clone();
            with_a.push(s.grid[a]);
            let v = if with_a.len() == 5 {
                value(&with_a)
            } else {
                // First pick: the best hand it could finish with another remembered card.
                known.iter().filter(|&&b| b != a).map(|&b| value(&[with_a.as_slice(), &[s.grid[b]]].concat())).max().unwrap_or(0)
            };
            if best.is_none_or(|(bv, _)| v > bv) {
                best = Some((v, a));
            }
        }
        let cell = match best {
            Some((_, c)) => c,
            None => *rng.pick(&free),
        };
        Move::Pick { cell }
    }
}

pub type Rules = MemoryPoker;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    fn all_ready(s: &mut State) {
        for id in players(s.seats.len()) {
            MemoryPoker::apply_move(s, &id, Move::Ready).unwrap();
        }
    }

    /// Card from rank (2–14) and suit (0–3).
    fn c(rank: u8, suit: u8) -> Card {
        (rank - 2) * 4 + suit
    }

    #[test]
    fn hands_are_ranked() {
        assert_eq!(evaluate(&[c(10, 0), c(11, 0), c(12, 0), c(13, 0), c(14, 0)]).0, 8);
        assert_eq!(evaluate(&[c(9, 0), c(9, 1), c(9, 2), c(9, 3), c(2, 0)]).0, 7);
        assert_eq!(evaluate(&[c(9, 0), c(9, 1), c(9, 2), c(4, 3), c(4, 0)]).0, 6);
        assert_eq!(evaluate(&[c(2, 1), c(7, 1), c(9, 1), c(11, 1), c(13, 1)]).0, 5);
        assert_eq!(evaluate(&[c(14, 1), c(2, 0), c(3, 2), c(4, 1), c(5, 1)]), (4, vec![5]));
        assert_eq!(evaluate(&[c(8, 0), c(8, 1), c(8, 2), c(4, 3), c(5, 0)]).0, 3);
        assert_eq!(evaluate(&[c(8, 0), c(8, 1), c(4, 2), c(4, 3), c(5, 0)]).0, 2);
        assert_eq!(evaluate(&[c(8, 0), c(8, 1), c(3, 2), c(4, 3), c(5, 0)]).0, 1);
        assert_eq!(evaluate(&[c(13, 0), c(8, 1), c(3, 2), c(4, 3), c(5, 0)]).0, 0);
        assert!(evaluate(&[c(9, 0), c(9, 1), c(2, 2), c(3, 3), c(4, 0)]) > evaluate(&[c(8, 0), c(8, 1), c(12, 2), c(13, 3), c(14, 0)]));
    }

    #[test]
    fn study_shows_the_grid_then_picks_stay_secret() {
        let mut s = MemoryPoker::setup(&players(2), 1).unwrap();
        assert!(MemoryPoker::view_for(&s, "p2").grid.iter().all(|g| g.card.is_some()));
        MemoryPoker::apply_move(&mut s, "p1", Move::Ready).unwrap();
        assert_eq!(s.phase, Phase::Study);
        MemoryPoker::apply_move(&mut s, "p2", Move::Ready).unwrap();
        assert_eq!(s.phase, Phase::Pick);
        assert!(MemoryPoker::apply_move(&mut s, "p2", Move::Pick { cell: 0 }).is_err());
        MemoryPoker::apply_move(&mut s, "p1", Move::Pick { cell: 5 }).unwrap();
        let other = MemoryPoker::view_for(&s, "p2");
        assert!(other.grid[5].card.is_none());
        assert_eq!(other.grid[5].taken_by.as_deref(), Some("p1"));
        assert!(other.seats[0].cards[0].is_none());
        assert!(MemoryPoker::view_for(&s, "p1").grid[5].card.is_some());
        assert!(MemoryPoker::apply_move(&mut s, "p2", Move::Pick { cell: 5 }).is_err());
    }

    #[test]
    fn snake_order_and_showdown_points() {
        let mut s = MemoryPoker::setup(&players(2), 2).unwrap();
        all_ready(&mut s);
        for (p, cell) in [("p1", 0), ("p2", 1), ("p2", 2), ("p1", 3)] {
            MemoryPoker::apply_move(&mut s, p, Move::Pick { cell }).unwrap();
        }
        assert_eq!(s.phase, Phase::Show);
        let v = MemoryPoker::view_for(&s, "p1");
        assert!(v.seats[1].cards.iter().all(Option::is_some));
        assert!(v.seats[1].hand.is_some());
        let h1 = evaluate(&s.hand_of(0));
        let h2 = evaluate(&s.hand_of(1));
        let bonus = |me: &(usize, Vec<u8>), other: &(usize, Vec<u8>)| if me >= other { BEST_BONUS } else { 0 };
        assert_eq!(s.seats[0].points, HAND_POINTS[h1.0] + bonus(&h1, &h2));
        assert_eq!(s.seats[1].points, HAND_POINTS[h2.0] + bonus(&h2, &h1));
        MemoryPoker::apply_move(&mut s, "p1", Move::Next).unwrap();
        // Round 2 starts with the second seat.
        all_ready(&mut s);
        assert_eq!(MemoryPoker::to_move(&s), vec!["p2".to_string()]);
    }

    #[test]
    fn study_and_picks_wait_for_the_players() {
        let mut s = MemoryPoker::setup(&players(2), 3).unwrap();
        MemoryPoker::tick(&mut s, 3_600_000);
        assert_eq!(s.phase, Phase::Study);
        all_ready(&mut s);
        MemoryPoker::apply_move(&mut s, "p1", Move::Pick { cell: 0 }).unwrap();
        MemoryPoker::tick(&mut s, 3_600_000);
        assert_eq!((s.phase, s.pick), (Phase::Pick, 1));
        assert_eq!(MemoryPoker::to_move(&s), vec!["p2".to_string()]);
    }

    #[test]
    fn bot_with_memory_beats_random_picks() {
        let (mut bot, mut random) = (0, 0);
        for seed in 1..=200 {
            let mut s = MemoryPoker::setup(&players(1), seed).unwrap();
            all_ready(&mut s);
            let mv = MemoryPoker::bot_move(&s, "p1", seed);
            MemoryPoker::apply_move(&mut s, "p1", mv).unwrap();
            let mv = MemoryPoker::bot_move(&s, "p1", seed);
            MemoryPoker::apply_move(&mut s, "p1", mv).unwrap();
            bot += s.seats[0].round_points;
            random += HAND_POINTS[evaluate(&[s.grid[0], s.grid[1], s.shared[0], s.shared[1], s.shared[2]]).0] + BEST_BONUS;
        }
        assert!(bot > random * 3 / 2, "bot {bot}, random {random}");
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<MemoryPoker>(1000, 1, 300);
        bot_games_with::<MemoryPoker>(300, 4, 600);
    }

    #[test]
    fn a_straight_or_better_at_the_showdown_is_a_replay_moment() {
        let mut s = MemoryPoker::setup(&players(2), 4).unwrap();
        s.shared = vec![c(10, 0), c(11, 0), c(12, 1)];
        s.grid[..4].copy_from_slice(&[c(13, 2), c(2, 3), c(7, 2), c(14, 3)]);
        all_ready(&mut s);
        for (p, cell) in [("p1", 0), ("p2", 1), ("p2", 2)] {
            MemoryPoker::apply_move(&mut s, p, Move::Pick { cell }).unwrap();
        }
        // Picks are secret until the showdown: no moment yet.
        assert!(MemoryPoker::moments(&s, "p2").is_empty());
        MemoryPoker::apply_move(&mut s, "p1", Move::Pick { cell: 3 }).unwrap();
        // p1: 10-J-Q-K-A straight, the best hand: 180 + 60. p2's high card is no moment.
        let m = MemoryPoker::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Straight from memory (+240)"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<MemoryPoker>(300, 4, 600);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 10 <= stats.games * 8, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

//! Tiger and Goat (final duel, 2 players). Rules page: docs/games/tiger-and-goat.md
//!
//! The hunt game on two boards at once: on board 0 the first player has the
//! tigers, on board 1 the goats. In every step each player moves once, on the
//! board where their side is to move (goats in even steps, tigers in odd).

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const SIZE: usize = 5;
pub const GOATS: u8 = 20;
pub const CAPTURES_TO_WIN: u8 = 5;
pub const MAX_PLIES: u32 = 100;
pub const STEP_MS: i64 = 60_000;
pub const WIN_BONUS: i64 = 250;
pub const PER_GOAT: i64 = 50;

const EMPTY: u8 = 0;
const TIGER: u8 = 1;
const GOAT: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Tiger,
    Goat,
}

fn xy(i: usize) -> (i32, i32) {
    ((i % SIZE) as i32, (i / SIZE) as i32)
}

fn at(x: i32, y: i32) -> Option<usize> {
    (x >= 0 && y >= 0 && x < SIZE as i32 && y < SIZE as i32).then(|| y as usize * SIZE + x as usize)
}

/// The line directions from a point: orthogonal always, diagonal on even points.
fn dirs(i: usize) -> &'static [(i32, i32)] {
    const ALL: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    let (x, y) = xy(i);
    if (x + y) % 2 == 0 { &ALL } else { &ALL[..4] }
}

#[derive(Clone, Copy, Debug)]
pub struct Board {
    pub cells: [u8; SIZE * SIZE],
    pub hand: u8,
    pub captured: u8,
    pub plies: u32,
    pub last: Option<(Option<usize>, usize)>,
    pub winner: Option<Side>,
    pub ended: bool,
}

impl Board {
    fn new() -> Board {
        let mut cells = [EMPTY; SIZE * SIZE];
        for c in [0, SIZE - 1, SIZE * (SIZE - 1), SIZE * SIZE - 1] {
            cells[c] = TIGER;
        }
        Board { cells, hand: GOATS, captured: 0, plies: 0, last: None, winner: None, ended: false }
    }

    pub fn side_to_move(&self) -> Side {
        if self.plies.is_multiple_of(2) { Side::Goat } else { Side::Tiger }
    }

    /// Tiger moves (slides and jumps) as (from, to, jumps?).
    fn tiger_moves(&self, mut f: impl FnMut(usize, usize, bool)) {
        for i in 0..SIZE * SIZE {
            if self.cells[i] != TIGER {
                continue;
            }
            let (x, y) = xy(i);
            for &(dx, dy) in dirs(i) {
                let Some(n) = at(x + dx, y + dy) else { continue };
                if self.cells[n] == EMPTY {
                    f(i, n, false);
                } else if self.cells[n] == GOAT
                    && let Some(m) = at(x + 2 * dx, y + 2 * dy)
                    && self.cells[m] == EMPTY
                {
                    f(i, m, true);
                }
            }
        }
    }

    /// Legal moves of the side to move: (from, to), `from` None for a placement.
    pub fn moves(&self) -> Vec<(Option<usize>, usize)> {
        let mut out = Vec::new();
        if self.ended {
            return out;
        }
        match self.side_to_move() {
            Side::Tiger => self.tiger_moves(|f, t, _| out.push((Some(f), t))),
            Side::Goat if self.hand > 0 => out.extend((0..SIZE * SIZE).filter(|&i| self.cells[i] == EMPTY).map(|i| (None, i))),
            Side::Goat => {
                for i in (0..SIZE * SIZE).filter(|&i| self.cells[i] == GOAT) {
                    let (x, y) = xy(i);
                    for &(dx, dy) in dirs(i) {
                        if let Some(n) = at(x + dx, y + dy)
                            && self.cells[n] == EMPTY
                        {
                            out.push((Some(i), n));
                        }
                    }
                }
            }
        }
        out
    }

    /// Plays a move known to be legal.
    fn play(&mut self, from: Option<usize>, to: usize) {
        match from {
            None => {
                self.hand -= 1;
                self.cells[to] = GOAT;
            }
            Some(f) => {
                let piece = self.cells[f];
                self.cells[f] = EMPTY;
                self.cells[to] = piece;
                let ((fx, fy), (tx, ty)) = (xy(f), xy(to));
                if piece == TIGER && ((fx - tx).abs() == 2 || (fy - ty).abs() == 2) {
                    let mid = at((fx + tx) / 2, (fy + ty) / 2).expect("jump middle");
                    self.cells[mid] = EMPTY;
                    self.captured += 1;
                }
            }
        }
        self.plies += 1;
        self.last = Some((from, to));
        if self.captured >= CAPTURES_TO_WIN {
            self.winner = Some(Side::Tiger);
        } else if self.moves().is_empty() {
            // Trapped tigers lose; goats that cannot move lose too.
            self.winner = Some(if self.side_to_move() == Side::Tiger { Side::Goat } else { Side::Tiger });
        }
        self.ended = self.winner.is_some() || self.plies >= MAX_PLIES;
    }

    /// How good the board is for the tigers; `jump_weight` is what an open jump is worth.
    fn tiger_score(&self, jump_weight: i64) -> i64 {
        if self.captured >= CAPTURES_TO_WIN {
            return 10_000;
        }
        let (mut slides, mut jumps) = (0, 0);
        self.tiger_moves(|_, _, j| if j { jumps += 1 } else { slides += 1 });
        if slides + jumps == 0 {
            return -10_000;
        }
        100 * i64::from(self.captured) + jump_weight * jumps + 3 * (slides + jumps)
    }

    /// Points this board gives the player of `side`.
    fn points(&self, side: Side) -> i64 {
        let c = i64::from(self.captured.min(CAPTURES_TO_WIN));
        let base = match side {
            Side::Tiger => PER_GOAT * c,
            Side::Goat => PER_GOAT * (i64::from(CAPTURES_TO_WIN) - c),
        };
        base + if self.winner == Some(side) { WIN_BONUS } else { 0 }
    }
}

/// The bot's choice for the side to move: one ply, with the next reply priced into the score.
pub fn best_move(b: &Board, rng: &mut Rng) -> Option<(Option<usize>, usize)> {
    let mut options = b.moves();
    rng.shuffle(&mut options);
    let tiger = b.side_to_move() == Side::Tiger;
    options.into_iter().max_by_key(|&(f, t)| {
        let mut next = *b;
        next.play(f, t);
        // Goats move next after a tiger move and can block one jump; after a goat move the tigers
        // jump at once, so for goats every open jump is nearly a lost goat.
        if tiger { next.tiger_score(12) } else { -next.tiger_score(90) }
    })
}

#[derive(Clone, Debug)]
pub struct TigerState {
    players: [String; 2],
    boards: [Board; 2],
    /// Which boards have had their move in this step.
    moved: [bool; 2],
    step: u32,
    time_left_ms: i64,
    /// Replay moments: every tiger trapped; a board held for all 100 turns; a fifth capture
    /// once all 20 goats are on the board.
    pub moments: Moments,
}

/// The boards' names on the page.
const BOARD_NAMES: [&str; 2] = ["A", "B"];

impl TigerState {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }
    /// The seat that moves next on board `b`: the tigers are seat `b`.
    fn mover(&self, b: usize) -> usize {
        match self.boards[b].side_to_move() {
            Side::Tiger => b,
            Side::Goat => 1 - b,
        }
    }
    fn waiting(&self, b: usize) -> bool {
        !self.boards[b].ended && !self.moved[b]
    }
    /// The board `seat` must move on in this step.
    fn board_for(&self, seat: usize) -> Option<usize> {
        (0..2).find(|&b| self.waiting(b) && self.mover(b) == seat)
    }
    fn over(&self) -> bool {
        self.boards.iter().all(|b| b.ended)
    }
    /// Plays a legal move on board `b`. Replay moments (the boards are public): the goats trap
    /// every tiger or hold out for all 100 turns; the tigers take the fifth goat after all 20
    /// were placed. The tigers on board `b` are seat `b`, the goats seat `1 - b`.
    fn play(&mut self, b: usize, from: Option<usize>, to: usize) {
        let board = &mut self.boards[b];
        board.play(from, to);
        let name = BOARD_NAMES[b];
        let (seat, text) = match board.winner {
            Some(Side::Goat) => (1 - b, format!("Every tiger trapped: board {name} won")),
            Some(Side::Tiger) if board.captured >= CAPTURES_TO_WIN && board.hand == 0 => {
                (b, format!("Board {name} won: fifth goat taken after all 20 placed"))
            }
            None if board.ended => (1 - b, format!("Board {name} held for all {MAX_PLIES} turns")),
            _ => return,
        };
        self.moments.push(&self.players[seat].clone(), text);
    }
    fn after_move(&mut self) {
        if (0..2).all(|b| !self.waiting(b)) {
            self.moved = [false; 2];
            self.step += 1;
            self.time_left_ms = STEP_MS;
        }
    }
    fn points(&self, seat: usize) -> i64 {
        // Seat s has the tigers on board s and the goats on the other.
        100 + self.boards[seat].points(Side::Tiger) + self.boards[1 - seat].points(Side::Goat)
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TigerMove {
    Place { board: usize, to: usize },
    Move { board: usize, from: usize, to: usize },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardView {
    /// 0 empty, 1 tiger, 2 goat.
    pub cells: Vec<u8>,
    pub side: Side,
    pub to_move: Side,
    pub your_turn: bool,
    pub hand: u8,
    pub captured: u8,
    pub turns: u32,
    pub max_turns: u32,
    pub ended: bool,
    pub winner: Option<Side>,
    /// Your legal moves as [from, to]; from is −1 for a placement.
    pub moves: Vec<[i32; 2]>,
    pub last: Option<[i32; 2]>,
    pub points: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TigerView {
    pub boards: Vec<BoardView>,
    pub opponent: String,
    pub step: u32,
    pub score: i64,
    pub their_score: i64,
    pub time_left_ms: i64,
    pub step_ms: i64,
    pub over: bool,
}

pub struct TigerAndGoat;

impl Game for TigerAndGoat {
    type State = TigerState;
    type Move = TigerMove;
    type View = TigerView;

    fn info() -> GameInfo {
        GameInfo {
            id: "tiger-and-goat",
            kind: "final-duel",
            difficulty: "hard",
            tags: &["board"],
            min_players: 2,
            max_players: 2,
            name: "Tiger and Goat",
            short: "The hunt game on two boards at once: tigers on one, goats on the other.",
        }
    }

    fn setup(players: &[String], _seed: u64) -> Result<TigerState, String> {
        let [a, b] = players else { return Err("Tiger and Goat is for 2 players".into()) };
        Ok(TigerState {
            players: [a.clone(), b.clone()],
            boards: [Board::new(), Board::new()],
            moved: [false; 2],
            step: 0,
            time_left_ms: STEP_MS,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut TigerState, player: &str, mv: TigerMove) -> Result<(), String> {
        if s.over() {
            return Err("Game is over".into());
        }
        let seat = s.seat(player).ok_or("Not in this game")?;
        let (b, from, to) = match mv {
            TigerMove::Place { board, to } => (board, None, to),
            TigerMove::Move { board, from, to } => (board, Some(from), to),
        };
        if s.board_for(seat) != Some(b) {
            return Err("Not your turn on that board".into());
        }
        if !s.boards[b].moves().contains(&(from, to)) {
            return Err("That move is not allowed".into());
        }
        s.play(b, from, to);
        s.moved[b] = true;
        s.after_move();
        Ok(())
    }

    fn tick(s: &mut TigerState, ms: u64) {
        if s.over() {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            for b in 0..2 {
                if s.waiting(b) {
                    let mut rng = Rng::new(u64::from(s.step) * 2 + b as u64 + 1);
                    match best_move(&s.boards[b], &mut rng) {
                        Some((f, t)) => s.play(b, f, t),
                        None => s.boards[b].ended = true,
                    }
                    s.moved[b] = true;
                }
            }
            s.after_move();
        }
    }

    fn view_for(s: &TigerState, player: &str) -> TigerView {
        let me = s.seat(player).unwrap_or(0);
        let boards = (0..2)
            .map(|b| {
                let bd = &s.boards[b];
                let side = if b == me { Side::Tiger } else { Side::Goat };
                let your_turn = s.board_for(me) == Some(b);
                BoardView {
                    cells: bd.cells.to_vec(),
                    side,
                    to_move: bd.side_to_move(),
                    your_turn,
                    hand: bd.hand,
                    captured: bd.captured,
                    turns: bd.plies,
                    max_turns: MAX_PLIES,
                    ended: bd.ended,
                    winner: bd.winner,
                    moves: if your_turn {
                        bd.moves().into_iter().map(|(f, t)| [f.map_or(-1, |f| f as i32), t as i32]).collect()
                    } else {
                        Vec::new()
                    },
                    last: bd.last.map(|(f, t)| [f.map_or(-1, |f| f as i32), t as i32]),
                    points: bd.points(side),
                }
            })
            .collect();
        TigerView {
            boards,
            opponent: s.players[1 - me].clone(),
            step: s.step + 1,
            score: s.points(me),
            their_score: s.points(1 - me),
            time_left_ms: s.time_left_ms.max(0),
            step_ms: STEP_MS,
            over: s.over(),
        }
    }

    fn is_over(s: &TigerState) -> bool {
        s.over()
    }

    fn results(s: &TigerState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = (0..2).map(|p| (s.players[p].clone(), s.points(p))).collect();
        rank_by_score(&scores)
    }

    fn to_move(s: &TigerState) -> Vec<String> {
        (0..2).filter(|&p| s.board_for(p).is_some()).map(|p| s.players[p].clone()).collect()
    }

    fn bot_move(s: &TigerState, player: &str, seed: u64) -> TigerMove {
        let seat = s.seat(player).unwrap_or(0);
        let b = s.board_for(seat).unwrap_or(0);
        match best_move(&s.boards[b], &mut Rng::new(seed)) {
            Some((None, to)) => TigerMove::Place { board: b, to },
            Some((Some(from), to)) => TigerMove::Move { board: b, from, to },
            None => TigerMove::Place { board: b, to: 0 },
        }
    }

    fn moments(s: &TigerState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub const READY: bool = true;
pub type Rules = TigerAndGoat;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn new() -> TigerState {
        TigerAndGoat::setup(&["a".into(), "b".into()], 1).unwrap()
    }

    #[test]
    fn each_player_moves_once_per_step() {
        let mut s = new();
        let mut both = TigerAndGoat::to_move(&s);
        both.sort();
        assert_eq!(both, vec!["a".to_string(), "b".to_string()]);
        // Step 0: goats move; a has the goats on board 1.
        assert!(TigerAndGoat::apply_move(&mut s, "a", TigerMove::Place { board: 0, to: 12 }).is_err());
        TigerAndGoat::apply_move(&mut s, "a", TigerMove::Place { board: 1, to: 12 }).unwrap();
        assert_eq!(TigerAndGoat::to_move(&s), vec!["b".to_string()]);
        assert!(TigerAndGoat::apply_move(&mut s, "a", TigerMove::Move { board: 1, from: 0, to: 1 }).is_err());
        TigerAndGoat::apply_move(&mut s, "b", TigerMove::Place { board: 0, to: 7 }).unwrap();
        assert_eq!(s.step, 1);
        // Step 1: tigers; a's tigers are on board 0.
        TigerAndGoat::apply_move(&mut s, "a", TigerMove::Move { board: 0, from: 0, to: 1 }).unwrap();
        assert_eq!(TigerAndGoat::to_move(&s), vec!["b".to_string()]);
    }

    #[test]
    fn diagonals_only_from_even_points() {
        let mut b = Board::new();
        b.plies = 1;
        // Corner tiger at 0 (even) may go diagonally to 6; a tiger on 1 (odd) may not reach 5+2=7.
        let m = b.moves();
        assert!(m.contains(&(Some(0), 6)));
        b.cells = [EMPTY; 25];
        b.cells[1] = TIGER;
        let m = b.moves();
        assert!(!m.contains(&(Some(1), 7)) && !m.contains(&(Some(1), 5)));
        assert!(m.contains(&(Some(1), 6)));
    }

    #[test]
    fn a_jump_captures_and_five_win() {
        let mut b = Board::new();
        b.cells[1] = GOAT;
        b.hand = 0;
        b.plies = 1;
        b.captured = 4;
        assert!(b.moves().contains(&(Some(0), 2)));
        b.play(Some(0), 2);
        assert_eq!(b.cells[1], EMPTY);
        assert_eq!(b.captured, 5);
        assert_eq!(b.winner, Some(Side::Tiger));
        assert!(b.ended);
    }

    #[test]
    fn trapped_tigers_lose() {
        let mut b = Board::new();
        b.cells = [EMPTY; 25];
        b.cells[0] = TIGER;
        // Fill both lines out of the corner two deep.
        for i in [1, 2, 5, 10, 6, 12] {
            b.cells[i] = GOAT;
        }
        b.hand = 1;
        b.plies = 0;
        b.play(None, 24);
        assert_eq!(b.winner, Some(Side::Goat));
    }

    #[test]
    fn the_bot_captures_and_the_goats_avoid_jumps() {
        let mut b = Board::new();
        b.cells[1] = GOAT;
        b.hand = 19;
        b.plies = 1;
        assert_eq!(best_move(&b, &mut Rng::new(1)), Some((Some(0), 2)));
        // Goats to move with 19 in hand: never place next to a tiger with open space behind.
        b.plies = 2;
        b.cells[1] = EMPTY;
        b.hand = 20;
        let (_, to) = best_move(&b, &mut Rng::new(1)).unwrap();
        let mut next = b;
        next.play(None, to);
        let mut jumps = 0;
        next.tiger_moves(|_, _, j| jumps += i32::from(j));
        assert_eq!(jumps, 0);
    }

    #[test]
    fn points_add_both_boards() {
        let mut s = new();
        s.boards[0].captured = 5;
        s.boards[0].winner = Some(Side::Tiger);
        s.boards[1].captured = 2;
        assert_eq!(s.points(0), 100 + 250 + 250 + 150);
        assert_eq!(s.points(1), 100 + 100);
    }

    #[test]
    fn time_up_moves_for_both() {
        let mut s = new();
        TigerAndGoat::tick(&mut s, STEP_MS as u64);
        assert_eq!(s.step, 1);
        assert_eq!(s.boards[0].hand, GOATS - 1);
        assert_eq!(s.boards[1].hand, GOATS - 1);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<TigerAndGoat>(1000, 2, 3000);
    }

    #[test]
    fn a_trap_a_hold_out_and_a_late_fifth_capture_are_replay_moments() {
        // Board B: the goats (seat a) shut in the last tiger.
        let mut s = new();
        s.boards[1].cells = [EMPTY; 25];
        s.boards[1].cells[0] = TIGER;
        for i in [1, 2, 5, 10, 6, 12] {
            s.boards[1].cells[i] = GOAT;
        }
        s.boards[1].hand = 1;
        s.play(1, None, 24);
        // Board A: the tigers (seat a) take the fifth goat with all 20 placed.
        s.boards[0].cells[1] = GOAT;
        s.boards[0].hand = 0;
        s.boards[0].plies = 41;
        s.boards[0].captured = 4;
        s.play(0, Some(0), 2);
        let m = TigerAndGoat::moments(&s, "b");
        assert_eq!(m.len(), 2);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("a", "Every tiger trapped: board B won"));
        assert_eq!((m[1].by.as_str(), m[1].text.as_str()), ("a", "Board A won: fifth goat taken after all 20 placed"));
        // Board A again: the goats (seat b) last all 100 turns.
        let mut s = new();
        s.boards[0].plies = MAX_PLIES - 1;
        s.boards[0].captured = 2;
        s.play(0, Some(0), 1);
        let m = TigerAndGoat::moments(&s, "a");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("b", "Board A held for all 100 turns"));
        // A fifth capture while goats are still being placed is no moment.
        let mut s = new();
        s.boards[0].cells[1] = GOAT;
        s.boards[0].hand = 10;
        s.boards[0].plies = 21;
        s.boards[0].captured = 4;
        s.play(0, Some(0), 2);
        assert!(s.boards[0].ended && s.moments.is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<TigerAndGoat>(300, 2, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 2, "too many in one game: {stats:?}");
    }
}

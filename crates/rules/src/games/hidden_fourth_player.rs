//! Hidden Fourth Player (mini team game, 3 players). Rules page: docs/games/hidden-fourth-player.md
//!
//! Three players and the Shadow (run by the rules) place stones on a 7×7 board.
//! Three in a row is a line. Players score their own lines, but if the Shadow
//! makes a line first the game ends and everyone's points shrink: block it together.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const N: usize = 7;
const CELLS: usize = N * N;
pub const SHADOW: u8 = 4;
const EMPTY: u8 = 0;
pub const SURVIVE_BONUS: i64 = 300;

/// Every 3-square stretch: across, down and both diagonals.
fn windows() -> &'static [[usize; 3]] {
    static W: std::sync::OnceLock<Vec<[usize; 3]>> = std::sync::OnceLock::new();
    W.get_or_init(|| {
        let mut out = Vec::new();
        for y in 0..N as i32 {
            for x in 0..N as i32 {
                for (dx, dy) in [(1, 0), (0, 1), (1, 1), (1, -1)] {
                    let (ex, ey) = (x + 2 * dx, y + 2 * dy);
                    if (0..N as i32).contains(&ex) && (0..N as i32).contains(&ey) {
                        out.push([0, 1, 2].map(|k| ((y + k * dy) * N as i32 + x + k * dx) as usize));
                    }
                }
            }
        }
        out
    })
}

/// The stretches through each square.
fn windows_at(c: usize) -> impl Iterator<Item = &'static [usize; 3]> {
    static AT: std::sync::OnceLock<Vec<Vec<usize>>> = std::sync::OnceLock::new();
    let at = AT.get_or_init(|| (0..CELLS).map(|c| (0..windows().len()).filter(|&i| windows()[i].contains(&c)).collect()).collect());
    at[c].iter().map(|&i| &windows()[i])
}

fn count(board: &[u8; CELLS], w: &[usize; 3], v: u8) -> usize {
    w.iter().filter(|&&c| board[c] == v).count()
}

/// Lines (full 3-stretches) of colour `v`.
pub fn lines(board: &[u8; CELLS], v: u8) -> usize {
    windows().iter().filter(|w| count(board, w, v) == 3).count()
}

/// Empty squares where colour `v` would complete a line now.
pub fn completing(board: &[u8; CELLS], v: u8) -> Vec<usize> {
    let mut out: Vec<usize> = windows()
        .iter()
        .filter(|w| count(board, w, v) == 2 && count(board, w, EMPTY) == 1)
        .filter_map(|w| w.iter().copied().find(|&c| board[c] == EMPTY))
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// The Shadow's choice: complete a line, else make the most open twos.
fn shadow_square(board: &[u8; CELLS], rng: &mut Rng) -> Option<usize> {
    if let Some(&c) = completing(board, SHADOW).first() {
        return Some(c);
    }
    let mut empty: Vec<usize> = (0..CELLS).filter(|&c| board[c] == EMPTY).collect();
    rng.shuffle(&mut empty);
    empty.into_iter().max_by_key(|&c| {
        windows_at(c)
            .filter(|w| count(board, w, EMPTY) + count(board, w, SHADOW) == 3)
            .map(|w| if count(board, w, SHADOW) == 1 { 10 } else { 1 })
            .sum::<i32>()
    })
}

/// How good square `c` is for seat `seat` (colour seat + 1) to play now; `threats` are the
/// squares where the Shadow would complete a line.
fn square_value(board: &[u8; CELLS], seat: usize, c: usize, threats: &[usize]) -> i64 {
    let me = seat as u8 + 1;
    // Players still to move before the Shadow, this one included.
    let to_go = 3 - seat;
    let mut v = 0;
    if threats.contains(&c) {
        v += if threats.len() >= to_go { 10_000 } else { 300 };
    }
    let mut shadow_new = 0;
    for w in windows_at(c) {
        let (mine, shadow, empty) = (count(board, w, me), count(board, w, SHADOW), count(board, w, EMPTY));
        if shadow == 1 && empty == 2 {
            shadow_new += 1;
        }
        match (mine, empty) {
            (2, 1) => v += 400,
            (1, 2) => v += 30,
            (0, 3) => v += 3,
            _ => {}
        }
        if empty == 1 && shadow == 0 && mine == 0 && (1..=3).any(|o| o != me && count(board, w, o) == 2) {
            v += 40;
        }
    }
    v += if shadow_new >= 2 { 60 * shadow_new } else { 15 * shadow_new };
    let (x, y) = ((c % N) as i64, (c / N) as i64);
    v + 6 - (x - 3).abs() - (y - 3).abs()
}

#[derive(Clone, Debug)]
pub struct HiddenState {
    players: [String; 3],
    board: [u8; CELLS],
    turn: usize,
    rng: Rng,
    round: u32,
    /// The stones of the last round: (colour, square).
    last: Vec<(u8, usize)>,
    shadow_won: bool,
    over: bool,
    /// Replay moments: two lines with one stone, a stone that blocks the Shadow and makes a line.
    pub moments: Moments,
}

impl HiddenState {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }
    fn full(&self) -> bool {
        self.board.iter().all(|&c| c != EMPTY)
    }
    fn place(&mut self, colour: u8, c: usize) {
        if colour == 1 {
            self.last.clear();
        }
        self.board[c] = colour;
        self.last.push((colour, c));
    }
    fn after_player(&mut self) {
        if self.full() {
            self.over = true;
            return;
        }
        if self.turn < 2 {
            self.turn += 1;
            return;
        }
        // The Shadow plays at once.
        if let Some(c) = shadow_square(&self.board, &mut self.rng) {
            self.place(SHADOW, c);
        }
        self.round += 1;
        self.turn = 0;
        if lines(&self.board, SHADOW) > 0 {
            self.shadow_won = true;
            self.over = true;
        } else if self.full() {
            self.over = true;
        }
    }
    fn points(&self, seat: usize) -> i64 {
        let l = lines(&self.board, seat as u8 + 1) as i64;
        if self.shadow_won { 100 + 50 * l } else { 100 + 100 * l + SURVIVE_BONUS }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HiddenMove {
    Place { x: usize, y: usize },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HiddenView {
    pub size: usize,
    /// 0 empty, 1–3 the seats in order, 4 the Shadow.
    pub board: Vec<u8>,
    pub me: usize,
    pub players: Vec<String>,
    pub lines: Vec<usize>,
    pub shadow_lines: usize,
    pub turn: usize,
    pub your_turn: bool,
    /// Squares where the Shadow would complete a line now.
    pub danger: Vec<usize>,
    pub last: Vec<(u8, usize)>,
    pub round: u32,
    pub over: bool,
    pub shadow_won: bool,
}

pub struct HiddenFourthPlayer;

impl Game for HiddenFourthPlayer {
    type State = HiddenState;
    type Move = HiddenMove;
    type View = HiddenView;

    fn info() -> GameInfo {
        GameInfo {
            id: "hidden-fourth-player",
            kind: "mini-team",
            difficulty: "hard",
            tags: &["board", "alliance"],
            min_players: 3,
            max_players: 3,
            name: "Hidden Fourth Player",
            short: "Three in a row for three players, against a fourth that plays itself. Block it together or all lose.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<HiddenState, String> {
        let [a, b, c] = players else { return Err("Hidden Fourth Player is for 3 players".into()) };
        let mut rng = Rng::new(seed);
        let mut board = [EMPTY; CELLS];
        // The opening Shadow stone: somewhere in the middle 5×5.
        let (x, y) = (rng.int(1, 5) as usize, rng.int(1, 5) as usize);
        board[y * N + x] = SHADOW;
        Ok(HiddenState {
            players: [a.clone(), b.clone(), c.clone()],
            board,
            turn: 0,
            rng,
            round: 1,
            last: vec![(SHADOW, y * N + x)],
            shadow_won: false,
            over: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut HiddenState, player: &str, mv: HiddenMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.seat(player).ok_or("Not in this game")?;
        if seat != s.turn {
            return Err("Not your turn".into());
        }
        let HiddenMove::Place { x, y } = mv;
        if x >= N || y >= N || s.board[y * N + x] != EMPTY {
            return Err("That square is taken".into());
        }
        let (c, me) = (y * N + x, seat as u8 + 1);
        let lines_before = lines(&s.board, me);
        let blocks = windows_at(c).any(|w| count(&s.board, w, SHADOW) == 2);
        s.place(me, c);
        // Replay moments (the board is public).
        let made = lines(&s.board, me) - lines_before;
        if made >= 2 {
            s.moments.push(player, format!("{made} lines with one stone"));
        } else if made == 1 && blocks {
            s.moments.push(player, "One stone: Shadow blocked, line made");
        }
        s.after_player();
        Ok(())
    }

    fn view_for(s: &HiddenState, player: &str) -> HiddenView {
        let me = s.seat(player).unwrap_or(0);
        HiddenView {
            size: N,
            board: s.board.to_vec(),
            me,
            players: s.players.to_vec(),
            lines: (0..3).map(|i| lines(&s.board, i as u8 + 1)).collect(),
            shadow_lines: lines(&s.board, SHADOW),
            turn: s.turn,
            your_turn: !s.over && s.turn == me,
            danger: if s.over { Vec::new() } else { completing(&s.board, SHADOW) },
            last: s.last.clone(),
            round: s.round,
            over: s.over,
            shadow_won: s.shadow_won,
        }
    }

    fn is_over(s: &HiddenState) -> bool {
        s.over
    }

    fn results(s: &HiddenState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = (0..3).map(|i| (s.players[i].clone(), s.points(i))).collect();
        rank_by_score(&scores)
    }

    fn to_move(s: &HiddenState) -> Vec<String> {
        if s.over { Vec::new() } else { vec![s.players[s.turn].clone()] }
    }

    fn bot_move(s: &HiddenState, player: &str, seed: u64) -> HiddenMove {
        let seat = s.seat(player).unwrap_or(0);
        let mut empty: Vec<usize> = (0..CELLS).filter(|&c| s.board[c] == EMPTY).collect();
        Rng::new(seed).shuffle(&mut empty);
        let threats = completing(&s.board, SHADOW);
        let c = empty.into_iter().max_by_key(|&c| square_value(&s.board, seat, c, &threats)).unwrap_or(0);
        HiddenMove::Place { x: c % N, y: c / N }
    }

    fn moments(s: &HiddenState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub type Rules = HiddenFourthPlayer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn new() -> HiddenState {
        let mut s = HiddenFourthPlayer::setup(&["a".into(), "b".into(), "c".into()], 1).unwrap();
        s.board = [EMPTY; CELLS];
        s
    }

    fn at(x: usize, y: usize) -> usize {
        y * N + x
    }

    #[test]
    fn there_are_120_stretches() {
        assert_eq!(windows().len(), 35 + 35 + 25 + 25);
    }

    #[test]
    fn the_shadow_plays_after_the_third_player() {
        let mut s = new();
        for p in ["a", "b"] {
            let before = s.board.iter().filter(|&&c| c == SHADOW).count();
            let HiddenMove::Place { x, y } = HiddenFourthPlayer::bot_move(&s, p, 1);
            HiddenFourthPlayer::apply_move(&mut s, p, HiddenMove::Place { x, y }).unwrap();
            assert_eq!(s.board.iter().filter(|&&c| c == SHADOW).count(), before);
        }
        assert!(HiddenFourthPlayer::apply_move(&mut s, "a", HiddenMove::Place { x: 0, y: 0 }).is_err());
        HiddenFourthPlayer::apply_move(&mut s, "c", HiddenMove::Place { x: 0, y: 0 }).unwrap();
        assert_eq!(s.board.iter().filter(|&&c| c == SHADOW).count(), 1);
        assert_eq!(s.turn, 0);
        assert_eq!(s.round, 2);
    }

    #[test]
    fn a_shadow_line_ends_the_game_and_halves_lines() {
        let mut s = new();
        s.board[at(0, 0)] = SHADOW;
        s.board[at(1, 0)] = SHADOW;
        for x in 0..3 {
            s.board[at(x, 6)] = 1;
        }
        s.turn = 2;
        HiddenFourthPlayer::apply_move(&mut s, "c", HiddenMove::Place { x: 6, y: 6 }).unwrap();
        assert!(s.over && s.shadow_won);
        assert_eq!(s.points(0), 150);
        assert_eq!(s.points(1), 100);
    }

    #[test]
    fn the_last_player_blocks_and_others_may_not() {
        let mut s = new();
        s.board[at(2, 2)] = SHADOW;
        s.board[at(3, 2)] = SHADOW;
        s.board[at(1, 2)] = 2;
        // One threat at (4,2). The last player before the Shadow must block it.
        let HiddenMove::Place { x, y } = HiddenFourthPlayer::bot_move(&s, "c", 1);
        assert_eq!((x, y), (4, 2));
        // The first player may complete its own line instead.
        s.board[at(0, 5)] = 1;
        s.board[at(1, 5)] = 1;
        let HiddenMove::Place { x, y } = HiddenFourthPlayer::bot_move(&s, "a", 1);
        assert_eq!((x, y), (2, 5));
    }

    #[test]
    fn the_shadow_completes_its_line() {
        let mut b = [EMPTY; CELLS];
        b[at(4, 4)] = SHADOW;
        b[at(5, 5)] = SHADOW;
        assert_eq!(completing(&b, SHADOW), vec![at(3, 3), at(6, 6)]);
        let c = shadow_square(&b, &mut Rng::new(1)).unwrap();
        assert!(c == at(3, 3) || c == at(6, 6));
    }

    #[test]
    fn the_game_waits_for_the_player() {
        let mut s = new();
        HiddenFourthPlayer::tick(&mut s, 600_000);
        assert_eq!(s.turn, 0);
        assert!(s.board.iter().all(|&c| c == EMPTY));
        assert_eq!(HiddenFourthPlayer::to_move(&s), vec!["a".to_string()]);
    }

    #[test]
    fn bots_usually_hold_off_the_shadow() {
        let mut survived = 0;
        for seed in 0..40 {
            let mut s = HiddenFourthPlayer::setup(&["a".into(), "b".into(), "c".into()], seed).unwrap();
            while !s.over {
                let p = s.players[s.turn].clone();
                let mv = HiddenFourthPlayer::bot_move(&s, &p, seed);
                HiddenFourthPlayer::apply_move(&mut s, &p, mv).unwrap();
            }
            survived += usize::from(!s.shadow_won);
        }
        // 30 of 40 when written: the Shadow wins now and then, so blocking matters.
        assert!((20..40).contains(&survived), "survived {survived} of 40");
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<HiddenFourthPlayer>(1000, 3, 1500);
    }

    #[test]
    fn two_lines_at_once_and_a_blocking_line_are_replay_moments() {
        let mut s = new();
        // a: (0,0) (1,0) _ (3,0) (4,0): the gap at (2,0) makes 3 lines at once (x 0–2, 1–3, 2–4).
        for x in [0, 1, 3, 4] {
            s.board[at(x, 0)] = 1;
        }
        HiddenFourthPlayer::apply_move(&mut s, "a", HiddenMove::Place { x: 2, y: 0 }).unwrap();
        let m = HiddenFourthPlayer::moments(&s, "b");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("a", "3 lines with one stone"));
        // b: (5,6) closes its own line down column 5 and blocks the Shadow's (3,6) (4,6).
        s.board[at(5, 4)] = 2;
        s.board[at(5, 5)] = 2;
        s.board[at(3, 6)] = SHADOW;
        s.board[at(4, 6)] = SHADOW;
        HiddenFourthPlayer::apply_move(&mut s, "b", HiddenMove::Place { x: 5, y: 6 }).unwrap();
        // c: a plain stone is no moment.
        HiddenFourthPlayer::apply_move(&mut s, "c", HiddenMove::Place { x: 0, y: 3 }).unwrap();
        let m = HiddenFourthPlayer::moments(&s, "a");
        assert_eq!(m.len(), 2);
        assert_eq!((m[1].by.as_str(), m[1].text.as_str()), ("b", "One stone: Shadow blocked, line made"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<HiddenFourthPlayer>(300, 3, 1_500);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

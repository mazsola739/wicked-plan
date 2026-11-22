//! Nine Men's Morris (final duel, 2 players). Rules page: docs/games/nine-mens-morris.md
//!
//! 24 points: outer square 0–7, middle 8–15, inner 16–23, each clockwise from
//! the top-left corner. Place 9 pieces, then slide them (fly with 3). A mill
//! (three in a line) removes an opponent piece.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const PIECES: u8 = 9;
pub const TURN_MS: i64 = 60_000;
pub const MAX_PLIES: u32 = 200;
pub const QUIET_PLIES: u32 = 60;

const EMPTY: i8 = -1;

/// The 16 lines of three.
pub const MILLS: [[usize; 3]; 16] = [
    [0, 1, 2],
    [2, 3, 4],
    [4, 5, 6],
    [6, 7, 0],
    [8, 9, 10],
    [10, 11, 12],
    [12, 13, 14],
    [14, 15, 8],
    [16, 17, 18],
    [18, 19, 20],
    [20, 21, 22],
    [22, 23, 16],
    [1, 9, 17],
    [3, 11, 19],
    [5, 13, 21],
    [7, 15, 23],
];

/// Neighbours of each point, `NONE` where there are fewer than 4. A table
/// because the bot's search asks for them very often.
const NONE: usize = usize::MAX;
const ADJ: [[usize; 4]; 24] = {
    let mut adj = [[NONE; 4]; 24];
    let mut i = 0;
    while i < 24 {
        let (ring, k) = (i / 8, i % 8);
        adj[i][0] = ring * 8 + (k + 1) % 8;
        adj[i][1] = ring * 8 + (k + 7) % 8;
        if k % 2 == 1 && ring > 0 {
            adj[i][2] = i - 8;
        }
        if k % 2 == 1 && ring < 2 {
            adj[i][3] = i + 8;
        }
        i += 1;
    }
    adj
};

/// The two mills through each point.
const POINT_MILLS: [[usize; 2]; 24] = {
    let mut out = [[0; 2]; 24];
    let mut seen = [0; 24];
    let mut m = 0;
    while m < MILLS.len() {
        let mut k = 0;
        while k < 3 {
            let i = MILLS[m][k];
            out[i][seen[i]] = m;
            seen[i] += 1;
            k += 1;
        }
        m += 1;
    }
    out
};

pub fn neighbours(i: usize) -> impl Iterator<Item = usize> {
    ADJ[i].into_iter().filter(|&j| j != NONE)
}

/// A position for the rules and the bot's search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub board: [i8; 24],
    pub hand: [u8; 2],
}

/// One whole turn: a placement (`from` None) or a slide/jump, and the removal it earns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Action {
    pub from: Option<usize>,
    pub to: usize,
    pub remove: Option<usize>,
}

impl Pos {
    fn on_board(&self, p: usize) -> usize {
        let mut n = 0;
        for &c in &self.board {
            n += usize::from(c == p as i8);
        }
        n
    }
    fn pieces(&self, p: usize) -> usize {
        self.on_board(p) + self.hand[p] as usize
    }
    fn in_mill(&self, i: usize) -> bool {
        let p = self.board[i];
        p != EMPTY && POINT_MILLS[i].iter().any(|&m| MILLS[m].iter().all(|&j| self.board[j] == p))
    }
    fn flying(&self, p: usize) -> bool {
        self.hand[p] == 0 && self.on_board(p) == 3
    }
    /// Placements and slides/jumps for `p`, without removals.
    pub fn steps(&self, p: usize) -> Vec<(Option<usize>, usize)> {
        let empty: Vec<usize> = (0..24).filter(|&i| self.board[i] == EMPTY).collect();
        if self.hand[p] > 0 {
            return empty.into_iter().map(|to| (None, to)).collect();
        }
        let fly = self.flying(p);
        let mut out = Vec::new();
        for from in (0..24).filter(|&i| self.board[i] == p as i8) {
            if fly {
                out.extend(empty.iter().map(|&to| (Some(from), to)));
            } else {
                out.extend(neighbours(from).filter(|&to| self.board[to] == EMPTY).map(|to| (Some(from), to)));
            }
        }
        out
    }
    /// The opponent pieces `p` may remove.
    pub fn removable(&self, p: usize) -> Vec<usize> {
        let o = (1 - p) as i8;
        let theirs: Vec<usize> = (0..24).filter(|&i| self.board[i] == o).collect();
        let free: Vec<usize> = theirs.iter().copied().filter(|&i| !self.in_mill(i)).collect();
        if free.is_empty() { theirs } else { free }
    }
    fn step(&mut self, p: usize, from: Option<usize>, to: usize) -> bool {
        match from {
            None => self.hand[p] -= 1,
            Some(f) => self.board[f] = EMPTY,
        }
        self.board[to] = p as i8;
        self.in_mill(to)
    }
    pub fn actions(&self, p: usize) -> Vec<Action> {
        let mut out = Vec::new();
        for (from, to) in self.steps(p) {
            let mut next = *self;
            if next.step(p, from, to) {
                out.extend(next.removable(p).into_iter().map(|r| Action { from, to, remove: Some(r) }));
            } else {
                out.push(Action { from, to, remove: None });
            }
        }
        out
    }
    pub fn after(&self, p: usize, a: Action) -> Pos {
        let mut next = *self;
        next.step(p, a.from, a.to);
        if let Some(r) = a.remove {
            next.board[r] = EMPTY;
        }
        next
    }
    /// How many placements or slides/jumps `p` has (no allocation: the bot calls it a lot).
    pub fn mobility(&self, p: usize) -> usize {
        let mut empty = 0;
        let mut mine = 0;
        let mut slides = 0;
        for (&c, adj) in self.board.iter().zip(&ADJ) {
            if c == EMPTY {
                empty += 1;
            } else if c == p as i8 {
                mine += 1;
                for &j in adj {
                    slides += usize::from(j != NONE && self.board[j] == EMPTY);
                }
            }
        }
        if self.hand[p] > 0 {
            empty
        } else if mine == 3 {
            3 * empty
        } else {
            slides
        }
    }
    /// `p` has lost when down to 2 pieces or unable to move on their turn.
    pub fn lost(&self, p: usize) -> bool {
        self.pieces(p) < 3 || self.mobility(p) == 0
    }
}

/// Score of a position for `p`, who has just moved (the opponent is to move).
fn eval(pos: &Pos, p: usize) -> i64 {
    let o = 1 - p;
    if pos.lost(o) {
        return 100_000;
    }
    let mut v = (pos.pieces(p) as i64 - pos.pieces(o) as i64) * 100;
    for m in &MILLS {
        let (mut mine, mut theirs) = (0, 0);
        for &i in m {
            mine += i32::from(pos.board[i] == p as i8);
            theirs += i32::from(pos.board[i] == o as i8);
        }
        if mine == 2 && theirs == 0 {
            v += 8;
        }
        if theirs == 2 && mine == 0 {
            // The opponent moves next: their open two is a real threat.
            v -= 25;
        }
    }
    if pos.hand == [0, 0] {
        v += 2 * (pos.mobility(p) as i64 - pos.mobility(o) as i64);
    }
    v
}

/// How many of the best-looking turns the bot looks at more deeply.
const SEARCH_WIDTH: usize = 8;

/// Best whole turn for `p`: the most promising actions, each against the opponent's best reply.
pub fn best_action(pos: &Pos, p: usize, rng: &mut Rng) -> Option<Action> {
    let mut actions = pos.actions(p);
    rng.shuffle(&mut actions);
    // A stable sort keeps the shuffled order among equals.
    actions.sort_by_cached_key(|&a| -eval(&pos.after(p, a), p));
    actions.truncate(SEARCH_WIDTH);
    let o = 1 - p;
    let mut best: Option<(Action, i64)> = None;
    for a in actions {
        let next = pos.after(p, a);
        let value = if next.lost(o) {
            100_000
        } else {
            let mut worst = i64::MAX;
            for b in next.actions(o) {
                let v = -eval(&next.after(o, b), o);
                worst = worst.min(v);
                if best.is_some_and(|(_, bv)| worst <= bv) {
                    break;
                }
            }
            worst
        };
        if best.is_none_or(|(_, bv)| value > bv) {
            best = Some((a, value));
        }
    }
    best.map(|(a, _)| a)
}

#[derive(Clone, Debug)]
pub struct MorrisState {
    players: [String; 2],
    pos: Pos,
    turn: usize,
    /// The point a mill was just made on: the mover must remove a piece.
    pending_remove: bool,
    plies: u32,
    quiet: u32,
    last: Option<(Option<usize>, usize)>,
    last_removed: Option<usize>,
    time_left_ms: i64,
    over: bool,
    winner: Option<usize>,
}

impl MorrisState {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }
    fn end_turn(&mut self, removed: bool) {
        self.pending_remove = false;
        self.plies += 1;
        self.quiet = if removed || self.pos.hand != [0, 0] { 0 } else { self.quiet + 1 };
        self.turn = 1 - self.turn;
        self.time_left_ms = TURN_MS;
        if self.pos.lost(self.turn) {
            self.over = true;
            self.winner = Some(1 - self.turn);
        } else if self.plies >= MAX_PLIES || self.quiet >= QUIET_PLIES {
            self.over = true;
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MorrisMove {
    Place { to: usize },
    Move { from: usize, to: usize },
    Remove { at: usize },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MorrisView {
    /// 1 yours, −1 theirs, 0 empty.
    pub board: Vec<i8>,
    pub hand: u8,
    pub their_hand: u8,
    pub on_board: usize,
    pub their_on_board: usize,
    pub opponent: String,
    pub your_turn: bool,
    /// place | move | fly | remove | wait | over
    pub phase: &'static str,
    /// Your legal steps as [from, to]; from is −1 for a placement.
    pub moves: Vec<[i32; 2]>,
    pub removable: Vec<usize>,
    pub last: Option<[i32; 2]>,
    pub last_removed: Option<usize>,
    pub turn_no: u32,
    pub time_left_ms: i64,
    pub turn_ms: i64,
    pub over: bool,
}

pub struct NineMensMorris;

impl Game for NineMensMorris {
    type State = MorrisState;
    type Move = MorrisMove;
    type View = MorrisView;

    fn info() -> GameInfo {
        GameInfo {
            id: "nine-mens-morris",
            kind: "final-duel",
            min_players: 2,
            max_players: 2,
            name: "Nine Men's Morris",
            short: "The mill game: place, slide and fly your nine pieces; three in a line takes an enemy piece.",
        }
    }

    fn setup(players: &[String], _seed: u64) -> Result<MorrisState, String> {
        let [a, b] = players else { return Err("Nine Men's Morris is for 2 players".into()) };
        Ok(MorrisState {
            players: [a.clone(), b.clone()],
            pos: Pos { board: [EMPTY; 24], hand: [PIECES; 2] },
            turn: 0,
            pending_remove: false,
            plies: 0,
            quiet: 0,
            last: None,
            last_removed: None,
            time_left_ms: TURN_MS,
            over: false,
            winner: None,
        })
    }

    fn apply_move(s: &mut MorrisState, player: &str, mv: MorrisMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let p = s.seat(player).ok_or("Not in this game")?;
        if p != s.turn {
            return Err("Not your turn".into());
        }
        let (from, to) = match mv {
            MorrisMove::Remove { at } => {
                if !s.pending_remove {
                    return Err("No mill to remove with".into());
                }
                if !s.pos.removable(p).contains(&at) {
                    return Err("That piece cannot be removed".into());
                }
                s.pos.board[at] = EMPTY;
                s.last_removed = Some(at);
                s.end_turn(true);
                return Ok(());
            }
            _ if s.pending_remove => return Err("Remove a piece first".into()),
            MorrisMove::Place { to } => (None, to),
            MorrisMove::Move { from, to } => (Some(from), to),
        };
        if !s.pos.steps(p).contains(&(from, to)) {
            return Err("You cannot move there".into());
        }
        s.last = Some((from, to));
        s.last_removed = None;
        if s.pos.step(p, from, to) {
            s.pending_remove = true;
            s.time_left_ms = TURN_MS;
        } else {
            s.end_turn(false);
        }
        Ok(())
    }

    fn tick(s: &mut MorrisState, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            let p = s.players[s.turn].clone();
            let mv = Self::bot_move(s, &p, u64::from(s.plies) + 1);
            if Self::apply_move(s, &p, mv).is_err() {
                // Cannot happen with a legal bot move; never stall.
                s.end_turn(false);
            }
        }
    }

    fn view_for(s: &MorrisState, player: &str) -> MorrisView {
        let me = s.seat(player).unwrap_or(0);
        let your_turn = !s.over && s.turn == me;
        let phase = if s.over {
            "over"
        } else if !your_turn {
            "wait"
        } else if s.pending_remove {
            "remove"
        } else if s.pos.hand[me] > 0 {
            "place"
        } else if s.pos.flying(me) {
            "fly"
        } else {
            "move"
        };
        let moves = if your_turn && !s.pending_remove {
            s.pos.steps(me).into_iter().map(|(f, t)| [f.map_or(-1, |f| f as i32), t as i32]).collect()
        } else {
            Vec::new()
        };
        MorrisView {
            board: s.pos.board.iter().map(|&c| if c == EMPTY { 0 } else if c == me as i8 { 1 } else { -1 }).collect(),
            hand: s.pos.hand[me],
            their_hand: s.pos.hand[1 - me],
            on_board: s.pos.on_board(me),
            their_on_board: s.pos.on_board(1 - me),
            opponent: s.players[1 - me].clone(),
            your_turn,
            phase,
            moves,
            removable: if your_turn && s.pending_remove { s.pos.removable(me) } else { Vec::new() },
            last: s.last.map(|(f, t)| [f.map_or(-1, |f| f as i32), t as i32]),
            last_removed: s.last_removed,
            turn_no: s.plies / 2 + 1,
            time_left_ms: s.time_left_ms.max(0),
            turn_ms: TURN_MS,
            over: s.over,
        }
    }

    fn is_over(s: &MorrisState) -> bool {
        s.over
    }

    fn results(s: &MorrisState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = (0..2)
            .map(|p| {
                let bonus = match s.winner {
                    Some(w) if w == p => 400,
                    Some(_) => 0,
                    None => 150,
                };
                (s.players[p].clone(), 100 + 50 * s.pos.pieces(p) as i64 + bonus)
            })
            .collect();
        rank_by_score(&scores)
    }

    fn to_move(s: &MorrisState) -> Vec<String> {
        if s.over { Vec::new() } else { vec![s.players[s.turn].clone()] }
    }

    fn bot_move(s: &MorrisState, player: &str, seed: u64) -> MorrisMove {
        let mut rng = Rng::new(seed);
        let p = s.seat(player).unwrap_or(0);
        if s.pending_remove {
            // Pick the removal whose position is best after the opponent's reply.
            let o = 1 - p;
            let mut options = s.pos.removable(p);
            rng.shuffle(&mut options);
            let at = options
                .iter()
                .copied()
                .max_by_key(|&r| {
                    let mut next = s.pos;
                    next.board[r] = EMPTY;
                    if next.lost(o) {
                        return 100_000;
                    }
                    next.actions(o).iter().map(|&b| -eval(&next.after(o, b), o)).min().unwrap_or(100_000)
                })
                .unwrap_or(0);
            return MorrisMove::Remove { at };
        }
        match best_action(&s.pos, p, &mut rng) {
            Some(Action { from: None, to, .. }) => MorrisMove::Place { to },
            Some(Action { from: Some(from), to, .. }) => MorrisMove::Move { from, to },
            None => MorrisMove::Place { to: 0 },
        }
    }
}

pub const READY: bool = true;
pub type Rules = NineMensMorris;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn new() -> MorrisState {
        NineMensMorris::setup(&["a".into(), "b".into()], 1).unwrap()
    }

    fn go(s: &mut MorrisState, p: &str, mv: MorrisMove) {
        NineMensMorris::apply_move(s, p, mv).unwrap();
    }

    #[test]
    fn the_board_is_connected_both_ways() {
        let mut edges = 0;
        for i in 0..24 {
            for j in neighbours(i) {
                assert!(neighbours(j).any(|k| k == i), "{i}-{j}");
                edges += 1;
            }
        }
        assert_eq!(edges, 2 * 32);
    }

    #[test]
    fn a_mill_removes_a_piece() {
        let mut s = new();
        go(&mut s, "a", MorrisMove::Place { to: 0 });
        go(&mut s, "b", MorrisMove::Place { to: 8 });
        go(&mut s, "a", MorrisMove::Place { to: 1 });
        go(&mut s, "b", MorrisMove::Place { to: 9 });
        assert!(NineMensMorris::apply_move(&mut s, "b", MorrisMove::Place { to: 2 }).is_err());
        go(&mut s, "a", MorrisMove::Place { to: 2 });
        assert!(s.pending_remove);
        assert_eq!(NineMensMorris::to_move(&s), vec!["a".to_string()]);
        assert!(NineMensMorris::apply_move(&mut s, "a", MorrisMove::Place { to: 5 }).is_err());
        assert!(NineMensMorris::apply_move(&mut s, "a", MorrisMove::Remove { at: 0 }).is_err());
        go(&mut s, "a", MorrisMove::Remove { at: 9 });
        assert_eq!(s.pos.pieces(1), 8);
        assert_eq!(s.turn, 1);
    }

    #[test]
    fn pieces_in_a_mill_are_safe_unless_all_are() {
        let mut pos = Pos { board: [EMPTY; 24], hand: [0, 0] };
        for i in [0, 1, 2, 10] {
            pos.board[i] = 1;
        }
        assert_eq!(pos.removable(0), vec![10]);
        pos.board[10] = EMPTY;
        assert_eq!(pos.removable(0), vec![0, 1, 2]);
    }

    #[test]
    fn three_pieces_fly_and_two_lose() {
        let mut pos = Pos { board: [EMPTY; 24], hand: [0, 0] };
        for i in [0, 9, 20] {
            pos.board[i] = 0;
        }
        for i in [4, 5, 6, 12] {
            pos.board[i] = 1;
        }
        assert_eq!(pos.steps(0).len(), 3 * 17);
        assert!(!pos.lost(0));
        pos.board[20] = EMPTY;
        assert!(pos.lost(0));
    }

    #[test]
    fn the_bot_completes_a_mill_and_blocks_one() {
        let mut pos = Pos { board: [EMPTY; 24], hand: [7, 7] };
        pos.board[0] = 0;
        pos.board[1] = 0;
        pos.board[16] = 1;
        pos.board[17] = 1;
        let a = best_action(&pos, 0, &mut Rng::new(3)).unwrap();
        assert_eq!(a.to, 2);
        assert!(a.remove.is_some());
        pos.board[0] = EMPTY;
        pos.board[1] = EMPTY;
        pos.board[8] = 0;
        pos.board[12] = 0;
        let a = best_action(&pos, 0, &mut Rng::new(3)).unwrap();
        assert_eq!(a.to, 18, "blocks the open two");
    }

    #[test]
    fn time_up_plays_for_you() {
        let mut s = new();
        NineMensMorris::tick(&mut s, TURN_MS as u64);
        assert_eq!(s.turn, 1);
        assert_eq!(s.pos.hand[0], PIECES - 1);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<NineMensMorris>(1000, 2, 2000);
    }
}

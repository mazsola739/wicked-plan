//! Polyomino Territory (Main game, 2–4 players). Rules page: docs/games/polyomino-territory.md
//!
//! Grow your territory on a 14×14 board with polyomino pieces that must touch
//! it edge to edge. Key squares give an encore (place again) or a leap (one
//! piece anywhere). Most squares wins.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const SIZE: i32 = 14;
pub const HAND: usize = 3;
pub const MAX_TURNS_EACH: u32 = 40;
pub const POINTS_PER_SQUARE: i64 = 10;

pub type Cell = (i32, i32);
/// A placement: hand index, the squares covered, whether it uses a leap.
pub type Choice = (usize, Vec<Cell>, bool);

/// The 17 piece shapes, as cells of their base orientation.
pub const SHAPES: [&[Cell]; 17] = [
    &[(0, 0)],
    &[(0, 0), (1, 0)],
    &[(0, 0), (1, 0), (2, 0)],
    &[(0, 0), (0, 1), (1, 1)],
    &[(0, 0), (1, 0), (2, 0), (3, 0)],
    &[(0, 0), (1, 0), (0, 1), (1, 1)],
    &[(0, 0), (1, 0), (2, 0), (1, 1)],
    &[(0, 0), (0, 1), (0, 2), (1, 2)],
    &[(1, 0), (2, 0), (0, 1), (1, 1)],
    &[(0, 0), (1, 0), (0, 1), (1, 1), (0, 2)],
    &[(0, 0), (0, 1), (0, 2), (0, 3), (1, 3)],
    &[(0, 0), (1, 0), (2, 0), (1, 1), (1, 2)],
    &[(0, 0), (2, 0), (0, 1), (1, 1), (2, 1)],
    &[(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)],
    &[(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)],
    &[(0, 0), (1, 0), (1, 1), (1, 2), (2, 2)],
    &[(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum KeyKind {
    Encore,
    Leap,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    pub x: i32,
    pub y: i32,
    pub kind: KeyKind,
}

/// Cells shifted so the smallest x and y are 0, sorted.
pub fn normalize(cells: &[Cell]) -> Vec<Cell> {
    let mx = cells.iter().map(|c| c.0).min().unwrap_or(0);
    let my = cells.iter().map(|c| c.1).min().unwrap_or(0);
    let mut out: Vec<Cell> = cells.iter().map(|&(x, y)| (x - mx, y - my)).collect();
    out.sort_by_key(|&(x, y)| (y, x));
    out
}

/// Orientation 0–7: turned a quarter `k % 4` times, then mirrored when `k >= 4`.
pub fn orient(cells: &[Cell], k: usize) -> Vec<Cell> {
    let turned: Vec<Cell> = cells
        .iter()
        .map(|&(x, y)| {
            let (mut x, mut y) = (x, y);
            for _ in 0..k % 4 {
                (x, y) = (-y, x);
            }
            if k >= 4 { (-x, y) } else { (x, y) }
        })
        .collect();
    normalize(&turned)
}

/// The different orientations of a shape.
pub fn orientations(cells: &[Cell]) -> Vec<Vec<Cell>> {
    let mut out: Vec<Vec<Cell>> = Vec::new();
    for k in 0..8 {
        let o = orient(cells, k);
        if !out.contains(&o) {
            out.push(o);
        }
    }
    out
}

static ORIENTATIONS: std::sync::OnceLock<Vec<Vec<Vec<Cell>>>> = std::sync::OnceLock::new();

/// The orientations of every shape, worked out once.
fn all_orientations() -> &'static Vec<Vec<Vec<Cell>>> {
    ORIENTATIONS.get_or_init(|| SHAPES.iter().map(|s| orientations(s)).collect())
}

fn idx(x: i32, y: i32) -> usize {
    (y * SIZE + x) as usize
}

fn inside(x: i32, y: i32) -> bool {
    (0..SIZE).contains(&x) && (0..SIZE).contains(&y)
}

const DIRS: [Cell; 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub seat: usize,
    pub cells: Vec<Cell>,
    pub leap: bool,
    pub keys: Vec<KeyKind>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub players: Vec<String>,
    /// -1 empty, else the owner's seat.
    pub board: Vec<i8>,
    pub keys: Vec<Key>,
    pub hands: Vec<Vec<usize>>,
    pub leaps: Vec<u32>,
    pub turn: usize,
    pub encores: u32,
    /// The player to move is on an encore.
    pub on_encore: bool,
    pub turns: Vec<u32>,
    pub last: Option<Placement>,
    pub over: bool,
    /// Replay moments: a leap that lands on a key square.
    pub moments: Moments,
    deck: Rng,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Place {
        piece: usize,
        cells: Vec<Cell>,
        #[serde(default)]
        leap: bool,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: Vec<String>,
    pub you: Option<usize>,
    pub size: i32,
    pub board: Vec<i8>,
    pub keys: Vec<Key>,
    /// Your pieces, each in its base orientation.
    pub hand: Vec<Vec<Cell>>,
    pub leaps: Vec<u32>,
    pub squares: Vec<i64>,
    pub turn: usize,
    pub your_turn: bool,
    pub encore: bool,
    pub turns_left: u32,
    pub last: Option<Placement>,
    pub over: bool,
}

impl State {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }

    pub fn squares(&self, seat: usize) -> i64 {
        self.board.iter().filter(|&&b| b == seat as i8).count() as i64
    }

    fn touches(&self, seat: usize, cells: &[Cell]) -> bool {
        cells.iter().any(|&(x, y)| DIRS.iter().any(|&(dx, dy)| inside(x + dx, y + dy) && self.board[idx(x + dx, y + dy)] == seat as i8))
    }

    /// Why `cells` cannot be placed for `seat` with piece `piece`, if not.
    fn check(&self, seat: usize, piece: usize, cells: &[Cell], leap: bool) -> Result<(), String> {
        let shape = *self.hands[seat].get(piece).ok_or("No such piece")?;
        if !orientations(SHAPES[shape]).contains(&normalize(cells)) {
            return Err("That is not the shape of the piece".into());
        }
        if cells.iter().any(|&(x, y)| !inside(x, y) || self.board[idx(x, y)] != -1) {
            return Err("The piece must go on empty squares".into());
        }
        if leap {
            if self.leaps[seat] == 0 {
                return Err("You have no leap".into());
            }
        } else if !self.touches(seat, cells) {
            return Err("The piece must touch your territory".into());
        }
        Ok(())
    }

    /// Every legal placement for `seat`: (hand index, cells, leap).
    pub fn placements(&self, seat: usize, with_leaps: bool) -> Vec<Choice> {
        let mut out = Vec::new();
        self.scan(seat, with_leaps, &mut |h, cells, leap| {
            out.push((h, cells.to_vec(), leap));
            false
        });
        out
    }

    /// Calls `found` for every legal placement until it returns true.
    fn scan(&self, seat: usize, with_leaps: bool, found: &mut dyn FnMut(usize, &[Cell], bool) -> bool) {
        // Empty squares next to the territory: a piece touches it when it covers one of them.
        let mut near = vec![false; self.board.len()];
        for y in 0..SIZE {
            for x in 0..SIZE {
                if self.board[idx(x, y)] == seat as i8 {
                    for (dx, dy) in DIRS {
                        if inside(x + dx, y + dy) && self.board[idx(x + dx, y + dy)] == -1 {
                            near[idx(x + dx, y + dy)] = true;
                        }
                    }
                }
            }
        }
        let leaps = with_leaps && self.leaps[seat] > 0;
        let mut cells: Vec<Cell> = Vec::with_capacity(5);
        for (h, &shape) in self.hands[seat].iter().enumerate() {
            if self.hands[seat][..h].contains(&shape) {
                continue;
            }
            for o in &all_orientations()[shape] {
                let w = o.iter().map(|c| c.0).max().unwrap_or(0);
                let ht = o.iter().map(|c| c.1).max().unwrap_or(0);
                for y in 0..SIZE - ht {
                    for x in 0..SIZE - w {
                        if o.iter().any(|&(cx, cy)| self.board[idx(cx + x, cy + y)] != -1) {
                            continue;
                        }
                        let touching = o.iter().any(|&(cx, cy)| near[idx(cx + x, cy + y)]);
                        if !touching && !leaps {
                            continue;
                        }
                        cells.clear();
                        cells.extend(o.iter().map(|&(cx, cy)| (cx + x, cy + y)));
                        if found(h, &cells, !touching) {
                            return;
                        }
                    }
                }
            }
        }
    }

    fn can_move(&self, seat: usize) -> bool {
        let mut any = false;
        self.scan(seat, true, &mut |_, _, _| {
            any = true;
            true
        });
        any
    }

    fn place(&mut self, seat: usize, piece: usize, cells: Vec<Cell>, leap: bool) {
        for &(x, y) in &cells {
            self.board[idx(x, y)] = seat as i8;
        }
        if leap {
            self.leaps[seat] -= 1;
        }
        let mut got = Vec::new();
        self.keys.retain(|k| {
            let hit = cells.contains(&(k.x, k.y));
            if hit {
                got.push(k.kind);
            }
            !hit
        });
        for k in &got {
            match k {
                KeyKind::Encore => self.encores += 1,
                KeyKind::Leap => self.leaps[seat] += 1,
            }
        }
        // Replay moment (the board, the key squares and the leaps are public): a leap that lands on a key square.
        if leap && !got.is_empty() {
            let text = match (got.contains(&KeyKind::Encore), got.contains(&KeyKind::Leap)) {
                (true, true) => "Leap grabs two key squares",
                (true, false) => "Leap grabs an encore square",
                _ => "Leap grabs another leap",
            };
            self.moments.push(&self.players[seat].clone(), text);
        }
        self.hands[seat].remove(piece);
        let next = self.deck.index(SHAPES.len());
        self.hands[seat].push(next);
        self.turns[seat] += 1;
        self.last = Some(Placement { seat, cells, leap, keys: got });
        self.on_encore = self.encores > 0 && self.turns[seat] < MAX_TURNS_EACH;
        if self.on_encore {
            self.encores -= 1;
        } else {
            self.encores = 0;
            self.turn = (self.turn + 1) % self.players.len();
        }
        self.settle();
    }

    /// Skips players who cannot move; ends the game when nobody can.
    fn settle(&mut self) {
        for _ in 0..self.players.len() {
            if self.turns[self.turn] < MAX_TURNS_EACH && self.can_move(self.turn) {
                return;
            }
            self.encores = 0;
            self.on_encore = false;
            self.turn = (self.turn + 1) % self.players.len();
        }
        self.over = true;
    }

    /// The bot's placement for `seat`, if it has one.
    pub fn bot_choice(&self, seat: usize, seed: u64) -> Option<Choice> {
        let mut rng = Rng::new(seed);
        let me = seat as i8;
        let center = (SIZE - 1) as f64 / 2.0;
        let mut best: Option<(f64, Choice)> = None;
        for p in self.placements(seat, true) {
            let cells = &p.1;
            let mut v = cells.len() as f64 * 10.0;
            for k in &self.keys {
                if cells.contains(&(k.x, k.y)) {
                    v += if k.kind == KeyKind::Encore { 35.0 } else { 25.0 };
                }
            }
            for &(x, y) in cells {
                for (dx, dy) in DIRS {
                    let (nx, ny) = (x + dx, y + dy);
                    if !inside(nx, ny) || cells.contains(&(nx, ny)) {
                        continue;
                    }
                    let b = self.board[idx(nx, ny)];
                    if b == -1 {
                        v += 1.5;
                    } else if b != me {
                        v += 2.5;
                    }
                }
                v -= ((x as f64 - center).abs() + (y as f64 - center).abs()) * 0.15;
            }
            if p.2 {
                // A leap is worth keeping unless this jump is clearly better.
                v -= 30.0;
            }
            v += rng.next_f64() * 4.0;
            if best.as_ref().is_none_or(|(b, _)| v > *b) {
                best = Some((v, p));
            }
        }
        best.map(|(_, p)| p)
    }
}

pub struct PolyominoTerritory;

impl Game for PolyominoTerritory {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "polyomino-territory",
            kind: "main",
            difficulty: "hard",
            tags: &["board"],
            min_players: 2,
            max_players: 4,
            name: "Polyomino Territory",
            short: "Grow your territory with polyomino pieces; key squares give encores and leaps.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        let n = players.len();
        if !(2..=4).contains(&n) {
            return Err("Polyomino Territory needs 2–4 players".into());
        }
        let mut rng = Rng::new(seed);
        let mut board = vec![-1i8; (SIZE * SIZE) as usize];
        let m = SIZE - 1;
        let starts: Vec<Cell> = match n {
            2 => vec![(0, 0), (m, m)],
            3 => vec![(0, 0), (m, 0), (SIZE / 2, m)],
            _ => vec![(0, 0), (m, 0), (m, m), (0, m)],
        };
        for (k, &(x, y)) in starts.iter().enumerate() {
            board[idx(x, y)] = k as i8;
        }
        // One encore and one leap in the top-left quarter, mirrored into the others.
        let half = SIZE / 2;
        let mut quarter: Vec<Cell> = Vec::new();
        while quarter.len() < 2 {
            let c = (rng.int(2, i64::from(half - 2)) as i32, rng.int(2, i64::from(half - 2)) as i32);
            if !quarter.contains(&c) {
                quarter.push(c);
            }
        }
        let mut keys = Vec::new();
        for (i, &(x, y)) in quarter.iter().enumerate() {
            let kind = if i == 0 { KeyKind::Encore } else { KeyKind::Leap };
            for (kx, ky) in [(x, y), (m - x, y), (x, m - y), (m - x, m - y)] {
                keys.push(Key { x: kx, y: ky, kind });
            }
        }
        let mut deck = Rng::new(seed ^ 0xdec4);
        let hands = (0..n).map(|_| (0..HAND).map(|_| deck.index(SHAPES.len())).collect()).collect();
        Ok(State {
            players: players.to_vec(),
            board,
            keys,
            hands,
            leaps: vec![0; n],
            turn: 0,
            encores: 0,
            on_encore: false,
            turns: vec![0; n],
            last: None,
            over: false,
            moments: Moments::default(),
            deck,
        })
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.seat(player).ok_or("Not in this game")?;
        if seat != s.turn {
            return Err("Not your turn".into());
        }
        let Move::Place { piece, cells, leap } = mv;
        s.check(seat, piece, &cells, leap)?;
        s.place(seat, piece, cells, leap);
        Ok(())
    }

    fn view_for(s: &State, player: &str) -> View {
        let you = s.seat(player);
        View {
            players: s.players.clone(),
            you,
            size: SIZE,
            board: s.board.clone(),
            keys: s.keys.clone(),
            hand: you.map(|k| s.hands[k].iter().map(|&h| SHAPES[h].to_vec()).collect()).unwrap_or_default(),
            leaps: s.leaps.clone(),
            squares: (0..s.players.len()).map(|k| s.squares(k)).collect(),
            turn: s.turn,
            your_turn: !s.over && you == Some(s.turn),
            encore: s.on_encore,
            turns_left: you.map_or(0, |k| MAX_TURNS_EACH.saturating_sub(s.turns[k])),
            last: s.last.clone(),
            over: s.over,
        }
    }

    fn is_over(s: &State) -> bool {
        s.over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.players.iter().enumerate().map(|(k, p)| (p.clone(), s.squares(k) * POINTS_PER_SQUARE)).collect();
        rank_by_score(&scores)
    }

    fn to_move(s: &State) -> Vec<String> {
        if s.over { Vec::new() } else { vec![s.players[s.turn].clone()] }
    }

    fn bot_move(s: &State, player: &str, seed: u64) -> Move {
        let seat = s.seat(player).unwrap_or(0);
        match s.bot_choice(seat, seed) {
            Some((piece, cells, leap)) => Move::Place { piece, cells, leap },
            None => Move::Place { piece: 0, cells: Vec::new(), leap: false },
        }
    }

    fn moments(s: &State, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub type Rules = PolyominoTerritory;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn shapes_and_orientations() {
        assert_eq!(orientations(SHAPES[0]).len(), 1);
        assert_eq!(orientations(SHAPES[5]).len(), 1, "the square");
        assert_eq!(orientations(SHAPES[13]).len(), 1, "the plus");
        assert_eq!(orientations(SHAPES[9]).len(), 8, "the P");
        for s in SHAPES {
            for o in orientations(s) {
                assert_eq!(o.len(), s.len());
            }
        }
        // No two shapes are the same piece.
        for (i, a) in SHAPES.iter().enumerate() {
            for (j, b) in SHAPES.iter().enumerate().take(i) {
                assert!(!orientations(a).contains(&normalize(b)), "{i} and {j}");
            }
        }
    }

    #[test]
    fn pieces_must_touch_and_fit() {
        let mut s = PolyominoTerritory::setup(&players(2), 1).unwrap();
        s.hands[0] = vec![1, 0, 2];
        // A domino next to the corner start, turned upright.
        assert!(PolyominoTerritory::apply_move(&mut s, "p1", Move::Place { piece: 0, cells: vec![(5, 5), (5, 6)], leap: false }).is_err());
        assert!(PolyominoTerritory::apply_move(&mut s, "p1", Move::Place { piece: 0, cells: vec![(0, 0), (0, 1)], leap: false }).is_err());
        assert!(PolyominoTerritory::apply_move(&mut s, "p1", Move::Place { piece: 0, cells: vec![(1, 0), (2, 1)], leap: false }).is_err());
        PolyominoTerritory::apply_move(&mut s, "p1", Move::Place { piece: 0, cells: vec![(0, 1), (0, 2)], leap: false }).unwrap();
        assert_eq!(s.squares(0), 3);
        assert_eq!(s.hands[0].len(), HAND);
        assert_eq!(s.turn, 1);
    }

    #[test]
    fn key_squares_give_an_encore_and_a_leap() {
        let mut s = PolyominoTerritory::setup(&players(2), 2).unwrap();
        let encore = s.keys.iter().find(|k| k.kind == KeyKind::Encore).unwrap().clone();
        let leap = s.keys.iter().find(|k| k.kind == KeyKind::Leap).unwrap().clone();
        // Give p1 territory next to both keys.
        s.board[idx(encore.x - 1, encore.y)] = 0;
        s.board[idx(leap.x - 1, leap.y)] = 0;
        s.hands[0] = vec![0, 0, 0];
        PolyominoTerritory::apply_move(&mut s, "p1", Move::Place { piece: 0, cells: vec![(encore.x, encore.y)], leap: false }).unwrap();
        assert_eq!(s.turn, 0, "encore");
        PolyominoTerritory::apply_move(&mut s, "p1", Move::Place { piece: 0, cells: vec![(leap.x, leap.y)], leap: false }).unwrap();
        assert_eq!(s.leaps[0], 1);
        assert_eq!(s.turn, 1);
        s.turn = 0;
        s.hands[0] = vec![0, 0, 0];
        let far = (SIZE / 2, 0);
        assert!(PolyominoTerritory::apply_move(&mut s, "p1", Move::Place { piece: 0, cells: vec![far], leap: false }).is_err());
        PolyominoTerritory::apply_move(&mut s, "p1", Move::Place { piece: 0, cells: vec![far], leap: true }).unwrap();
        assert_eq!(s.leaps[0], 0);
    }

    #[test]
    fn stuck_players_are_skipped_and_the_game_ends() {
        let mut s = PolyominoTerritory::setup(&players(2), 3).unwrap();
        // Fill the board except one square next to p2's corner.
        for i in 0..s.board.len() {
            s.board[i] = 1;
        }
        s.board[idx(0, 0)] = 0;
        s.board[idx(SIZE - 1, SIZE - 2)] = -1;
        s.hands[1] = vec![0, 1, 2];
        s.settle();
        assert_eq!(s.turn, 1);
        PolyominoTerritory::apply_move(&mut s, "p2", Move::Place { piece: 0, cells: vec![(SIZE - 1, SIZE - 2)], leap: false }).unwrap();
        assert!(s.over);
        let r = PolyominoTerritory::results(&s);
        assert_eq!(r[1].rank, 1);
        assert_eq!(r[1].points, (SIZE * SIZE - 1) as i64 * POINTS_PER_SQUARE);
    }

    #[test]
    fn a_turn_waits_for_its_player() {
        let mut s = PolyominoTerritory::setup(&players(2), 5).unwrap();
        PolyominoTerritory::tick(&mut s, 600_000);
        assert_eq!(s.turns, vec![0, 0]);
        assert_eq!(PolyominoTerritory::to_move(&s), vec!["p1".to_string()]);
    }

    #[test]
    fn hands_are_private() {
        let s = PolyominoTerritory::setup(&players(3), 4).unwrap();
        let v = PolyominoTerritory::view_for(&s, "p2");
        assert_eq!(v.hand.len(), HAND);
        assert_eq!(v.hand[0], SHAPES[s.hands[1][0]].to_vec());
    }

    #[test]
    fn moves_arrive_as_json() {
        let mv: Move = serde_json::from_value(serde_json::json!({"type": "place", "piece": 1, "cells": [[0, 1], [0, 2]]})).unwrap();
        assert!(matches!(mv, Move::Place { piece: 1, leap: false, .. }));
    }

    #[test]
    fn bot_games() {
        // Every bot move searches all placements, so fewer games than usual keep the test quick.
        for seats in 2..=4 {
            bot_games_with::<PolyominoTerritory>(150, seats, 5000);
        }
    }

    #[test]
    fn a_leap_onto_a_key_square_is_a_replay_moment() {
        let mut s = PolyominoTerritory::setup(&players(2), 2).unwrap();
        let encore = s.keys.iter().find(|k| k.kind == KeyKind::Encore).unwrap().clone();
        s.hands[0] = vec![0, 0, 0];
        // A plain placement next to the start is no moment.
        PolyominoTerritory::apply_move(&mut s, "p1", Move::Place { piece: 0, cells: vec![(1, 0)], leap: false }).unwrap();
        assert!(PolyominoTerritory::moments(&s, "p2").is_empty());
        s.turn = 0;
        s.leaps[0] = 1;
        PolyominoTerritory::apply_move(&mut s, "p1", Move::Place { piece: 0, cells: vec![(encore.x, encore.y)], leap: true }).unwrap();
        let m = PolyominoTerritory::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].by, "p1");
        assert_eq!(m[0].text, "Leap grabs an encore square");
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<PolyominoTerritory>(150, 4, 5000);
        assert!(stats.with_any * 3 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

//! Spinning Puzzle Wheel (Team mini game, 1–4 players). Rules page: docs/games/spinning-puzzle-wheel.md
//!
//! Six silhouettes on a wheel, each with its own tray of pieces. The wheel
//! turns from player to player; each gets a few seconds to place (or take back)
//! one piece on the puzzle in front of them. The team finishes the shapes.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult};
use crate::games::polyomino_territory::{Cell, SHAPES, normalize, orientations};
use crate::rng::Rng;

pub const GRID: i32 = 5;
pub const PUZZLES: usize = 6;
pub const PIECES_PER_PUZZLE: usize = 3;
pub const MAX_TURNS: u32 = 36;
pub const TURN_MS: i64 = 12_000;
pub const PUZZLE_POINTS: i64 = 100;
pub const SPARE_TURN_POINTS: i64 = 10;
pub const BOT_SLOW: f64 = 0.1;
/// Shapes of 3–5 squares (indices into `SHAPES`).
const FIRST_SHAPE: usize = 2;

#[derive(Clone, Debug)]
pub struct Puzzle {
    pub target: Vec<Cell>,
    pub pieces: Vec<usize>,
    pub placed: Vec<Option<Vec<Cell>>>,
}

impl Puzzle {
    pub fn covered(&self) -> usize {
        self.placed.iter().flatten().map(Vec::len).sum()
    }

    pub fn done(&self) -> bool {
        self.covered() == self.target.len()
    }

    /// Target squares not covered yet, as a 5×5 mask.
    fn open(&self) -> [bool; 25] {
        let mut m = [false; 25];
        for &(x, y) in &self.target {
            m[(y * GRID + x) as usize] = true;
        }
        for &(x, y) in self.placed.iter().flatten().flatten() {
            m[(y * GRID + x) as usize] = false;
        }
        m
    }

    fn fits(&self, cells: &[Cell]) -> bool {
        let open = self.open();
        cells.iter().all(|&(x, y)| (0..GRID).contains(&x) && (0..GRID).contains(&y) && open[(y * GRID + x) as usize])
    }

    /// A way to cover the open squares with unplaced tray pieces, if there is one.
    pub fn solve(&self) -> Option<Vec<(usize, Vec<Cell>)>> {
        let free: Vec<usize> = (0..self.pieces.len()).filter(|&i| self.placed[i].is_none()).collect();
        let mut out = Vec::new();
        solve_rec(&mut self.open(), &self.pieces, &free, &mut out).then_some(out)
    }
}

fn solve_rec(open: &mut [bool; 25], shapes: &[usize], free: &[usize], out: &mut Vec<(usize, Vec<Cell>)>) -> bool {
    let Some(first) = open.iter().position(|&o| o) else { return true };
    let (fx, fy) = (first as i32 % GRID, first as i32 / GRID);
    for (k, &p) in free.iter().enumerate() {
        if free[..k].iter().any(|&q| shapes[q] == shapes[p]) {
            continue;
        }
        for o in orientations(SHAPES[shapes[p]]) {
            // The piece covering the first open square has its own first square there.
            let (dx, dy) = (fx - o[0].0, fy - o[0].1);
            let cells: Vec<Cell> = o.iter().map(|&(x, y)| (x + dx, y + dy)).collect();
            if !cells.iter().all(|&(x, y)| (0..GRID).contains(&x) && (0..GRID).contains(&y) && open[(y * GRID + x) as usize]) {
                continue;
            }
            for &(x, y) in &cells {
                open[(y * GRID + x) as usize] = false;
            }
            let rest: Vec<usize> = free.iter().copied().filter(|&q| q != p).collect();
            out.push((p, cells.clone()));
            if solve_rec(open, shapes, &rest, out) {
                return true;
            }
            out.pop();
            for &(x, y) in &cells {
                open[(y * GRID + x) as usize] = true;
            }
        }
    }
    false
}

fn touches(a: &[Cell], b: &[Cell]) -> bool {
    a.iter().any(|&(x, y)| b.iter().any(|&(u, v)| (x - u).abs() + (y - v).abs() == 1))
}

/// A silhouette glued from three random pieces, centred on the grid, plus a decoy in the tray.
pub fn generate(rng: &mut Rng) -> Puzzle {
    'again: loop {
        let mut shapes: Vec<usize> = (0..PIECES_PER_PUZZLE).map(|_| rng.int(FIRST_SHAPE as i64, SHAPES.len() as i64 - 1) as usize).collect();
        let mut occ: Vec<Cell> = Vec::new();
        for (i, &sh) in shapes.iter().enumerate() {
            let os = orientations(SHAPES[sh]);
            let mut done = false;
            for _ in 0..200 {
                let o = rng.pick(&os);
                let w = o.iter().map(|c| c.0).max().unwrap_or(0) + 1;
                let h = o.iter().map(|c| c.1).max().unwrap_or(0) + 1;
                if w > GRID || h > GRID {
                    continue;
                }
                let (x, y) = (rng.int(0, i64::from(GRID - w)) as i32, rng.int(0, i64::from(GRID - h)) as i32);
                let cells: Vec<Cell> = o.iter().map(|&(cx, cy)| (cx + x, cy + y)).collect();
                if cells.iter().any(|c| occ.contains(c)) || (i > 0 && !touches(&cells, &occ)) {
                    continue;
                }
                occ.extend(cells);
                done = true;
                break;
            }
            if !done {
                continue 'again;
            }
        }
        let norm = normalize(&occ);
        let w = norm.iter().map(|c| c.0).max().unwrap_or(0) + 1;
        let h = norm.iter().map(|c| c.1).max().unwrap_or(0) + 1;
        let target = norm.iter().map(|&(x, y)| (x + (GRID - w) / 2, y + (GRID - h) / 2)).collect();
        shapes.push(rng.int(FIRST_SHAPE as i64, SHAPES.len() as i64 - 1) as usize);
        rng.shuffle(&mut shapes);
        let n = shapes.len();
        return Puzzle { target, pieces: shapes, placed: vec![None; n] };
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub players: Vec<String>,
    pub puzzles: Vec<Puzzle>,
    pub current: usize,
    pub seat: usize,
    pub turns: u32,
    pub time_left_ms: i64,
    pub last: Option<String>,
    pub over: bool,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Place { piece: usize, cells: Vec<Cell> },
    Remove { piece: usize },
    Pass,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleView {
    pub target: Vec<Cell>,
    /// Each tray piece in its base orientation.
    pub pieces: Vec<Vec<Cell>>,
    pub placed: Vec<Option<Vec<Cell>>>,
    pub done: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: Vec<String>,
    pub you: Option<usize>,
    pub grid: i32,
    pub puzzles: Vec<PuzzleView>,
    pub current: usize,
    pub turn: usize,
    pub your_turn: bool,
    pub turns_left: u32,
    pub done: usize,
    pub time_left_ms: i64,
    pub turn_ms: i64,
    pub last: Option<String>,
    pub score: i64,
    pub over: bool,
}

impl State {
    fn seat_of(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }

    pub fn done(&self) -> usize {
        self.puzzles.iter().filter(|p| p.done()).count()
    }

    pub fn score(&self) -> i64 {
        let done = self.done();
        let mut pts = done as i64 * PUZZLE_POINTS;
        if done == PUZZLES {
            pts += i64::from(MAX_TURNS - self.turns) * SPARE_TURN_POINTS;
        }
        pts
    }

    fn spin(&mut self, what: String) {
        self.last = Some(format!("{}: {what}", self.players[self.seat]));
        self.turns += 1;
        if self.done() == PUZZLES || self.turns >= MAX_TURNS {
            self.over = true;
            self.time_left_ms = 0;
            return;
        }
        self.seat = (self.seat + 1) % self.players.len();
        self.current = (1..=PUZZLES).map(|d| (self.current + d) % PUZZLES).find(|&i| !self.puzzles[i].done()).unwrap_or(0);
        self.time_left_ms = TURN_MS;
    }
}

pub struct SpinningPuzzleWheel;

impl Game for SpinningPuzzleWheel {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "spinning-puzzle-wheel",
            kind: "mini-team",
            difficulty: "medium",
            min_players: 1,
            max_players: 4,
            name: "Spinning Puzzle Wheel",
            short: "The wheel turns from player to player: a few seconds each to place one piece. Finish the shapes together.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if !(1..=4).contains(&players.len()) {
            return Err("Spinning Puzzle Wheel is for 1–4 players".into());
        }
        let mut rng = Rng::new(seed);
        Ok(State {
            players: players.to_vec(),
            puzzles: (0..PUZZLES).map(|_| generate(&mut rng)).collect(),
            current: 0,
            seat: 0,
            turns: 0,
            time_left_ms: TURN_MS,
            last: None,
            over: false,
        })
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.seat_of(player).ok_or("Not in this game")?;
        if seat != s.seat {
            return Err("Not your turn".into());
        }
        let p = &mut s.puzzles[s.current];
        match mv {
            Move::Place { piece, cells } => {
                let shape = *p.pieces.get(piece).ok_or("No such piece")?;
                if p.placed[piece].is_some() {
                    return Err("That piece is already placed".into());
                }
                if !orientations(SHAPES[shape]).contains(&normalize(&cells)) {
                    return Err("That is not the shape of the piece".into());
                }
                if !p.fits(&cells) {
                    return Err("The piece must go on empty squares of the shape".into());
                }
                p.placed[piece] = Some(cells);
                s.spin("placed a piece".into());
            }
            Move::Remove { piece } => {
                if p.placed.get(piece).is_none_or(Option::is_none) {
                    return Err("That piece is not placed".into());
                }
                p.placed[piece] = None;
                s.spin("took a piece back".into());
            }
            Move::Pass => s.spin("passed".into()),
        }
        Ok(())
    }

    fn tick(s: &mut State, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.spin("ran out of time".into());
        }
    }

    fn view_for(s: &State, player: &str) -> View {
        let you = s.seat_of(player);
        View {
            players: s.players.clone(),
            you,
            grid: GRID,
            puzzles: s
                .puzzles
                .iter()
                .map(|p| PuzzleView {
                    target: p.target.clone(),
                    pieces: p.pieces.iter().map(|&sh| SHAPES[sh].to_vec()).collect(),
                    placed: p.placed.clone(),
                    done: p.done(),
                })
                .collect(),
            current: s.current,
            turn: s.seat,
            your_turn: !s.over && you == Some(s.seat),
            turns_left: MAX_TURNS - s.turns,
            done: s.done(),
            time_left_ms: s.time_left_ms,
            turn_ms: TURN_MS,
            last: s.last.clone(),
            score: s.score(),
            over: s.over,
        }
    }

    fn is_over(s: &State) -> bool {
        s.over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        let pts = s.score();
        s.players.iter().map(|p| PlayerResult { player_id: p.clone(), rank: 1, points: pts }).collect()
    }

    fn to_move(s: &State) -> Vec<String> {
        if s.over { Vec::new() } else { vec![s.players[s.seat].clone()] }
    }

    fn bot_move(s: &State, _player: &str, seed: u64) -> Move {
        let mut rng = Rng::new(seed);
        if s.over || rng.next_f64() < BOT_SLOW {
            return Move::Pass;
        }
        let p = &s.puzzles[s.current];
        match p.solve() {
            Some(sol) if !sol.is_empty() => {
                let (piece, cells) = sol[0].clone();
                Move::Place { piece, cells }
            }
            _ => match (0..p.placed.len()).rev().find(|&i| p.placed[i].is_some()) {
                Some(piece) => Move::Remove { piece },
                None => Move::Pass,
            },
        }
    }
}

pub const READY: bool = true;
pub type Rules = SpinningPuzzleWheel;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn team(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn generated_puzzles_are_solvable_and_fit_the_grid() {
        let mut rng = Rng::new(5);
        for _ in 0..500 {
            let p = generate(&mut rng);
            assert_eq!(p.pieces.len(), PIECES_PER_PUZZLE + 1);
            assert!(p.target.iter().all(|&(x, y)| (0..GRID).contains(&x) && (0..GRID).contains(&y)));
            let sol = p.solve().expect("solvable");
            assert_eq!(sol.iter().map(|(_, c)| c.len()).sum::<usize>(), p.target.len());
        }
    }

    #[test]
    fn placing_taking_back_and_turns() {
        let mut s = SpinningPuzzleWheel::setup(&team(2), 1).unwrap();
        let (piece, cells) = s.puzzles[0].solve().unwrap()[0].clone();
        assert!(SpinningPuzzleWheel::apply_move(&mut s, "p2", Move::Pass).is_err());
        let shifted: Vec<Cell> = cells.iter().map(|&(x, y)| (x + 10, y)).collect();
        assert!(SpinningPuzzleWheel::apply_move(&mut s, "p1", Move::Place { piece, cells: shifted }).is_err());
        SpinningPuzzleWheel::apply_move(&mut s, "p1", Move::Place { piece, cells: cells.clone() }).unwrap();
        assert_eq!((s.seat, s.current), (1, 1));
        SpinningPuzzleWheel::apply_move(&mut s, "p2", Move::Pass).unwrap();
        // The wheel comes back round to puzzle 0 after five more turns.
        for _ in 0..4 {
            let p = s.players[s.seat].clone();
            SpinningPuzzleWheel::apply_move(&mut s, &p, Move::Pass).unwrap();
        }
        assert_eq!(s.current, 0);
        assert!(SpinningPuzzleWheel::apply_move(&mut s, "p1", Move::Place { piece, cells }).is_err(), "already placed");
        SpinningPuzzleWheel::apply_move(&mut s, "p1", Move::Remove { piece }).unwrap();
        assert!(s.puzzles[0].placed[piece].is_none());
        SpinningPuzzleWheel::tick(&mut s, TURN_MS as u64);
        assert_eq!(s.turns, 8);
    }

    #[test]
    fn finished_puzzles_are_skipped_and_all_six_score_a_bonus() {
        let mut s = SpinningPuzzleWheel::setup(&team(1), 2).unwrap();
        let mut steps = 0;
        while !s.over {
            let sol = s.puzzles[s.current].solve().unwrap();
            let (piece, cells) = sol[0].clone();
            SpinningPuzzleWheel::apply_move(&mut s, "p1", Move::Place { piece, cells }).unwrap();
            steps += 1;
        }
        assert_eq!(s.done(), PUZZLES);
        assert_eq!(s.score(), PUZZLES as i64 * PUZZLE_POINTS + i64::from(MAX_TURNS - steps) * SPARE_TURN_POINTS);
    }

    #[test]
    fn bots_finish_most_puzzles() {
        let mut total = 0;
        for seed in 0..30 {
            let mut s = SpinningPuzzleWheel::setup(&team(4), seed).unwrap();
            while !s.over {
                let p = s.players[s.seat].clone();
                let mv = SpinningPuzzleWheel::bot_move(&s, &p, seed * 100 + u64::from(s.turns));
                SpinningPuzzleWheel::apply_move(&mut s, &p, mv).unwrap();
            }
            total += s.done();
        }
        assert!(total >= 30 * 5, "finished {total} of 180");
    }

    #[test]
    fn thousand_bot_games() {
        for seats in 1..=4 {
            bot_games_with::<SpinningPuzzleWheel>(1000, seats, 3000);
        }
    }
}

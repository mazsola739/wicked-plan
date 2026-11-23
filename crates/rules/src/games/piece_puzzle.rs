//! Piece Puzzle (exile, a race for 1–4). Rules page: docs/games/piece-puzzle.md
//!
//! Fill a target shape exactly with the given polyominoes (turned and flipped
//! as you like). Seat 0 solves; the other seats are bots that "solve" at seeded
//! times. `Race` below is the shared race clock, also used by Fence Building
//! and Symmetry Puzzle.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

// --- The race --------------------------------------------------------------

/// Points by finishing place among those who finished the round.
pub const PLACE_POINTS: [i64; 4] = [200, 140, 90, 60];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceFinish {
    pub id: String,
    /// None: did not finish.
    pub time_ms: Option<i64>,
    pub place: Option<u32>,
    pub points: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Racer {
    pub id: String,
    pub score: i64,
    /// Finished the current round.
    pub done: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceView {
    pub round: usize,
    pub rounds: usize,
    pub time_left_ms: i64,
    pub round_time_ms: i64,
    pub racers: Vec<Racer>,
    pub last: Option<Vec<RaceFinish>>,
}

/// Rounds against the clock. Seat 0 plays; the other seats finish each round
/// at a seeded time between 25% and 110% of the limit (over 100% = not finished).
#[derive(Clone, Debug)]
pub struct Race {
    pub players: Vec<String>,
    pub round: usize,
    pub round_ms: Vec<i64>,
    pub elapsed_ms: i64,
    bot_times: Vec<Option<i64>>,
    pub scores: Vec<i64>,
    pub last: Option<Vec<RaceFinish>>,
    pub over: bool,
    seed: u64,
}

impl Race {
    pub fn new(players: &[String], seed: u64, round_ms: &[i64]) -> Result<Self, String> {
        if players.is_empty() || players.len() > PLACE_POINTS.len() {
            return Err("This race is for 1 to 4 players".into());
        }
        let mut race = Race {
            players: players.to_vec(),
            round: 0,
            round_ms: round_ms.to_vec(),
            elapsed_ms: 0,
            bot_times: Vec::new(),
            scores: vec![0; players.len()],
            last: None,
            over: false,
            seed,
        };
        race.start_round();
        Ok(race)
    }

    pub fn limit(&self) -> i64 {
        self.round_ms[self.round.min(self.round_ms.len() - 1)]
    }

    fn start_round(&mut self) {
        self.elapsed_ms = 0;
        let limit = self.limit();
        let mut rng = Rng::new(self.seed ^ (0xb07 + self.round as u64 * 0x9e37_79b9));
        self.bot_times = (0..self.players.len())
            .map(|seat| {
                let t = (limit as f64 * (0.25 + 0.85 * rng.next_f64())) as i64;
                (seat > 0 && t <= limit).then_some(t)
            })
            .collect();
    }

    /// Is `player` the one who plays (seat 0)?
    pub fn is_solver(&self, player: &str) -> bool {
        self.players[0] == player
    }

    pub fn check_move(&self, player: &str) -> Result<(), String> {
        if self.over {
            return Err("Game is over".into());
        }
        if !self.is_solver(player) {
            return Err("That seat races by itself".into());
        }
        Ok(())
    }

    /// Time passes; true when the round ended (the time ran out).
    pub fn tick(&mut self, ms: u64) -> bool {
        if self.over {
            return false;
        }
        self.elapsed_ms += ms as i64;
        if self.elapsed_ms >= self.limit() {
            self.end_round(None);
            return true;
        }
        false
    }

    /// Seat 0 solved (true) or gave up (false). Ends the round at once.
    pub fn solver_done(&mut self, solved: bool) {
        let t = solved.then_some(self.elapsed_ms.min(self.limit()));
        self.end_round(t);
    }

    fn end_round(&mut self, solver_time: Option<i64>) {
        let limit = self.limit();
        let mut times = self.bot_times.clone();
        times[0] = solver_time;
        let finishes = (0..self.players.len())
            .map(|seat| {
                let place = times[seat].map(|t| 1 + times.iter().flatten().filter(|&&o| o < t).count() as u32);
                let points = match (times[seat], place) {
                    (Some(t), Some(p)) => PLACE_POINTS[p as usize - 1] + (limit - t) / 2000,
                    _ => 0,
                };
                self.scores[seat] += points;
                RaceFinish { id: self.players[seat].clone(), time_ms: times[seat], place, points }
            })
            .collect();
        self.last = Some(finishes);
        self.round += 1;
        if self.round >= self.round_ms.len() {
            self.over = true;
            self.elapsed_ms = 0;
        } else {
            self.start_round();
        }
    }

    pub fn to_move(&self) -> Vec<String> {
        if self.over { Vec::new() } else { vec![self.players[0].clone()] }
    }

    pub fn view(&self) -> RaceView {
        RaceView {
            round: (self.round + 1).min(self.round_ms.len()),
            rounds: self.round_ms.len(),
            time_left_ms: if self.over { 0 } else { self.limit() - self.elapsed_ms },
            round_time_ms: self.limit(),
            racers: (0..self.players.len())
                .map(|seat| Racer {
                    id: self.players[seat].clone(),
                    score: self.scores[seat],
                    done: !self.over && self.bot_times[seat].is_some_and(|t| t <= self.elapsed_ms),
                })
                .collect(),
            last: self.last.clone(),
        }
    }

    pub fn results(&self) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = self.players.iter().cloned().zip(self.scores.iter().copied()).collect();
        rank_by_score(&scores)
    }
}

// --- Polyominoes -----------------------------------------------------------

pub type Cell = (i32, i32);

/// Shifted so the smallest x and y are 0, sorted.
pub fn normalize(cells: &[Cell]) -> Vec<Cell> {
    let mx = cells.iter().map(|c| c.0).min().unwrap_or(0);
    let my = cells.iter().map(|c| c.1).min().unwrap_or(0);
    let mut out: Vec<Cell> = cells.iter().map(|&(x, y)| (x - mx, y - my)).collect();
    out.sort_by_key(|&(x, y)| (y, x));
    out
}

/// Every distinct turn and flip of a shape, normalized.
pub fn orientations(shape: &[Cell]) -> Vec<Vec<Cell>> {
    let mut out: Vec<Vec<Cell>> = Vec::new();
    for flip in [false, true] {
        let mut cur: Vec<Cell> = shape.iter().map(|&(x, y)| if flip { (-x, y) } else { (x, y) }).collect();
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

#[derive(Clone, Debug)]
pub struct Puzzle {
    pub width: i32,
    pub height: i32,
    pub target: Vec<bool>,
    /// Shapes as handed out.
    pub pieces: Vec<Vec<Cell>>,
    /// Where each piece lay when the target was cut.
    pub solution: Vec<Vec<Cell>>,
}

pub const AREAS: [usize; 3] = [16, 24, 32];
pub const PIECES: [usize; 3] = [4, 5, 7];
pub const BOXES: [i32; 3] = [5, 6, 7];
pub const ROUND_MS: [i64; 3] = [2 * 60_000, 3 * 60_000, 4 * 60_000];

const DIRS: [Cell; 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// No empty cell is shut in: every empty cell reaches the border.
fn has_no_holes(w: i32, h: i32, filled: &[bool]) -> bool {
    let mut seen = vec![false; (w * h) as usize];
    let mut stack: Vec<Cell> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let border = x == 0 || y == 0 || x == w - 1 || y == h - 1;
            if border && !filled[(y * w + x) as usize] {
                seen[(y * w + x) as usize] = true;
                stack.push((x, y));
            }
        }
    }
    while let Some((x, y)) = stack.pop() {
        for (dx, dy) in DIRS {
            let (nx, ny) = (x + dx, y + dy);
            if nx >= 0 && ny >= 0 && nx < w && ny < h {
                let i = (ny * w + nx) as usize;
                if !filled[i] && !seen[i] {
                    seen[i] = true;
                    stack.push((nx, ny));
                }
            }
        }
    }
    (0..filled.len()).all(|i| filled[i] || seen[i])
}

pub fn generate(round: usize, rng: &mut Rng) -> Puzzle {
    let (area, k, size) = (AREAS[round], PIECES[round], BOXES[round]);
    loop {
        // A connected blob without holes, grown from the middle.
        let mut filled = vec![false; (size * size) as usize];
        let mut cells = vec![(size / 2, size / 2)];
        filled[((size / 2) * size + size / 2) as usize] = true;
        while cells.len() < area {
            let (x, y) = *rng.pick(&cells);
            let (dx, dy) = DIRS[rng.index(4)];
            let (nx, ny) = (x + dx, y + dy);
            if nx >= 0 && ny >= 0 && nx < size && ny < size && !filled[(ny * size + nx) as usize] {
                filled[(ny * size + nx) as usize] = true;
                cells.push((nx, ny));
            }
        }
        if !has_no_holes(size, size, &filled) {
            continue;
        }
        // Cut it: k seeds grow until every cell has an owner; the smallest piece grows first.
        let mut owner = vec![usize::MAX; (size * size) as usize];
        let mut order = cells.clone();
        rng.shuffle(&mut order);
        let mut pieces: Vec<Vec<Cell>> = order[..k].iter().map(|&c| vec![c]).collect();
        for (p, &(x, y)) in order[..k].iter().enumerate() {
            owner[(y * size + x) as usize] = p;
        }
        let free = |owner: &[usize], (x, y): Cell| {
            DIRS.iter()
                .map(|&(dx, dy)| (x + dx, y + dy))
                .filter(|&(nx, ny)| nx >= 0 && ny >= 0 && nx < size && ny < size)
                .filter(|&(nx, ny)| filled[(ny * size + nx) as usize] && owner[(ny * size + nx) as usize] == usize::MAX)
                .collect::<Vec<Cell>>()
        };
        while pieces.iter().map(Vec::len).sum::<usize>() < area {
            // The pieces that can still grow; mostly the smallest of them grows.
            let growing: Vec<(usize, Vec<Cell>)> = (0..k)
                .filter(|&p| pieces[p].len() < 6)
                .map(|p| (p, pieces[p].iter().flat_map(|&c| free(&owner, c)).collect::<Vec<Cell>>()))
                .filter(|(_, f)| !f.is_empty())
                .collect();
            let Some(min) = growing.iter().map(|(p, _)| pieces[*p].len()).min() else { break };
            let small: Vec<&(usize, Vec<Cell>)> = growing.iter().filter(|(p, _)| pieces[*p].len() == min).collect();
            let (p, frontier) = if rng.next_f64() < 0.8 { *rng.pick(&small) } else { rng.pick(&growing) };
            let (nx, ny) = *rng.pick(frontier);
            owner[(ny * size + nx) as usize] = *p;
            pieces[*p].push((nx, ny));
        }
        if pieces.iter().any(|p| p.len() < 3 || p.len() > 6) || pieces.iter().map(Vec::len).sum::<usize>() < area {
            continue;
        }
        // Crop the box to the blob.
        let x0 = cells.iter().map(|c| c.0).min().unwrap_or(0);
        let y0 = cells.iter().map(|c| c.1).min().unwrap_or(0);
        let width = cells.iter().map(|c| c.0).max().unwrap_or(0) - x0 + 1;
        let height = cells.iter().map(|c| c.1).max().unwrap_or(0) - y0 + 1;
        let mut target = vec![false; (width * height) as usize];
        for &(x, y) in &cells {
            target[((y - y0) * width + x - x0) as usize] = true;
        }
        let solution: Vec<Vec<Cell>> = pieces.iter().map(|p| p.iter().map(|&(x, y)| (x - x0, y - y0)).collect()).collect();
        let handed = solution
            .iter()
            .map(|p| {
                let all = orientations(p);
                all[rng.index(all.len())].clone()
            })
            .collect();
        return Puzzle { width, height, target, pieces: handed, solution };
    }
}

/// Counts the coverings of the target with every piece once (stops at `limit`).
pub fn count_solutions(p: &Puzzle, limit: usize) -> usize {
    fn go(p: &Puzzle, shapes: &[Vec<Vec<Cell>>], used: &mut [bool], grid: &mut [bool], found: &mut usize, limit: usize) {
        let Some(first) = (0..grid.len()).find(|&i| p.target[i] && !grid[i]) else {
            *found += 1;
            return;
        };
        let (fx, fy) = ((first as i32) % p.width, (first as i32) / p.width);
        for k in 0..shapes.len() {
            if used[k] {
                continue;
            }
            for o in &shapes[k] {
                // The orientation's first cell (row-major) goes on the first empty cell.
                let (ax, ay) = o[0];
                let cells: Vec<usize> = o
                    .iter()
                    .map(|&(x, y)| (x - ax + fx, y - ay + fy))
                    .filter(|&(x, y)| x >= 0 && y >= 0 && x < p.width && y < p.height)
                    .map(|(x, y)| (y * p.width + x) as usize)
                    .collect();
                if cells.len() == o.len() && cells.iter().all(|&i| p.target[i] && !grid[i]) {
                    used[k] = true;
                    cells.iter().for_each(|&i| grid[i] = true);
                    go(p, shapes, used, grid, found, limit);
                    cells.iter().for_each(|&i| grid[i] = false);
                    used[k] = false;
                    if *found >= limit {
                        return;
                    }
                }
            }
        }
    }
    let shapes: Vec<Vec<Vec<Cell>>> = p.pieces.iter().map(|s| orientations(s)).collect();
    let mut found = 0;
    go(p, &shapes, &mut vec![false; shapes.len()], &mut vec![false; p.target.len()], &mut found, limit);
    found
}

// --- The game --------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct PieceState {
    pub race: Race,
    pub puzzle: Puzzle,
    /// Where each piece lies now.
    pub placed: Vec<Option<Vec<Cell>>>,
    /// Rounds the solver finished first.
    pub firsts: usize,
    /// Replay moments: a round won by a whisker, first place in every round.
    pub moments: Moments,
    seed: u64,
}

impl PieceState {
    fn load(&mut self) {
        if !self.race.over {
            self.puzzle = generate(self.race.round, &mut Rng::new(self.seed.wrapping_add(self.race.round as u64 * 1_000_003)));
            self.placed = vec![None; self.puzzle.pieces.len()];
        }
    }

    fn covered(&self) -> bool {
        self.placed.iter().all(Option::is_some)
    }

    /// After the solver covered the shape: replay moments for a first place won by less than a
    /// tenth of the round time, and for first place in every round (all times are public now).
    fn solved_moments(&mut self) {
        let Some(last) = &self.race.last else { return };
        let (Some(1), Some(t)) = (last[0].place, last[0].time_ms) else { return };
        let me = self.race.players[0].clone();
        let limit = self.race.round_ms[self.race.round - 1];
        let next = last[1..].iter().filter_map(|f| f.time_ms.map(|b| (b - t, &f.id))).filter(|&(gap, _)| gap > 0).min();
        if let Some((gap, id)) = next.filter(|&(gap, _)| gap * 10 <= limit) {
            self.moments.push(&me, format!("Beat {{{id}}} to the finish by {}.{} s", gap / 1000, gap % 1000 / 100));
        }
        self.firsts += 1;
        if self.race.over && self.firsts == self.race.round_ms.len() {
            self.moments.push(&me, "First to finish in all three rounds");
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PieceMove {
    Place { piece: usize, cells: Vec<Cell> },
    Remove { piece: usize },
    GiveUp,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceInfo {
    pub shape: Vec<Cell>,
    pub placed: Option<Vec<Cell>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceView {
    #[serde(flatten)]
    pub race: RaceView,
    pub width: i32,
    pub height: i32,
    pub target: Vec<bool>,
    pub pieces: Vec<PieceInfo>,
    pub over: bool,
}

pub struct Rules;

impl Game for Rules {
    type State = PieceState;
    type Move = PieceMove;
    type View = PieceView;

    fn info() -> GameInfo {
        GameInfo {
            id: "piece-puzzle",
            kind: "exile",
            difficulty: "medium",
            tags: &["logic", "speed"],
            min_players: 1,
            max_players: 4,
            name: "Piece Puzzle",
            short: "Fill the shape exactly with the given pieces, faster than the others. Three rounds.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<PieceState, String> {
        let race = Race::new(players, seed, &ROUND_MS)?;
        let mut s =
            PieceState { race, puzzle: generate(0, &mut Rng::new(seed)), placed: Vec::new(), firsts: 0, moments: Moments::default(), seed };
        s.load();
        Ok(s)
    }

    fn apply_move(s: &mut PieceState, player: &str, mv: PieceMove) -> Result<(), String> {
        s.race.check_move(player)?;
        match mv {
            PieceMove::Place { piece, cells } => {
                let shape = s.puzzle.pieces.get(piece).ok_or("No such piece")?;
                if !orientations(shape).contains(&normalize(&cells)) {
                    return Err("That is not the shape of this piece".into());
                }
                let p = &s.puzzle;
                let inside = |&(x, y): &Cell| x >= 0 && y >= 0 && x < p.width && y < p.height && p.target[(y * p.width + x) as usize];
                if !cells.iter().all(inside) {
                    return Err("The piece must lie inside the shape".into());
                }
                let overlaps =
                    s.placed.iter().enumerate().any(|(k, c)| k != piece && c.as_ref().is_some_and(|c| c.iter().any(|x| cells.contains(x))));
                if overlaps {
                    return Err("Pieces may not overlap".into());
                }
                s.placed[piece] = Some(cells);
                if s.covered() {
                    s.race.solver_done(true);
                    s.solved_moments();
                    s.load();
                }
            }
            PieceMove::Remove { piece } => {
                *s.placed.get_mut(piece).ok_or("No such piece")? = None;
            }
            PieceMove::GiveUp => {
                s.race.solver_done(false);
                s.load();
            }
        }
        Ok(())
    }

    fn tick(s: &mut PieceState, ms: u64) {
        if s.race.tick(ms) {
            s.load();
        }
    }

    fn view_for(s: &PieceState, _player: &str) -> PieceView {
        PieceView {
            race: s.race.view(),
            width: s.puzzle.width,
            height: s.puzzle.height,
            target: s.puzzle.target.clone(),
            pieces: s
                .puzzle
                .pieces
                .iter()
                .zip(&s.placed)
                .map(|(shape, placed)| PieceInfo { shape: shape.clone(), placed: placed.clone() })
                .collect(),
            over: s.race.over,
        }
    }

    fn is_over(s: &PieceState) -> bool {
        s.race.over
    }

    fn results(s: &PieceState) -> Vec<PlayerResult> {
        s.race.results()
    }

    fn to_move(s: &PieceState) -> Vec<String> {
        s.race.to_move()
    }

    fn moments(s: &PieceState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &PieceState, _player: &str, _seed: u64) -> PieceMove {
        // Take back a piece that is not where the puzzle was cut, else lay the next one.
        let sol = &s.puzzle.solution;
        let same = |a: &Vec<Cell>, b: &Vec<Cell>| normalize_set(a) == normalize_set(b);
        if let Some(k) = (0..sol.len()).find(|&k| s.placed[k].as_ref().is_some_and(|c| !same(c, &sol[k]))) {
            return PieceMove::Remove { piece: k };
        }
        match (0..sol.len()).find(|&k| s.placed[k].is_none()) {
            Some(k) => PieceMove::Place { piece: k, cells: sol[k].clone() },
            None => PieceMove::GiveUp,
        }
    }
}

fn normalize_set(cells: &[Cell]) -> Vec<Cell> {
    let mut v = cells.to_vec();
    v.sort();
    v
}

pub const READY: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::{bot_games, bot_games_with};

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn orientations_of_known_shapes() {
        assert_eq!(orientations(&[(0, 0), (1, 0), (0, 1), (1, 1)]).len(), 1); // square
        assert_eq!(orientations(&[(0, 0), (1, 0), (2, 0)]).len(), 2); // line
        assert_eq!(orientations(&[(0, 0), (0, 1), (0, 2), (1, 2)]).len(), 8); // L
        assert_eq!(orientations(&[(0, 0), (1, 0), (2, 0), (1, 1)]).len(), 4); // T
    }

    #[test]
    fn generated_puzzles_are_solvable() {
        for seed in 0..60 {
            for round in 0..3 {
                let p = generate(round, &mut Rng::new(seed));
                assert_eq!(p.target.iter().filter(|&&t| t).count(), AREAS[round]);
                assert_eq!(p.pieces.len(), PIECES[round]);
                assert!(p.pieces.iter().all(|s| (3..=6).contains(&s.len())));
                assert!(count_solutions(&p, 1) == 1, "seed {seed} round {round}");
            }
        }
    }

    #[test]
    fn placing_checks_shape_inside_and_overlap() {
        let mut s = Rules::setup(&players(1), 3).unwrap();
        let sol = s.puzzle.solution.clone();
        // Wrong shape: one cell short.
        assert!(Rules::apply_move(&mut s, "p1", PieceMove::Place { piece: 0, cells: sol[0][1..].to_vec() }).is_err());
        // Outside the box.
        let outside: Vec<Cell> = sol[0].iter().map(|&(x, y)| (x - 20, y)).collect();
        assert!(Rules::apply_move(&mut s, "p1", PieceMove::Place { piece: 0, cells: outside }).is_err());
        Rules::apply_move(&mut s, "p1", PieceMove::Place { piece: 0, cells: sol[0].clone() }).unwrap();
        // Piece 1 in piece 0's place overlaps (when the shapes match), else is the wrong shape: refused either way.
        assert!(Rules::apply_move(&mut s, "p1", PieceMove::Place { piece: 1, cells: sol[0].clone() }).is_err());
        for (k, cells) in sol.iter().enumerate().skip(1) {
            Rules::apply_move(&mut s, "p1", PieceMove::Place { piece: k, cells: cells.clone() }).unwrap();
        }
        // Covered: the round is over and won.
        assert_eq!(s.race.round, 1);
        let last = s.race.last.as_ref().unwrap();
        assert_eq!(last[0].place, Some(1));
        assert!(s.race.scores[0] >= 200);
    }

    #[test]
    fn bots_finish_on_time_and_late_ones_do_not() {
        let mut s = Rules::setup(&players(4), 11).unwrap();
        Rules::tick(&mut s, ROUND_MS[0] as u64);
        let last = s.race.last.clone().unwrap();
        assert_eq!(last[0].place, None);
        for f in &last[1..] {
            assert_eq!(f.place.is_some(), f.time_ms.is_some());
            assert!(f.time_ms.is_none_or(|t| t <= ROUND_MS[0]));
        }
        // Giving up plays the bots' times out at once.
        Rules::apply_move(&mut s, "p1", PieceMove::GiveUp).unwrap();
        assert_eq!(s.race.round, 2);
        assert!(Rules::apply_move(&mut s, "p2", PieceMove::GiveUp).is_err());
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<Rules>(1000, 3000);
        bot_games_with::<Rules>(1000, 4, 3000);
    }

    /// Lays every piece where the puzzle was cut.
    fn solve(s: &mut PieceState) {
        for (k, cells) in s.puzzle.solution.clone().into_iter().enumerate() {
            Rules::apply_move(s, "p1", PieceMove::Place { piece: k, cells }).unwrap();
        }
    }

    #[test]
    fn a_close_win_and_a_clean_sweep_are_replay_moments() {
        let mut s = Rules::setup(&players(4), 5).unwrap();
        // Round 1: p1 finishes at 15 s, p2 at 20 s.
        s.race.bot_times = vec![None, Some(20_000), Some(50_000), Some(90_000)];
        Rules::tick(&mut s, 15_000);
        solve(&mut s);
        let m = Rules::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Beat {p2} to the finish by 5.0 s"));
        // Rounds 2 and 3: easy wins, no close finish, but first every time.
        solve(&mut s);
        assert_eq!(Rules::moments(&s, "p1").len(), 1);
        solve(&mut s);
        let m = Rules::moments(&s, "p1");
        assert_eq!(m.len(), 2);
        assert_eq!((m[1].by.as_str(), m[1].text.as_str()), ("p1", "First to finish in all three rounds"));
        // Second place is no moment.
        let mut s = Rules::setup(&players(4), 5).unwrap();
        s.race.bot_times = vec![None, Some(1_000), None, None];
        Rules::tick(&mut s, 5_000);
        solve(&mut s);
        assert_eq!(s.race.last.as_ref().unwrap()[0].place, Some(2));
        assert!(Rules::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 3000);
        // The test solver lays a piece on nearly every step, so it wins almost every round and
        // sweeps most test games; someone racing the bots by hand wins a round far less often.
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

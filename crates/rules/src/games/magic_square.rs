//! Magic Square Trial (Hidden, solo). Rules page: docs/games/magic-square.md
//!
//! Three boards: 3×3, 4×4, 5×5. Fill in 1 … n² once each so that every row,
//! every column and both diagonals add up to the same number. A few numbers
//! are given and cannot change. Any correct square counts, not only ours.
//! Time per board: 2, 5 and 10 minutes.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult};
use crate::rng::Rng;

pub const SIZES: [usize; 3] = [3, 4, 5];
pub const CLUES: [usize; 3] = [1, 6, 6];
pub const BOARD_MS: [i64; 3] = [2 * 60_000, 5 * 60_000, 10 * 60_000];
pub const BOARD_POINTS: [i64; 3] = [150, 300, 500];

pub fn magic_sum(n: usize) -> i64 {
    (n * (n * n + 1) / 2) as i64
}

/// A square in row-major order.
pub fn is_magic(n: usize, cells: &[i64]) -> bool {
    if cells.len() != n * n {
        return false;
    }
    let mut seen = vec![false; n * n + 1];
    for &v in cells {
        if v < 1 || v as usize > n * n || seen[v as usize] {
            return false;
        }
        seen[v as usize] = true;
    }
    let m = magic_sum(n);
    let row = |r: usize| (0..n).map(|c| cells[r * n + c]).sum::<i64>();
    let col = |c: usize| (0..n).map(|r| cells[r * n + c]).sum::<i64>();
    (0..n).all(|i| row(i) == m && col(i) == m)
        && (0..n).map(|i| cells[i * n + i]).sum::<i64>() == m
        && (0..n).map(|i| cells[i * n + n - 1 - i]).sum::<i64>() == m
}

/// A random magic square: a classic construction, then a random symmetry
/// (rotation / mirror) and maybe the complement (n² + 1 − v), which all keep it magic.
pub fn generate(n: usize, rng: &mut Rng) -> Vec<i64> {
    let mut sq = vec![0i64; n * n];
    if n % 2 == 1 {
        // Siamese method.
        let (mut r, mut c) = (0usize, n / 2);
        for v in 1..=(n * n) as i64 {
            sq[r * n + c] = v;
            let (nr, nc) = ((r + n - 1) % n, (c + 1) % n);
            if sq[nr * n + nc] != 0 {
                r = (r + 1) % n;
            } else {
                (r, c) = (nr, nc);
            }
        }
    } else {
        assert_eq!(n, 4, "only 3, 4 and 5 are used");
        for r in 0..n {
            for c in 0..n {
                let v = (r * n + c + 1) as i64;
                let diagonal = r % 4 == c % 4 || (r + c) % 4 == 3;
                sq[r * n + c] = if diagonal { (n * n + 1) as i64 - v } else { v };
            }
        }
    }
    let turns = rng.int(0, 3);
    for _ in 0..turns {
        sq = (0..n * n).map(|i| sq[(n - 1 - i % n) * n + i / n]).collect();
    }
    if rng.next_f64() < 0.5 {
        sq = (0..n * n).map(|i| sq[(i / n) * n + n - 1 - i % n]).collect();
    }
    if rng.next_f64() < 0.5 {
        sq = sq.iter().map(|v| (n * n + 1) as i64 - v).collect();
    }
    sq
}

#[derive(Clone, Debug)]
pub struct MagicState {
    pub player: String,
    pub board: usize,
    pub solution: Vec<i64>,
    pub given: Vec<bool>,
    /// 0 = empty.
    pub cells: Vec<i64>,
    pub wrong_tries: u32,
    pub solved: Vec<bool>,
    pub score: i64,
    pub time_left_ms: i64,
    pub over: bool,
    seed: u64,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MagicMove {
    /// `value` 0 clears the cell.
    Set {
        cell: usize,
        value: i64,
    },
    Submit,
    /// Gives up the current board for 0 points.
    Skip,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MagicView {
    pub board: usize,
    pub boards: usize,
    pub size: usize,
    pub magic_sum: i64,
    pub cells: Vec<i64>,
    pub given: Vec<bool>,
    pub wrong_tries: u32,
    pub solved: Vec<bool>,
    pub score: i64,
    pub time_left_ms: i64,
    pub board_time_ms: i64,
    pub over: bool,
}

impl MagicState {
    pub fn size(&self) -> usize {
        SIZES[self.board.min(SIZES.len() - 1)]
    }

    fn load_board(&mut self) {
        let n = self.size();
        let mut rng = Rng::new(self.seed.wrapping_add(self.board as u64 * 1_000_003));
        self.solution = generate(n, &mut rng);
        let mut order: Vec<usize> = (0..n * n).collect();
        rng.shuffle(&mut order);
        // On the big board the 1 is always one of the given numbers, as in the show's trial.
        if n == 5 {
            let one = self.solution.iter().position(|&v| v == 1).expect("1 is in the square");
            order.retain(|&i| i != one);
            order.insert(0, one);
        }
        self.given = vec![false; n * n];
        for &i in order.iter().take(CLUES[self.board]) {
            self.given[i] = true;
        }
        self.cells = (0..n * n).map(|i| if self.given[i] { self.solution[i] } else { 0 }).collect();
        self.wrong_tries = 0;
        self.time_left_ms = BOARD_MS[self.board];
    }

    fn next_board(&mut self, solved: bool) {
        if solved {
            self.score += BOARD_POINTS[self.board] + self.time_left_ms / 1000;
        }
        self.solved.push(solved);
        self.board += 1;
        if self.board >= SIZES.len() {
            self.over = true;
            self.time_left_ms = 0;
        } else {
            self.load_board();
        }
    }
}

pub struct MagicSquare;

impl Game for MagicSquare {
    type State = MagicState;
    type Move = MagicMove;
    type View = MagicView;

    fn info() -> GameInfo {
        GameInfo {
            id: "magic-square",
            kind: "hidden",
            min_players: 1,
            max_players: 1,
            name: "Magic Square",
            short: "Fill the grid so every row, column and diagonal has the same sum. 3×3, 4×4, 5×5.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<MagicState, String> {
        let [player] = players else { return Err("Magic Square is a solo game".into()) };
        let mut s = MagicState {
            player: player.clone(),
            board: 0,
            solution: Vec::new(),
            given: Vec::new(),
            cells: Vec::new(),
            wrong_tries: 0,
            solved: Vec::new(),
            score: 0,
            time_left_ms: 0,
            over: false,
            seed,
        };
        s.load_board();
        Ok(s)
    }

    fn apply_move(s: &mut MagicState, player: &str, mv: MagicMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        if player != s.player {
            return Err("Not your game".into());
        }
        let n = s.size();
        match mv {
            MagicMove::Set { cell, value } => {
                if cell >= n * n || s.given[cell] || value < 0 || value as usize > n * n {
                    return Err("Illegal move".into());
                }
                s.cells[cell] = value;
            }
            MagicMove::Submit => {
                if is_magic(n, &s.cells) {
                    s.next_board(true);
                } else {
                    s.wrong_tries += 1;
                }
            }
            MagicMove::Skip => s.next_board(false),
        }
        Ok(())
    }

    fn tick(s: &mut MagicState, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.time_left_ms = 0;
            s.next_board(false);
        }
    }

    fn view_for(s: &MagicState, _player: &str) -> MagicView {
        let n = s.size();
        MagicView {
            board: (s.board + 1).min(SIZES.len()),
            boards: SIZES.len(),
            size: n,
            magic_sum: magic_sum(n),
            cells: s.cells.clone(),
            given: s.given.clone(),
            wrong_tries: s.wrong_tries,
            solved: s.solved.clone(),
            score: s.score,
            time_left_ms: s.time_left_ms,
            board_time_ms: BOARD_MS[s.board.min(SIZES.len() - 1)],
            over: s.over,
        }
    }

    fn is_over(s: &MagicState) -> bool {
        s.over
    }

    fn results(s: &MagicState) -> Vec<PlayerResult> {
        vec![PlayerResult { player_id: s.player.clone(), rank: 1, points: s.score }]
    }

    fn bot_move(s: &MagicState, _player: &str, seed: u64) -> MagicMove {
        let mut rng = Rng::new(seed);
        match (0..s.cells.len()).find(|&i| s.cells[i] != s.solution[i]) {
            // Now and then a slip that it has to fix later.
            Some(cell) if rng.next_f64() < 0.1 => MagicMove::Set { cell, value: rng.int(1, s.cells.len() as i64) },
            Some(cell) => MagicMove::Set { cell, value: s.solution[cell] },
            None => MagicMove::Submit,
        }
    }
}

pub const READY: bool = true;
pub type Rules = MagicSquare;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games;

    const P: &str = "p1";

    #[test]
    fn generated_squares_are_magic() {
        for seed in 0..300 {
            for n in SIZES {
                assert!(is_magic(n, &generate(n, &mut Rng::new(seed))), "n={n} seed={seed}");
            }
        }
    }

    #[test]
    fn checker_rejects_near_misses() {
        assert!(is_magic(3, &[2, 7, 6, 9, 5, 1, 4, 3, 8]));
        assert!(!is_magic(3, &[7, 2, 6, 9, 5, 1, 4, 3, 8]));
        assert!(!is_magic(3, &[5, 5, 5, 5, 5, 5, 5, 5, 5]));
        assert!(!is_magic(3, &[2, 7, 6, 9, 5, 1, 4, 3]));
    }

    #[test]
    fn clues_are_fixed_and_the_5x5_shows_the_one() {
        let mut s = MagicSquare::setup(&[P.into()], 9).unwrap();
        let clue = s.given.iter().position(|&g| g).unwrap();
        assert!(MagicSquare::apply_move(&mut s, P, MagicMove::Set { cell: clue, value: 1 }).is_err());
        MagicSquare::apply_move(&mut s, P, MagicMove::Skip).unwrap();
        MagicSquare::apply_move(&mut s, P, MagicMove::Skip).unwrap();
        assert_eq!(s.size(), 5);
        let one = s.cells.iter().position(|&v| v == 1).unwrap();
        assert!(s.given[one]);
        assert_eq!(s.given.iter().filter(|&&g| g).count(), CLUES[2]);
    }

    #[test]
    fn any_correct_square_counts() {
        let mut s = MagicSquare::setup(&[P.into()], 1).unwrap();
        // Fill the given square, then submit: next board, points.
        for i in 0..9 {
            if !s.given[i] {
                let v = s.solution[i];
                MagicSquare::apply_move(&mut s, P, MagicMove::Set { cell: i, value: v }).unwrap();
            }
        }
        MagicSquare::apply_move(&mut s, P, MagicMove::Submit).unwrap();
        assert_eq!(s.solved, vec![true]);
        assert!(s.score >= BOARD_POINTS[0]);
    }

    #[test]
    fn wrong_submit_is_counted_and_time_up_moves_on() {
        let mut s = MagicSquare::setup(&[P.into()], 2).unwrap();
        MagicSquare::apply_move(&mut s, P, MagicMove::Submit).unwrap();
        assert_eq!(s.wrong_tries, 1);
        MagicSquare::tick(&mut s, BOARD_MS[0] as u64);
        assert_eq!(s.board, 1);
        assert_eq!(s.solved, vec![false]);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<MagicSquare>(1000, 5000);
    }
}

//! Count the Cubes (Mini, solo). Rules page: docs/games/count-the-cubes.md
//!
//! Five rounds. Each round shows a stack of cubes in an isometric view; type
//! how many there are before the 20-second timer runs out.
//!
//! A puzzle is a height map: a column of `h` cubes stands on every tile (x to
//! the bottom-right, y to the bottom-left, the back corner is (0, 0)).
//! Readability rule ("staircase"): a column is never taller than the column
//! behind it on either axis, so no cube is hidden and the answer can be counted
//! from the picture.

use serde::Serialize;

use crate::game::GameInfo;
use crate::quiz::QuizPuzzle;
use crate::rng::Rng;

pub const ROUND_MS: i64 = 20_000;
pub const MAX_ANSWER: i64 = 999;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CubesPuzzle {
    pub cols: usize,
    pub rows: usize,
    /// Row-major: index = y * cols + x.
    pub heights: Vec<u32>,
}

pub struct Difficulty {
    pub cols: usize,
    pub rows: usize,
    pub max_height: u32,
}

/// Round 1 → 5 gets bigger and taller.
pub const DIFFICULTIES: [Difficulty; 5] = [
    Difficulty { cols: 3, rows: 3, max_height: 2 },
    Difficulty { cols: 3, rows: 3, max_height: 3 },
    Difficulty { cols: 4, rows: 3, max_height: 3 },
    Difficulty { cols: 4, rows: 4, max_height: 4 },
    Difficulty { cols: 5, rows: 4, max_height: 4 },
];

impl CubesPuzzle {
    pub fn height_at(&self, x: usize, y: usize) -> u32 {
        self.heights[y * self.cols + x]
    }

    pub fn count(&self) -> i64 {
        self.heights.iter().map(|&h| i64::from(h)).sum()
    }

    pub fn is_readable(&self) -> bool {
        if self.heights.len() != self.cols * self.rows || self.count() == 0 {
            return false;
        }
        (0..self.rows).all(|y| {
            (0..self.cols).all(|x| {
                let h = self.height_at(x, y);
                (x == 0 || h <= self.height_at(x - 1, y)) && (y == 0 || h <= self.height_at(x, y - 1))
            })
        })
    }

    /// `level` is 1-based.
    pub fn generate(rng: &mut Rng, level: usize) -> Self {
        let d = &DIFFICULTIES[level.clamp(1, DIFFICULTIES.len()) - 1];
        let min_cubes = (d.cols * d.rows) as f64 * f64::from(d.max_height) * 0.3;
        for _ in 0..50 {
            let mut heights: Vec<u32> = Vec::with_capacity(d.cols * d.rows);
            for y in 0..d.rows {
                for x in 0..d.cols {
                    let left = if x > 0 { heights[y * d.cols + x - 1] } else { d.max_height };
                    let up = if y > 0 { heights[(y - 1) * d.cols + x] } else { d.max_height };
                    // Mostly keep the height, sometimes step down by 1–2: a varied staircase.
                    let r = rng.next_f64();
                    let drop = if r < 0.45 {
                        0
                    } else if r < 0.85 {
                        1
                    } else {
                        2
                    };
                    heights.push(left.min(up).saturating_sub(drop));
                }
            }
            let p = Self { cols: d.cols, rows: d.rows, heights };
            if p.is_readable() && p.count() as f64 >= min_cubes {
                return p;
            }
        }
        // Fallback that always passes: a full block of the maximum height.
        Self { cols: d.cols, rows: d.rows, heights: vec![d.max_height; d.cols * d.rows] }
    }
}

impl QuizPuzzle for CubesPuzzle {
    type Answer = i64;
    fn is_correct(&self, answer: &i64) -> bool {
        *answer == self.count()
    }
    fn solution(&self) -> i64 {
        self.count()
    }
    fn validate(&self, answer: &i64) -> Result<(), String> {
        if (0..=MAX_ANSWER).contains(answer) { Ok(()) } else { Err("Illegal move".into()) }
    }
}

crate::quiz_game!(
    CountTheCubes,
    CubesPuzzle,
    GameInfo {
        id: "count-the-cubes",
        kind: "mini-solo",
        difficulty: "easy",
        min_players: 1,
        max_players: 1,
        name: "Count the Cubes",
        short: "How many cubes are in the drawing? Five rounds, 20 seconds each.",
    },
    ROUND_MS,
    |rng| (1..=DIFFICULTIES.len()).map(|level| CubesPuzzle::generate(rng, level)).collect(),
    |state, rng| {
        // Right 75% of the time, otherwise off by one or two.
        let answer = state.current_solution().unwrap_or(0);
        if rng.next_f64() < 0.75 { answer } else { (answer + rng.pick(&[-2, -1, 1, 2])).max(0) }
    }
);

pub const READY: bool = true;
pub type Rules = CountTheCubes;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, testing::bot_games};

    const P: &str = "p1";

    fn setup(seed: u64) -> <CountTheCubes as Game>::State {
        CountTheCubes::setup(&[P.to_string()], seed).unwrap()
    }

    #[test]
    fn generator_makes_readable_puzzles_of_the_right_size() {
        for seed in 1..=1000 {
            let mut rng = Rng::new(seed);
            for (i, d) in DIFFICULTIES.iter().enumerate() {
                let p = CubesPuzzle::generate(&mut rng, i + 1);
                assert!(p.is_readable());
                assert_eq!((p.cols, p.rows), (d.cols, d.rows));
                assert!(p.heights.iter().all(|&h| h <= d.max_height));
            }
        }
    }

    #[test]
    fn generator_is_deterministic_and_varied() {
        assert_eq!(CubesPuzzle::generate(&mut Rng::new(9), 4), CubesPuzzle::generate(&mut Rng::new(9), 4));
        let answers: std::collections::HashSet<i64> = (1..=50).map(|s| CubesPuzzle::generate(&mut Rng::new(s), 5).count()).collect();
        assert!(answers.len() > 5);
    }

    #[test]
    fn readability_rejects_hidden_cubes() {
        assert!(!CubesPuzzle { cols: 2, rows: 1, heights: vec![1, 2] }.is_readable());
        assert!(!CubesPuzzle { cols: 1, rows: 2, heights: vec![1, 2] }.is_readable());
        assert!(CubesPuzzle { cols: 2, rows: 2, heights: vec![3, 2, 2, 1] }.is_readable());
        assert!(!CubesPuzzle { cols: 2, rows: 1, heights: vec![0, 0] }.is_readable());
    }

    #[test]
    fn perfect_game_scores_every_round() {
        let mut s = setup(1);
        while !s.over {
            let a = s.current_solution().unwrap();
            CountTheCubes::apply_move(&mut s, P, QuizMove::Answer { value: a }).unwrap();
        }
        assert!(s.score() >= 5 * 100);
        assert!(s.rounds.iter().all(|r| r.correct));
    }

    #[test]
    fn speed_and_timeouts() {
        let mut slow = setup(3);
        CountTheCubes::tick(&mut slow, (ROUND_MS - 1000) as u64);
        let mut fast = setup(3);
        let a = fast.current_solution().unwrap();
        CountTheCubes::apply_move(&mut slow, P, QuizMove::Answer { value: a }).unwrap();
        CountTheCubes::apply_move(&mut fast, P, QuizMove::Answer { value: a }).unwrap();
        assert!(fast.rounds[0].points > slow.rounds[0].points);

        let mut s = setup(4);
        CountTheCubes::tick(&mut s, (ROUND_MS + 5) as u64);
        assert_eq!(s.current, 1);
        assert_eq!(s.rounds[0].given, Some(None));
        assert_eq!(s.time_left_ms, ROUND_MS);
    }

    #[test]
    fn rejects_illegal_moves_without_changing_state() {
        let mut s = setup(6);
        assert!(CountTheCubes::apply_move(&mut s, "someone-else", QuizMove::Answer { value: 1 }).is_err());
        assert!(CountTheCubes::apply_move(&mut s, P, QuizMove::Answer { value: -1 }).is_err());
        assert_eq!(s.current, 0);
    }

    #[test]
    fn view_hides_nothing_it_should_not() {
        let s = setup(5);
        let v = serde_json::to_value(CountTheCubes::view_for(&s, P)).unwrap();
        assert!(v["history"].as_array().unwrap().is_empty());
        assert!(v["puzzle"].get("solution").is_none());
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<CountTheCubes>(1000, 200);
    }
}

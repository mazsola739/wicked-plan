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

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult};
use crate::quiz::{QuizPuzzle, QuizState, QuizView};
use crate::rng::Rng;

pub const ROUND_MS: i64 = 20_000;
pub const MAX_ANSWER: i64 = 999;
/// Time left on the last (biggest) stack for a replay moment.
pub const FAST_MS: i64 = 10_000;

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

/// The shared quiz rounds plus the replay moments.
#[derive(Clone, Debug)]
pub struct CubesState {
    pub quiz: QuizState<CubesPuzzle>,
    /// Replay moments: the last (biggest) stack counted fast, and all five stacks right.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum QuizMove {
    Answer { value: i64 },
}

pub struct CountTheCubes;

impl Game for CountTheCubes {
    type State = CubesState;
    type Move = QuizMove;
    type View = QuizView<CubesPuzzle>;

    fn info() -> GameInfo {
        GameInfo {
            id: "count-the-cubes",
            kind: "mini-solo",
            difficulty: "normal",
            tags: &["logic-puzzles"],
            min_players: 1,
            max_players: 1,
            name: "Count the Cubes",
            short: "How many cubes are in the drawing? Five rounds, 20 seconds each.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<CubesState, String> {
        let mut rng = Rng::new(seed);
        let puzzles = (1..=DIFFICULTIES.len()).map(|level| CubesPuzzle::generate(&mut rng, level)).collect();
        Ok(CubesState { quiz: QuizState::new(players, puzzles, ROUND_MS, Self::info().name)?, moments: Moments::default() })
    }

    fn apply_move(s: &mut CubesState, player: &str, mv: QuizMove) -> Result<(), String> {
        let QuizMove::Answer { value } = mv;
        let left = s.quiz.time_left_ms;
        s.quiz.answer(player, value)?;
        // Replay moments, after the last round (a timeout is never right, so only an answer can make one).
        let q = &s.quiz;
        let last = &q.rounds[q.current - 1];
        if q.over && last.correct {
            if left >= FAST_MS {
                s.moments.push(&q.player, format!("{} cubes counted with {} s to spare", last.puzzle.count(), left / 1000));
            }
            if q.rounds.iter().all(|r| r.correct) {
                s.moments.push(&q.player, "All five stacks counted right");
            }
        }
        Ok(())
    }

    fn tick(s: &mut CubesState, ms: u64) {
        s.quiz.tick(ms);
    }

    fn view_for(s: &CubesState, _player: &str) -> QuizView<CubesPuzzle> {
        s.quiz.view()
    }

    fn is_over(s: &CubesState) -> bool {
        s.quiz.over
    }

    fn results(s: &CubesState) -> Vec<PlayerResult> {
        s.quiz.results()
    }

    fn bot_move(s: &CubesState, _player: &str, seed: u64) -> QuizMove {
        let mut rng = Rng::new(seed);
        // Right 75% of the time, otherwise off by one or two.
        let answer = s.quiz.current_solution().unwrap_or(0);
        let value = if rng.next_f64() < 0.75 { answer } else { (answer + rng.pick(&[-2, -1, 1, 2])).max(0) };
        QuizMove::Answer { value }
    }

    fn moments(s: &CubesState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub type Rules = CountTheCubes;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games;

    const P: &str = "p1";

    fn setup(seed: u64) -> CubesState {
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
        while !s.quiz.over {
            let a = s.quiz.current_solution().unwrap();
            CountTheCubes::apply_move(&mut s, P, QuizMove::Answer { value: a }).unwrap();
        }
        assert!(s.quiz.score() >= 5 * 100);
        assert!(s.quiz.rounds.iter().all(|r| r.correct));
    }

    #[test]
    fn a_fast_last_count_and_a_perfect_run_are_replay_moments() {
        let mut s = setup(1);
        while !s.quiz.over {
            let a = s.quiz.current_solution().unwrap();
            CountTheCubes::tick(&mut s, 7_500);
            CountTheCubes::apply_move(&mut s, P, QuizMove::Answer { value: a }).unwrap();
        }
        let cubes = s.quiz.rounds[4].puzzle.count();
        let m: Vec<(String, String)> = CountTheCubes::moments(&s, P).into_iter().map(|m| (m.by, m.text)).collect();
        assert_eq!(m, vec![
            (P.to_string(), format!("{cubes} cubes counted with 12 s to spare")),
            (P.to_string(), "All five stacks counted right".to_string()),
        ]);
        // A miss, then a slow last count: no moment.
        let mut s = setup(1);
        CountTheCubes::apply_move(&mut s, P, QuizMove::Answer { value: 0 }).unwrap();
        while !s.quiz.over {
            let a = s.quiz.current_solution().unwrap();
            CountTheCubes::tick(&mut s, 10_500);
            CountTheCubes::apply_move(&mut s, P, QuizMove::Answer { value: a }).unwrap();
        }
        assert!(CountTheCubes::moments(&s, P).is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<CountTheCubes>(300, 1, 200);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 2, "too many in one game: {stats:?}");
    }

    #[test]
    fn speed_and_timeouts() {
        let mut slow = setup(3);
        CountTheCubes::tick(&mut slow, (ROUND_MS - 1000) as u64);
        let mut fast = setup(3);
        let a = fast.quiz.current_solution().unwrap();
        CountTheCubes::apply_move(&mut slow, P, QuizMove::Answer { value: a }).unwrap();
        CountTheCubes::apply_move(&mut fast, P, QuizMove::Answer { value: a }).unwrap();
        assert!(fast.quiz.rounds[0].points > slow.quiz.rounds[0].points);

        let mut s = setup(4);
        CountTheCubes::tick(&mut s, (ROUND_MS + 5) as u64);
        assert_eq!(s.quiz.current, 1);
        assert_eq!(s.quiz.rounds[0].given, Some(None));
        assert_eq!(s.quiz.time_left_ms, ROUND_MS);
    }

    #[test]
    fn rejects_illegal_moves_without_changing_state() {
        let mut s = setup(6);
        assert!(CountTheCubes::apply_move(&mut s, "someone-else", QuizMove::Answer { value: 1 }).is_err());
        assert!(CountTheCubes::apply_move(&mut s, P, QuizMove::Answer { value: -1 }).is_err());
        assert_eq!(s.quiz.current, 0);
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

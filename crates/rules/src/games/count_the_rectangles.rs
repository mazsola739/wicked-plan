//! Count the Rectangles (Mini, solo). Rules page: docs/games/count-the-rectangles.md
//!
//! Five rounds. Each round shows a figure of straight lines on a grid; type how
//! many rectangles (squares included) its lines draw, within 30 seconds.
//!
//! The figure lives on a lattice of (cols + 1) × (rows + 1) points. It is a set
//! of unit segments: `h[y][x]` joins (x, y)–(x+1, y), `v[y][x]` joins
//! (x, y)–(x, y+1). A rectangle counts when all four of its sides are drawn.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult};
use crate::quiz::{QuizPuzzle, QuizState, QuizView};
use crate::rng::Rng;

pub const ROUND_MS: i64 = 30_000;
pub const MAX_ANSWER: i64 = 999;
/// Time left on the last (biggest) figure for a replay moment.
pub const FAST_MS: i64 = 15_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RectanglesPuzzle {
    pub cols: usize,
    pub rows: usize,
    /// (rows + 1) rows of `cols` horizontal unit segments.
    pub h: Vec<Vec<bool>>,
    /// `rows` rows of (cols + 1) vertical unit segments.
    pub v: Vec<Vec<bool>>,
}

/// (cols, rows, extra lines) for rounds 1 → 5.
pub const DIFFICULTIES: [(usize, usize, usize); 5] = [(2, 2, 1), (3, 2, 2), (3, 3, 3), (4, 3, 4), (4, 4, 5)];

impl RectanglesPuzzle {
    fn empty(cols: usize, rows: usize) -> Self {
        let mut p = Self { cols, rows, h: vec![vec![false; cols]; rows + 1], v: vec![vec![false; cols + 1]; rows] };
        p.h_line(0, 0, cols);
        p.h_line(rows, 0, cols);
        p.v_line(0, 0, rows);
        p.v_line(cols, 0, rows);
        p
    }

    fn h_line(&mut self, y: usize, x1: usize, x2: usize) {
        for x in x1..x2 {
            self.h[y][x] = true;
        }
    }

    fn v_line(&mut self, x: usize, y1: usize, y2: usize) {
        for y in y1..y2 {
            self.v[y][x] = true;
        }
    }

    fn h_full(&self, y: usize, x1: usize, x2: usize) -> bool {
        (x1..x2).all(|x| self.h[y][x])
    }

    fn v_full(&self, x: usize, y1: usize, y2: usize) -> bool {
        (y1..y2).all(|y| self.v[y][x])
    }

    /// The answer: every (x1, y1)–(x2, y2) whose four sides are drawn.
    pub fn count(&self) -> i64 {
        let mut n = 0;
        for x1 in 0..self.cols {
            for x2 in x1 + 1..=self.cols {
                for y1 in 0..self.rows {
                    for y2 in y1 + 1..=self.rows {
                        if self.h_full(y1, x1, x2) && self.h_full(y2, x1, x2) && self.v_full(x1, y1, y2) && self.v_full(x2, y1, y2) {
                            n += 1;
                        }
                    }
                }
            }
        }
        n
    }

    /// `level` is 1-based. Each extra line runs between two points of the
    /// figure's frame or of an earlier line, so nothing dangles in the air.
    pub fn generate(rng: &mut Rng, level: usize) -> Self {
        let (cols, rows, lines) = DIFFICULTIES[level.clamp(1, DIFFICULTIES.len()) - 1];
        let mut best = Self::empty(cols, rows);
        for _ in 0..20 {
            let mut p = Self::empty(cols, rows);
            for _ in 0..lines {
                if rng.next_f64() < 0.5 && rows > 1 {
                    let y = rng.int(1, rows as i64 - 1) as usize;
                    let (a, b) = Self::span(rng, cols);
                    p.h_line(y, a, b);
                } else if cols > 1 {
                    let x = rng.int(1, cols as i64 - 1) as usize;
                    let (a, b) = Self::span(rng, rows);
                    p.v_line(x, a, b);
                }
            }
            if p.count() > best.count() {
                best = p;
            }
            // Enough to count, few enough to count in time.
            if (3..=60).contains(&best.count()) && best.count() as usize >= level * 3 {
                break;
            }
        }
        best
    }

    /// Mostly full-length lines, sometimes a shorter piece.
    fn span(rng: &mut Rng, len: usize) -> (usize, usize) {
        if len < 2 || rng.next_f64() < 0.6 {
            return (0, len);
        }
        let a = rng.int(0, len as i64 - 1) as usize;
        let b = rng.int(a as i64 + 1, len as i64) as usize;
        (a, b)
    }
}

impl QuizPuzzle for RectanglesPuzzle {
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
pub struct RectanglesState {
    pub quiz: QuizState<RectanglesPuzzle>,
    /// Replay moments: the last (biggest) figure counted fast, and all five figures right.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum QuizMove {
    Answer { value: i64 },
}

pub struct CountTheRectangles;

impl Game for CountTheRectangles {
    type State = RectanglesState;
    type Move = QuizMove;
    type View = QuizView<RectanglesPuzzle>;

    fn info() -> GameInfo {
        GameInfo {
            id: "count-the-rectangles",
            kind: "mini-solo",
            difficulty: "easy",
            tags: &["numbers", "speed"],
            min_players: 1,
            max_players: 1,
            name: "Count the Rectangles",
            short: "How many rectangles do the lines draw? Five rounds, 30 seconds each.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<RectanglesState, String> {
        let mut rng = Rng::new(seed);
        let puzzles = (1..=DIFFICULTIES.len()).map(|level| RectanglesPuzzle::generate(&mut rng, level)).collect();
        Ok(RectanglesState { quiz: QuizState::new(players, puzzles, ROUND_MS, Self::info().name)?, moments: Moments::default() })
    }

    fn apply_move(s: &mut RectanglesState, player: &str, mv: QuizMove) -> Result<(), String> {
        let QuizMove::Answer { value } = mv;
        let left = s.quiz.time_left_ms;
        s.quiz.answer(player, value)?;
        // Replay moments, after the last round (a timeout is never right, so only an answer can make one).
        let q = &s.quiz;
        let last = &q.rounds[q.current - 1];
        if q.over && last.correct {
            if left >= FAST_MS {
                s.moments.push(&q.player, format!("{} rectangles counted with {} s to spare", last.puzzle.count(), left / 1000));
            }
            if q.rounds.iter().all(|r| r.correct) {
                s.moments.push(&q.player, "All five figures counted right");
            }
        }
        Ok(())
    }

    fn tick(s: &mut RectanglesState, ms: u64) {
        s.quiz.tick(ms);
    }

    fn view_for(s: &RectanglesState, _player: &str) -> QuizView<RectanglesPuzzle> {
        s.quiz.view()
    }

    fn is_over(s: &RectanglesState) -> bool {
        s.quiz.over
    }

    fn results(s: &RectanglesState) -> Vec<PlayerResult> {
        s.quiz.results()
    }

    fn bot_move(s: &RectanglesState, _player: &str, seed: u64) -> QuizMove {
        let mut rng = Rng::new(seed);
        let answer = s.quiz.current_solution().unwrap_or(0);
        let value = if rng.next_f64() < 0.7 { answer } else { (answer + rng.pick(&[-3, -1, 1, 2])).max(0) };
        QuizMove::Answer { value }
    }

    fn moments(s: &RectanglesState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub const READY: bool = true;
pub type Rules = CountTheRectangles;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games;

    #[test]
    fn counts_known_figures() {
        // A bare frame is one rectangle.
        assert_eq!(RectanglesPuzzle::empty(3, 2).count(), 1);
        // A full 2×2 grid: 9 rectangles (4 small, 4 halves, 1 whole).
        let mut p = RectanglesPuzzle::empty(2, 2);
        p.h_line(1, 0, 2);
        p.v_line(1, 0, 2);
        assert_eq!(p.count(), 9);
        // A full n×m grid has C(n+1,2)·C(m+1,2) rectangles: 3×3 → 36.
        let mut g = RectanglesPuzzle::empty(3, 3);
        for i in 1..3 {
            g.h_line(i, 0, 3);
            g.v_line(i, 0, 3);
        }
        assert_eq!(g.count(), 36);
        // A half line adds no rectangle on its own.
        let mut half = RectanglesPuzzle::empty(2, 2);
        half.h_line(1, 0, 1);
        assert_eq!(half.count(), 1);
    }

    #[test]
    fn generator_is_sized_deterministic_and_countable() {
        for seed in 1..=500 {
            let mut rng = Rng::new(seed);
            for (i, &(cols, rows, _)) in DIFFICULTIES.iter().enumerate() {
                let p = RectanglesPuzzle::generate(&mut rng, i + 1);
                assert_eq!((p.cols, p.rows), (cols, rows));
                assert_eq!(p.h.len(), rows + 1);
                assert_eq!(p.v.len(), rows);
                assert!((1..=MAX_ANSWER).contains(&p.count()));
            }
        }
        assert_eq!(RectanglesPuzzle::generate(&mut Rng::new(5), 4), RectanglesPuzzle::generate(&mut Rng::new(5), 4));
    }

    const P: &str = "p1";

    #[test]
    fn a_fast_last_count_and_a_perfect_run_are_replay_moments() {
        let mut s = CountTheRectangles::setup(&[P.to_string()], 1).unwrap();
        while !s.quiz.over {
            let a = s.quiz.current_solution().unwrap();
            CountTheRectangles::tick(&mut s, 14_500);
            CountTheRectangles::apply_move(&mut s, P, QuizMove::Answer { value: a }).unwrap();
        }
        let n = s.quiz.rounds[4].puzzle.count();
        let m: Vec<(String, String)> = CountTheRectangles::moments(&s, P).into_iter().map(|m| (m.by, m.text)).collect();
        assert_eq!(m, vec![
            (P.to_string(), format!("{n} rectangles counted with 15 s to spare")),
            (P.to_string(), "All five figures counted right".to_string()),
        ]);
        // A miss, then a slow last count: no moment.
        let mut s = CountTheRectangles::setup(&[P.to_string()], 1).unwrap();
        CountTheRectangles::apply_move(&mut s, P, QuizMove::Answer { value: 0 }).unwrap();
        while !s.quiz.over {
            let a = s.quiz.current_solution().unwrap();
            CountTheRectangles::tick(&mut s, 15_500);
            CountTheRectangles::apply_move(&mut s, P, QuizMove::Answer { value: a }).unwrap();
        }
        assert!(CountTheRectangles::moments(&s, P).is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<CountTheRectangles>(300, 1, 200);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 2, "too many in one game: {stats:?}");
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<CountTheRectangles>(1000, 200);
    }
}

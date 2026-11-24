//! Equation Pyramid (Prison, played solo here). Rules page: docs/games/equation-target.md
//!
//! Five rounds. Ten number cards (1–9) and a target are shown. Pick three
//! different cards and two operators (+ − ×); the expression is worked out
//! left to right, ((a ∘ b) ∘ c), and must equal the target.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult};
use crate::quiz::{QuizPuzzle, QuizState, QuizView};
use crate::rng::Rng;

pub const CARDS: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
    #[serde(rename = "+")]
    Add,
    #[serde(rename = "-")]
    Sub,
    #[serde(rename = "*")]
    Mul,
}

impl Op {
    pub const ALL: [Op; 3] = [Op::Add, Op::Sub, Op::Mul];
    fn apply(self, a: i64, b: i64) -> i64 {
        match self {
            Op::Add => a + b,
            Op::Sub => a - b,
            Op::Mul => a * b,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquationAnswer {
    /// Card indices, in the order they are used.
    pub picks: [usize; 3],
    pub ops: [Op; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EquationPuzzle {
    pub cards: Vec<i64>,
    pub target: i64,
    /// Kept on the server only; it could be brute-forced, but the phone does not need it.
    #[serde(skip)]
    pub known: Option<EquationAnswer>,
}

impl EquationPuzzle {
    pub fn eval(&self, a: &EquationAnswer) -> Option<i64> {
        let [i, j, k] = a.picks;
        if i == j || j == k || i == k || [i, j, k].iter().any(|&n| n >= self.cards.len()) {
            return None;
        }
        Some(a.ops[1].apply(a.ops[0].apply(self.cards[i], self.cards[j]), self.cards[k]))
    }

    /// Every right answer (used by the tests and to keep targets fair).
    pub fn solutions(&self) -> Vec<EquationAnswer> {
        let n = self.cards.len();
        let mut out = Vec::new();
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    for o1 in Op::ALL {
                        for o2 in Op::ALL {
                            let a = EquationAnswer { picks: [i, j, k], ops: [o1, o2] };
                            if self.eval(&a) == Some(self.target) {
                                out.push(a);
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// Later rounds use bigger targets, which need × more often.
    pub fn generate(rng: &mut Rng, level: usize) -> Self {
        let (lo, hi) = [(5, 20), (10, 30), (15, 45), (20, 70), (30, 99)][level.clamp(1, 5) - 1];
        loop {
            let cards: Vec<i64> = (0..CARDS).map(|_| rng.int(1, 9)).collect();
            let mut idx: Vec<usize> = (0..CARDS).collect();
            rng.shuffle(&mut idx);
            let a = EquationAnswer { picks: [idx[0], idx[1], idx[2]], ops: [*rng.pick(&Op::ALL), *rng.pick(&Op::ALL)] };
            let mut p = Self { cards, target: 0, known: None };
            let Some(target) = p.eval(&a) else { continue };
            if (lo..=hi).contains(&target) {
                p.target = target;
                p.known = Some(a);
                return p;
            }
        }
    }
}

impl QuizPuzzle for EquationPuzzle {
    type Answer = EquationAnswer;
    fn is_correct(&self, answer: &EquationAnswer) -> bool {
        self.eval(answer) == Some(self.target)
    }
    fn solution(&self) -> EquationAnswer {
        self.known.clone().unwrap_or_else(|| self.solutions().remove(0))
    }
    fn validate(&self, answer: &EquationAnswer) -> Result<(), String> {
        self.eval(answer).map(|_| ()).ok_or_else(|| "Pick three different cards".into())
    }
}

/// The shared quiz rounds plus the replay moments.
#[derive(Clone, Debug)]
pub struct TargetState {
    pub quiz: QuizState<EquationPuzzle>,
    /// Replay moment: all five targets hit.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum QuizMove {
    Answer { value: EquationAnswer },
}

pub struct EquationTarget;

impl Game for EquationTarget {
    type State = TargetState;
    type Move = QuizMove;
    type View = QuizView<EquationPuzzle>;

    fn info() -> GameInfo {
        GameInfo {
            id: "equation-target",
            kind: "prison",
            difficulty: "medium",
            tags: &["logic-puzzles"],
            min_players: 1,
            max_players: 1,
            name: "Equation Pyramid",
            short: "Pick three of ten number cards and two operators to hit the target.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<TargetState, String> {
        let mut rng = Rng::new(seed);
        let puzzles = (1..=5).map(|level| EquationPuzzle::generate(&mut rng, level)).collect();
        Ok(TargetState { quiz: QuizState::new(players, puzzles, 0, Self::info().name)?, moments: Moments::default() })
    }

    fn apply_move(s: &mut TargetState, player: &str, mv: QuizMove) -> Result<(), String> {
        let QuizMove::Answer { value } = mv;
        s.quiz.answer(player, value)?;
        // The replay moment, after the last round.
        let q = &s.quiz;
        if q.over && q.rounds.iter().all(|r| r.correct) {
            s.moments.push(&q.player, "All five targets hit");
        }
        Ok(())
    }

    fn view_for(s: &TargetState, _player: &str) -> QuizView<EquationPuzzle> {
        s.quiz.view()
    }

    fn is_over(s: &TargetState) -> bool {
        s.quiz.over
    }

    fn results(s: &TargetState) -> Vec<PlayerResult> {
        s.quiz.results()
    }

    fn bot_move(s: &TargetState, _player: &str, seed: u64) -> QuizMove {
        let mut rng = Rng::new(seed);
        let right = s.quiz.current_solution().expect("round in progress");
        let value = if rng.next_f64() < 0.7 {
            right
        } else {
            // A plausible miss: same cards, other operators.
            EquationAnswer { picks: right.picks, ops: [*rng.pick(&Op::ALL), *rng.pick(&Op::ALL)] }
        };
        QuizMove::Answer { value }
    }

    fn moments(s: &TargetState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub type Rules = EquationTarget;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games;

    #[test]
    fn evaluates_left_to_right() {
        let p = EquationPuzzle { cards: vec![2, 3, 4], target: 20, known: None };
        let a = EquationAnswer { picks: [0, 1, 2], ops: [Op::Add, Op::Mul] };
        assert_eq!(p.eval(&a), Some(20)); // (2 + 3) × 4
        assert!(p.is_correct(&a));
        assert_eq!(p.eval(&EquationAnswer { picks: [0, 0, 2], ops: [Op::Add, Op::Add] }), None);
        assert_eq!(p.eval(&EquationAnswer { picks: [0, 1, 9], ops: [Op::Add, Op::Add] }), None);
    }

    #[test]
    fn every_generated_target_is_reachable_and_in_range() {
        for seed in 1..=300 {
            let mut rng = Rng::new(seed);
            for level in 1..=5 {
                let p = EquationPuzzle::generate(&mut rng, level);
                assert_eq!(p.cards.len(), CARDS);
                assert!(p.cards.iter().all(|c| (1..=9).contains(c)));
                assert!(p.is_correct(&p.solution()));
                assert!(!p.solutions().is_empty());
            }
        }
    }

    #[test]
    fn the_phone_never_gets_the_known_answer() {
        let p = EquationPuzzle::generate(&mut Rng::new(1), 3);
        let v = serde_json::to_value(&p).unwrap();
        assert!(v.get("known").is_none());
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<EquationTarget>(1000, 200);
    }

    fn answer_right(s: &mut TargetState) {
        let value = s.quiz.current_solution().unwrap();
        EquationTarget::apply_move(s, "p1", QuizMove::Answer { value }).unwrap();
    }

    /// The right cards with operators that miss the target.
    fn answer_wrong(s: &mut TargetState) {
        let right = s.quiz.current_solution().unwrap();
        let puzzle = &s.quiz.rounds[s.quiz.current].puzzle;
        let value = Op::ALL
            .iter()
            .flat_map(|&a| Op::ALL.iter().map(move |&b| EquationAnswer { picks: right.picks, ops: [a, b] }))
            .find(|v| !puzzle.is_correct(v))
            .unwrap();
        EquationTarget::apply_move(s, "p1", QuizMove::Answer { value }).unwrap();
    }

    #[test]
    fn a_round_waits_for_the_answer() {
        let ids = vec!["p1".to_string()];
        let mut s = EquationTarget::setup(&ids, 7).unwrap();
        EquationTarget::tick(&mut s, 3_600_000);
        assert_eq!((s.quiz.current, s.quiz.time_left_ms), (0, 0));
        assert!(!s.quiz.over);
        answer_right(&mut s);
        assert_eq!((s.quiz.current, s.quiz.rounds[0].points), (1, 100));
    }

    #[test]
    fn a_perfect_run_is_a_replay_moment() {
        let ids = vec!["p1".to_string()];
        // Five right answers.
        let mut s = EquationTarget::setup(&ids, 7).unwrap();
        for _ in 0..4 {
            answer_right(&mut s);
            assert!(EquationTarget::moments(&s, "p1").is_empty(), "only the end makes moments");
        }
        answer_right(&mut s);
        let m = EquationTarget::moments(&s, "p1");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "All five targets hit"));
        assert_eq!(EquationTarget::results(&s)[0].points, 500);
        // A missed round, then four right answers: nothing.
        let mut s = EquationTarget::setup(&ids, 7).unwrap();
        answer_wrong(&mut s);
        for _ in 0..4 {
            answer_right(&mut s);
        }
        assert!(s.quiz.over);
        assert!(EquationTarget::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        // The only moment is a perfect run. The bot is right about 75% of a round (a random miss can
        // still hit), so about 0.75^5 ≈ 23% of games have it.
        let stats = crate::game::testing::moment_stats::<EquationTarget>(300, 1, 200);
        assert!(stats.with_any * 5 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 2, "too many in one game: {stats:?}");
    }
}

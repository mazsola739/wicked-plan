//! Shared engine for the timed solo puzzles that are "rounds of one question":
//! Count the Cubes, Count the Rectangles, Equation Target.
//! Each round shows a puzzle; answer before the timer runs out. A correct
//! answer scores `speed_points`; a wrong one or a timeout scores 0.

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::game::{PlayerResult, speed_points};

pub trait QuizPuzzle: Clone + std::fmt::Debug + Send + Serialize + 'static {
    /// What the player sends. Serialized in the history once the round is over.
    type Answer: Clone + std::fmt::Debug + PartialEq + Send + Serialize + DeserializeOwned + 'static;
    fn is_correct(&self, answer: &Self::Answer) -> bool;
    /// One right answer, shown after the round and used by the bot.
    fn solution(&self) -> Self::Answer;
    /// Rejects malformed answers (out of range numbers and so on).
    fn validate(&self, _answer: &Self::Answer) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct QuizRound<P: QuizPuzzle> {
    pub puzzle: P,
    /// None: not played yet. Some(None): time ran out.
    pub given: Option<Option<P::Answer>>,
    pub correct: bool,
    pub points: i64,
}

#[derive(Clone, Debug)]
pub struct QuizState<P: QuizPuzzle> {
    pub player: String,
    pub rounds: Vec<QuizRound<P>>,
    pub current: usize,
    pub time_left_ms: i64,
    pub round_ms: i64,
    pub over: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizHistory<P: QuizPuzzle> {
    pub puzzle: P,
    pub given: Option<P::Answer>,
    pub solution: P::Answer,
    pub correct: bool,
    pub points: i64,
}

/// The current round's puzzle carries no solution: `QuizPuzzle` types keep
/// the answer out of their serialized form.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizView<P: QuizPuzzle> {
    pub round: usize,
    pub rounds: usize,
    pub puzzle: Option<P>,
    pub time_left_ms: i64,
    pub round_time_ms: i64,
    pub history: Vec<QuizHistory<P>>,
    pub score: i64,
    pub over: bool,
}

impl<P: QuizPuzzle> QuizState<P> {
    pub fn new(players: &[String], puzzles: Vec<P>, round_ms: i64, name: &str) -> Result<Self, String> {
        let [player] = players else {
            return Err(format!("{name} is a solo game"));
        };
        let rounds = puzzles.into_iter().map(|puzzle| QuizRound { puzzle, given: None, correct: false, points: 0 }).collect();
        Ok(Self { player: player.clone(), rounds, current: 0, time_left_ms: round_ms, round_ms, over: false })
    }

    fn finish_round(&mut self, given: Option<P::Answer>) {
        let time_left = self.time_left_ms;
        let round_ms = self.round_ms;
        let round = &mut self.rounds[self.current];
        round.correct = given.as_ref().is_some_and(|a| round.puzzle.is_correct(a));
        round.points = if round.correct { speed_points(time_left, round_ms) } else { 0 };
        round.given = Some(given);
        self.current += 1;
        self.over = self.current >= self.rounds.len();
        self.time_left_ms = if self.over { 0 } else { self.round_ms };
    }

    pub fn answer(&mut self, player: &str, answer: P::Answer) -> Result<(), String> {
        if self.over {
            return Err("Game is over".into());
        }
        if player != self.player {
            return Err("Not your game".into());
        }
        self.rounds[self.current].puzzle.validate(&answer)?;
        self.finish_round(Some(answer));
        Ok(())
    }

    pub fn tick(&mut self, ms: u64) {
        if self.over {
            return;
        }
        self.time_left_ms -= ms as i64;
        if self.time_left_ms <= 0 {
            self.time_left_ms = 0;
            self.finish_round(None);
        }
    }

    pub fn score(&self) -> i64 {
        self.rounds.iter().map(|r| r.points).sum()
    }

    pub fn view(&self) -> QuizView<P> {
        QuizView {
            round: (self.current + 1).min(self.rounds.len()),
            rounds: self.rounds.len(),
            puzzle: (!self.over).then(|| self.rounds[self.current].puzzle.clone()),
            time_left_ms: self.time_left_ms,
            round_time_ms: self.round_ms,
            history: self
                .rounds
                .iter()
                .filter_map(|r| {
                    r.given.as_ref().map(|given| QuizHistory {
                        puzzle: r.puzzle.clone(),
                        given: given.clone(),
                        solution: r.puzzle.solution(),
                        correct: r.correct,
                        points: r.points,
                    })
                })
                .collect(),
            score: self.score(),
            over: self.over,
        }
    }

    pub fn results(&self) -> Vec<PlayerResult> {
        vec![PlayerResult { player_id: self.player.clone(), rank: 1, points: self.score() }]
    }

    pub fn current_solution(&self) -> Option<P::Answer> {
        self.rounds.get(self.current).map(|r| r.puzzle.solution())
    }
}

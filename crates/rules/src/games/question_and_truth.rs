//! Question and Truth (final duel, 2 players). Rules page: docs/games/question-and-truth.md
//!
//! Each player has a secret row of the cards 1–8. Players take turns spending
//! tokens on questions from a fixed menu, then guess the opponent's whole row.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const CARDS: usize = 8;
pub const TOKENS: u32 = 10;

/// A row: `row[i]` is the card (1–8) at position i + 1.
pub type Row = [u8; CARDS];

/// Every possible row, made once.
fn all_rows() -> &'static [Row] {
    static ALL: OnceLock<Vec<Row>> = OnceLock::new();
    ALL.get_or_init(|| {
        fn build(row: &mut Row, used: &mut [bool; CARDS], i: usize, out: &mut Vec<Row>) {
            if i == CARDS {
                out.push(*row);
                return;
            }
            for c in 0..CARDS {
                if !used[c] {
                    used[c] = true;
                    row[i] = c as u8 + 1;
                    build(row, used, i + 1, out);
                    used[c] = false;
                }
            }
        }
        let mut out = Vec::with_capacity(40_320);
        build(&mut [0; CARDS], &mut [false; CARDS], 0, &mut out);
        out
    })
}

fn pos_of(row: &Row, card: u8) -> u8 {
    row.iter().position(|&c| c == card).map_or(0, |p| p as u8 + 1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Question {
    CardAt { pos: u8 },
    WhereIs { card: u8 },
    Sum { a: u8, b: u8 },
    LeftOf { a: u8, b: u8 },
    Bigger { a: u8, b: u8 },
    LeftHalf { card: u8 },
}

impl Question {
    pub fn cost(self) -> u32 {
        match self {
            Question::CardAt { .. } | Question::WhereIs { .. } => 3,
            Question::Sum { .. } => 2,
            Question::LeftOf { .. } | Question::Bigger { .. } | Question::LeftHalf { .. } => 1,
        }
    }

    fn valid(self) -> bool {
        let ok = |n: u8| (1..=CARDS as u8).contains(&n);
        match self {
            Question::CardAt { pos: n } | Question::WhereIs { card: n } | Question::LeftHalf { card: n } => ok(n),
            Question::Sum { a, b } | Question::LeftOf { a, b } | Question::Bigger { a, b } => ok(a) && ok(b) && a != b,
        }
    }

    /// The true answer for `row`: a number, or 1 / 0 for yes / no.
    pub fn answer(self, row: &Row) -> u8 {
        let at = |p: u8| row[p as usize - 1];
        match self {
            Question::CardAt { pos } => at(pos),
            Question::WhereIs { card } => pos_of(row, card),
            Question::Sum { a, b } => at(a) + at(b),
            Question::LeftOf { a, b } => u8::from(pos_of(row, a) < pos_of(row, b)),
            Question::Bigger { a, b } => u8::from(at(a) > at(b)),
            Question::LeftHalf { card } => u8::from(pos_of(row, card) <= 4),
        }
    }

    fn yes_no(self) -> bool {
        matches!(self, Question::LeftOf { .. } | Question::Bigger { .. } | Question::LeftHalf { .. })
    }

    pub fn text(self) -> String {
        match self {
            Question::CardAt { pos } => format!("Which card is at position {pos}?"),
            Question::WhereIs { card } => format!("Where is card {card}?"),
            Question::Sum { a, b } => format!("Sum of the cards at positions {a} and {b}?"),
            Question::LeftOf { a, b } => format!("Is card {a} left of card {b}?"),
            Question::Bigger { a, b } => format!("Is the card at position {a} bigger than at position {b}?"),
            Question::LeftHalf { card } => format!("Is card {card} in the left half?"),
        }
    }

    fn answer_text(self, answer: u8) -> String {
        match self {
            _ if self.yes_no() => (if answer == 1 { "Yes" } else { "No" }).into(),
            Question::WhereIs { .. } => format!("Position {answer}"),
            Question::CardAt { .. } => format!("Card {answer}"),
            _ => answer.to_string(),
        }
    }
}

/// Every question on the menu.
fn menu() -> Vec<Question> {
    let mut out = Vec::new();
    for n in 1..=CARDS as u8 {
        out.extend([Question::CardAt { pos: n }, Question::WhereIs { card: n }, Question::LeftHalf { card: n }]);
        for m in n + 1..=CARDS as u8 {
            out.extend([Question::Sum { a: n, b: m }, Question::LeftOf { a: n, b: m }, Question::Bigger { a: n, b: m }]);
        }
    }
    out
}

#[derive(Clone, Debug)]
struct Seat {
    id: String,
    row: Row,
    tokens: u32,
    log: Vec<(Question, u8)>,
    guess: Option<Row>,
    /// Every row of the opponent that fits the answers this seat has had (for its bot).
    candidates: Vec<Row>,
}

#[derive(Clone, Debug)]
pub struct TruthState {
    seats: [Seat; 2],
    turn: usize,
    /// Replay moments: a row cracked with tokens to spare; a lucky whole-row guess.
    pub moments: Moments,
}

impl TruthState {
    fn seat(&self, player: &str) -> Option<usize> {
        self.seats.iter().position(|s| s.id == player)
    }
    fn over(&self) -> bool {
        self.seats.iter().all(|s| s.guess.is_some())
    }
    fn next_turn(&mut self) {
        if self.seats[1 - self.turn].guess.is_none() {
            self.turn = 1 - self.turn;
        }
    }
    /// How many positions of `i`'s guess are right.
    fn right(&self, i: usize) -> usize {
        let theirs = &self.seats[1 - i].row;
        self.seats[i].guess.map_or(0, |g| g.iter().zip(theirs).filter(|(a, b)| a == b).count())
    }
    fn points(&self, i: usize) -> i64 {
        if self.seats[i].guess.is_none() {
            return 0;
        }
        let right = self.right(i) as i64;
        100 * right + if right == CARDS as i64 { 100 } else { 0 } + 10 * i64::from(self.seats[i].tokens)
    }
}

/// The candidate row that agrees most, position by position, with the others.
fn likeliest(candidates: &[Row]) -> Row {
    let step = (candidates.len() / 400).max(1);
    let mut freq = [[0u32; CARDS + 1]; CARDS];
    for row in candidates.iter().step_by(step) {
        for (i, &c) in row.iter().enumerate() {
            freq[i][c as usize] += 1;
        }
    }
    let score = |row: &Row| row.iter().enumerate().map(|(i, &c)| freq[i][c as usize]).sum::<u32>();
    candidates.iter().step_by(step).copied().max_by_key(score).unwrap_or([1, 2, 3, 4, 5, 6, 7, 8])
}

/// The affordable question with the most expected information per token, if any tells anything.
pub fn best_question(candidates: &[Row], tokens: u32, rng: &mut Rng) -> Option<Question> {
    let step = (candidates.len() / 120).max(1);
    let sample: Vec<&Row> = candidates.iter().step_by(step).collect();
    let n = sample.len() as f64;
    let mut questions = menu();
    rng.shuffle(&mut questions);
    let mut best: Option<(Question, f64)> = None;
    for q in questions.into_iter().filter(|q| q.cost() <= tokens) {
        let mut counts = [0u32; 20];
        for row in &sample {
            counts[q.answer(row) as usize] += 1;
        }
        let entropy: f64 = counts.iter().filter(|&&c| c > 0).map(|&c| -(f64::from(c) / n) * (f64::from(c) / n).log2()).sum();
        let value = entropy / f64::from(q.cost());
        if entropy > 1e-9 && best.is_none_or(|(_, v)| value > v) {
            best = Some((q, value));
        }
    }
    best.map(|(q, _)| q)
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TruthMove {
    Ask { question: Question },
    Guess { order: Vec<u8> },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub question: String,
    pub answer: String,
    pub cost: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TruthView {
    pub row: Row,
    pub tokens: u32,
    pub their_tokens: u32,
    pub opponent: String,
    pub your_turn: bool,
    pub guessed: bool,
    pub they_guessed: bool,
    /// Your questions about their row.
    pub log: Vec<LogLine>,
    /// Their questions about your row.
    pub their_log: Vec<LogLine>,
    pub over: bool,
    /// After the end: their row, your guess, how many were right.
    pub their_row: Option<Row>,
    pub your_guess: Option<Row>,
    pub right: Option<usize>,
}

pub struct QuestionAndTruth;

impl Game for QuestionAndTruth {
    type State = TruthState;
    type Move = TruthMove;
    type View = TruthView;

    fn info() -> GameInfo {
        GameInfo {
            id: "question-and-truth",
            kind: "final-duel",
            difficulty: "medium",
            tags: &["bluff-deduction"],
            min_players: 2,
            max_players: 2,
            name: "Question and Truth",
            short: "Spend tokens on questions to work out the order of your opponent's eight secret cards.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<TruthState, String> {
        let [a, b] = players else { return Err("Question and Truth is for 2 players".into()) };
        let mut rng = Rng::new(seed);
        let mut seat = |id: &String| {
            let mut row: Row = [1, 2, 3, 4, 5, 6, 7, 8];
            rng.shuffle(&mut row);
            Seat { id: id.clone(), row, tokens: TOKENS, log: Vec::new(), guess: None, candidates: all_rows().to_vec() }
        };
        let seats = [seat(a), seat(b)];
        Ok(TruthState { seats, turn: 0, moments: Moments::default() })
    }

    fn apply_move(s: &mut TruthState, player: &str, mv: TruthMove) -> Result<(), String> {
        if s.over() {
            return Err("Game is over".into());
        }
        let i = s.seat(player).ok_or("Not in this game")?;
        if i != s.turn {
            return Err("Not your turn".into());
        }
        match mv {
            TruthMove::Ask { question } => {
                if !question.valid() {
                    return Err("That question is not on the menu".into());
                }
                if question.cost() > s.seats[i].tokens {
                    return Err("Not enough tokens".into());
                }
                let answer = question.answer(&s.seats[1 - i].row);
                let me = &mut s.seats[i];
                me.tokens -= question.cost();
                me.log.push((question, answer));
                me.candidates.retain(|r| question.answer(r) == answer);
                // Replay moment: only one row fits the answers and tokens are left. Every answer is
                // public, so anyone could see it; the row itself stays secret.
                if me.candidates.len() == 1 && me.tokens > 0 {
                    let (by, tokens) = (me.id.clone(), me.tokens);
                    let plural = if tokens == 1 { "" } else { "s" };
                    s.moments.push(&by, format!("Row cracked with {tokens} token{plural} to spare"));
                }
            }
            TruthMove::Guess { order } => {
                let mut sorted = order.clone();
                sorted.sort_unstable();
                if sorted != [1, 2, 3, 4, 5, 6, 7, 8] {
                    return Err("Put each card 1–8 in the row once".into());
                }
                s.seats[i].guess = Some(order.try_into().expect("8 cards"));
                // Replay moment, once both rows are open at the end: a whole row guessed right while
                // two or more rows still fitted the answers.
                if s.over() {
                    for k in 0..2 {
                        let left = s.seats[k].candidates.len();
                        if s.right(k) == CARDS && left > 1 {
                            let by = s.seats[k].id.clone();
                            s.moments.push(&by, format!("Lucky guess: the right row of {left} left"));
                        }
                    }
                }
            }
        }
        s.next_turn();
        Ok(())
    }

    fn view_for(s: &TruthState, player: &str) -> TruthView {
        let i = s.seat(player).unwrap_or(0);
        let o = 1 - i;
        let lines = |log: &[(Question, u8)]| {
            log.iter().map(|&(q, a)| LogLine { question: q.text(), answer: q.answer_text(a), cost: q.cost() }).collect()
        };
        let over = s.over();
        TruthView {
            row: s.seats[i].row,
            tokens: s.seats[i].tokens,
            their_tokens: s.seats[o].tokens,
            opponent: s.seats[o].id.clone(),
            your_turn: !over && s.turn == i,
            guessed: s.seats[i].guess.is_some(),
            they_guessed: s.seats[o].guess.is_some(),
            log: lines(&s.seats[i].log),
            their_log: lines(&s.seats[o].log),
            over,
            their_row: over.then_some(s.seats[o].row),
            your_guess: if over { s.seats[i].guess } else { None },
            right: over.then(|| s.right(i)),
        }
    }

    fn is_over(s: &TruthState) -> bool {
        s.over()
    }

    fn results(s: &TruthState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = (0..2).map(|i| (s.seats[i].id.clone(), s.points(i))).collect();
        rank_by_score(&scores)
    }

    fn to_move(s: &TruthState) -> Vec<String> {
        if s.over() { Vec::new() } else { vec![s.seats[s.turn].id.clone()] }
    }

    fn moments(s: &TruthState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &TruthState, player: &str, seed: u64) -> TruthMove {
        let me = &s.seats[s.seat(player).unwrap_or(0)];
        let guess = || TruthMove::Guess { order: likeliest(&me.candidates).to_vec() };
        if me.candidates.len() <= 1 {
            return guess();
        }
        match best_question(&me.candidates, me.tokens, &mut Rng::new(seed)) {
            Some(question) => TruthMove::Ask { question },
            None => guess(),
        }
    }
}

pub type Rules = QuestionAndTruth;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn new() -> TruthState {
        let mut s = QuestionAndTruth::setup(&["a".into(), "b".into()], 3).unwrap();
        s.seats[0].row = [1, 2, 3, 4, 5, 6, 7, 8];
        s.seats[1].row = [8, 6, 4, 2, 1, 3, 5, 7];
        s
    }

    fn ask(s: &mut TruthState, p: &str, question: Question) -> Result<(), String> {
        QuestionAndTruth::apply_move(s, p, TruthMove::Ask { question })
    }

    #[test]
    fn there_are_40320_rows_and_108_questions() {
        assert_eq!(all_rows().len(), 40_320);
        assert_eq!(menu().len(), 8 * 3 + 28 * 3);
        assert!(menu().iter().all(|q| q.valid()));
    }

    #[test]
    fn answers_are_true() {
        let row = [8, 6, 4, 2, 1, 3, 5, 7];
        assert_eq!(Question::CardAt { pos: 2 }.answer(&row), 6);
        assert_eq!(Question::WhereIs { card: 1 }.answer(&row), 5);
        assert_eq!(Question::Sum { a: 1, b: 8 }.answer(&row), 15);
        assert_eq!(Question::LeftOf { a: 2, b: 1 }.answer(&row), 1);
        assert_eq!(Question::Bigger { a: 5, b: 6 }.answer(&row), 0);
        assert_eq!(Question::LeftHalf { card: 7 }.answer(&row), 0);
    }

    #[test]
    fn asking_costs_tokens_and_passes_the_turn() {
        let mut s = new();
        assert!(ask(&mut s, "b", Question::CardAt { pos: 1 }).is_err());
        assert!(ask(&mut s, "a", Question::Sum { a: 1, b: 1 }).is_err());
        ask(&mut s, "a", Question::CardAt { pos: 1 }).unwrap();
        assert_eq!(s.seats[0].tokens, TOKENS - 3);
        assert_eq!(s.seats[0].candidates.len(), 5040);
        let v = serde_json::to_value(QuestionAndTruth::view_for(&s, "b")).unwrap();
        assert_eq!(v["theirLog"][0]["answer"], "Card 8");
        assert!(v["theirRow"].is_null());
        assert_eq!(s.turn, 1);
    }

    #[test]
    fn guesses_end_the_game_and_score() {
        let mut s = new();
        QuestionAndTruth::apply_move(&mut s, "a", TruthMove::Guess { order: vec![8, 6, 4, 2, 1, 3, 5, 7] }).unwrap();
        assert!(QuestionAndTruth::apply_move(&mut s, "b", TruthMove::Guess { order: vec![1, 1, 3, 4, 5, 6, 7, 8] }).is_err());
        ask(&mut s, "b", Question::LeftOf { a: 1, b: 2 }).unwrap();
        // a has guessed: b goes on alone.
        assert_eq!(s.turn, 1);
        QuestionAndTruth::apply_move(&mut s, "b", TruthMove::Guess { order: vec![2, 1, 3, 4, 5, 6, 7, 8] }).unwrap();
        assert!(s.over());
        assert_eq!(s.points(0), 900 + 10 * TOKENS as i64);
        assert_eq!(s.points(1), 600 + 10 * (TOKENS as i64 - 1));
    }

    #[test]
    fn the_bot_finds_the_row() {
        let mut s = new();
        s.seats[0].tokens = 40;
        let mut steps = 0;
        while s.seats[0].guess.is_none() {
            if s.turn == 1 {
                QuestionAndTruth::apply_move(&mut s, "b", TruthMove::Guess { order: vec![1, 2, 3, 4, 5, 6, 7, 8] }).unwrap();
                continue;
            }
            let mv = QuestionAndTruth::bot_move(&s, "a", steps);
            QuestionAndTruth::apply_move(&mut s, "a", mv).unwrap();
            steps += 1;
        }
        assert_eq!(s.right(0), CARDS);
    }

    #[test]
    fn no_clock_the_turn_waits_for_you() {
        let mut s = new();
        QuestionAndTruth::tick(&mut s, 600_000);
        assert_eq!(s.turn, 0);
        assert!(s.seats[0].log.is_empty() && s.seats[0].guess.is_none());
        assert_eq!(s.seats[0].tokens, TOKENS);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<QuestionAndTruth>(1000, 2, 500);
    }

    /// Balance: with 10 tokens a good player usually, not always, finds the whole row
    /// (200 games in release: 7.2 positions right on average, 85% fully right).
    #[test]
    fn ten_tokens_are_tight_but_enough() {
        let (mut right, mut full) = (0, 0);
        for seed in 0..20u64 {
            let mut s = QuestionAndTruth::setup(&["a".into(), "b".into()], seed).unwrap();
            let mut k = 0;
            while !s.over() {
                let p = s.seats[s.turn].id.clone();
                let mv = QuestionAndTruth::bot_move(&s, &p, seed * 100 + k);
                QuestionAndTruth::apply_move(&mut s, &p, mv).unwrap();
                k += 1;
            }
            for i in 0..2 {
                right += s.right(i);
                full += usize::from(s.right(i) == CARDS);
            }
        }
        assert!(right >= 40 * 6, "bots find most positions: {right}");
        assert!(full < 40, "not every row is found");
    }

    #[test]
    fn a_row_cracked_with_tokens_left_and_a_lucky_guess_are_replay_moments() {
        let mut s = new();
        let other = [6, 8, 4, 2, 1, 3, 5, 7];
        s.seats[0].candidates = vec![[8, 6, 4, 2, 1, 3, 5, 7], other];
        ask(&mut s, "a", Question::CardAt { pos: 1 }).unwrap();
        let m = QuestionAndTruth::moments(&s, "b");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("a", "Row cracked with 7 tokens to spare"));
        // b guesses its way to the right row while two rows still fit: lucky, shown once the game ends.
        s.seats[1].candidates = vec![[1, 2, 3, 4, 5, 6, 7, 8], [2, 1, 3, 4, 5, 6, 7, 8]];
        QuestionAndTruth::apply_move(&mut s, "b", TruthMove::Guess { order: vec![1, 2, 3, 4, 5, 6, 7, 8] }).unwrap();
        assert_eq!(QuestionAndTruth::moments(&s, "a").len(), 1, "nothing before the end");
        QuestionAndTruth::apply_move(&mut s, "a", TruthMove::Guess { order: vec![8, 6, 4, 2, 1, 3, 5, 7] }).unwrap();
        let m = QuestionAndTruth::moments(&s, "a");
        assert_eq!(m.len(), 2, "a's right guess from one row left is no luck");
        assert_eq!((m[1].by.as_str(), m[1].text.as_str()), ("b", "Lucky guess: the right row of 2 left"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<QuestionAndTruth>(300, 2, 500);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 2, "too many in one game: {stats:?}");
    }
}

//! Secret Number (main game, 4–8 seats). Rules page: docs/games/secret-number.md
//!
//! Everyone has a different secret number 1–100. Trade clues (you learn about
//! them, they learn about you), then guess exact numbers: right exposes the
//! target, wrong tells you higher or lower.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 5;
pub const TRADE_MS: i64 = 20_000;
pub const GUESS_MS: i64 = 25_000;
pub const RIGHT: i64 = 50;
pub const CAUGHT: i64 = -30;
pub const WRONG: i64 = -5;
pub const SAFE: i64 = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clue {
    Even(bool),
    /// The number is greater than `x` (or, with `false`, at most `x`).
    Above(i64, bool),
    Tens(i64),
    Last(i64),
    Div3(bool),
}

impl Clue {
    pub fn about(kind: usize, v: i64, rng: &mut Rng) -> Clue {
        match kind {
            0 => Clue::Even(v % 2 == 0),
            1 => {
                let x = 5 * rng.int(2, 18);
                Clue::Above(x, v > x)
            }
            2 => Clue::Tens(v / 10),
            3 => Clue::Last(v % 10),
            _ => Clue::Div3(v % 3 == 0),
        }
    }

    pub fn holds(&self, v: i64) -> bool {
        match *self {
            Clue::Even(e) => (v % 2 == 0) == e,
            Clue::Above(x, yes) => (v > x) == yes,
            Clue::Tens(d) => v / 10 == d,
            Clue::Last(d) => v % 10 == d,
            Clue::Div3(yes) => (v % 3 == 0) == yes,
        }
    }

    pub fn text(&self) -> String {
        match *self {
            Clue::Even(true) => "even".into(),
            Clue::Even(false) => "odd".into(),
            Clue::Above(x, true) => format!("higher than {x}"),
            Clue::Above(x, false) => format!("{x} or lower"),
            Clue::Tens(d) => format!("tens digit {d}"),
            Clue::Last(d) => format!("last digit {d}"),
            Clue::Div3(true) => "divisible by 3".into(),
            Clue::Div3(false) => "not divisible by 3".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Trade,
    Guess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Skip,
    Trade(usize),
    Pass,
    Guess(usize, i64),
}

#[derive(Clone, Debug)]
pub struct SecretState {
    pub players: Vec<String>,
    pub rng: Rng,
    pub numbers: Vec<i64>,
    pub exposed: Vec<bool>,
    pub scores: Vec<i64>,
    /// Per seat: (about whom, clue).
    pub clues: Vec<Vec<(usize, Clue)>>,
    pub round: usize,
    pub phase: Phase,
    pub actions: Vec<Option<Action>>,
    pub time_left_ms: i64,
    pub log: Vec<String>,
    pub notes: Vec<Vec<String>>,
    pub over: bool,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SecretMove {
    Trade { seat: usize },
    Skip,
    Guess { seat: usize, number: i64 },
    Pass,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Seat {
    pub id: String,
    pub score: i64,
    pub exposed: Option<i64>,
    pub done: bool,
    /// What you know about this player (empty for yourself).
    pub clues: Vec<String>,
    /// How many numbers this player could still have, as far as you know.
    pub possible: usize,
    pub low: i64,
    pub high: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretView {
    pub me: usize,
    pub number: i64,
    pub round: usize,
    pub rounds: usize,
    /// "trade" | "guess" | "over"
    pub phase: &'static str,
    pub seats: Vec<Seat>,
    pub acted: bool,
    pub time_left_ms: i64,
    pub phase_ms: i64,
    pub log: Vec<String>,
    pub notes: Vec<String>,
    /// Every number, after the game.
    pub numbers: Option<Vec<i64>>,
    pub over: bool,
}

impl SecretState {
    /// Numbers `seat` thinks `other` could have.
    pub fn candidates(&self, seat: usize, other: usize) -> Vec<i64> {
        if self.exposed[other] || seat == other {
            return vec![self.numbers[other]];
        }
        let known: Vec<i64> = (0..self.players.len()).filter(|&o| o == seat || self.exposed[o]).map(|o| self.numbers[o]).collect();
        (1..=100).filter(|v| !known.contains(v) && self.clues[seat].iter().filter(|(a, _)| *a == other).all(|(_, c)| c.holds(*v))).collect()
    }

    fn resolve_trades(&mut self) {
        let n = self.players.len();
        for seat in 0..n {
            let Some(Action::Trade(other)) = self.actions[seat] else { continue };
            let kind = self.rng.index(5);
            let about_other = Clue::about(kind, self.numbers[other], &mut self.rng);
            let about_seat = Clue::about(kind, self.numbers[seat], &mut self.rng);
            self.clues[seat].push((other, about_other));
            self.clues[other].push((seat, about_seat));
            let (a, b) = (self.players[seat].clone(), self.players[other].clone());
            self.notes[seat].push(format!("{b}'s number: {}. {b} learned yours: {}.", about_other.text(), about_seat.text()));
            self.notes[other].push(format!("{a}'s number: {}. {a} learned yours: {}.", about_seat.text(), about_other.text()));
            self.log.push(format!("{} traded clues with {}.", self.players[seat], self.players[other]));
        }
        self.phase = Phase::Guess;
        self.time_left_ms = GUESS_MS;
        self.actions = vec![None; n];
    }

    fn resolve_guesses(&mut self) {
        let n = self.players.len();
        let mut caught = vec![false; n];
        for seat in 0..n {
            let Some(Action::Guess(other, number)) = self.actions[seat] else { continue };
            let v = self.numbers[other];
            if number == v {
                self.scores[seat] += RIGHT;
                caught[other] = true;
                self.log.push(format!("{} guessed {}'s number: right, {v}!", self.players[seat], self.players[other]));
            } else {
                self.scores[seat] += WRONG;
                let clue = if v > number { Clue::Above(number, true) } else { Clue::Above(number - 1, false) };
                self.clues[seat].push((other, clue));
                let way = if v > number { "higher" } else { "lower" };
                self.notes[seat].push(format!("{}'s number is {way} than {number}.", self.players[other]));
                self.log.push(format!("{} guessed {}'s number: wrong.", self.players[seat], self.players[other]));
            }
        }
        for (o, &c) in caught.iter().enumerate() {
            if c {
                self.scores[o] += CAUGHT;
                self.exposed[o] = true;
            }
        }
        self.round += 1;
        self.actions = vec![None; n];
        self.phase = Phase::Trade;
        self.time_left_ms = TRADE_MS;
        if self.round >= ROUNDS || (0..n).filter(|&o| !self.exposed[o]).count() <= 1 {
            for o in 0..n {
                if !self.exposed[o] {
                    self.scores[o] += SAFE;
                }
            }
            self.over = true;
            self.time_left_ms = 0;
            self.log.push("Game over: every hidden number scores +40.".into());
        } else {
            self.log.push(format!("Round {} begins.", self.round + 1));
        }
    }

    fn after_move(&mut self) {
        if self.actions.iter().all(Option::is_some) {
            match self.phase {
                Phase::Trade => self.resolve_trades(),
                Phase::Guess => self.resolve_guesses(),
            }
        }
    }
}

/// Ranks from scores; points 1000 for first down to 100 for last.
pub fn rank_points(scores: &[(String, i64)]) -> Vec<PlayerResult> {
    let n = scores.len() as i64;
    rank_by_score(scores)
        .into_iter()
        .map(|mut r| {
            r.points = 100 + 900 * (n - i64::from(r.rank)) / (n - 1).max(1);
            r
        })
        .collect()
}

pub struct Rules;

impl Game for Rules {
    type State = SecretState;
    type Move = SecretMove;
    type View = SecretView;

    fn info() -> GameInfo {
        GameInfo {
            id: "secret-number",
            kind: "main",
            min_players: 4,
            max_players: 8,
            name: "Secret Number",
            short: "Everyone has a secret number from 1 to 100; trade clues, guess the others, protect your own.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<SecretState, String> {
        if !(4..=8).contains(&players.len()) {
            return Err("Secret Number needs 4–8 players".into());
        }
        let n = players.len();
        let mut rng = Rng::new(seed);
        let mut all: Vec<i64> = (1..=100).collect();
        rng.shuffle(&mut all);
        Ok(SecretState {
            players: players.to_vec(),
            rng,
            numbers: all[..n].to_vec(),
            exposed: vec![false; n],
            scores: vec![0; n],
            clues: vec![Vec::new(); n],
            round: 0,
            phase: Phase::Trade,
            actions: vec![None; n],
            time_left_ms: TRADE_MS,
            log: vec!["Round 1 begins.".into()],
            notes: vec![Vec::new(); n],
            over: false,
        })
    }

    fn apply_move(s: &mut SecretState, player: &str, mv: SecretMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.players.iter().position(|p| p == player).ok_or("Not in this game")?;
        if s.actions[seat].is_some() {
            return Err("You already moved this step".into());
        }
        let n = s.players.len();
        let action = match (mv, s.phase) {
            (SecretMove::Skip, Phase::Trade) => Action::Skip,
            (SecretMove::Trade { seat: o }, Phase::Trade) if o < n && o != seat => Action::Trade(o),
            (SecretMove::Pass, Phase::Guess) => Action::Pass,
            (SecretMove::Guess { seat: o, number }, Phase::Guess) if o < n && o != seat && (1..=100).contains(&number) => {
                if s.exposed[o] {
                    return Err("That number is already exposed".into());
                }
                Action::Guess(o, number)
            }
            _ => return Err("Not a valid choice now".into()),
        };
        s.actions[seat] = Some(action);
        s.after_move();
        Ok(())
    }

    fn tick(s: &mut SecretState, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms > 0 {
            return;
        }
        let default = if s.phase == Phase::Trade { Action::Skip } else { Action::Pass };
        for a in &mut s.actions {
            a.get_or_insert(default);
        }
        s.after_move();
    }

    fn view_for(s: &SecretState, player: &str) -> SecretView {
        let me = s.players.iter().position(|p| p == player).unwrap_or(0);
        let seats = (0..s.players.len())
            .map(|o| {
                let cands = s.candidates(me, o);
                Seat {
                    id: s.players[o].clone(),
                    score: s.scores[o],
                    exposed: s.exposed[o].then_some(s.numbers[o]),
                    done: s.actions[o].is_some(),
                    clues: if o == me {
                        Vec::new()
                    } else {
                        s.clues[me].iter().filter(|(a, _)| *a == o).map(|(_, c)| c.text()).collect()
                    },
                    possible: cands.len(),
                    low: cands.first().copied().unwrap_or(0),
                    high: cands.last().copied().unwrap_or(0),
                }
            })
            .collect();
        let notes = &s.notes[me];
        SecretView {
            me,
            number: s.numbers[me],
            round: (s.round + 1).min(ROUNDS),
            rounds: ROUNDS,
            phase: if s.over {
                "over"
            } else if s.phase == Phase::Trade {
                "trade"
            } else {
                "guess"
            },
            seats,
            acted: s.actions[me].is_some(),
            time_left_ms: s.time_left_ms,
            phase_ms: if s.phase == Phase::Trade { TRADE_MS } else { GUESS_MS },
            log: s.log[s.log.len().saturating_sub(10)..].to_vec(),
            notes: notes[notes.len().saturating_sub(10)..].to_vec(),
            numbers: s.over.then(|| s.numbers.clone()),
            over: s.over,
        }
    }

    fn is_over(s: &SecretState) -> bool {
        s.over
    }

    fn results(s: &SecretState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.players.iter().cloned().zip(s.scores.iter().copied()).collect();
        rank_points(&scores)
    }

    fn to_move(s: &SecretState) -> Vec<String> {
        if s.over {
            return Vec::new();
        }
        (0..s.players.len()).filter(|&i| s.actions[i].is_none()).map(|i| s.players[i].clone()).collect()
    }

    fn bot_move(s: &SecretState, player: &str, seed: u64) -> SecretMove {
        let mut rng = Rng::new(seed);
        let seat = s.players.iter().position(|p| p == player).unwrap_or(0);
        let others: Vec<(usize, Vec<i64>)> =
            (0..s.players.len()).filter(|&o| o != seat && !s.exposed[o]).map(|o| (o, s.candidates(seat, o))).collect();
        if others.is_empty() {
            return if s.phase == Phase::Trade { SecretMove::Skip } else { SecretMove::Pass };
        }
        if s.phase == Phase::Trade {
            if rng.next_f64() < 0.15 {
                return SecretMove::Skip;
            }
            let (o, _) = others.iter().max_by_key(|(o, c)| (c.len() + rng.index(10), *o)).expect("others");
            return SecretMove::Trade { seat: *o };
        }
        let (o, cands) = others.iter().min_by_key(|(o, c)| (c.len(), *o)).expect("others");
        let last = s.round + 1 >= ROUNDS;
        let k = cands.len();
        if k <= 4 || (last && k <= 20) {
            return SecretMove::Guess { seat: *o, number: *rng.pick(cands) };
        }
        if k <= 14 && rng.next_f64() < 0.5 {
            return SecretMove::Guess { seat: *o, number: cands[k / 2] };
        }
        SecretMove::Pass
    }
}

pub const READY: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn ids(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn numbers_are_different() {
        for seed in 1..30 {
            let s = Rules::setup(&ids(8), seed).unwrap();
            let mut v = s.numbers.clone();
            v.sort();
            v.dedup();
            assert_eq!(v.len(), 8);
        }
    }

    #[test]
    fn clues_are_true() {
        let mut rng = Rng::new(9);
        for v in 1..=100 {
            for kind in 0..5 {
                assert!(Clue::about(kind, v, &mut rng).holds(v));
            }
        }
    }

    #[test]
    fn a_trade_gives_both_sides_a_clue() {
        let mut s = Rules::setup(&ids(4), 1).unwrap();
        Rules::apply_move(&mut s, "p1", SecretMove::Trade { seat: 2 }).unwrap();
        assert!(Rules::apply_move(&mut s, "p2", SecretMove::Trade { seat: 1 }).is_err(), "not yourself");
        for p in ["p2", "p3", "p4"] {
            Rules::apply_move(&mut s, p, SecretMove::Skip).unwrap();
        }
        assert_eq!(s.phase, Phase::Guess);
        assert_eq!(s.clues[0].len(), 1);
        assert_eq!(s.clues[2].len(), 1);
        assert_eq!(s.clues[0][0].0, 2);
        assert_eq!(s.clues[2][0].0, 0);
        assert!(s.clues[0][0].1.holds(s.numbers[2]));
        assert!(s.clues[2][0].1.holds(s.numbers[0]));
        let v = Rules::view_for(&s, "p1");
        assert_eq!(v.seats[2].clues.len(), 1);
        assert!(v.seats[2].possible < 100);
    }

    #[test]
    fn guesses_score_and_expose() {
        let mut s = Rules::setup(&ids(4), 2).unwrap();
        Rules::tick(&mut s, TRADE_MS as u64);
        let target = s.numbers[1];
        let wrong = if target == 100 { 99 } else { target + 1 };
        Rules::apply_move(&mut s, "p1", SecretMove::Guess { seat: 1, number: target }).unwrap();
        Rules::apply_move(&mut s, "p3", SecretMove::Guess { seat: 3, number: wrong }).unwrap();
        Rules::tick(&mut s, GUESS_MS as u64);
        assert_eq!(s.scores[0], RIGHT);
        assert_eq!(s.scores[1], CAUGHT);
        assert!(s.exposed[1]);
        assert_eq!(Rules::view_for(&s, "p4").seats[1].exposed, Some(target));
        assert_eq!(s.phase, Phase::Trade);
        assert_eq!(s.round, 1);
        Rules::tick(&mut s, TRADE_MS as u64);
        assert!(Rules::apply_move(&mut s, "p3", SecretMove::Guess { seat: 1, number: target }).is_err(), "already exposed");
    }

    #[test]
    fn wrong_guess_says_higher_or_lower() {
        let mut s = Rules::setup(&ids(4), 3).unwrap();
        s.numbers = vec![10, 50, 70, 90];
        Rules::tick(&mut s, TRADE_MS as u64);
        Rules::apply_move(&mut s, "p1", SecretMove::Guess { seat: 1, number: 40 }).unwrap();
        Rules::tick(&mut s, GUESS_MS as u64);
        assert_eq!(s.scores[0], WRONG);
        let c = s.candidates(0, 1);
        assert!(c.iter().all(|&v| v > 40));
        assert!(s.notes[0].iter().any(|n| n.contains("higher than 40")));
    }

    #[test]
    fn hidden_numbers_score_at_the_end() {
        let mut s = Rules::setup(&ids(4), 4).unwrap();
        for _ in 0..ROUNDS * 2 {
            Rules::tick(&mut s, GUESS_MS as u64);
        }
        assert!(s.over);
        assert_eq!(s.scores, vec![SAFE; 4]);
        assert!(Rules::view_for(&s, "p1").numbers.is_some());
    }

    #[test]
    fn view_hides_other_numbers() {
        let s = Rules::setup(&ids(5), 5).unwrap();
        let v = Rules::view_for(&s, "p2");
        assert!(v.numbers.is_none());
        assert!(v.seats.iter().all(|x| x.exposed.is_none()));
        assert_eq!(v.number, s.numbers[1]);
    }

    #[test]
    fn thousand_bot_games() {
        for n in [4, 8] {
            bot_games_with::<Rules>(500, n, 3000);
        }
    }
}

//! Secret Code (exile game, a race). Rules page: docs/games/secret-code.md
//!
//! Four dice hide a code of faces 1–6; the pip total is shown. Guess the code:
//! each guess answers with exact and near counts. Everyone races on the same
//! code; rivals (the seats after the first) are played by the rules with
//! seeded timing, so the race has human-like speeds.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const LEN: usize = 4;
pub const CODES: usize = 3;
pub const ROUND_MS: i64 = 120_000;
pub const REVEAL_MS: i64 = 4_000;
pub const MAX_GUESSES: usize = 8;
pub const FIRST_BONUS: i64 = 50;

pub type Code = [u8; LEN];

/// Exact (right face, right place) and near (right face, wrong place).
pub fn answer(code: &Code, guess: &Code) -> (u8, u8) {
    let exact = (0..LEN).filter(|&i| code[i] == guess[i]).count() as u8;
    let common: u8 = (1..=6u8).map(|f| code.iter().filter(|&&c| c == f).count().min(guess.iter().filter(|&&g| g == f).count()) as u8).sum();
    (exact, common - exact)
}

pub fn all_codes() -> Vec<Code> {
    (0..6usize.pow(LEN as u32))
        .map(|mut n| {
            let mut c = [0u8; LEN];
            for d in c.iter_mut() {
                *d = (n % 6) as u8 + 1;
                n /= 6;
            }
            c
        })
        .collect()
}

pub fn pips(code: &Code) -> u32 {
    code.iter().map(|&d| u32::from(d)).sum()
}

/// A random code that fits the total and every answer so far.
fn consistent_guess(total: u32, guesses: &[Guess], rng: &mut Rng) -> Code {
    let fits: Vec<Code> =
        all_codes().into_iter().filter(|c| pips(c) == total && guesses.iter().all(|g| answer(c, &g.code) == (g.exact, g.near))).collect();
    if fits.is_empty() { [rng.int(1, 6) as u8, rng.int(1, 6) as u8, rng.int(1, 6) as u8, rng.int(1, 6) as u8] } else { *rng.pick(&fits) }
}

pub fn code_points(guesses_used: usize) -> i64 {
    60 + 25 * (MAX_GUESSES as i64 + 1 - guesses_used as i64)
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Guess {
    pub code: Code,
    pub exact: u8,
    pub near: u8,
    /// Round time (ms since the code was shown) when it was made.
    #[serde(skip)]
    pub at: i64,
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    /// This round's guesses. For rivals the whole plan, revealed as time passes.
    pub guesses: Vec<Guess>,
    pub points: i64,
    pub round_points: i64,
}

impl Seat {
    fn made(&self, now: i64) -> impl Iterator<Item = &Guess> {
        self.guesses.iter().filter(move |g| g.at <= now)
    }
    fn solved_at(&self, now: i64) -> Option<(i64, usize)> {
        self.made(now).enumerate().find(|(_, g)| g.exact as usize == LEN).map(|(i, g)| (g.at, i + 1))
    }
    fn done(&self, now: i64) -> bool {
        self.solved_at(now).is_some() || self.made(now).count() >= MAX_GUESSES
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub seats: Vec<Seat>,
    pub codes: Vec<Code>,
    pub current: usize,
    /// Time since this code was shown.
    pub elapsed_ms: i64,
    pub revealing: bool,
    pub reveal_left_ms: i64,
    pub rng: Rng,
    pub over: bool,
    /// Replay moments: a code cracked in one guess, or in two by the fastest solver.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Guess { code: Code },
    Next,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RivalView {
    pub id: String,
    pub guesses: usize,
    pub solved: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreView {
    pub id: String,
    pub points: i64,
    pub round_points: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round: usize,
    pub rounds: usize,
    pub total: u32,
    pub time_left_ms: i64,
    pub round_ms: i64,
    pub max_guesses: usize,
    pub guesses: Vec<Guess>,
    pub done: bool,
    pub rivals: Vec<RivalView>,
    pub revealing: bool,
    pub reveal_left_ms: i64,
    /// Only in the reveal and after the game.
    pub code: Option<Code>,
    pub scores: Vec<ScoreView>,
    pub over: bool,
}

impl State {
    fn code(&self) -> Code {
        self.codes[self.current]
    }

    /// A rival's whole round, decided at the start: guesses and their times.
    fn plan_rival(&mut self) -> Vec<Guess> {
        let code = self.code();
        let total = pips(&code);
        let mut out: Vec<Guess> = Vec::new();
        let mut at = self.rng.int(4_000, 9_000);
        while out.len() < MAX_GUESSES && at <= ROUND_MS {
            let guess = if self.rng.next_f64() < 0.75 {
                consistent_guess(total, &out, &mut self.rng)
            } else {
                let mut g = consistent_guess(total, &[], &mut self.rng);
                g.swap(0, self.rng.index(LEN));
                g
            };
            let (exact, near) = answer(&code, &guess);
            out.push(Guess { code: guess, exact, near, at });
            if exact as usize == LEN {
                break;
            }
            at += self.rng.int(5_000, 12_000);
        }
        out
    }

    fn start_code(&mut self) {
        self.elapsed_ms = 0;
        self.revealing = false;
        for i in 0..self.seats.len() {
            self.seats[i].round_points = 0;
            self.seats[i].guesses = if i == 0 { Vec::new() } else { self.plan_rival() };
        }
    }

    fn end_code(&mut self) {
        let now = self.elapsed_ms;
        let best = self.seats.iter().filter_map(|s| s.solved_at(now)).map(|(t, _)| t).min();
        for s in &mut self.seats {
            if let Some((t, used)) = s.solved_at(now) {
                s.round_points = code_points(used) + if Some(t) == best { FIRST_BONUS } else { 0 };
                s.points += s.round_points;
                // Replay moment (the code and every rival's guess count are shown in the reveal): a
                // code cracked in one guess, or in two by the fastest solver.
                let first = Some(t) == best;
                if used == 1 {
                    self.moments.push(&s.id, "Code cracked in one guess");
                } else if used == 2 && first {
                    self.moments.push(&s.id, "Cracked in 2 guesses, fastest at the table");
                }
            }
        }
        self.revealing = true;
        self.reveal_left_ms = REVEAL_MS;
    }

    fn next_code(&mut self) {
        self.current += 1;
        if self.current >= CODES {
            self.over = true;
            self.revealing = false;
        } else {
            self.start_code();
        }
    }

    /// The round ends at the time limit, or at once when the first seat is done
    /// (the rivals' planned guesses up to the limit still count).
    fn check_end(&mut self) {
        if self.seats[0].done(self.elapsed_ms) || self.elapsed_ms >= ROUND_MS {
            self.elapsed_ms = ROUND_MS;
            self.end_code();
        }
    }
}

pub struct SecretCode;

impl Game for SecretCode {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "secret-code",
            kind: "exile",
            difficulty: "medium",
            tags: &["logic", "dice"],
            min_players: 1,
            max_players: 4,
            name: "Secret Code",
            short: "Four dice under the cup: crack their faces from exact and near hints, faster than the rivals.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if players.is_empty() || players.len() > 4 {
            return Err("Secret Code needs 1–4 players".into());
        }
        let mut rng = Rng::new(seed);
        let codes = (0..CODES).map(|_| [0; LEN].map(|_: u8| rng.int(1, 6) as u8)).collect();
        let seats = players.iter().map(|id| Seat { id: id.clone(), guesses: Vec::new(), points: 0, round_points: 0 }).collect();
        let mut s = State {
            seats,
            codes,
            current: 0,
            elapsed_ms: 0,
            revealing: false,
            reveal_left_ms: 0,
            rng,
            over: false,
            moments: Moments::default(),
        };
        s.start_code();
        Ok(s)
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        if player != s.seats[0].id {
            return Err("Rivals play by themselves".into());
        }
        match mv {
            Move::Guess { code } if !s.revealing => {
                if code.iter().any(|&d| !(1..=6).contains(&d)) {
                    return Err("Faces are 1–6".into());
                }
                let (exact, near) = answer(&s.code(), &code);
                let at = s.elapsed_ms;
                s.seats[0].guesses.push(Guess { code, exact, near, at });
                s.check_end();
                Ok(())
            }
            Move::Next if s.revealing => {
                s.next_code();
                Ok(())
            }
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut State, ms: u64) {
        if s.over {
            return;
        }
        if s.revealing {
            s.reveal_left_ms -= ms as i64;
            if s.reveal_left_ms <= 0 {
                s.next_code();
            }
        } else {
            s.elapsed_ms += ms as i64;
            s.check_end();
        }
    }

    fn view_for(s: &State, player: &str) -> View {
        let now = s.elapsed_ms;
        let me = s.seats.iter().position(|x| x.id == player).unwrap_or(0);
        let shown = s.current.min(CODES - 1);
        View {
            round: shown + 1,
            rounds: CODES,
            total: pips(&s.codes[shown]),
            time_left_ms: (ROUND_MS - now).max(0),
            round_ms: ROUND_MS,
            max_guesses: MAX_GUESSES,
            guesses: s.seats[me].made(now).cloned().collect(),
            done: s.seats[me].done(now),
            rivals: s
                .seats
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != me)
                .map(|(_, x)| RivalView { id: x.id.clone(), guesses: x.made(now).count(), solved: x.solved_at(now).is_some() })
                .collect(),
            revealing: s.revealing,
            reveal_left_ms: s.reveal_left_ms.max(0),
            code: (s.revealing || s.over).then(|| s.codes[shown]),
            scores: s.seats.iter().map(|x| ScoreView { id: x.id.clone(), points: x.points, round_points: x.round_points }).collect(),
            over: s.over,
        }
    }

    fn is_over(s: &State) -> bool {
        s.over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        rank_by_score(&s.seats.iter().map(|x| (x.id.clone(), x.points)).collect::<Vec<_>>())
    }

    fn to_move(s: &State) -> Vec<String> {
        if s.over || s.seats[0].done(s.elapsed_ms) { Vec::new() } else { vec![s.seats[0].id.clone()] }
    }

    fn moments(s: &State, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &State, _player: &str, seed: u64) -> Move {
        if s.revealing {
            return Move::Next;
        }
        let mut rng = Rng::new(seed);
        Move::Guess { code: consistent_guess(pips(&s.code()), &s.seats[0].guesses, &mut rng) }
    }
}

pub type Rules = SecretCode;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn answers_count_each_die_once() {
        assert_eq!(answer(&[1, 2, 3, 4], &[1, 2, 3, 4]), (4, 0));
        assert_eq!(answer(&[1, 1, 2, 2], &[2, 2, 1, 1]), (0, 4));
        assert_eq!(answer(&[1, 1, 2, 3], &[1, 4, 1, 1]), (1, 1));
        assert_eq!(answer(&[6, 6, 6, 6], &[1, 2, 3, 4]), (0, 0));
        assert_eq!(all_codes().len(), 1296);
    }

    #[test]
    fn the_code_stays_hidden_until_the_reveal() {
        let mut s = SecretCode::setup(&players(3), 1).unwrap();
        let v = SecretCode::view_for(&s, "p1");
        assert!(v.code.is_none());
        assert_eq!(v.total, pips(&s.codes[0]));
        let text = serde_json::to_string(&SecretCode::view_for(&s, "p1")).unwrap();
        assert!(!text.contains("\"at\""));
        let code = s.codes[0];
        SecretCode::apply_move(&mut s, "p1", Move::Guess { code }).unwrap();
        let v = SecretCode::view_for(&s, "p1");
        assert!(v.revealing);
        assert_eq!(v.code, Some(code));
    }

    #[test]
    fn rival_guesses_appear_only_with_time() {
        let mut s = SecretCode::setup(&players(2), 5).unwrap();
        assert_eq!(SecretCode::view_for(&s, "p1").rivals[0].guesses, 0);
        SecretCode::tick(&mut s, 30_000);
        assert!(SecretCode::view_for(&s, "p1").rivals[0].guesses >= 2);
        assert!(SecretCode::apply_move(&mut s, "p2", Move::Next).is_err());
    }

    #[test]
    fn a_one_guess_solve_scores_most_and_gets_the_first_bonus() {
        let mut s = SecretCode::setup(&players(4), 9).unwrap();
        let code = s.codes[0];
        SecretCode::apply_move(&mut s, "p1", Move::Guess { code }).unwrap();
        assert_eq!(s.seats[0].points, code_points(1) + FIRST_BONUS);
        SecretCode::apply_move(&mut s, "p1", Move::Next).unwrap();
        assert_eq!(s.current, 1);
        assert!(!s.revealing);
    }

    #[test]
    fn eight_wrong_guesses_end_the_round() {
        let mut s = SecretCode::setup(&players(1), 3).unwrap();
        let wrong = if s.codes[0] == [1, 1, 1, 1] { [2, 2, 2, 2] } else { [1, 1, 1, 1] };
        for _ in 0..MAX_GUESSES {
            SecretCode::apply_move(&mut s, "p1", Move::Guess { code: wrong }).unwrap();
        }
        assert!(s.revealing);
        assert_eq!(s.seats[0].points, 0);
        assert!(SecretCode::apply_move(&mut s, "p1", Move::Guess { code: [7, 1, 1, 1] }).is_err());
    }

    #[test]
    fn bot_solves_most_codes() {
        let mut solved = 0;
        for seed in 1..=50 {
            let mut s = SecretCode::setup(&players(1), seed).unwrap();
            let mut n = 0;
            while !s.revealing {
                let mv = SecretCode::bot_move(&s, "p1", seed * 31 + n);
                SecretCode::apply_move(&mut s, "p1", mv).unwrap();
                n += 1;
            }
            solved += usize::from(s.seats[0].points > 0);
        }
        assert!(solved >= 45, "solved {solved}");
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<SecretCode>(1000, 1, 300);
        bot_games_with::<SecretCode>(300, 4, 300);
    }

    #[test]
    fn a_one_or_fast_two_guess_crack_is_a_replay_moment() {
        let mine =
            |s: &State| -> Vec<String> { SecretCode::moments(s, "p2").into_iter().filter(|m| m.by == "p1").map(|m| m.text).collect() };
        let mut s = SecretCode::setup(&players(2), 9).unwrap();
        let code = s.codes[0];
        SecretCode::apply_move(&mut s, "p1", Move::Guess { code }).unwrap();
        assert_eq!(mine(&s), vec!["Code cracked in one guess"]);
        // The next code: one miss, then the code, before the rival's first guess.
        SecretCode::apply_move(&mut s, "p1", Move::Next).unwrap();
        let code = s.codes[1];
        let wrong = if code == [1, 1, 1, 1] { [2, 2, 2, 2] } else { [1, 1, 1, 1] };
        SecretCode::apply_move(&mut s, "p1", Move::Guess { code: wrong }).unwrap();
        SecretCode::apply_move(&mut s, "p1", Move::Guess { code }).unwrap();
        assert_eq!(mine(&s), vec!["Code cracked in one guess", "Cracked in 2 guesses, fastest at the table"]);
        // Three guesses is no moment.
        SecretCode::apply_move(&mut s, "p1", Move::Next).unwrap();
        let code = s.codes[2];
        let wrong = if code == [1, 1, 1, 1] { [2, 2, 2, 2] } else { [1, 1, 1, 1] };
        SecretCode::apply_move(&mut s, "p1", Move::Guess { code: wrong }).unwrap();
        SecretCode::apply_move(&mut s, "p1", Move::Guess { code: wrong }).unwrap();
        SecretCode::apply_move(&mut s, "p1", Move::Guess { code }).unwrap();
        assert_eq!(mine(&s).len(), 2);
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<SecretCode>(300, 4, 300);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

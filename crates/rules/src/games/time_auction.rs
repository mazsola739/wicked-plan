//! Time Auction (prison game, 2–6 seats). Rules page: docs/games/time-auction.md
//!
//! Everyone has a time bank. Each round all hold a button; after a countdown
//! the clock runs and holders pay their bank while they hold. The longest hold
//! wins the round's token.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 7;
pub const BANK_MS: i64 = 100_000;
pub const READY_MS: i64 = 15_000;
pub const COUNTDOWN_MS: i64 = 5_000;
pub const RESULT_MS: i64 = 4_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Ready,
    Countdown,
    Bidding,
    Result,
    Over,
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    pub bank_ms: i64,
    pub tokens: u32,
    pub holding: bool,
    /// Still in this round's bid (held through the countdown).
    pub bidding: bool,
    pub held_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundLog {
    pub round: usize,
    pub winner: Option<String>,
    pub bidders: usize,
}

#[derive(Clone, Debug)]
pub struct TaState {
    pub seats: Vec<Seat>,
    pub seed: u64,
    pub phase: Phase,
    pub round: usize,
    pub time_left_ms: i64,
    pub log: Vec<RoundLog>,
    /// Each seat's hold time in the last round (each seat sees only its own).
    pub last_held: Vec<i64>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TaMove {
    Press,
    Release,
    Wait,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub tokens: u32,
    /// Only during Ready: whether this seat already holds the button.
    pub ready: Option<bool>,
    pub out_of_time: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaView {
    pub phase: Phase,
    pub round: usize,
    pub rounds: usize,
    pub time_left_ms: i64,
    pub phase_ms: i64,
    pub bank_ms: i64,
    pub start_bank_ms: i64,
    pub holding: bool,
    pub bidding: bool,
    pub held_ms: i64,
    pub last_held_ms: i64,
    pub seats: Vec<SeatView>,
    pub log: Vec<RoundLog>,
}

impl TaState {
    fn seat(&self, id: &str) -> Option<usize> {
        self.seats.iter().position(|s| s.id == id)
    }

    fn start_round(&mut self) {
        if self.round >= ROUNDS || self.seats.iter().all(|s| s.bank_ms == 0) {
            self.phase = Phase::Over;
            self.time_left_ms = 0;
            return;
        }
        self.round += 1;
        self.phase = Phase::Ready;
        self.time_left_ms = READY_MS;
        for s in &mut self.seats {
            s.holding = false;
            s.bidding = false;
            s.held_ms = 0;
        }
    }

    fn start_countdown(&mut self) {
        self.phase = Phase::Countdown;
        self.time_left_ms = COUNTDOWN_MS;
    }

    fn start_bidding(&mut self) {
        for s in &mut self.seats {
            s.bidding = s.holding;
        }
        self.phase = Phase::Bidding;
        self.time_left_ms = 0;
        if self.seats.iter().all(|s| !s.holding) {
            self.end_round();
        }
    }

    fn end_round(&mut self) {
        let bidders = self.seats.iter().filter(|s| s.bidding).count();
        let best = self.seats.iter().filter(|s| s.bidding).map(|s| s.held_ms).max().unwrap_or(0);
        let top: Vec<usize> = (0..self.seats.len()).filter(|&i| self.seats[i].bidding && self.seats[i].held_ms == best).collect();
        let winner = if top.len() == 1 && best > 0 { Some(top[0]) } else { None };
        if let Some(w) = winner {
            self.seats[w].tokens += 1;
        }
        self.log.push(RoundLog { round: self.round, winner: winner.map(|w| self.seats[w].id.clone()), bidders });
        self.last_held = self.seats.iter().map(|s| s.held_ms).collect();
        for s in &mut self.seats {
            s.holding = false;
            s.bidding = false;
        }
        self.phase = Phase::Result;
        self.time_left_ms = RESULT_MS;
    }

    fn all_ready(&self) -> bool {
        self.seats.iter().filter(|s| s.bank_ms > 0).all(|s| s.holding)
    }

    /// The bot's secret plan for this round: pass in the countdown, or hold this long.
    fn bot_plan(&self, i: usize) -> Option<i64> {
        let s = &self.seats[i];
        let mut rng = Rng::new(self.seed ^ (self.round as u64 * 0x9e37_79b9) ^ ((i as u64 + 1) << 40));
        let rounds_left = (ROUNDS + 1 - self.round).max(1) as i64;
        let share = s.bank_ms / rounds_left;
        let pass_chance = if share < 4_000 { 0.5 } else { 0.15 };
        if rng.next_f64() < pass_chance {
            return None;
        }
        let best = self.seats.iter().map(|o| o.tokens).max().unwrap_or(0);
        let behind = if s.tokens < best { 1.3 } else { 1.0 };
        let factor = 0.4 + rng.next_f64() * 1.8;
        Some(((share as f64 * factor * behind) as i64).max(500).min(s.bank_ms))
    }
}

pub struct Rules;

impl Game for Rules {
    type State = TaState;
    type Move = TaMove;
    type View = TaView;

    fn info() -> GameInfo {
        GameInfo {
            id: "time-auction",
            kind: "prison",
            difficulty: "easy",
            min_players: 2,
            max_players: 6,
            name: "Time Auction",
            short: "Hold a button to bid time from a limited bank; the longest press wins the round.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<TaState, String> {
        if !(2..=6).contains(&players.len()) {
            return Err("Time Auction needs 2–6 players".into());
        }
        let seats = players
            .iter()
            .map(|id| Seat { id: id.clone(), bank_ms: BANK_MS, tokens: 0, holding: false, bidding: false, held_ms: 0 })
            .collect();
        let mut s = TaState {
            seats,
            seed,
            phase: Phase::Ready,
            round: 0,
            time_left_ms: 0,
            log: Vec::new(),
            last_held: vec![0; players.len()],
        };
        s.start_round();
        Ok(s)
    }

    fn apply_move(s: &mut TaState, player: &str, mv: TaMove) -> Result<(), String> {
        let i = s.seat(player).ok_or("Not at this table")?;
        match (mv, s.phase) {
            (TaMove::Wait, Phase::Over) => Err("Game is over".into()),
            (TaMove::Wait, _) => Ok(()),
            (TaMove::Press, Phase::Ready) => {
                if s.seats[i].bank_ms == 0 {
                    return Err("Your time bank is empty".into());
                }
                s.seats[i].holding = true;
                if s.all_ready() {
                    s.start_countdown();
                }
                Ok(())
            }
            (TaMove::Release, Phase::Ready | Phase::Countdown | Phase::Bidding) => {
                if !s.seats[i].holding {
                    return Err("You are not holding".into());
                }
                s.seats[i].holding = false;
                if s.phase == Phase::Bidding && s.seats.iter().all(|x| !x.holding) {
                    s.end_round();
                }
                Ok(())
            }
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut TaState, ms: u64) {
        let mut ms = ms as i64;
        while ms > 0 {
            match s.phase {
                Phase::Over => return,
                Phase::Bidding => {
                    for seat in s.seats.iter_mut().filter(|x| x.holding) {
                        let take = ms.min(seat.bank_ms);
                        seat.bank_ms -= take;
                        seat.held_ms += take;
                        if seat.bank_ms == 0 {
                            seat.holding = false;
                        }
                    }
                    if s.seats.iter().all(|x| !x.holding) {
                        s.end_round();
                    }
                    return;
                }
                Phase::Ready | Phase::Countdown | Phase::Result => {
                    let step = ms.min(s.time_left_ms);
                    s.time_left_ms -= step;
                    ms -= step;
                    if s.time_left_ms > 0 {
                        return;
                    }
                    match s.phase {
                        Phase::Ready => s.start_countdown(),
                        Phase::Countdown => s.start_bidding(),
                        _ => s.start_round(),
                    }
                }
            }
        }
    }

    fn view_for(s: &TaState, player: &str) -> TaView {
        let me = s.seat(player);
        let mine = me.map(|i| &s.seats[i]);
        TaView {
            phase: s.phase,
            round: s.round,
            rounds: ROUNDS,
            time_left_ms: s.time_left_ms,
            phase_ms: match s.phase {
                Phase::Ready => READY_MS,
                Phase::Countdown => COUNTDOWN_MS,
                Phase::Result => RESULT_MS,
                _ => 0,
            },
            bank_ms: mine.map_or(0, |m| m.bank_ms),
            start_bank_ms: BANK_MS,
            holding: mine.is_some_and(|m| m.holding),
            bidding: mine.is_some_and(|m| m.bidding),
            held_ms: mine.map_or(0, |m| m.held_ms),
            last_held_ms: me.map_or(0, |i| s.last_held[i]),
            seats: s
                .seats
                .iter()
                .map(|x| SeatView {
                    id: x.id.clone(),
                    tokens: x.tokens,
                    ready: (s.phase == Phase::Ready).then_some(x.holding),
                    out_of_time: x.bank_ms == 0,
                })
                .collect(),
            log: s.log.clone(),
        }
    }

    fn is_over(s: &TaState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &TaState) -> Vec<PlayerResult> {
        let keyed: Vec<(String, i64)> = s.seats.iter().map(|x| (x.id.clone(), i64::from(x.tokens) * 1_000_000 + x.bank_ms)).collect();
        points_by_rank(&keyed)
    }

    fn to_move(s: &TaState) -> Vec<String> {
        let list = |f: &dyn Fn(&Seat) -> bool| s.seats.iter().filter(|x| f(x)).map(|x| x.id.clone()).collect();
        match s.phase {
            Phase::Ready => list(&|x| x.bank_ms > 0 && !x.holding),
            Phase::Countdown | Phase::Bidding => list(&|x| x.holding),
            _ => Vec::new(),
        }
    }

    fn bot_move(s: &TaState, player: &str, _seed: u64) -> TaMove {
        let Some(i) = s.seat(player) else { return TaMove::Wait };
        let seat = &s.seats[i];
        match s.phase {
            Phase::Ready if !seat.holding => TaMove::Press,
            Phase::Countdown if s.bot_plan(i).is_none() => TaMove::Release,
            Phase::Bidding if s.bot_plan(i).is_none_or(|target| seat.held_ms >= target) => TaMove::Release,
            _ => TaMove::Wait,
        }
    }
}

/// Ranks by `key` (higher is better), then 1000 points for the first down to 100 for the last.
fn points_by_rank(keyed: &[(String, i64)]) -> Vec<PlayerResult> {
    let n = keyed.len().max(2) as i64;
    rank_by_score(keyed)
        .into_iter()
        .map(|r| PlayerResult { points: 100 + 900 * (n - i64::from(r.rank)) / (n - 1), ..r })
        .collect()
}

pub const READY: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn ids(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    fn press_all(s: &mut TaState) {
        for id in ids(s.seats.len()) {
            Rules::apply_move(s, &id, TaMove::Press).unwrap();
        }
    }

    #[test]
    fn longest_hold_wins_and_everyone_pays() {
        let mut s = Rules::setup(&ids(3), 1).unwrap();
        press_all(&mut s);
        assert_eq!(s.phase, Phase::Countdown);
        // p3 passes in the countdown and pays nothing.
        Rules::apply_move(&mut s, "p3", TaMove::Release).unwrap();
        Rules::tick(&mut s, COUNTDOWN_MS as u64 + 2_000);
        assert_eq!(s.phase, Phase::Bidding);
        Rules::apply_move(&mut s, "p1", TaMove::Release).unwrap();
        Rules::tick(&mut s, 3_000);
        Rules::apply_move(&mut s, "p2", TaMove::Release).unwrap();
        assert_eq!(s.phase, Phase::Result);
        assert_eq!(s.seats[0].bank_ms, BANK_MS - 2_000);
        assert_eq!(s.seats[1].bank_ms, BANK_MS - 5_000);
        assert_eq!(s.seats[2].bank_ms, BANK_MS);
        assert_eq!(s.seats[1].tokens, 1);
        assert_eq!(s.log[0].winner.as_deref(), Some("p2"));
        assert_eq!(s.log[0].bidders, 2);
    }

    #[test]
    fn a_tie_gives_no_token_and_an_empty_bank_lets_go() {
        let mut s = Rules::setup(&ids(2), 2).unwrap();
        press_all(&mut s);
        Rules::tick(&mut s, COUNTDOWN_MS as u64);
        Rules::tick(&mut s, BANK_MS as u64 + 10_000);
        assert_eq!(s.phase, Phase::Result);
        assert!(s.seats.iter().all(|x| x.bank_ms == 0 && x.tokens == 0));
        assert_eq!(s.log[0].winner, None);
        Rules::tick(&mut s, RESULT_MS as u64);
        assert!(Rules::is_over(&s));
    }

    #[test]
    fn ready_timeout_leaves_the_slow_ones_out_and_hides_holders_later() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        Rules::apply_move(&mut s, "p1", TaMove::Press).unwrap();
        assert_eq!(Rules::view_for(&s, "p2").seats[0].ready, Some(true));
        Rules::tick(&mut s, (READY_MS + COUNTDOWN_MS) as u64);
        assert_eq!(s.phase, Phase::Bidding);
        assert!(Rules::view_for(&s, "p2").seats.iter().all(|x| x.ready.is_none()));
        assert_eq!(Rules::to_move(&s), vec!["p1".to_string()]);
        assert!(Rules::apply_move(&mut s, "p2", TaMove::Press).is_err());
    }

    #[test]
    fn ranks_by_tokens_then_bank() {
        let mut s = Rules::setup(&ids(3), 4).unwrap();
        s.seats[0].tokens = 2;
        s.seats[1].tokens = 2;
        s.seats[1].bank_ms = 1;
        let r = Rules::results(&s);
        assert_eq!((r[0].rank, r[0].points), (1, 1000));
        assert_eq!((r[1].rank, r[1].points), (2, 550));
        assert_eq!((r[2].rank, r[2].points), (3, 100));
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(400, 4, 20_000);
        bot_games_with::<Rules>(300, 2, 20_000);
        bot_games_with::<Rules>(300, 6, 20_000);
    }
}

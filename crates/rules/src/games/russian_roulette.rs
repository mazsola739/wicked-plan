//! Russian Roulette (prison game, 3–6 seats). Rules page: docs/games/russian-roulette.md
//!
//! A six-chamber drum with two paint charges goes round the table. Before
//! pulling, a player may spend secret credits to peek, spin or secretly turn
//! the drum on. A hit costs a life.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const CHAMBERS: usize = 6;
pub const CHARGES: u32 = 2;
pub const LIVES: u32 = 2;
pub const CREDITS: u32 = 16;
pub const PEEK: u32 = 3;
pub const SPIN: u32 = 2;
pub const TAMPER: u32 = 4;
pub const PULLS_PER_PLAYER: usize = 6;
pub const SHOT_MS: i64 = 2_500;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Turn,
    Shot,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Spin,
    Click,
    Hit,
    Reload,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: String,
    pub event: Event,
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    pub lives: u32,
    pub credits: u32,
    /// What this seat's last peek showed, while the drum has not moved since.
    pub peek: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct RrState {
    pub seats: Vec<Seat>,
    pub rng: Rng,
    pub chambers: [bool; CHAMBERS],
    pub pointer: usize,
    pub charges_left: u32,
    pub pulls_since_mix: usize,
    pub pulls: usize,
    pub max_pulls: usize,
    pub actor: usize,
    pub phase: Phase,
    pub time_left_ms: i64,
    pub log: Vec<LogEntry>,
    /// Replay moments: a click on the last life against bad odds; the hit that leaves one standing.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RrMove {
    Peek,
    Spin,
    Tamper,
    Pull,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub lives: u32,
    pub to_act: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RrView {
    pub phase: Phase,
    pub my_turn: bool,
    pub lives: u32,
    pub credits: u32,
    pub peek: Option<bool>,
    pub charges_left: u32,
    pub pulls_since_mix: usize,
    pub pulls: usize,
    pub max_pulls: usize,
    pub seats: Vec<SeatView>,
    pub log: Vec<LogEntry>,
}

impl RrState {
    fn seat(&self, id: &str) -> Option<usize> {
        self.seats.iter().position(|s| s.id == id)
    }

    fn reload(&mut self) {
        self.chambers = [false; CHAMBERS];
        let mut placed = 0;
        while placed < CHARGES {
            let k = self.rng.index(CHAMBERS);
            if !self.chambers[k] {
                self.chambers[k] = true;
                placed += 1;
            }
        }
        self.pointer = self.rng.index(CHAMBERS);
        self.charges_left = CHARGES;
        self.pulls_since_mix = 0;
    }

    fn drum_moved(&mut self) {
        for s in &mut self.seats {
            s.peek = None;
        }
    }

    fn alive(&self) -> usize {
        self.seats.iter().filter(|s| s.lives > 0).count()
    }

    /// Public odds that the next pull hits: charges left ÷ chambers not fired since the last mix.
    pub fn public_odds(&self) -> f64 {
        f64::from(self.charges_left) / (CHAMBERS - self.pulls_since_mix.min(CHAMBERS - 1)) as f64
    }

    fn act(&mut self, i: usize, mv: RrMove) -> Result<(), String> {
        let cost = match mv {
            RrMove::Peek => PEEK,
            RrMove::Spin => SPIN,
            RrMove::Tamper => TAMPER,
            RrMove::Pull => 0,
        };
        if self.seats[i].credits < cost {
            return Err("Not enough credits".into());
        }
        self.seats[i].credits -= cost;
        match mv {
            RrMove::Peek => self.seats[i].peek = Some(self.chambers[self.pointer]),
            RrMove::Spin => {
                self.pointer = self.rng.index(CHAMBERS);
                self.pulls_since_mix = 0;
                self.drum_moved();
                self.log.push(LogEntry { id: self.seats[i].id.clone(), event: Event::Spin });
            }
            RrMove::Tamper => {
                self.pointer = (self.pointer + 1) % CHAMBERS;
                self.drum_moved();
            }
            RrMove::Pull => self.pull(i),
        }
        Ok(())
    }

    fn pull(&mut self, i: usize) {
        let hit = self.chambers[self.pointer];
        let id = self.seats[i].id.clone();
        // Replay moments, from what the table sees (the odds shown in every view; never a peek or a
        // tamper): a click on the last life with 2 charges in 3 chambers or worse, and the hit that
        // leaves one player standing.
        let (charges, chambers) = (self.charges_left as usize, CHAMBERS.saturating_sub(self.pulls_since_mix));
        if !hit && self.seats[i].lives == 1 && chambers >= 2 && 3 * charges >= 2 * chambers {
            self.moments.push(&id, format!("Last life, {charges} charges in {chambers} chambers: click"));
        }
        if hit && self.seats[i].lives == 1 && self.alive() == 2 {
            let last = self.seats.iter().find(|s| s.lives > 0 && s.id != id).map(|s| s.id.clone()).unwrap_or_default();
            self.moments.push(&id, format!("Out: {{{last}}} is the last one standing"));
        }
        if hit {
            self.chambers[self.pointer] = false;
            self.charges_left -= 1;
            self.seats[i].lives -= 1;
        }
        self.log.push(LogEntry { id: id.clone(), event: if hit { Event::Hit } else { Event::Click } });
        self.pointer = (self.pointer + 1) % CHAMBERS;
        self.pulls_since_mix += 1;
        self.pulls += 1;
        self.drum_moved();
        if self.charges_left == 0 {
            self.reload();
            self.log.push(LogEntry { id, event: Event::Reload });
        }
        self.phase = Phase::Shot;
        self.time_left_ms = SHOT_MS;
    }

    fn next_turn(&mut self) {
        if self.alive() <= 1 || self.pulls >= self.max_pulls {
            self.phase = Phase::Over;
            self.time_left_ms = 0;
            return;
        }
        let n = self.seats.len();
        self.actor = (1..=n).map(|k| (self.actor + k) % n).find(|&j| self.seats[j].lives > 0).unwrap_or(self.actor);
        self.phase = Phase::Turn;
    }
}

pub struct Rules;

impl Game for Rules {
    type State = RrState;
    type Move = RrMove;
    type View = RrView;

    fn info() -> GameInfo {
        GameInfo {
            id: "russian-roulette",
            kind: "prison",
            difficulty: "expert",
            tags: &["bluff-deduction", "speed-risk"],
            min_players: 3,
            max_players: 6,
            name: "Russian Roulette",
            short: "Spend 16 credits on clues, drum spins and secret tampering to dodge the paint charge.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<RrState, String> {
        if !(3..=6).contains(&players.len()) {
            return Err("Russian Roulette needs 3–6 players".into());
        }
        let mut rng = Rng::new(seed);
        let actor = rng.index(players.len());
        let seats = players.iter().map(|id| Seat { id: id.clone(), lives: LIVES, credits: CREDITS, peek: None }).collect();
        let mut s = RrState {
            seats,
            rng,
            chambers: [false; CHAMBERS],
            pointer: 0,
            charges_left: 0,
            pulls_since_mix: 0,
            pulls: 0,
            max_pulls: PULLS_PER_PLAYER * players.len(),
            actor,
            phase: Phase::Turn,
            time_left_ms: 0,
            log: Vec::new(),
            moments: Moments::default(),
        };
        s.reload();
        Ok(s)
    }

    fn apply_move(s: &mut RrState, player: &str, mv: RrMove) -> Result<(), String> {
        let i = s.seat(player).ok_or("Not at this table")?;
        if s.phase != Phase::Turn || i != s.actor {
            return Err("Not your turn".into());
        }
        s.act(i, mv)
    }

    fn tick(s: &mut RrState, ms: u64) {
        if s.phase != Phase::Shot {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.next_turn();
        }
    }

    fn view_for(s: &RrState, player: &str) -> RrView {
        let me = s.seat(player);
        let mine = me.map(|i| &s.seats[i]);
        RrView {
            phase: s.phase,
            my_turn: s.phase == Phase::Turn && me == Some(s.actor),
            lives: mine.map_or(0, |m| m.lives),
            credits: mine.map_or(0, |m| m.credits),
            peek: mine.and_then(|m| m.peek),
            charges_left: s.charges_left,
            pulls_since_mix: s.pulls_since_mix,
            pulls: s.pulls,
            max_pulls: s.max_pulls,
            seats: s
                .seats
                .iter()
                .enumerate()
                .map(|(i, x)| SeatView { id: x.id.clone(), lives: x.lives, to_act: s.phase != Phase::Over && i == s.actor })
                .collect(),
            log: s.log.clone(),
        }
    }

    fn is_over(s: &RrState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &RrState) -> Vec<PlayerResult> {
        let keyed: Vec<(String, i64)> = s.seats.iter().map(|x| (x.id.clone(), i64::from(x.lives) * 100 + i64::from(x.credits))).collect();
        points_by_rank(&keyed)
    }

    fn to_move(s: &RrState) -> Vec<String> {
        if s.phase == Phase::Turn { vec![s.seats[s.actor].id.clone()] } else { Vec::new() }
    }

    fn moments(s: &RrState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &RrState, player: &str, seed: u64) -> RrMove {
        let mut rng = Rng::new(seed);
        let me = &s.seats[s.seat(player).unwrap_or(0)];
        let credits = me.credits;
        match me.peek {
            Some(true) if credits >= TAMPER => return RrMove::Tamper,
            Some(true) if credits >= SPIN => return RrMove::Spin,
            Some(_) => return RrMove::Pull,
            None => {}
        }
        let odds = s.public_odds();
        // Early in the game keep some credits back unless the odds are really bad.
        let reserve = if s.pulls * 2 < s.max_pulls { 4 } else { 0 };
        let nerve = rng.next_f64() * 0.1;
        let can_peek = credits >= PEEK + reserve || (credits >= PEEK && odds >= 0.45);
        if odds >= 0.3 + nerve && can_peek {
            return RrMove::Peek;
        }
        if odds >= 0.5 && credits >= SPIN {
            return RrMove::Spin;
        }
        RrMove::Pull
    }
}

/// Ranks by `key` (higher is better), then 1000 points for the first down to 100 for the last.
fn points_by_rank(keyed: &[(String, i64)]) -> Vec<PlayerResult> {
    let n = keyed.len().max(2) as i64;
    rank_by_score(keyed).into_iter().map(|r| PlayerResult { points: 100 + 900 * (n - i64::from(r.rank)) / (n - 1), ..r }).collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn ids(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    fn rig(s: &mut RrState, chambers: [bool; CHAMBERS], pointer: usize) {
        s.chambers = chambers;
        s.pointer = pointer;
        s.actor = 0;
    }

    #[test]
    fn a_hit_costs_a_life_and_the_drum_turns() {
        let mut s = Rules::setup(&ids(3), 1).unwrap();
        rig(&mut s, [true, false, false, true, false, false], 0);
        Rules::apply_move(&mut s, "p1", RrMove::Pull).unwrap();
        assert_eq!(s.seats[0].lives, LIVES - 1);
        assert_eq!(s.charges_left, 1);
        assert_eq!(s.pointer, 1);
        assert_eq!(s.phase, Phase::Shot);
        assert!(Rules::apply_move(&mut s, "p2", RrMove::Pull).is_err());
        Rules::tick(&mut s, SHOT_MS as u64);
        assert_eq!(s.actor, 1);
        // No time limit: the turn waits for p2 to pull.
        Rules::tick(&mut s, 600_000);
        assert_eq!((s.phase, s.actor, s.log.len()), (Phase::Turn, 1, 1));
        Rules::apply_move(&mut s, "p2", RrMove::Pull).unwrap();
        assert_eq!(s.seats[1].lives, LIVES);
        assert_eq!(s.log.iter().map(|e| e.event).collect::<Vec<_>>(), vec![Event::Hit, Event::Click]);
    }

    #[test]
    fn peek_and_tamper_are_secret_and_cost_credits() {
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        rig(&mut s, [true, false, false, false, false, true], 0);
        Rules::apply_move(&mut s, "p1", RrMove::Peek).unwrap();
        assert_eq!(Rules::view_for(&s, "p1").peek, Some(true));
        assert_eq!(Rules::view_for(&s, "p2").peek, None);
        Rules::apply_move(&mut s, "p1", RrMove::Tamper).unwrap();
        assert_eq!(s.seats[0].credits, CREDITS - PEEK - TAMPER);
        assert_eq!(Rules::view_for(&s, "p1").peek, None);
        assert!(s.log.is_empty());
        Rules::apply_move(&mut s, "p1", RrMove::Pull).unwrap();
        assert_eq!(s.seats[0].lives, LIVES);
        // Credits of others never show.
        assert!(serde_json::to_value(Rules::view_for(&s, "p2")).unwrap()["seats"][0].get("credits").is_none());
    }

    #[test]
    fn spending_more_than_you_have_is_refused() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        s.actor = 0;
        s.seats[0].credits = 3;
        Rules::apply_move(&mut s, "p1", RrMove::Peek).unwrap();
        assert!(Rules::apply_move(&mut s, "p1", RrMove::Spin).is_err());
    }

    #[test]
    fn reloads_and_ends_when_one_is_left() {
        let mut s = Rules::setup(&ids(3), 4).unwrap();
        s.seats[1].lives = 0;
        s.seats[2].lives = 1;
        rig(&mut s, [false, false, false, false, true, true], 4);
        s.charges_left = 2;
        Rules::apply_move(&mut s, "p1", RrMove::Pull).unwrap();
        Rules::tick(&mut s, SHOT_MS as u64);
        assert_eq!(s.actor, 2, "skips the player who is out");
        Rules::apply_move(&mut s, "p3", RrMove::Pull).unwrap();
        assert_eq!(s.charges_left, CHARGES, "reloaded");
        assert_eq!(s.log.last().unwrap().event, Event::Reload);
        Rules::tick(&mut s, SHOT_MS as u64);
        assert!(Rules::is_over(&s));
        let r = Rules::results(&s);
        assert_eq!(r[0].rank, 1);
        assert_eq!(r[0].points, 1000);
    }

    #[test]
    fn bot_dodges_what_it_peeked() {
        let mut s = Rules::setup(&ids(3), 5).unwrap();
        rig(&mut s, [true; CHAMBERS], 0);
        s.seats[0].peek = Some(true);
        assert!(matches!(Rules::bot_move(&s, "p1", 1), RrMove::Tamper));
        s.seats[0].credits = 2;
        assert!(matches!(Rules::bot_move(&s, "p1", 1), RrMove::Spin));
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(400, 4, 5_000);
        bot_games_with::<Rules>(300, 3, 5_000);
        bot_games_with::<Rules>(300, 6, 5_000);
    }

    #[test]
    fn a_last_life_click_against_the_odds_and_the_final_hit_are_replay_moments() {
        let mut s = Rules::setup(&ids(3), 6).unwrap();
        rig(&mut s, [false, false, false, false, true, true], 3);
        s.seats[0].lives = 1;
        s.pulls_since_mix = 3;
        Rules::apply_move(&mut s, "p1", RrMove::Pull).unwrap();
        let m = Rules::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Last life, 2 charges in 3 chambers: click"));
        // p2 is out, p1 is hit on its last life: p3 is the last one standing.
        let mut s = Rules::setup(&ids(3), 6).unwrap();
        rig(&mut s, [true, false, false, false, false, true], 0);
        s.seats[0].lives = 1;
        s.seats[1].lives = 0;
        Rules::apply_move(&mut s, "p1", RrMove::Pull).unwrap();
        let m = Rules::moments(&s, "p3");
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Out: {p3} is the last one standing"));
        // A last-life click at 2 charges in 6 is no moment.
        let mut s = Rules::setup(&ids(3), 6).unwrap();
        rig(&mut s, [false, true, true, false, false, false], 0);
        s.seats[0].lives = 1;
        Rules::apply_move(&mut s, "p1", RrMove::Pull).unwrap();
        assert!(Rules::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

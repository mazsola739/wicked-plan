//! Heavy 13 (prison game, 3–6 seats). Rules page: docs/games/heavy-13.md
//!
//! Everyone secretly adds a card to one of four coloured stacks (or passes);
//! a stack over 13 is overloaded and costs its contributors, exactly 13 pays a bonus.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 10;
pub const HAND: usize = 8;
pub const PASSES: u32 = 2;
pub const LIMIT: u32 = 13;
pub const FULL_BONUS: i64 = 3;
pub const RESULT_MS: i64 = 5_000;
pub const COLOUR_NAMES: [&str; 4] = ["red", "blue", "green", "gold"];

/// A card: colour * 6 + (value - 1).
pub type Card = u8;

pub fn value(c: Card) -> u32 {
    u32::from(c % 6) + 1
}

pub fn colour(c: Card) -> u8 {
    c / 6
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Play,
    Result,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Play {
    /// None for a pass.
    pub card: Option<Card>,
    pub stack: u8,
    pub rainbow: bool,
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    pub hand: Vec<Card>,
    pub rainbow: bool,
    pub passes: u32,
    pub play: Option<Play>,
    pub score: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Outcome {
    Stays,
    Full,
    Overloaded,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StackLog {
    pub before: u32,
    pub added: u32,
    pub outcome: Outcome,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundLog {
    pub round: usize,
    /// (seat, play, points this round)
    pub plays: Vec<(String, Play, i64)>,
    pub stacks: Vec<StackLog>,
}

#[derive(Clone, Debug)]
pub struct HvState {
    pub seats: Vec<Seat>,
    pub stacks: [u32; 4],
    pub phase: Phase,
    pub round: usize,
    pub time_left_ms: i64,
    pub log: Vec<RoundLog>,
    /// Replay moments: a rainbow card that fills a stack to exactly 13, the lead stolen in the last round.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HvMove {
    Play { card: Card, stack: Option<u8> },
    Pass,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub score: i64,
    pub cards: usize,
    pub rainbow: bool,
    pub passes: u32,
    pub done: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HvView {
    pub phase: Phase,
    pub round: usize,
    pub rounds: usize,
    pub stacks: [u32; 4],
    pub hand: Vec<Card>,
    pub rainbow: bool,
    pub passes: u32,
    pub my_play: Option<Play>,
    pub seats: Vec<SeatView>,
    pub log: Vec<RoundLog>,
}

/// Points one seat makes from its play, given the stack totals after everyone played.
fn play_points(play: &Play, before: &[u32; 4], added: &[u32; 4]) -> i64 {
    let Some(card) = play.card else { return 0 };
    let v = i64::from(value(card));
    let total = before[play.stack as usize] + added[play.stack as usize];
    v + match total.cmp(&LIMIT) {
        std::cmp::Ordering::Greater => -2 * v,
        std::cmp::Ordering::Equal => FULL_BONUS,
        std::cmp::Ordering::Less => 0,
    }
}

impl HvState {
    fn seat(&self, id: &str) -> Option<usize> {
        self.seats.iter().position(|s| s.id == id)
    }

    fn start_round(&mut self) {
        if self.round >= ROUNDS {
            self.phase = Phase::Over;
            self.time_left_ms = 0;
            return;
        }
        self.round += 1;
        for s in &mut self.seats {
            s.play = None;
        }
        self.phase = Phase::Play;
    }

    fn resolve(&mut self) {
        let before = self.stacks;
        let top_before = self.seats.iter().map(|s| s.score).max().unwrap_or(0);
        let mut added = [0u32; 4];
        for p in self.seats.iter().filter_map(|s| s.play) {
            if let Some(c) = p.card {
                added[p.stack as usize] += value(c);
            }
        }
        let mut plays = Vec::new();
        for s in &mut self.seats {
            let play = s.play.unwrap_or(Play { card: None, stack: 0, rainbow: false });
            let pts = play_points(&play, &before, &added);
            s.score += pts;
            plays.push((s.id.clone(), play, pts));
        }
        let mut stacks = Vec::new();
        for k in 0..4 {
            let total = before[k] + added[k];
            let outcome = match total.cmp(&LIMIT) {
                std::cmp::Ordering::Greater => Outcome::Overloaded,
                std::cmp::Ordering::Equal => Outcome::Full,
                std::cmp::Ordering::Less => Outcome::Stays,
            };
            self.stacks[k] = if outcome == Outcome::Stays { total } else { 0 };
            stacks.push(StackLog { before: before[k], added: added[k], outcome });
            // A replay moment: the rainbow chip fills another colour's stack to exactly 13 (every play is shown now).
            if outcome == Outcome::Full {
                for s in &self.seats {
                    if let Some(Play { card: Some(c), rainbow: true, .. }) = s.play.filter(|p| p.stack as usize == k) {
                        self.moments.push(&s.id, format!("Rainbow {} fills {} to exactly 13", value(c), COLOUR_NAMES[k]));
                    }
                }
            }
        }
        // A replay moment: behind before the last round, alone in the lead after it.
        if self.round == ROUNDS {
            let top = self.seats.iter().map(|s| s.score).max().unwrap_or(0);
            if let [leader] = self.seats.iter().filter(|s| s.score == top).collect::<Vec<_>>()[..] {
                let gain = plays.iter().find(|p| p.0 == leader.id).map_or(0, |p| p.2);
                if top - gain < top_before {
                    self.moments.push(&leader.id, "Lead stolen in the very last round");
                }
            }
        }
        self.log.push(RoundLog { round: self.round, plays, stacks });
        self.phase = Phase::Result;
        self.time_left_ms = RESULT_MS;
    }

    fn choose(&mut self, i: usize, mv: HvMove) -> Result<(), String> {
        let s = &mut self.seats[i];
        if s.play.is_some() {
            return Err("You have already played".into());
        }
        match mv {
            HvMove::Pass => {
                if s.passes == 0 {
                    return Err("No passes left".into());
                }
                s.passes -= 1;
                s.play = Some(Play { card: None, stack: 0, rainbow: false });
            }
            HvMove::Play { card, stack } => {
                let pos = s.hand.iter().position(|&c| c == card).ok_or("That card is not in your hand")?;
                let target = stack.unwrap_or(colour(card));
                if target > 3 {
                    return Err("No such stack".into());
                }
                let rainbow = target != colour(card);
                if rainbow && !s.rainbow {
                    return Err("Your rainbow chip is used up".into());
                }
                s.hand.remove(pos);
                if rainbow {
                    s.rainbow = false;
                }
                s.play = Some(Play { card: Some(card), stack: target, rainbow });
            }
        }
        if self.seats.iter().all(|x| x.play.is_some()) {
            self.resolve();
        }
        Ok(())
    }
}

pub struct Rules;

impl Game for Rules {
    type State = HvState;
    type Move = HvMove;
    type View = HvView;

    fn info() -> GameInfo {
        GameInfo {
            id: "heavy-13",
            kind: "prison",
            difficulty: "hard",
            tags: &["cards", "numbers"],
            min_players: 3,
            max_players: 6,
            name: "Heavy 13",
            short: "Number cards, rainbow chips and passes stack coloured rocks; an overloaded stack costs points.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<HvState, String> {
        if !(3..=6).contains(&players.len()) {
            return Err("Heavy 13 needs 3–6 players".into());
        }
        let mut rng = Rng::new(seed);
        let mut deck: Vec<Card> = (0..24).chain(0..24).collect();
        rng.shuffle(&mut deck);
        let seats = players
            .iter()
            .map(|id| {
                let mut hand: Vec<Card> = deck.drain(..HAND).collect();
                hand.sort();
                Seat { id: id.clone(), hand, rainbow: true, passes: PASSES, play: None, score: 0 }
            })
            .collect();
        let mut s =
            HvState { seats, stacks: [0; 4], phase: Phase::Play, round: 0, time_left_ms: 0, log: Vec::new(), moments: Moments::default() };
        s.start_round();
        Ok(s)
    }

    fn apply_move(s: &mut HvState, player: &str, mv: HvMove) -> Result<(), String> {
        let i = s.seat(player).ok_or("Not at this table")?;
        if s.phase != Phase::Play {
            return Err("Not now".into());
        }
        s.choose(i, mv)
    }

    fn tick(s: &mut HvState, ms: u64) {
        if s.phase != Phase::Result {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.start_round();
        }
    }

    fn view_for(s: &HvState, player: &str) -> HvView {
        let mine = s.seat(player).map(|i| &s.seats[i]);
        HvView {
            phase: s.phase,
            round: s.round,
            rounds: ROUNDS,
            stacks: s.stacks,
            hand: mine.map(|m| m.hand.clone()).unwrap_or_default(),
            rainbow: mine.is_some_and(|m| m.rainbow),
            passes: mine.map_or(0, |m| m.passes),
            my_play: mine.and_then(|m| m.play),
            seats: s
                .seats
                .iter()
                .map(|x| {
                    // While the round runs, show the counts from before this round's secret choice.
                    let p = x.play.filter(|_| s.phase == Phase::Play);
                    SeatView {
                        id: x.id.clone(),
                        score: x.score,
                        cards: x.hand.len() + usize::from(p.is_some_and(|p| p.card.is_some())),
                        rainbow: x.rainbow || p.is_some_and(|p| p.rainbow),
                        passes: x.passes + u32::from(p.is_some_and(|p| p.card.is_none())),
                        done: s.phase != Phase::Play || x.play.is_some(),
                    }
                })
                .collect(),
            log: s.log.clone(),
        }
    }

    fn is_over(s: &HvState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &HvState) -> Vec<PlayerResult> {
        points_by_rank(&s.seats.iter().map(|x| (x.id.clone(), x.score)).collect::<Vec<_>>())
    }

    fn to_move(s: &HvState) -> Vec<String> {
        if s.phase != Phase::Play {
            return Vec::new();
        }
        s.seats.iter().filter(|x| x.play.is_none()).map(|x| x.id.clone()).collect()
    }

    fn bot_move(s: &HvState, player: &str, seed: u64) -> HvMove {
        let mut rng = Rng::new(seed);
        let i = s.seat(player).unwrap_or(0);
        let me = &s.seats[i];
        // Imagined additions of the other seats to each stack.
        let samples: Vec<[u32; 4]> = (0..40)
            .map(|_| {
                let mut add = [0u32; 4];
                for _ in 0..s.seats.len() - 1 {
                    if rng.next_f64() < 0.25 {
                        continue;
                    }
                    add[rng.index(4)] += rng.int(1, 6) as u32;
                }
                add
            })
            .collect();
        let average = |play: &Play| -> f64 {
            samples
                .iter()
                .map(|add| {
                    let mut add = *add;
                    if let Some(c) = play.card {
                        add[play.stack as usize] += value(c);
                    }
                    play_points(play, &s.stacks, &add) as f64
                })
                .sum::<f64>()
                / samples.len() as f64
        };
        let mut best: Option<(f64, HvMove)> = None;
        let mut consider = |score: f64, mv: HvMove| {
            if best.as_ref().is_none_or(|b| score > b.0) {
                best = Some((score, mv));
            }
        };
        if me.passes > 0 {
            consider(-1.5 + rng.next_f64() * 0.5, HvMove::Pass);
        }
        for &card in &me.hand {
            let own = Play { card: Some(card), stack: colour(card), rainbow: false };
            consider(average(&own) + rng.next_f64() * 0.5, HvMove::Play { card, stack: None });
            if me.rainbow {
                for k in (0..4u8).filter(|&k| k != colour(card)) {
                    let p = Play { card: Some(card), stack: k, rainbow: true };
                    consider(average(&p) - 1.5 + rng.next_f64() * 0.5, HvMove::Play { card, stack: Some(k) });
                }
            }
        }
        best.map_or(HvMove::Pass, |b| b.1)
    }

    fn moments(s: &HvState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
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

    /// Card from colour (0–3) and value (1–6).
    fn c(colour: u8, v: u8) -> Card {
        colour * 6 + v - 1
    }

    fn give(s: &mut HvState, hands: &[&[Card]]) {
        for (seat, h) in s.seats.iter_mut().zip(hands) {
            seat.hand = h.to_vec();
        }
    }

    #[test]
    fn overload_costs_and_exact_13_pays() {
        let mut s = Rules::setup(&ids(3), 1).unwrap();
        give(&mut s, &[&[c(0, 6)], &[c(0, 5)], &[c(1, 3)]]);
        s.stacks = [4, 10, 0, 0];
        Rules::apply_move(&mut s, "p1", HvMove::Play { card: c(0, 6), stack: None }).unwrap();
        Rules::apply_move(&mut s, "p2", HvMove::Play { card: c(0, 5), stack: None }).unwrap();
        // Red: 4 + 6 + 5 = 15 → overloaded. Blue: 10 + 3 = 13 → full.
        Rules::apply_move(&mut s, "p3", HvMove::Play { card: c(1, 3), stack: None }).unwrap();
        assert_eq!(s.phase, Phase::Result);
        assert_eq!(s.seats.iter().map(|x| x.score).collect::<Vec<_>>(), vec![-6, -5, 6]);
        assert_eq!(s.stacks, [0, 0, 0, 0]);
        assert_eq!(s.log[0].stacks[0].outcome, Outcome::Overloaded);
        assert_eq!(s.log[0].stacks[1].outcome, Outcome::Full);
    }

    #[test]
    fn rainbow_chip_and_passes_are_limited() {
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        give(&mut s, &[&[c(0, 2), c(1, 2)], &[c(2, 1)], &[c(3, 1)]]);
        assert!(Rules::apply_move(&mut s, "p1", HvMove::Play { card: c(2, 5), stack: None }).is_err());
        Rules::apply_move(&mut s, "p1", HvMove::Play { card: c(0, 2), stack: Some(3) }).unwrap();
        assert!(!s.seats[0].rainbow);
        // The others cannot tell yet what p1 did.
        let seen = &Rules::view_for(&s, "p2").seats[0];
        assert_eq!((seen.cards, seen.rainbow, seen.passes), (2, true, PASSES));
        assert!(Rules::apply_move(&mut s, "p1", HvMove::Pass).is_err());
        Rules::apply_move(&mut s, "p2", HvMove::Pass).unwrap();
        Rules::apply_move(&mut s, "p3", HvMove::Pass).unwrap();
        assert_eq!(s.stacks, [0, 0, 0, 2]);
        Rules::tick(&mut s, RESULT_MS as u64);
        assert!(Rules::apply_move(&mut s, "p1", HvMove::Play { card: c(1, 2), stack: Some(0) }).is_err());
        // Hands stay secret.
        let v = serde_json::to_value(Rules::view_for(&s, "p2")).unwrap();
        assert_eq!(v["seats"][0]["cards"], 1);
        assert!(v["seats"][0].get("hand").is_none());
    }

    #[test]
    fn the_round_waits_for_every_seat() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        Rules::apply_move(&mut s, "p1", HvMove::Pass).unwrap();
        Rules::apply_move(&mut s, "p2", HvMove::Pass).unwrap();
        Rules::tick(&mut s, 600_000);
        assert_eq!((s.phase, s.round), (Phase::Play, 1));
        assert_eq!(Rules::to_move(&s), vec!["p3".to_string()]);
        assert_eq!((s.seats[2].passes, s.seats[2].hand.len()), (PASSES, HAND));
    }

    #[test]
    fn bot_avoids_a_sure_overload() {
        let mut s = Rules::setup(&ids(3), 4).unwrap();
        give(&mut s, &[&[c(0, 6), c(1, 1)], &[c(2, 1)], &[c(3, 1)]]);
        s.seats[0].rainbow = false;
        s.stacks = [12, 0, 0, 0];
        for seed in 0..20 {
            assert!(!matches!(Rules::bot_move(&s, "p1", seed), HvMove::Play { card, .. } if card == c(0, 6)));
        }
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(400, 4, 5_000);
        bot_games_with::<Rules>(300, 3, 5_000);
        bot_games_with::<Rules>(300, 6, 5_000);
    }

    #[test]
    fn a_rainbow_fill_and_a_last_round_steal_are_replay_moments() {
        let mut s = Rules::setup(&ids(3), 5).unwrap();
        give(&mut s, &[&[c(0, 4)], &[c(1, 3)], &[]]);
        s.stacks = [0, 10, 0, 9];
        // p1's red 4 on gold with the rainbow chip: 9 + 4 = 13. p2's blue 3 fills blue the normal way.
        Rules::apply_move(&mut s, "p1", HvMove::Play { card: c(0, 4), stack: Some(3) }).unwrap();
        Rules::apply_move(&mut s, "p2", HvMove::Play { card: c(1, 3), stack: None }).unwrap();
        Rules::apply_move(&mut s, "p3", HvMove::Pass).unwrap();
        let m = Rules::moments(&s, "p3");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Rainbow 4 fills gold to exactly 13"));
        // The last round: p1 trails p2 by 2 and plays a 6.
        let mut s = Rules::setup(&ids(3), 5).unwrap();
        s.round = ROUNDS;
        give(&mut s, &[&[c(0, 6)], &[c(1, 1)], &[]]);
        for (seat, score) in s.seats.iter_mut().zip([10, 12, 0]) {
            seat.score = score;
        }
        Rules::apply_move(&mut s, "p1", HvMove::Play { card: c(0, 6), stack: None }).unwrap();
        Rules::apply_move(&mut s, "p2", HvMove::Play { card: c(1, 1), stack: None }).unwrap();
        Rules::apply_move(&mut s, "p3", HvMove::Pass).unwrap();
        let m = Rules::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Lead stolen in the very last round"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 5_000);
        // Without timeouts there are no forced passes or lowest cards, so fewer last-round swings.
        assert!(stats.with_any * 5 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 3, "too many in one game: {stats:?}");
    }
}

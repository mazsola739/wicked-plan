//! Triple Dice (prison game, 2–6 seats). Rules page: docs/games/dice-prediction.md
//!
//! Three dice each: one open, one secret (only you see it), one blind (everyone
//! but you sees it). Reroll the secret die once if you like, then predict your
//! own rank at the table.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 6;
pub const RESULT_MS: i64 = 6_000;
/// Exact rank calls in a row for a replay moment.
pub const STREAK: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Build,
    Predict,
    Result,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Nothing,
    Pair,
    Straight,
    Triple,
}

pub fn kind(d: [u8; 3]) -> Kind {
    let mut s = d;
    s.sort();
    if s[0] == s[2] {
        Kind::Triple
    } else if s[0] + 1 == s[1] && s[1] + 1 == s[2] {
        Kind::Straight
    } else if s[0] == s[1] || s[1] == s[2] {
        Kind::Pair
    } else {
        Kind::Nothing
    }
}

/// Bigger is better: kind first, then the sum.
pub fn hand_value(d: [u8; 3]) -> u32 {
    kind(d) as u32 * 100 + d.iter().map(|&x| u32::from(x)).sum::<u32>()
}

/// Rank of each hand: 1 + the number of strictly better hands.
pub fn ranks(values: &[u32]) -> Vec<usize> {
    values.iter().map(|v| 1 + values.iter().filter(|o| *o > v).count()).collect()
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    /// open, secret, blind
    pub dice: [u8; 3],
    pub built: bool,
    pub rerolled: bool,
    pub prediction: Option<usize>,
    pub score: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatLog {
    pub id: String,
    pub dice: [u8; 3],
    pub kind: Kind,
    pub rank: usize,
    pub prediction: usize,
    pub points: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundLog {
    pub round: usize,
    pub seats: Vec<SeatLog>,
}

#[derive(Clone, Debug)]
pub struct DpState {
    pub seats: Vec<Seat>,
    pub rng: Rng,
    pub phase: Phase,
    pub round: usize,
    pub time_left_ms: i64,
    pub log: Vec<RoundLog>,
    /// Replay moments: a triple whose rank was called exactly, 4 or more exact calls in a row.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DpMove {
    Keep,
    Reroll,
    Predict { rank: usize },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub score: i64,
    pub open: u8,
    /// Other seats' blind dice; null for your own.
    pub blind: Option<u8>,
    pub rerolled: bool,
    pub done: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DpView {
    pub phase: Phase,
    pub round: usize,
    pub rounds: usize,
    pub players: usize,
    pub open: u8,
    pub secret: u8,
    pub rerolled: bool,
    pub built: bool,
    pub prediction: Option<usize>,
    pub seats: Vec<SeatView>,
    pub log: Vec<RoundLog>,
}

impl DpState {
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
            s.dice = [0; 3].map(|_| self.rng.int(1, 6) as u8);
            s.built = false;
            s.rerolled = false;
            s.prediction = None;
        }
        self.phase = Phase::Build;
    }

    fn build(&mut self, i: usize, reroll: bool) {
        if reroll {
            self.seats[i].dice[1] = self.rng.int(1, 6) as u8;
            self.seats[i].rerolled = true;
        }
        self.seats[i].built = true;
        if self.seats.iter().all(|s| s.built) {
            self.phase = Phase::Predict;
        }
    }

    fn score_round(&mut self) {
        let n = self.seats.len();
        let values: Vec<u32> = self.seats.iter().map(|s| hand_value(s.dice)).collect();
        let r = ranks(&values);
        let mut seats = Vec::new();
        for (i, s) in self.seats.iter_mut().enumerate() {
            let prediction = s.prediction.unwrap_or(n.div_ceil(2));
            let mut points = match prediction.abs_diff(r[i]) {
                0 => 3,
                1 => 1,
                _ => 0,
            };
            if r[i] == 1 {
                points += 1;
            }
            s.score += points;
            // Replay moments (every die and prediction is shown now).
            if prediction == r[i] {
                let streak = 1 + self.log.iter().rev().take_while(|l| l.seats[i].prediction == l.seats[i].rank).count();
                if kind(s.dice) == Kind::Triple {
                    self.moments.push(&s.id, format!("Triple {}s, rank {} called", s.dice[0], r[i]));
                } else if streak >= STREAK {
                    self.moments.push(&s.id, format!("{streak} exact rank calls in a row"));
                }
            }
            seats.push(SeatLog { id: s.id.clone(), dice: s.dice, kind: kind(s.dice), rank: r[i], prediction, points });
        }
        self.log.push(RoundLog { round: self.round, seats });
        self.phase = Phase::Result;
        self.time_left_ms = RESULT_MS;
    }
}

pub struct Rules;

impl Game for Rules {
    type State = DpState;
    type Move = DpMove;
    type View = DpView;

    fn info() -> GameInfo {
        GameInfo {
            id: "dice-prediction",
            kind: "prison",
            difficulty: "medium",
            tags: &["bluff-deduction", "speed-risk"],
            min_players: 2,
            max_players: 6,
            name: "Triple Dice",
            short: "Build a three-dice hand with hidden dice, then predict your own rank.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<DpState, String> {
        if !(2..=6).contains(&players.len()) {
            return Err("Triple Dice needs 2–6 players".into());
        }
        let seats = players
            .iter()
            .map(|id| Seat { id: id.clone(), dice: [1; 3], built: false, rerolled: false, prediction: None, score: 0 })
            .collect();
        let mut s = DpState {
            seats,
            rng: Rng::new(seed),
            phase: Phase::Build,
            round: 0,
            time_left_ms: 0,
            log: Vec::new(),
            moments: Moments::default(),
        };
        s.start_round();
        Ok(s)
    }

    fn apply_move(s: &mut DpState, player: &str, mv: DpMove) -> Result<(), String> {
        let i = s.seat(player).ok_or("Not at this table")?;
        match (mv, s.phase) {
            (DpMove::Keep | DpMove::Reroll, Phase::Build) if s.seats[i].built => Err("You have already decided".into()),
            (DpMove::Keep, Phase::Build) => {
                s.build(i, false);
                Ok(())
            }
            (DpMove::Reroll, Phase::Build) => {
                s.build(i, true);
                Ok(())
            }
            (DpMove::Predict { rank }, Phase::Predict) => {
                if s.seats[i].prediction.is_some() {
                    return Err("You have already predicted".into());
                }
                if rank < 1 || rank > s.seats.len() {
                    return Err("Pick a rank at this table".into());
                }
                s.seats[i].prediction = Some(rank);
                if s.seats.iter().all(|x| x.prediction.is_some()) {
                    s.score_round();
                }
                Ok(())
            }
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut DpState, ms: u64) {
        if s.phase != Phase::Result {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.start_round();
        }
    }

    fn view_for(s: &DpState, player: &str) -> DpView {
        let me = s.seat(player);
        let mine = me.map(|i| &s.seats[i]);
        DpView {
            phase: s.phase,
            round: s.round,
            rounds: ROUNDS,
            players: s.seats.len(),
            open: mine.map_or(0, |m| m.dice[0]),
            secret: mine.map_or(0, |m| m.dice[1]),
            rerolled: mine.is_some_and(|m| m.rerolled),
            built: mine.is_some_and(|m| m.built),
            prediction: mine.and_then(|m| m.prediction),
            seats: s
                .seats
                .iter()
                .enumerate()
                .map(|(i, x)| SeatView {
                    id: x.id.clone(),
                    score: x.score,
                    open: x.dice[0],
                    blind: (Some(i) != me).then_some(x.dice[2]),
                    rerolled: x.rerolled,
                    done: match s.phase {
                        Phase::Build => x.built,
                        Phase::Predict => x.prediction.is_some(),
                        _ => true,
                    },
                })
                .collect(),
            log: s.log.clone(),
        }
    }

    fn is_over(s: &DpState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &DpState) -> Vec<PlayerResult> {
        points_by_rank(&s.seats.iter().map(|x| (x.id.clone(), x.score)).collect::<Vec<_>>())
    }

    fn to_move(s: &DpState) -> Vec<String> {
        let waiting = |x: &&Seat| match s.phase {
            Phase::Build => !x.built,
            Phase::Predict => x.prediction.is_none(),
            _ => false,
        };
        s.seats.iter().filter(waiting).map(|x| x.id.clone()).collect()
    }

    fn bot_move(s: &DpState, player: &str, seed: u64) -> DpMove {
        let mut rng = Rng::new(seed);
        let i = s.seat(player).unwrap_or(0);
        let [open, secret, _] = s.seats[i].dice;
        if s.phase == Phase::Build {
            // Average value over the blind die (unknown to this seat).
            let keep: u32 = (1..=6).map(|b| hand_value([open, secret, b])).sum();
            let reroll: u32 = (1..=6).flat_map(|n| (1..=6).map(move |b| hand_value([open, n, b]))).sum::<u32>() / 6;
            let wants = reroll > keep;
            return if wants != (rng.next_f64() < 0.1) { DpMove::Reroll } else { DpMove::Keep };
        }
        // Deal the unknown dice at random and count how often each rank comes up.
        let n = s.seats.len();
        let mut counts = vec![0u32; n + 1];
        for _ in 0..120 {
            let values: Vec<u32> = s
                .seats
                .iter()
                .enumerate()
                .map(|(j, x)| {
                    let unknown = rng.int(1, 6) as u8;
                    if j == i { hand_value([open, secret, unknown]) } else { hand_value([x.dice[0], unknown, x.dice[2]]) }
                })
                .collect();
            counts[ranks(&values)[i]] += 1;
        }
        let rank = (1..=n).max_by_key(|&r| counts[r]).unwrap_or(1);
        DpMove::Predict { rank }
    }

    fn moments(s: &DpState, player: &str) -> Vec<Moment> {
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

    #[test]
    fn hand_kinds_and_ranks() {
        assert_eq!(kind([4, 4, 4]), Kind::Triple);
        assert_eq!(kind([5, 3, 4]), Kind::Straight);
        assert_eq!(kind([2, 6, 2]), Kind::Pair);
        assert_eq!(kind([1, 3, 6]), Kind::Nothing);
        assert!(hand_value([1, 1, 1]) > hand_value([4, 5, 6]));
        assert!(hand_value([1, 2, 3]) > hand_value([6, 6, 5]));
        assert_eq!(ranks(&[10, 30, 10, 5]), vec![2, 1, 2, 4]);
    }

    #[test]
    fn blind_die_is_hidden_from_its_owner_only() {
        let s = Rules::setup(&ids(3), 1).unwrap();
        let v = Rules::view_for(&s, "p1");
        assert_eq!(v.seats[0].blind, None);
        assert_eq!(v.seats[1].blind, Some(s.seats[1].dice[2]));
        assert_eq!(v.secret, s.seats[0].dice[1]);
        // Nothing else in the view carries the other seats' secret dice.
        assert!(serde_json::to_value(&v).unwrap()["seats"][1].get("secret").is_none());
    }

    #[test]
    fn predictions_score() {
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        s.seats[0].dice = [6, 6, 6];
        s.seats[1].dice = [1, 2, 3];
        s.seats[2].dice = [1, 3, 5];
        for id in ids(3) {
            Rules::apply_move(&mut s, &id, DpMove::Keep).unwrap();
        }
        assert!(Rules::apply_move(&mut s, "p1", DpMove::Predict { rank: 4 }).is_err());
        Rules::apply_move(&mut s, "p1", DpMove::Predict { rank: 1 }).unwrap();
        Rules::apply_move(&mut s, "p2", DpMove::Predict { rank: 3 }).unwrap();
        // No time limit: the round waits for p3.
        Rules::tick(&mut s, 600_000);
        assert_eq!(s.phase, Phase::Predict);
        Rules::apply_move(&mut s, "p3", DpMove::Predict { rank: 2 }).unwrap();
        // p1 exact + best hand, p2 off by one, p3 predicts 2 (rank 3): off by one.
        assert_eq!(s.seats.iter().map(|x| x.score).collect::<Vec<_>>(), vec![4, 1, 1]);
        assert_eq!(s.phase, Phase::Result);
    }

    /// Plays one round with these dice and predictions (everyone keeps).
    fn round(s: &mut DpState, dice: &[[u8; 3]], predict: &[usize]) {
        for (i, d) in dice.iter().enumerate() {
            s.seats[i].dice = *d;
            Rules::apply_move(s, &format!("p{}", i + 1), DpMove::Keep).unwrap();
        }
        for (i, &rank) in predict.iter().enumerate() {
            Rules::apply_move(s, &format!("p{}", i + 1), DpMove::Predict { rank }).unwrap();
        }
        Rules::tick(s, RESULT_MS as u64);
    }

    #[test]
    fn a_called_triple_and_a_streak_of_exact_calls_are_replay_moments() {
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        // p1 calls its triple; p2's triple is called wrong, no moment.
        round(&mut s, &[[5, 5, 5], [2, 2, 2], [1, 3, 6]], &[1, 1, 3]);
        let m = Rules::moments(&s, "p3");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Triple 5s, rank 1 called"));
        // p3 calls rank 3 four times in a row: the fourth is a moment.
        for _ in 0..3 {
            round(&mut s, &[[6, 6, 1], [1, 2, 3], [1, 3, 6]], &[3, 3, 3]);
        }
        let m = Rules::moments(&s, "p2");
        assert_eq!(m.len(), 2);
        assert_eq!((m[1].by.as_str(), m[1].text.as_str()), ("p3", "4 exact rank calls in a row"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        // Without timeouts every seat predicts (no defaulted middle ranks), so a lucky game has a few more.
        assert!(stats.most <= 6, "too many in one game: {stats:?}");
    }

    #[test]
    fn bot_predicts_first_with_a_strong_hand() {
        let mut s = Rules::setup(&ids(2), 3).unwrap();
        s.seats[0].dice = [6, 6, 6];
        s.seats[1].dice = [1, 2, 6];
        s.phase = Phase::Predict;
        assert!(matches!(Rules::bot_move(&s, "p1", 5), DpMove::Predict { rank: 1 }));
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(400, 4, 5_000);
        bot_games_with::<Rules>(300, 2, 5_000);
        bot_games_with::<Rules>(300, 6, 5_000);
    }
}

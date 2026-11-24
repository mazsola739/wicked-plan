//! Remove One (prison game, 3–6 seats). Rules page: docs/games/minus-one.md
//!
//! Everyone shows two cards from 1–10, then keeps one; the lowest unique kept
//! card scores the sum of all kept cards.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 7;
pub const RESULT_MS: i64 = 5_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Show,
    Keep,
    Result,
    Over,
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    pub hand: Vec<u8>,
    pub pair: Option<[u8; 2]>,
    pub kept: Option<u8>,
    pub score: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundLog {
    pub round: usize,
    /// (seat, kept card), in seat order.
    pub kept: Vec<(String, u8)>,
    pub winner: Option<String>,
    pub pot: i64,
}

#[derive(Clone, Debug)]
pub struct MoState {
    pub seats: Vec<Seat>,
    pub phase: Phase,
    pub round: usize,
    /// The Result pause.
    pub time_left_ms: i64,
    pub log: Vec<RoundLog>,
    /// Replay moments: a 1 or a 2 that takes a pot of 22 or more.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MoMove {
    Show { cards: [u8; 2] },
    Keep { card: u8 },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub score: i64,
    pub hand_size: usize,
    /// Whether this seat has made its choice in the current phase.
    pub done: bool,
    /// Shown once everyone has shown (Keep and Result phases).
    pub pair: Option<[u8; 2]>,
    /// Shown in the Result phase.
    pub kept: Option<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoView {
    pub phase: Phase,
    pub round: usize,
    pub rounds: usize,
    pub hand: Vec<u8>,
    pub my_pair: Option<[u8; 2]>,
    pub my_kept: Option<u8>,
    pub seats: Vec<SeatView>,
    pub log: Vec<RoundLog>,
}

/// The lowest card kept by exactly one seat, as a seat index.
pub fn lowest_unique(kept: &[u8]) -> Option<usize> {
    let unique = |c: u8| kept.iter().filter(|&&k| k == c).count() == 1;
    kept.iter().enumerate().filter(|&(_, &c)| unique(c)).min_by_key(|&(_, &c)| c).map(|(i, _)| i)
}

impl MoState {
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
            s.pair = None;
            s.kept = None;
        }
        self.phase = Phase::Show;
    }

    fn after_move(&mut self) {
        match self.phase {
            Phase::Show if self.seats.iter().all(|s| s.pair.is_some()) => self.phase = Phase::Keep,
            Phase::Keep if self.seats.iter().all(|s| s.kept.is_some()) => self.score_round(),
            _ => {}
        }
    }

    fn score_round(&mut self) {
        let kept: Vec<u8> = self.seats.iter().map(|s| s.kept.unwrap_or(0)).collect();
        let pot: i64 = kept.iter().map(|&k| i64::from(k)).sum();
        let winner = lowest_unique(&kept);
        if let Some(w) = winner {
            self.seats[w].score += pot;
            // A replay moment: a 1 or a 2 snatching a pot of 22 or more (every kept card is public now).
            let card = i64::from(kept[w]);
            if card <= 2 && pot >= 22 {
                self.moments.push(&self.seats[w].id.clone(), format!("A lone {card} takes the pot of {pot}"));
            }
        }
        for (s, &k) in self.seats.iter_mut().zip(&kept) {
            if let Some(i) = s.hand.iter().position(|&c| c == k) {
                s.hand.remove(i);
            }
        }
        self.log.push(RoundLog {
            round: self.round,
            kept: self.seats.iter().zip(&kept).map(|(s, &k)| (s.id.clone(), k)).collect(),
            winner: winner.map(|w| self.seats[w].id.clone()),
            pot: if winner.is_some() { pot } else { 0 },
        });
        self.phase = Phase::Result;
        self.time_left_ms = RESULT_MS;
    }
}

pub struct Rules;

impl Game for Rules {
    type State = MoState;
    type Move = MoMove;
    type View = MoView;

    fn info() -> GameInfo {
        GameInfo {
            id: "minus-one",
            kind: "prison",
            difficulty: "medium",
            tags: &["strategy-territory", "bluff-deduction"],
            min_players: 3,
            max_players: 6,
            name: "Remove One",
            short: "Show two cards, keep one; the lowest unique card wins the round.",
        }
    }

    fn setup(players: &[String], _seed: u64) -> Result<MoState, String> {
        if !(3..=6).contains(&players.len()) {
            return Err("Remove One needs 3–6 players".into());
        }
        let seats =
            players.iter().map(|id| Seat { id: id.clone(), hand: (1..=10).collect(), pair: None, kept: None, score: 0 }).collect();
        let mut s = MoState { seats, phase: Phase::Show, round: 0, time_left_ms: 0, log: Vec::new(), moments: Moments::default() };
        s.start_round();
        Ok(s)
    }

    fn apply_move(s: &mut MoState, player: &str, mv: MoMove) -> Result<(), String> {
        let i = s.seat(player).ok_or("Not at this table")?;
        let seat = &mut s.seats[i];
        match (mv, s.phase) {
            (MoMove::Show { cards: [a, b] }, Phase::Show) => {
                if seat.pair.is_some() {
                    return Err("You have already shown".into());
                }
                if a == b || !seat.hand.contains(&a) || !seat.hand.contains(&b) {
                    return Err("Pick two different cards from your hand".into());
                }
                seat.pair = Some([a.min(b), a.max(b)]);
            }
            (MoMove::Keep { card }, Phase::Keep) => {
                if seat.kept.is_some() {
                    return Err("You have already kept a card".into());
                }
                if !seat.pair.is_some_and(|p| p.contains(&card)) {
                    return Err("Keep one of the two cards you showed".into());
                }
                seat.kept = Some(card);
            }
            _ => return Err("Not now".into()),
        }
        s.after_move();
        Ok(())
    }

    fn tick(s: &mut MoState, ms: u64) {
        if s.phase != Phase::Result {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.start_round();
        }
    }

    fn view_for(s: &MoState, player: &str) -> MoView {
        let me = s.seat(player).map(|i| &s.seats[i]);
        let open = matches!(s.phase, Phase::Keep | Phase::Result);
        MoView {
            phase: s.phase,
            round: s.round,
            rounds: ROUNDS,
            hand: me.map(|m| m.hand.clone()).unwrap_or_default(),
            my_pair: me.and_then(|m| m.pair),
            my_kept: me.and_then(|m| m.kept),
            seats: s
                .seats
                .iter()
                .map(|x| SeatView {
                    id: x.id.clone(),
                    score: x.score,
                    hand_size: x.hand.len(),
                    done: match s.phase {
                        Phase::Show => x.pair.is_some(),
                        Phase::Keep => x.kept.is_some(),
                        _ => true,
                    },
                    pair: if open { x.pair } else { None },
                    kept: if s.phase == Phase::Result { x.kept } else { None },
                })
                .collect(),
            log: s.log.clone(),
        }
    }

    fn is_over(s: &MoState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &MoState) -> Vec<PlayerResult> {
        points_by_rank(&s.seats.iter().map(|x| (x.id.clone(), x.score)).collect::<Vec<_>>())
    }

    fn to_move(s: &MoState) -> Vec<String> {
        let waiting = |x: &&Seat| match s.phase {
            Phase::Show => x.pair.is_none(),
            Phase::Keep => x.kept.is_none(),
            _ => false,
        };
        s.seats.iter().filter(waiting).map(|x| x.id.clone()).collect()
    }

    fn bot_move(s: &MoState, player: &str, seed: u64) -> MoMove {
        let mut rng = Rng::new(seed);
        let i = s.seat(player).unwrap_or(0);
        let me = &s.seats[i];
        if s.phase != Phase::Keep {
            let mut h = me.hand.clone();
            h.sort();
            let low = h[rng.index(h.len().min(3))];
            let others: Vec<u8> = h.iter().copied().filter(|&c| c != low).collect();
            let other = *rng.pick(&others);
            let cards = if rng.next_f64() < 0.15 {
                rng.shuffle(&mut h);
                [h[0], h[1]]
            } else {
                [low, other]
            };
            return MoMove::Show { cards };
        }
        let [a, b] = me.pair.unwrap_or([0, 0]);
        if rng.next_f64() < 0.1 {
            return MoMove::Keep { card: if rng.next_f64() < 0.5 { a } else { b } };
        }
        let rounds_left = ROUNDS - s.round;
        let value = |c: u8| keep_value(s, i, c) - if rounds_left > 0 { f64::from(11 - c) * 0.4 } else { 0.0 };
        MoMove::Keep { card: if value(a) >= value(b) { a } else { b } }
    }

    fn moments(s: &MoState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

/// Expected pot won by keeping `card`, over every way the other seats may keep
/// (each keeps its lower card 60% of the time).
fn keep_value(s: &MoState, me: usize, card: u8) -> f64 {
    let others: Vec<[u8; 2]> = (0..s.seats.len()).filter(|&j| j != me).map(|j| s.seats[j].pair.unwrap_or([0, 0])).collect();
    let mut total = 0.0;
    for mask in 0..(1u32 << others.len()) {
        let mut p = 1.0;
        let mut kept = vec![card];
        for (k, pair) in others.iter().enumerate() {
            let high = mask & (1 << k) != 0;
            p *= if high { 0.4 } else { 0.6 };
            kept.push(if high { pair[1] } else { pair[0] });
        }
        if lowest_unique(&kept) == Some(0) {
            total += p * kept.iter().map(|&c| f64::from(c)).sum::<f64>();
        }
    }
    total
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

    fn play(s: &mut MoState, pairs: &[[u8; 2]], keeps: &[u8]) {
        for (i, &cards) in pairs.iter().enumerate() {
            Rules::apply_move(s, &format!("p{}", i + 1), MoMove::Show { cards }).unwrap();
        }
        for (i, &card) in keeps.iter().enumerate() {
            Rules::apply_move(s, &format!("p{}", i + 1), MoMove::Keep { card }).unwrap();
        }
    }

    #[test]
    fn lowest_unique_card_wins_the_pot() {
        assert_eq!(lowest_unique(&[1, 1, 3, 5]), Some(2));
        assert_eq!(lowest_unique(&[2, 2, 4, 4]), None);
        let mut s = Rules::setup(&ids(4), 1).unwrap();
        play(&mut s, &[[1, 5], [1, 9], [3, 4], [2, 7]], &[1, 1, 3, 7]);
        assert_eq!(s.phase, Phase::Result);
        assert_eq!(s.seats[2].score, 12);
        assert!(!s.seats[0].hand.contains(&1));
        assert!(s.seats[0].hand.contains(&5));
        assert_eq!(s.seats[0].hand.len(), 9);
        assert_eq!(s.log[0].winner.as_deref(), Some("p3"));
    }

    #[test]
    fn pairs_stay_secret_until_everyone_has_shown() {
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        Rules::apply_move(&mut s, "p1", MoMove::Show { cards: [2, 6] }).unwrap();
        assert!(Rules::view_for(&s, "p2").seats[0].pair.is_none());
        assert!(Rules::apply_move(&mut s, "p2", MoMove::Show { cards: [2, 2] }).is_err());
        assert!(Rules::apply_move(&mut s, "p2", MoMove::Keep { card: 2 }).is_err());
        // No time limit: a long wait changes nothing, the round waits for p2 and p3.
        Rules::tick(&mut s, 600_000);
        assert_eq!(s.phase, Phase::Show);
        assert_eq!(Rules::to_move(&s), vec!["p2".to_string(), "p3".to_string()]);
        Rules::apply_move(&mut s, "p2", MoMove::Show { cards: [1, 2] }).unwrap();
        Rules::apply_move(&mut s, "p3", MoMove::Show { cards: [1, 2] }).unwrap();
        assert_eq!(s.phase, Phase::Keep);
        assert_eq!(Rules::view_for(&s, "p2").seats[0].pair, Some([2, 6]));
        assert_eq!(s.seats[1].pair, Some([1, 2]));
        assert!(Rules::view_for(&s, "p2").seats[0].kept.is_none());
        for (p, card) in [("p1", 2), ("p2", 1), ("p3", 1)] {
            Rules::apply_move(&mut s, p, MoMove::Keep { card }).unwrap();
        }
        // Kept 2, 1, 1 → 2 is the lowest unique one.
        assert_eq!(s.seats[0].score, 4);
    }

    #[test]
    fn bot_keeps_the_card_that_wins() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        // p1 shows 2 and 10; the others show 3–6, all different: the 2 always wins, the 10 never.
        play(&mut s, &[[2, 10], [3, 4], [5, 6]], &[]);
        assert_eq!(keep_value(&s, 0, 10), 0.0);
        assert!(keep_value(&s, 0, 2) > 10.0);
        assert!(matches!(Rules::bot_move(&s, "p1", 1), MoMove::Keep { card: 2 }) || matches!(Rules::bot_move(&s, "p1", 2), MoMove::Keep { card: 2 }));
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(400, 4, 5_000);
        bot_games_with::<Rules>(300, 3, 5_000);
        bot_games_with::<Rules>(300, 6, 5_000);
    }

    #[test]
    fn a_low_card_taking_a_big_pot_is_a_replay_moment() {
        let mut s = Rules::setup(&ids(4), 1).unwrap();
        // Kept 1, 9, 8, 7: the lone 1 takes a pot of 25.
        play(&mut s, &[[1, 5], [9, 10], [8, 6], [7, 4]], &[1, 9, 8, 7]);
        let m = Rules::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].by, "p1");
        assert_eq!(m[0].text, "A lone 1 takes the pot of 25");
        // A small pot is no moment.
        let mut s = Rules::setup(&ids(4), 1).unwrap();
        play(&mut s, &[[1, 5], [1, 2], [3, 4], [2, 7]], &[1, 1, 3, 2]);
        assert!(Rules::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

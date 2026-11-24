//! Piggy Bank (prison game, 3–6 seats). Rules page: docs/games/piggy-bank.md
//!
//! Drop white/black coins into a bank and pass it; the receiver sees the last
//! drop and claims how many coins of the round's colour are inside. Others may
//! doubt: the bank opens and the wrong side loses a point.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 6;
pub const FULL: u32 = 10;
pub const RESULT_MS: i64 = 5_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Colour {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Drop,
    Claim,
    Doubt,
    Result,
    Over,
}

#[derive(Clone, Copy, Debug)]
pub struct Drop {
    pub seat: usize,
    pub white: u32,
    pub black: u32,
}

impl Drop {
    fn of(&self, c: Colour) -> u32 {
        if c == Colour::White { self.white } else { self.black }
    }
    fn count(&self) -> u32 {
        self.white + self.black
    }
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    pub score: i64,
    /// Doubt (true) or trust (false) for the current claim.
    pub choice: Option<bool>,
    /// What this seat believed at its own last claim: (drops made by then, coins of the colour).
    pub belief: Option<(usize, u32)>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimView {
    pub id: String,
    pub count: u32,
    pub coins: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Opening {
    pub round: usize,
    pub colour: Colour,
    pub claimer: String,
    pub claim: u32,
    pub truth: u32,
    pub white: u32,
    pub black: u32,
    pub doubters: Vec<String>,
    pub losers: Vec<String>,
    /// Opened because the bank was full, not because of a doubt.
    pub full: bool,
}

#[derive(Clone, Debug)]
pub struct PbState {
    pub seats: Vec<Seat>,
    pub phase: Phase,
    pub round: usize,
    pub colour: Colour,
    pub holder: usize,
    pub drops: Vec<Drop>,
    pub claims: Vec<(usize, u32, u32)>,
    /// The last claim nobody doubted this round (0 before any).
    pub accepted: u32,
    pub time_left_ms: i64,
    pub log: Vec<Opening>,
    /// Replay moments: a right claim that two or more doubters pay for; a lone doubter
    /// catching a lie in a bank of 6+ coins.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PbMove {
    Drop { white: u32, black: u32 },
    Claim { count: u32 },
    Doubt,
    Trust,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub score: i64,
    pub holder: bool,
    /// In the doubt step: whether this seat has chosen (not what).
    pub done: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PbView {
    pub phase: Phase,
    pub round: usize,
    pub rounds: usize,
    pub colour: Colour,
    pub my_turn: bool,
    pub coins: u32,
    /// (seat, coins) for every drop this round.
    pub drops: Vec<(String, u32)>,
    pub claims: Vec<ClaimView>,
    pub accepted: u32,
    /// Your own coins of the round's colour and of the other colour this round.
    pub mine: (u32, u32),
    /// As the receiver: the coins (white, black) the previous holder dropped.
    pub seen: Option<(u32, u32)>,
    pub my_choice: Option<bool>,
    pub seats: Vec<SeatView>,
    pub log: Vec<Opening>,
}

impl PbState {
    fn seat(&self, id: &str) -> Option<usize> {
        self.seats.iter().position(|s| s.id == id)
    }

    fn coins(&self) -> u32 {
        self.drops.iter().map(Drop::count).sum()
    }

    fn truth(&self) -> u32 {
        self.drops.iter().map(|d| d.of(self.colour)).sum()
    }

    fn seen_colour(&self) -> u32 {
        self.drops.last().map_or(0, |d| d.of(self.colour))
    }

    fn claimer(&self) -> usize {
        self.claims.last().map_or(self.holder, |c| c.0)
    }

    fn start_round(&mut self, starter: usize) {
        if self.round >= ROUNDS {
            self.phase = Phase::Over;
            self.time_left_ms = 0;
            return;
        }
        self.round += 1;
        if self.round > 1 {
            self.colour = if self.colour == Colour::White { Colour::Black } else { Colour::White };
        }
        self.drops.clear();
        self.claims.clear();
        self.accepted = 0;
        for s in &mut self.seats {
            s.belief = None;
            s.choice = None;
        }
        self.holder = starter;
        self.phase = Phase::Drop;
    }

    fn drop_coins(&mut self, i: usize, white: u32, black: u32) -> Result<(), String> {
        if !(1..=2).contains(&(white + black)) {
            return Err("Drop 1 or 2 coins".into());
        }
        self.drops.push(Drop { seat: i, white, black });
        self.holder = (i + 1) % self.seats.len();
        self.phase = Phase::Claim;
        Ok(())
    }

    fn claim(&mut self, i: usize, count: u32) -> Result<(), String> {
        if count > self.coins() {
            return Err("There are not that many coins inside".into());
        }
        let honest = (self.accepted + self.seen_colour()).min(self.coins());
        self.seats[i].belief = Some((self.drops.len(), honest));
        self.claims.push((i, count, self.coins()));
        for s in &mut self.seats {
            s.choice = None;
        }
        self.phase = Phase::Doubt;
        Ok(())
    }

    fn resolve_doubts(&mut self) {
        let claimer = self.claimer();
        let doubters: Vec<usize> = (0..self.seats.len()).filter(|&j| j != claimer && self.seats[j].choice == Some(true)).collect();
        if !doubters.is_empty() {
            self.open(doubters, false);
        } else if self.coins() >= FULL {
            self.open(Vec::new(), true);
        } else {
            self.accepted = self.claims.last().map_or(0, |c| c.1);
            self.holder = claimer;
            self.phase = Phase::Drop;
        }
    }

    fn open(&mut self, doubters: Vec<usize>, full: bool) {
        let claimer = self.claimer();
        let claim = self.claims.last().map_or(0, |c| c.1);
        let truth = self.truth();
        let losers = if claim == truth { doubters.clone() } else { vec![claimer] };
        for &l in &losers {
            self.seats[l].score -= 1;
        }
        // Replay moments (the opened bank, the doubters and the losers are public): a right claim
        // that two or more doubters pay for, or a lone doubter catching a lie in a bank of 6+ coins.
        let colour = if self.colour == Colour::White { "white" } else { "black" };
        if claim == truth && doubters.len() >= 2 {
            self.moments.push(&self.seats[claimer].id.clone(), format!("Claim of {claim} {colour} holds: {} doubters pay", doubters.len()));
        } else if claim != truth && doubters.len() == 1 && self.coins() >= 6 {
            let by = self.seats[doubters[0]].id.clone();
            let liar = self.seats[claimer].id.clone();
            self.moments.push(&by, format!("Lone doubt: {{{liar}}} claimed {claim} {colour}, there were {truth}"));
        }
        let ids = |v: &[usize]| v.iter().map(|&j| self.seats[j].id.clone()).collect::<Vec<_>>();
        self.log.push(Opening {
            round: self.round,
            colour: self.colour,
            claimer: self.seats[claimer].id.clone(),
            claim,
            truth,
            white: self.drops.iter().map(|d| d.white).sum(),
            black: self.drops.iter().map(|d| d.black).sum(),
            doubters: ids(&doubters),
            losers: ids(&losers),
            full,
        });
        self.holder = claimer;
        self.phase = Phase::Result;
        self.time_left_ms = RESULT_MS;
    }

    /// Whether `j` doubts the current claim (bots).
    fn bot_doubts(&self, j: usize, rng: &mut Rng) -> bool {
        let Some(&(_, claim, coins)) = self.claims.last() else { return false };
        let own = |from: usize, c: Colour| -> u32 { self.drops[from..].iter().filter(|d| d.seat == j).map(|d| d.of(c)).sum() };
        let other = if self.colour == Colour::White { Colour::Black } else { Colour::White };
        // Hard limits from this seat's own coins.
        let mut lo = own(0, self.colour);
        let mut hi = coins - own(0, other);
        // What it believed at its own claim, plus what was dropped since.
        if let Some((at, belief)) = self.seats[j].belief {
            let since: u32 = self.drops[at..].iter().map(Drop::count).sum();
            lo = lo.max(belief + own(at, self.colour));
            hi = hi.min((belief + since).saturating_sub(own(at, other)));
        }
        let new_coins = self.drops.last().map_or(0, Drop::count);
        if claim < lo || claim > hi || claim < self.accepted {
            return rng.next_f64() < 0.9;
        }
        if claim > self.accepted + new_coins {
            return rng.next_f64() < 0.8;
        }
        rng.next_f64() < 0.05
    }
}

pub struct Rules;

impl Game for Rules {
    type State = PbState;
    type Move = PbMove;
    type View = PbView;

    fn info() -> GameInfo {
        GameInfo {
            id: "piggy-bank",
            kind: "prison",
            difficulty: "hard",
            tags: &["bluff-deduction", "memory-observation"],
            min_players: 3,
            max_players: 6,
            name: "Piggy Bank",
            short: "Drop coins in a piggy bank and pass it; the receiver says how many of a colour are inside, others may doubt.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<PbState, String> {
        if !(3..=6).contains(&players.len()) {
            return Err("Piggy Bank needs 3–6 players".into());
        }
        let mut rng = Rng::new(seed);
        let colour = if rng.next_f64() < 0.5 { Colour::White } else { Colour::Black };
        let starter = rng.index(players.len());
        let seats = players.iter().map(|id| Seat { id: id.clone(), score: 0, choice: None, belief: None }).collect();
        let mut s = PbState {
            seats,
            phase: Phase::Drop,
            round: 0,
            colour,
            holder: starter,
            drops: Vec::new(),
            claims: Vec::new(),
            accepted: 0,
            time_left_ms: 0,
            log: Vec::new(),
            moments: Moments::default(),
        };
        s.start_round(starter);
        Ok(s)
    }

    fn apply_move(s: &mut PbState, player: &str, mv: PbMove) -> Result<(), String> {
        let i = s.seat(player).ok_or("Not at this table")?;
        match (mv, s.phase) {
            (PbMove::Drop { white, black }, Phase::Drop) if i == s.holder => s.drop_coins(i, white, black),
            (PbMove::Claim { count }, Phase::Claim) if i == s.holder => s.claim(i, count),
            (mv @ (PbMove::Doubt | PbMove::Trust), Phase::Doubt) => {
                if i == s.claimer() {
                    return Err("You cannot doubt your own claim".into());
                }
                if s.seats[i].choice.is_some() {
                    return Err("You have already chosen".into());
                }
                s.seats[i].choice = Some(matches!(mv, PbMove::Doubt));
                let claimer = s.claimer();
                if (0..s.seats.len()).all(|j| j == claimer || s.seats[j].choice.is_some()) {
                    s.resolve_doubts();
                }
                Ok(())
            }
            (PbMove::Drop { .. } | PbMove::Claim { .. }, Phase::Drop | Phase::Claim) => Err("Not your turn".into()),
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut PbState, ms: u64) {
        if s.phase != Phase::Result {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            let holder = s.holder;
            s.start_round(holder);
        }
    }

    fn view_for(s: &PbState, player: &str) -> PbView {
        let me = s.seat(player);
        let mine_of = |c: Colour| s.drops.iter().filter(|d| Some(d.seat) == me).map(|d| d.of(c)).sum::<u32>();
        let other = if s.colour == Colour::White { Colour::Black } else { Colour::White };
        let claimer = s.claimer();
        PbView {
            phase: s.phase,
            round: s.round,
            rounds: ROUNDS,
            colour: s.colour,
            my_turn: match s.phase {
                Phase::Drop | Phase::Claim => me == Some(s.holder),
                Phase::Doubt => me.is_some_and(|i| i != claimer && s.seats[i].choice.is_none()),
                _ => false,
            },
            coins: s.coins(),
            drops: s.drops.iter().map(|d| (s.seats[d.seat].id.clone(), d.count())).collect(),
            claims: s.claims.iter().map(|&(j, count, coins)| ClaimView { id: s.seats[j].id.clone(), count, coins }).collect(),
            accepted: s.accepted,
            mine: (mine_of(s.colour), mine_of(other)),
            seen: (s.phase == Phase::Claim && me == Some(s.holder)).then(|| s.drops.last().map_or((0, 0), |d| (d.white, d.black))),
            my_choice: me.and_then(|i| s.seats[i].choice),
            seats: s
                .seats
                .iter()
                .enumerate()
                .map(|(i, x)| SeatView {
                    id: x.id.clone(),
                    score: x.score,
                    holder: matches!(s.phase, Phase::Drop | Phase::Claim) && i == s.holder,
                    done: s.phase == Phase::Doubt && (i == claimer || x.choice.is_some()),
                })
                .collect(),
            log: s.log.clone(),
        }
    }

    fn is_over(s: &PbState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &PbState) -> Vec<PlayerResult> {
        points_by_rank(&s.seats.iter().map(|x| (x.id.clone(), x.score)).collect::<Vec<_>>())
    }

    fn to_move(s: &PbState) -> Vec<String> {
        match s.phase {
            Phase::Drop | Phase::Claim => vec![s.seats[s.holder].id.clone()],
            Phase::Doubt => {
                let claimer = s.claimer();
                (0..s.seats.len()).filter(|&j| j != claimer && s.seats[j].choice.is_none()).map(|j| s.seats[j].id.clone()).collect()
            }
            _ => Vec::new(),
        }
    }

    fn moments(s: &PbState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &PbState, player: &str, seed: u64) -> PbMove {
        let mut rng = Rng::new(seed);
        let i = s.seat(player).unwrap_or(0);
        match s.phase {
            Phase::Claim => {
                let coins = s.coins();
                let honest = (s.accepted + s.seen_colour()).min(coins);
                let count = if rng.next_f64() < 0.15 {
                    if honest == 0 || (honest < coins && rng.next_f64() < 0.5) { honest + 1 } else { honest - 1 }
                } else {
                    honest
                };
                PbMove::Claim { count }
            }
            Phase::Doubt => {
                if s.bot_doubts(i, &mut rng) {
                    PbMove::Doubt
                } else {
                    PbMove::Trust
                }
            }
            _ => {
                let n = rng.int(1, 2) as u32;
                let white = (0..n).filter(|_| rng.next_f64() < 0.5).count() as u32;
                PbMove::Drop { white, black: n - white }
            }
        }
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

    /// A 3-seat table where p1 starts and the colour is white.
    fn table() -> PbState {
        let mut s = Rules::setup(&ids(3), 1).unwrap();
        s.colour = Colour::White;
        s.holder = 0;
        s
    }

    #[test]
    fn honest_chain_and_a_wrong_doubt() {
        let mut s = table();
        Rules::apply_move(&mut s, "p1", PbMove::Drop { white: 2, black: 0 }).unwrap();
        assert_eq!(s.holder, 1);
        assert_eq!(Rules::view_for(&s, "p2").seen, Some((2, 0)));
        assert_eq!(Rules::view_for(&s, "p3").seen, None);
        Rules::apply_move(&mut s, "p2", PbMove::Claim { count: 2 }).unwrap();
        Rules::apply_move(&mut s, "p1", PbMove::Trust).unwrap();
        Rules::apply_move(&mut s, "p3", PbMove::Trust).unwrap();
        assert_eq!((s.phase, s.holder, s.accepted), (Phase::Drop, 1, 2));
        Rules::apply_move(&mut s, "p2", PbMove::Drop { white: 1, black: 1 }).unwrap();
        Rules::apply_move(&mut s, "p3", PbMove::Claim { count: 3 }).unwrap();
        assert!(Rules::apply_move(&mut s, "p3", PbMove::Doubt).is_err());
        Rules::apply_move(&mut s, "p1", PbMove::Doubt).unwrap();
        Rules::apply_move(&mut s, "p2", PbMove::Trust).unwrap();
        assert_eq!(s.phase, Phase::Result);
        assert_eq!(s.seats.iter().map(|x| x.score).collect::<Vec<_>>(), vec![-1, 0, 0]);
        assert_eq!(s.log[0].truth, 3);
        Rules::tick(&mut s, RESULT_MS as u64);
        assert_eq!((s.round, s.holder, s.colour), (2, 2, Colour::Black));
    }

    #[test]
    fn a_doubted_lie_costs_the_claimer_and_a_full_bank_opens() {
        let mut s = table();
        Rules::apply_move(&mut s, "p1", PbMove::Drop { white: 0, black: 2 }).unwrap();
        assert!(Rules::apply_move(&mut s, "p2", PbMove::Claim { count: 3 }).is_err());
        Rules::apply_move(&mut s, "p2", PbMove::Claim { count: 1 }).unwrap();
        // p1 dropped two black coins: one white is impossible, the bot doubts.
        let doubts = (0..20).filter(|&k| s.bot_doubts(0, &mut Rng::new(k))).count();
        assert!(doubts >= 12);
        Rules::apply_move(&mut s, "p1", PbMove::Doubt).unwrap();
        Rules::apply_move(&mut s, "p3", PbMove::Trust).unwrap();
        assert_eq!(s.seats[1].score, -1);

        let mut s = table();
        s.drops = vec![Drop { seat: 2, white: 5, black: 4 }];
        s.holder = 2;
        Rules::apply_move(&mut s, "p3", PbMove::Drop { white: 1, black: 0 }).unwrap();
        Rules::apply_move(&mut s, "p1", PbMove::Claim { count: 6 }).unwrap();
        Rules::apply_move(&mut s, "p2", PbMove::Trust).unwrap();
        Rules::apply_move(&mut s, "p3", PbMove::Trust).unwrap();
        assert!(s.log[0].full);
        assert!(s.log[0].losers.is_empty());
    }

    #[test]
    fn the_game_waits_for_every_move() {
        let mut s = table();
        Rules::tick(&mut s, 600_000);
        assert_eq!((s.phase, s.holder), (Phase::Drop, 0));
        Rules::apply_move(&mut s, "p1", PbMove::Drop { white: 1, black: 0 }).unwrap();
        Rules::tick(&mut s, 600_000);
        assert_eq!((s.phase, s.holder), (Phase::Claim, 1));
        Rules::apply_move(&mut s, "p2", PbMove::Claim { count: 1 }).unwrap();
        Rules::apply_move(&mut s, "p1", PbMove::Trust).unwrap();
        Rules::tick(&mut s, 600_000);
        assert_eq!(s.phase, Phase::Doubt);
        assert_eq!(Rules::to_move(&s), vec!["p3".to_string()]);
    }

    #[test]
    fn the_liar_doubts_the_poisoned_claim() {
        let mut s = table();
        Rules::apply_move(&mut s, "p1", PbMove::Drop { white: 1, black: 1 }).unwrap();
        // p2 saw one white and one black but claims 2 whites, and nobody doubts.
        Rules::apply_move(&mut s, "p2", PbMove::Claim { count: 2 }).unwrap();
        Rules::apply_move(&mut s, "p1", PbMove::Trust).unwrap();
        Rules::apply_move(&mut s, "p3", PbMove::Trust).unwrap();
        Rules::apply_move(&mut s, "p2", PbMove::Drop { white: 0, black: 1 }).unwrap();
        // p3 trusts the chain: 2 + 0 = 2. p2 believes 1.
        Rules::apply_move(&mut s, "p3", PbMove::Claim { count: 2 }).unwrap();
        let doubts = (0..20).filter(|&k| s.bot_doubts(1, &mut Rng::new(k))).count();
        assert!(doubts >= 12);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(400, 4, 5_000);
        bot_games_with::<Rules>(300, 3, 5_000);
        bot_games_with::<Rules>(300, 6, 5_000);
    }

    #[test]
    fn a_right_claim_two_doubters_pay_for_is_a_replay_moment() {
        let mut s = table();
        Rules::apply_move(&mut s, "p1", PbMove::Drop { white: 2, black: 0 }).unwrap();
        Rules::apply_move(&mut s, "p2", PbMove::Claim { count: 2 }).unwrap();
        Rules::apply_move(&mut s, "p1", PbMove::Doubt).unwrap();
        Rules::apply_move(&mut s, "p3", PbMove::Doubt).unwrap();
        let m = Rules::moments(&s, "p3");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].by, "p2");
        assert_eq!(m[0].text, "Claim of 2 white holds: 2 doubters pay");
        // One doubter paying is no moment.
        let mut s = table();
        Rules::apply_move(&mut s, "p1", PbMove::Drop { white: 2, black: 0 }).unwrap();
        Rules::apply_move(&mut s, "p2", PbMove::Claim { count: 2 }).unwrap();
        Rules::apply_move(&mut s, "p1", PbMove::Doubt).unwrap();
        Rules::apply_move(&mut s, "p3", PbMove::Trust).unwrap();
        assert!(Rules::moments(&s, "p1").is_empty());
    }

    #[test]
    fn a_lone_doubt_catching_a_lie_in_a_big_bank_is_a_replay_moment() {
        let mut s = table();
        s.drops = vec![Drop { seat: 2, white: 2, black: 3 }];
        s.accepted = 2;
        s.holder = 2;
        Rules::apply_move(&mut s, "p3", PbMove::Drop { white: 1, black: 0 }).unwrap();
        // Six coins, three white; p1 claims four.
        Rules::apply_move(&mut s, "p1", PbMove::Claim { count: 4 }).unwrap();
        Rules::apply_move(&mut s, "p2", PbMove::Doubt).unwrap();
        Rules::apply_move(&mut s, "p3", PbMove::Trust).unwrap();
        let m = Rules::moments(&s, "p1");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].by, "p2");
        assert_eq!(m[0].text, "Lone doubt: {p1} claimed 4 white, there were 3");
        // The same catch in a small bank is no moment.
        let mut s = table();
        Rules::apply_move(&mut s, "p1", PbMove::Drop { white: 1, black: 1 }).unwrap();
        Rules::apply_move(&mut s, "p2", PbMove::Claim { count: 2 }).unwrap();
        Rules::apply_move(&mut s, "p1", PbMove::Doubt).unwrap();
        Rules::apply_move(&mut s, "p3", PbMove::Trust).unwrap();
        assert_eq!(s.seats[1].score, -1);
        assert!(Rules::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

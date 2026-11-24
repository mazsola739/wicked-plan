//! Sniper Hold'em (prison game, 2–6 seats). Rules page: docs/games/sniper-holdem.md
//!
//! Two private cards and four shared ones from a 40-card deck (1–10 in four
//! colours), one betting round, then everyone may "snipe" a hand rank: every
//! hand of that rank is knocked out of the showdown.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const HANDS: usize = 6;
pub const START_CHIPS: i64 = 25;
pub const ANTE: i64 = 1;
pub const RAISE: i64 = 2;
pub const MAX_RAISES: u32 = 3;
pub const RESULT_MS: i64 = 6_000;

/// A card: colour * 10 + (value - 1).
pub type Card = u8;

pub fn value(c: Card) -> u8 {
    c % 10 + 1
}

pub fn colour(c: Card) -> u8 {
    c / 10
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Rank {
    HighCard,
    Pair,
    TwoPair,
    Trips,
    Straight,
    Flush,
    FullHouse,
    Quads,
    StraightFlush,
}

pub const RANKS: [Rank; 9] = [
    Rank::HighCard,
    Rank::Pair,
    Rank::TwoPair,
    Rank::Trips,
    Rank::Straight,
    Rank::Flush,
    Rank::FullHouse,
    Rank::Quads,
    Rank::StraightFlush,
];

/// Strength of five cards: (rank, deciding values packed so that bigger is better).
pub fn eval5(cards: &[Card]) -> (Rank, u32) {
    let mut counts = [0u8; 11];
    for &c in cards {
        counts[value(c) as usize] += 1;
    }
    // Values ordered by (count, value), highest first.
    let mut groups = [(0u8, 0u8); 5];
    let mut len = 0;
    for v in (1..=10u8).rev() {
        if counts[v as usize] > 0 {
            groups[len] = (counts[v as usize], v);
            len += 1;
        }
    }
    let groups = &mut groups[..len];
    groups.sort_by(|a, b| b.cmp(a));
    let flush = cards.iter().all(|&c| colour(c) == colour(cards[0]));
    let straight = len == 5 && groups[0].1 - groups[4].1 == 4;
    let mut shape = [0u8; 5];
    for (k, g) in groups.iter().enumerate() {
        shape[k] = g.0;
    }
    let rank = match (straight, flush, &shape[..len]) {
        (true, true, _) => Rank::StraightFlush,
        (_, _, [4, 1]) => Rank::Quads,
        (_, _, [3, 2]) => Rank::FullHouse,
        (_, true, _) => Rank::Flush,
        (true, _, _) => Rank::Straight,
        (_, _, [3, 1, 1]) => Rank::Trips,
        (_, _, [2, 2, 1]) => Rank::TwoPair,
        (_, _, [2, 1, 1, 1]) => Rank::Pair,
        _ => Rank::HighCard,
    };
    let mut tie = 0u32;
    for (count, v) in groups.iter() {
        for _ in 0..*count {
            tie = tie * 16 + u32::from(*v);
        }
    }
    (rank, tie)
}

/// The best five of five or six cards.
pub fn best(cards: &[Card]) -> (Rank, u32) {
    if cards.len() <= 5 {
        return eval5(cards);
    }
    (0..cards.len())
        .map(|skip| {
            let mut five = [0; 5];
            let mut k = 0;
            for (i, &c) in cards.iter().enumerate().take(6) {
                if i != skip && k < 5 {
                    five[k] = c;
                    k += 1;
                }
            }
            eval5(&five)
        })
        .max()
        .expect("cards")
}

/// A rank in words, for replay captions.
pub fn rank_name(r: Rank) -> &'static str {
    match r {
        Rank::HighCard => "high card",
        Rank::Pair => "pair",
        Rank::TwoPair => "two pair",
        Rank::Trips => "three of a kind",
        Rank::Straight => "straight",
        Rank::Flush => "flush",
        Rank::FullHouse => "full house",
        Rank::Quads => "four of a kind",
        Rank::StraightFlush => "straight flush",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Betting,
    Snipe,
    Result,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Ante,
    Check,
    Call,
    Raise,
    Fold,
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    pub chips: i64,
    pub cards: Vec<Card>,
    pub in_hand: bool,
    pub bet: i64,
    pub acted: bool,
    pub last: Option<Action>,
    /// Some(choice) once chosen in the snipe step.
    pub snipe: Option<Option<Rank>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShownHand {
    pub id: String,
    pub cards: Vec<Card>,
    pub rank: Rank,
    pub sniped: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HandLog {
    pub hand: usize,
    pub community: Vec<Card>,
    pub pot: i64,
    pub winners: Vec<String>,
    /// True when every hand was knocked out and the pot carried over.
    pub carried: bool,
    pub snipes: Vec<(String, Option<Rank>)>,
    pub shown: Vec<ShownHand>,
}

#[derive(Clone, Debug)]
pub struct ShState {
    pub seats: Vec<Seat>,
    pub rng: Rng,
    pub phase: Phase,
    pub hand: usize,
    pub dealer: usize,
    pub actor: usize,
    pub community: Vec<Card>,
    pub pot: i64,
    pub carry: i64,
    pub raises: u32,
    pub cap: i64,
    pub time_left_ms: i64,
    pub log: Vec<HandLog>,
    /// Replay moments: a winner who sniped the best hand at the table (pot of 10 or more).
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ShMove {
    Call,
    Raise,
    Fold,
    Snipe { rank: Option<Rank> },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub chips: i64,
    pub bet: i64,
    pub in_hand: bool,
    pub last: Option<Action>,
    pub dealer: bool,
    pub to_act: bool,
    /// Whether this seat has chosen its snipe (the choice itself shows at the showdown).
    pub sniped_yet: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShView {
    pub phase: Phase,
    pub hand: usize,
    pub hands: usize,
    pub cards: Vec<Card>,
    pub community: Vec<Card>,
    pub my_rank: Option<Rank>,
    pub pot: i64,
    pub carry: i64,
    pub to_call: i64,
    pub can_raise: bool,
    pub my_turn: bool,
    pub snipe_chosen: bool,
    pub my_snipe: Option<Rank>,
    pub seats: Vec<SeatView>,
    pub log: Vec<HandLog>,
}

impl ShState {
    fn seat(&self, id: &str) -> Option<usize> {
        self.seats.iter().position(|s| s.id == id)
    }

    fn current_bet(&self) -> i64 {
        self.seats.iter().filter(|s| s.in_hand).map(|s| s.bet).max().unwrap_or(0)
    }

    fn in_hand(&self) -> Vec<usize> {
        (0..self.seats.len()).filter(|&i| self.seats[i].in_hand).collect()
    }

    fn start_hand(&mut self) {
        let alive = self.seats.iter().filter(|s| s.chips > 0).count();
        if self.hand >= HANDS || alive <= 1 {
            self.phase = Phase::Over;
            self.time_left_ms = 0;
            return;
        }
        self.hand += 1;
        let n = self.seats.len();
        if self.hand > 1 {
            self.dealer = (1..=n).map(|k| (self.dealer + k) % n).find(|&i| self.seats[i].chips > 0).unwrap_or(0);
        } else {
            while self.seats[self.dealer].chips == 0 {
                self.dealer = (self.dealer + 1) % n;
            }
        }
        let mut deck: Vec<Card> = (0..40).collect();
        self.rng.shuffle(&mut deck);
        self.pot = self.carry;
        self.carry = 0;
        self.raises = 0;
        for s in &mut self.seats {
            s.in_hand = s.chips > 0;
            s.bet = 0;
            s.acted = false;
            s.snipe = None;
            s.cards.clear();
            s.last = None;
            if s.in_hand {
                s.chips -= ANTE;
                self.pot += ANTE;
                s.cards = vec![deck.pop().expect("deck"), deck.pop().expect("deck")];
                s.last = Some(Action::Ante);
            }
        }
        self.community = (0..4).map(|_| deck.pop().expect("deck")).collect();
        self.cap = self.seats.iter().filter(|s| s.in_hand).map(|s| s.chips).min().unwrap_or(0);
        self.phase = Phase::Betting;
        self.actor = self.next_in_hand(self.dealer);
    }

    fn next_in_hand(&self, from: usize) -> usize {
        let n = self.seats.len();
        (1..=n).map(|k| (from + k) % n).find(|&i| self.seats[i].in_hand).unwrap_or(from)
    }

    fn to_call(&self, i: usize) -> i64 {
        self.current_bet() - self.seats[i].bet
    }

    fn can_raise(&self) -> bool {
        self.raises < MAX_RAISES && self.current_bet() + RAISE <= self.cap
    }

    fn act(&mut self, i: usize, mv: &ShMove) -> Result<(), String> {
        let to_call = self.to_call(i);
        match mv {
            ShMove::Call => {
                let seat = &mut self.seats[i];
                seat.chips -= to_call;
                seat.bet += to_call;
                self.pot += to_call;
                seat.last = Some(if to_call == 0 { Action::Check } else { Action::Call });
            }
            ShMove::Raise => {
                if !self.can_raise() {
                    return Err("No more raises in this hand".into());
                }
                let add = to_call + RAISE;
                let seat = &mut self.seats[i];
                seat.chips -= add;
                seat.bet += add;
                self.pot += add;
                seat.last = Some(Action::Raise);
                self.raises += 1;
                for (j, s) in self.seats.iter_mut().enumerate() {
                    if j != i {
                        s.acted = false;
                    }
                }
            }
            ShMove::Fold => {
                self.seats[i].in_hand = false;
                self.seats[i].last = Some(Action::Fold);
            }
            ShMove::Snipe { .. } => return Err("Not now".into()),
        }
        self.seats[i].acted = true;
        let left = self.in_hand();
        if left.len() == 1 {
            self.finish(left, Vec::new(), Vec::new(), false);
            return Ok(());
        }
        let bet = self.current_bet();
        if left.iter().all(|&j| self.seats[j].acted && self.seats[j].bet == bet) {
            self.phase = Phase::Snipe;
        } else {
            self.actor = self.next_in_hand(i);
        }
        Ok(())
    }

    fn showdown(&mut self) {
        let left = self.in_hand();
        let snipes: Vec<(String, Option<Rank>)> = left.iter().map(|&i| (self.seats[i].id.clone(), self.seats[i].snipe.flatten())).collect();
        let named: Vec<Rank> = snipes.iter().filter_map(|s| s.1).collect();
        let mut shown = Vec::new();
        let mut alive: Vec<(usize, (Rank, u32))> = Vec::new();
        let mut hands: Vec<(usize, (Rank, u32))> = Vec::new();
        for &i in &left {
            let mut all = self.seats[i].cards.clone();
            all.extend(&self.community);
            let strength = best(&all);
            hands.push((i, strength));
            let sniped = named.contains(&strength.0);
            shown.push(ShownHand { id: self.seats[i].id.clone(), cards: self.seats[i].cards.clone(), rank: strength.0, sniped });
            if !sniped {
                alive.push((i, strength));
            }
        }
        let top = alive.iter().map(|a| a.1).max();
        let winners: Vec<usize> = alive.iter().filter(|a| Some(a.1) == top).map(|a| a.0).collect();
        let carried = winners.is_empty();
        // Replay moment (the showdown shows every hand still in, its rank and every snipe): a
        // winner who sniped the rank of the best hand at the table, for a pot of 10 or more.
        let favourite = hands.iter().max_by_key(|h| h.1).copied();
        if let Some((fav, (rank, _))) = favourite
            && self.pot >= 10
            && !winners.contains(&fav)
        {
            let n = winners.len() as i64;
            for (k, &w) in winners.iter().enumerate() {
                if self.seats[w].snipe.flatten() == Some(rank) {
                    let gain = self.pot / n + if k == 0 { self.pot % n } else { 0 };
                    let text = format!("{{{}}}'s {} sniped (+{gain})", self.seats[fav].id, rank_name(rank));
                    self.moments.push(&self.seats[w].id.clone(), text);
                }
            }
        }
        self.finish(winners, snipes, shown, carried);
    }

    fn finish(&mut self, winners: Vec<usize>, snipes: Vec<(String, Option<Rank>)>, shown: Vec<ShownHand>, carried: bool) {
        let pot = self.pot;
        if carried {
            self.carry = pot;
        } else {
            let share = pot / winners.len() as i64;
            for (k, &w) in winners.iter().enumerate() {
                self.seats[w].chips += share + if k == 0 { pot % winners.len() as i64 } else { 0 };
            }
        }
        self.pot = 0;
        self.log.push(HandLog {
            hand: self.hand,
            community: self.community.clone(),
            pot,
            winners: winners.iter().map(|&w| self.seats[w].id.clone()).collect(),
            carried,
            snipes,
            shown,
        });
        self.phase = Phase::Result;
        self.time_left_ms = RESULT_MS;
    }
}

pub struct Rules;

impl Game for Rules {
    type State = ShState;
    type Move = ShMove;
    type View = ShView;

    fn info() -> GameInfo {
        GameInfo {
            id: "sniper-holdem",
            kind: "prison",
            difficulty: "medium",
            tags: &["cards", "bluff"],
            min_players: 2,
            max_players: 6,
            name: "Sniper Hold'em",
            short: "Poker where you can \"snipe\" a hand rank to knock it out.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<ShState, String> {
        if !(2..=6).contains(&players.len()) {
            return Err("Sniper Hold'em needs 2–6 players".into());
        }
        let mut rng = Rng::new(seed);
        let dealer = rng.index(players.len());
        let seats = players
            .iter()
            .map(|id| Seat {
                id: id.clone(),
                chips: START_CHIPS,
                cards: Vec::new(),
                in_hand: false,
                bet: 0,
                acted: false,
                last: None,
                snipe: None,
            })
            .collect();
        let mut s = ShState {
            seats,
            rng,
            phase: Phase::Betting,
            hand: 0,
            dealer,
            actor: 0,
            community: Vec::new(),
            pot: 0,
            carry: 0,
            raises: 0,
            cap: 0,
            time_left_ms: 0,
            log: Vec::new(),
            moments: Moments::default(),
        };
        s.start_hand();
        Ok(s)
    }

    fn apply_move(s: &mut ShState, player: &str, mv: ShMove) -> Result<(), String> {
        let i = s.seat(player).ok_or("Not at this table")?;
        match s.phase {
            Phase::Betting => {
                if i != s.actor {
                    return Err("Not your turn".into());
                }
                s.act(i, &mv)
            }
            Phase::Snipe => {
                let ShMove::Snipe { rank } = mv else { return Err("Choose a snipe".into()) };
                if !s.seats[i].in_hand {
                    return Err("You are not in this hand".into());
                }
                if s.seats[i].snipe.is_some() {
                    return Err("You have already chosen".into());
                }
                s.seats[i].snipe = Some(rank);
                if s.in_hand().iter().all(|&j| s.seats[j].snipe.is_some()) {
                    s.showdown();
                }
                Ok(())
            }
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut ShState, ms: u64) {
        if s.phase != Phase::Result {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.start_hand();
        }
    }

    fn view_for(s: &ShState, player: &str) -> ShView {
        let me = s.seat(player);
        let mine = me.map(|i| &s.seats[i]);
        let cards = mine.map(|m| m.cards.clone()).unwrap_or_default();
        let my_rank = (!cards.is_empty() && s.phase != Phase::Over).then(|| {
            let mut all = cards.clone();
            all.extend(&s.community);
            best(&all).0
        });
        ShView {
            phase: s.phase,
            hand: s.hand,
            hands: HANDS,
            cards,
            community: s.community.clone(),
            my_rank,
            pot: s.pot,
            carry: s.carry,
            to_call: me.filter(|_| s.phase == Phase::Betting).map_or(0, |i| s.to_call(i)),
            can_raise: s.phase == Phase::Betting && s.can_raise(),
            my_turn: s.phase == Phase::Betting && me == Some(s.actor),
            snipe_chosen: mine.is_some_and(|m| m.snipe.is_some()),
            my_snipe: mine.and_then(|m| m.snipe.flatten()),
            seats: s
                .seats
                .iter()
                .enumerate()
                .map(|(i, x)| SeatView {
                    id: x.id.clone(),
                    chips: x.chips,
                    bet: x.bet,
                    in_hand: x.in_hand,
                    last: x.last,
                    dealer: i == s.dealer,
                    to_act: s.phase == Phase::Betting && i == s.actor,
                    sniped_yet: x.snipe.is_some(),
                })
                .collect(),
            log: s.log.clone(),
        }
    }

    fn is_over(s: &ShState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &ShState) -> Vec<PlayerResult> {
        points_by_rank(&s.seats.iter().map(|x| (x.id.clone(), x.chips)).collect::<Vec<_>>())
    }

    fn to_move(s: &ShState) -> Vec<String> {
        match s.phase {
            Phase::Betting => vec![s.seats[s.actor].id.clone()],
            Phase::Snipe => s.seats.iter().filter(|x| x.in_hand && x.snipe.is_none()).map(|x| x.id.clone()).collect(),
            _ => Vec::new(),
        }
    }

    fn moments(s: &ShState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &ShState, player: &str, seed: u64) -> ShMove {
        let mut rng = Rng::new(seed);
        let i = s.seat(player).unwrap_or(0);
        let samples = sample_opponents(s, i, &mut rng, 30);
        if s.phase == Phase::Snipe {
            return ShMove::Snipe { rank: pick_snipe(&samples) };
        }
        let others = s.in_hand().len().saturating_sub(1).max(1) as i32;
        let win_one = samples.iter().filter(|o| o.mine >= o.theirs).count() as f64 / samples.len().max(1) as f64;
        let win = win_one.powi(others);
        let to_call = s.to_call(i);
        let bluff = rng.next_f64() < 0.1;
        if (win > 0.55 || bluff) && s.can_raise() {
            ShMove::Raise
        } else if to_call == 0 || win > 0.25 || (win > 0.12 && to_call <= RAISE) {
            ShMove::Call
        } else {
            ShMove::Fold
        }
    }
}

struct Sample {
    mine: (Rank, u32),
    theirs: (Rank, u32),
}

/// Deals the cards this seat cannot see to imaginary opponents, `n` times.
fn sample_opponents(s: &ShState, me: usize, rng: &mut Rng, n: usize) -> Vec<Sample> {
    let seen: Vec<Card> = s.seats[me].cards.iter().chain(&s.community).copied().collect();
    if s.seats[me].cards.is_empty() {
        return Vec::new();
    }
    let mut mine_all = seen.clone();
    mine_all.truncate(6);
    let mine = best(&mine_all);
    let mut unseen: Vec<Card> = (0..40).filter(|c| !seen.contains(c)).collect();
    (0..n)
        .map(|_| {
            let a = rng.index(unseen.len());
            unseen.swap(a, 0);
            let b = 1 + rng.index(unseen.len() - 1);
            let mut theirs = [unseen[0], unseen[b], 0, 0, 0, 0];
            for (k, &c) in s.community.iter().enumerate().take(4) {
                theirs[2 + k] = c;
            }
            Sample { mine, theirs: best(&theirs) }
        })
        .collect()
}

/// The rank that most often beats this hand in the samples, if it is common enough.
fn pick_snipe(samples: &[Sample]) -> Option<Rank> {
    let mine = samples.first()?.mine.0;
    let count = |r: Rank| samples.iter().filter(|o| o.theirs > o.mine && o.theirs.0 == r).count();
    let target = RANKS.into_iter().filter(|&r| r != mine).max_by_key(|&r| count(r))?;
    (count(target) * 6 >= samples.len()).then_some(target)
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

    /// Card from colour (0–3) and value (1–10).
    fn c(colour: u8, v: u8) -> Card {
        colour * 10 + v - 1
    }

    #[test]
    fn ranks_hands() {
        assert_eq!(eval5(&[c(0, 1), c(1, 2), c(2, 3), c(3, 4), c(0, 5)]).0, Rank::Straight);
        assert_eq!(eval5(&[c(0, 1), c(0, 2), c(0, 3), c(0, 4), c(0, 5)]).0, Rank::StraightFlush);
        assert_eq!(eval5(&[c(0, 1), c(0, 2), c(0, 3), c(0, 4), c(0, 9)]).0, Rank::Flush);
        assert_eq!(eval5(&[c(0, 7), c(1, 7), c(2, 7), c(3, 2), c(0, 2)]).0, Rank::FullHouse);
        assert_eq!(eval5(&[c(0, 7), c(1, 7), c(2, 3), c(3, 3), c(0, 2)]).0, Rank::TwoPair);
        assert_eq!(eval5(&[c(0, 7), c(1, 7), c(2, 7), c(3, 7), c(0, 2)]).0, Rank::Quads);
        // A pair of 9s beats a pair of 8s; kickers decide equal pairs.
        assert!(eval5(&[c(0, 9), c(1, 9), c(2, 1), c(3, 2), c(0, 3)]) > eval5(&[c(0, 8), c(1, 8), c(2, 10), c(3, 6), c(0, 5)]));
        assert!(eval5(&[c(0, 9), c(1, 9), c(2, 1), c(3, 2), c(0, 4)]) > eval5(&[c(2, 9), c(3, 9), c(2, 1), c(3, 2), c(0, 3)]));
        // Best five of six.
        assert_eq!(best(&[c(0, 1), c(1, 2), c(2, 3), c(3, 4), c(0, 5), c(1, 5)]).0, Rank::Straight);
    }

    fn rig(s: &mut ShState, hands: &[[Card; 2]], community: [Card; 4]) {
        for (seat, h) in s.seats.iter_mut().zip(hands) {
            seat.cards = h.to_vec();
        }
        s.community = community.to_vec();
    }

    #[test]
    fn sniping_knocks_out_the_best_hand() {
        let mut s = Rules::setup(&ids(3), 1).unwrap();
        // Shared: 2 3 4 9 (mixed colours). p1 makes a straight, p2 a pair of 9s, p3 high card.
        rig(&mut s, &[[c(0, 5), c(1, 6)], [c(2, 9), c(0, 1)], [c(3, 7), c(3, 8)]], [c(1, 2), c(2, 3), c(3, 4), c(0, 9)]);
        let pot = s.pot;
        for _ in 0..3 {
            let actor = s.seats[s.actor].id.clone();
            Rules::apply_move(&mut s, &actor, ShMove::Call).unwrap();
        }
        assert_eq!(s.phase, Phase::Snipe);
        assert!(Rules::apply_move(&mut s, "p1", ShMove::Call).is_err());
        Rules::apply_move(&mut s, "p1", ShMove::Snipe { rank: None }).unwrap();
        Rules::apply_move(&mut s, "p2", ShMove::Snipe { rank: Some(Rank::Straight) }).unwrap();
        Rules::apply_move(&mut s, "p3", ShMove::Snipe { rank: None }).unwrap();
        assert_eq!(s.phase, Phase::Result);
        let log = s.log.last().unwrap();
        assert_eq!(log.winners, vec!["p2".to_string()]);
        assert!(log.shown[0].sniped);
        assert_eq!(s.seats[1].chips, START_CHIPS - ANTE + pot);
    }

    #[test]
    fn all_knocked_out_carries_the_pot() {
        let mut s = Rules::setup(&ids(2), 2).unwrap();
        rig(&mut s, &[[c(0, 1), c(1, 1)], [c(2, 5), c(3, 5)]], [c(0, 7), c(1, 8), c(2, 2), c(3, 10)]);
        for _ in 0..2 {
            let actor = s.seats[s.actor].id.clone();
            Rules::apply_move(&mut s, &actor, ShMove::Call).unwrap();
        }
        Rules::apply_move(&mut s, "p1", ShMove::Snipe { rank: Some(Rank::Pair) }).unwrap();
        Rules::apply_move(&mut s, "p2", ShMove::Snipe { rank: None }).unwrap();
        assert!(s.log[0].carried);
        assert_eq!(s.carry, 2 * ANTE);
        Rules::tick(&mut s, RESULT_MS as u64);
        assert_eq!(s.pot, 4 * ANTE);
    }

    #[test]
    fn raises_respect_the_table_limit_and_folds_end_the_hand() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        s.cap = 3;
        let actor = s.seats[s.actor].id.clone();
        Rules::apply_move(&mut s, &actor, ShMove::Raise).unwrap();
        let next = s.seats[s.actor].id.clone();
        assert!(Rules::apply_move(&mut s, &next, ShMove::Raise).is_err());
        assert_eq!(Rules::view_for(&s, &next).to_call, RAISE);
        Rules::apply_move(&mut s, &next, ShMove::Fold).unwrap();
        let third = s.seats[s.actor].id.clone();
        Rules::apply_move(&mut s, &third, ShMove::Fold).unwrap();
        assert_eq!(s.phase, Phase::Result);
        assert_eq!(s.log[0].winners, vec![actor]);
        // Hidden cards never leak into another seat's view.
        let v = serde_json::to_string(&Rules::view_for(&s, "p1")).unwrap();
        assert!(!v.contains("\"snipe\":"));
    }

    #[test]
    fn betting_and_snipes_wait_for_the_players() {
        let mut s = Rules::setup(&ids(2), 4).unwrap();
        let actor = s.actor;
        Rules::tick(&mut s, 600_000);
        assert_eq!((s.phase, s.actor), (Phase::Betting, actor));
        assert!(s.seats.iter().all(|x| x.last == Some(Action::Ante)));
        for _ in 0..2 {
            let actor = s.seats[s.actor].id.clone();
            Rules::apply_move(&mut s, &actor, ShMove::Call).unwrap();
        }
        Rules::apply_move(&mut s, "p1", ShMove::Snipe { rank: None }).unwrap();
        Rules::tick(&mut s, 600_000);
        assert_eq!(s.phase, Phase::Snipe);
        assert_eq!(Rules::to_move(&s), vec!["p2".to_string()]);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(400, 4, 5_000);
        bot_games_with::<Rules>(300, 2, 5_000);
        bot_games_with::<Rules>(300, 6, 5_000);
    }

    #[test]
    fn sniping_the_best_hand_for_a_big_pot_is_a_replay_moment() {
        let showdown = |snipes: [Option<Rank>; 3]| {
            let mut s = Rules::setup(&ids(3), 1).unwrap();
            // p1 makes a straight, p2 a pair of 9s, p3 high card.
            rig(&mut s, &[[c(0, 5), c(1, 6)], [c(2, 9), c(0, 1)], [c(3, 7), c(3, 8)]], [c(1, 2), c(2, 3), c(3, 4), c(0, 9)]);
            s.pot += 9;
            for _ in 0..3 {
                let actor = s.seats[s.actor].id.clone();
                Rules::apply_move(&mut s, &actor, ShMove::Call).unwrap();
            }
            for (p, rank) in ["p1", "p2", "p3"].into_iter().zip(snipes) {
                Rules::apply_move(&mut s, p, ShMove::Snipe { rank }).unwrap();
            }
            s
        };
        let s = showdown([None, Some(Rank::Straight), None]);
        let m = Rules::moments(&s, "p3");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p2", "{p1}'s straight sniped (+12)"));
        // p3's snipe knocks the straight out, but p2 wins: no moment.
        let s = showdown([None, None, Some(Rank::Straight)]);
        assert_eq!(s.log[0].winners, vec!["p2".to_string()]);
        assert!(Rules::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

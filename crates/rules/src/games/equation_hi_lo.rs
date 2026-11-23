//! Equation Hi-Lo (main game, 3–6 seats). Rules page: docs/games/equation-hi-lo.md
//!
//! Four number cards each (two face up), the symbols + − × ÷: build an equation
//! with three of them and declare Low (near 1) or High (near 20). Fixed-limit
//! betting before and after the build; the pot is split between the sides.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const HANDS: usize = 3;
pub const CHIPS: i64 = 20;
pub const ANTE: i64 = 1;
pub const RAISE: i64 = 2;
pub const MAX_RAISES: usize = 2;
pub const BET_MS: i64 = 20_000;
pub const BUILD_MS: i64 = 60_000;
pub const LOW: f64 = 1.0;
pub const HIGH: f64 = 20.0;
pub const SYMBOLS: [char; 4] = ['+', '-', '*', '/'];
/// Replay moments: an exact 1 or 20 alone on its side winning this many chips or more,
/// and a pot of this size or more taken without a showdown.
const EXACT_MOMENT_CHIPS: i64 = 18;
const FOLD_MOMENT_POT: i64 = 10;

/// Value of a op b op c op d with × ÷ before + −; None when dividing by 0.
pub fn evaluate(nums: &[i64; 4], ops: &[char; 3]) -> Option<f64> {
    // A running sum of finished terms, and the term being multiplied out.
    let mut sum = 0.0;
    let mut term = nums[0] as f64;
    for (i, &op) in ops.iter().enumerate() {
        let n = nums[i + 1] as f64;
        match op {
            '+' => {
                sum += term;
                term = n;
            }
            '-' => {
                sum += term;
                term = -n;
            }
            '*' => term *= n,
            '/' if n != 0.0 => term /= n,
            _ => return None,
        }
    }
    Some(sum + term)
}

/// Every (order, ops) of these cards: 24 orders × 24 symbol sequences.
fn all_builds() -> Vec<([usize; 4], [char; 3])> {
    let mut orders = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                if a != b && b != c && a != c {
                    orders.push([a, b, c, 6 - a - b - c]);
                }
            }
        }
    }
    let mut ops = Vec::new();
    for x in SYMBOLS {
        for y in SYMBOLS {
            for z in SYMBOLS {
                if x != y && y != z && x != z {
                    ops.push([x, y, z]);
                }
            }
        }
    }
    orders.iter().flat_map(|&o| ops.iter().map(move |&p| (o, p))).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Low,
    High,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Build {
    pub order: [usize; 4],
    pub ops: [char; 3],
    pub side: Side,
    pub value: Option<f64>,
    pub text: String,
}

fn make_build(cards: &[i64; 4], order: [usize; 4], ops: [char; 3], side: Side) -> Build {
    let nums = order.map(|i| cards[i]);
    let value = evaluate(&nums, &ops);
    let sym = |c: char| match c {
        '*' => '×',
        '/' => '÷',
        '-' => '−',
        c => c,
    };
    let text = format!("{} {} {} {} {} {} {}", nums[0], sym(ops[0]), nums[1], sym(ops[1]), nums[2], sym(ops[2]), nums[3]);
    Build { order, ops, side, value, text }
}

/// The best Low and High build of these cards with their distances.
pub fn best_builds(cards: &[i64; 4]) -> (Build, f64, Build, f64) {
    let mut low = (f64::INFINITY, ([0, 1, 2, 3], ['+', '-', '*']));
    let mut high = low;
    for (order, ops) in all_builds() {
        let Some(v) = evaluate(&order.map(|i| cards[i]), &ops) else { continue };
        if (v - LOW).abs() < low.0 {
            low = ((v - LOW).abs(), (order, ops));
        }
        if (v - HIGH).abs() < high.0 {
            high = ((v - HIGH).abs(), (order, ops));
        }
    }
    (make_build(cards, low.1.0, low.1.1, Side::Low), low.0, make_build(cards, high.1.0, high.1.1, Side::High), high.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Bet1,
    Build,
    Bet2,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Showdown {
    pub hand: usize,
    pub cards: Vec<Option<[i64; 4]>>,
    pub builds: Vec<Option<Build>>,
    pub gains: Vec<i64>,
    pub pot: i64,
}

#[derive(Clone, Debug)]
pub struct HiLoState {
    pub players: Vec<String>,
    pub rng: Rng,
    pub chips: Vec<i64>,
    pub hand: usize,
    pub dealer: usize,
    pub phase: Phase,
    pub cards: Vec<[i64; 4]>,
    /// In this hand (has paid the ante) and not folded.
    pub live: Vec<bool>,
    pub in_hand: Vec<bool>,
    pub put: Vec<i64>,
    pub put_round: Vec<i64>,
    pub level: i64,
    pub raises: usize,
    pub cap: i64,
    pub acted: Vec<bool>,
    pub turn: usize,
    pub builds: Vec<Option<Build>>,
    /// Best Low and High equation per seat for this hand (for bots and timeouts).
    pub best: Vec<(Build, f64, Build, f64)>,
    pub pot: i64,
    pub time_left_ms: i64,
    pub log: Vec<String>,
    pub last: Option<Showdown>,
    pub over: bool,
    /// Replay moments: an exact 1 or 20 that wins a big share alone, a big pot won without a showdown.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HiLoMove {
    Call,
    Raise,
    Fold,
    Build { order: [usize; 4], ops: [char; 3], side: Side },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub chips: i64,
    pub put: i64,
    pub in_hand: bool,
    pub folded: bool,
    /// Face-up cards (both visible to all).
    pub up: Option<[i64; 2]>,
    pub built: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HiLoView {
    pub me: usize,
    pub hand: usize,
    pub hands: usize,
    /// "bet1" | "build" | "bet2" | "over"
    pub phase: &'static str,
    pub seats: Vec<SeatView>,
    pub cards: Option<[i64; 4]>,
    pub turn: Option<usize>,
    pub to_call: i64,
    pub can_raise: bool,
    pub pot: i64,
    pub build: Option<Build>,
    pub time_left_ms: i64,
    pub phase_ms: i64,
    pub log: Vec<String>,
    pub last: Option<Showdown>,
    pub over: bool,
}

impl HiLoState {
    fn n(&self) -> usize {
        self.players.len()
    }

    fn start_hand(&mut self) {
        let n = self.n();
        let seats: Vec<usize> = (0..n).filter(|&s| self.chips[s] >= ANTE).collect();
        if seats.len() < 2 || self.hand >= HANDS {
            self.over = true;
            self.time_left_ms = 0;
            self.log.push("Game over.".into());
            return;
        }
        let mut deck: Vec<i64> = (0..4).flat_map(|_| 0..=10).collect();
        self.rng.shuffle(&mut deck);
        self.cap = seats.iter().map(|&s| self.chips[s]).min().unwrap_or(0);
        self.in_hand = (0..n).map(|s| seats.contains(&s)).collect();
        self.live = self.in_hand.clone();
        self.put = vec![0; n];
        self.pot = 0;
        for (i, &s) in seats.iter().enumerate() {
            self.cards[s] = [deck[4 * i], deck[4 * i + 1], deck[4 * i + 2], deck[4 * i + 3]];
            self.chips[s] -= ANTE;
            self.put[s] = ANTE;
            self.pot += ANTE;
        }
        self.builds = vec![None; n];
        self.best = self.cards.iter().map(best_builds).collect();
        self.log.push(format!("Hand {}: ante {ANTE} each.", self.hand + 1));
        self.start_betting(Phase::Bet1);
    }

    fn start_betting(&mut self, phase: Phase) {
        let n = self.n();
        self.phase = phase;
        self.put_round = vec![0; n];
        self.level = 0;
        self.raises = 0;
        self.acted = vec![false; n];
        self.turn = self.dealer;
        self.time_left_ms = BET_MS;
        self.advance_turn();
    }

    /// Moves to the next live seat that still has to act, or ends the betting.
    fn advance_turn(&mut self) {
        let n = self.n();
        if self.live.iter().filter(|&&l| l).count() <= 1 {
            self.showdown();
            return;
        }
        for i in 1..=n {
            let s = (self.turn + i) % n;
            if self.live[s] && (!self.acted[s] || self.put_round[s] < self.level) {
                self.turn = s;
                self.time_left_ms = BET_MS;
                return;
            }
        }
        if self.phase == Phase::Bet1 {
            self.phase = Phase::Build;
            self.time_left_ms = BUILD_MS;
        } else {
            self.showdown();
        }
    }

    fn can_raise(&self, seat: usize) -> bool {
        self.raises < MAX_RAISES && self.put[seat] + (self.level - self.put_round[seat]) + RAISE <= self.cap
    }

    fn pay(&mut self, seat: usize, amount: i64) {
        self.chips[seat] -= amount;
        self.put[seat] += amount;
        self.put_round[seat] += amount;
        self.pot += amount;
    }

    fn bet_move(&mut self, seat: usize, mv: HiLoMove) -> Result<(), String> {
        let to_call = self.level - self.put_round[seat];
        let who = self.players[seat].clone();
        match mv {
            HiLoMove::Call => {
                self.pay(seat, to_call);
                self.log.push(if to_call == 0 { format!("{who} checks.") } else { format!("{who} calls {to_call}.") });
            }
            HiLoMove::Raise => {
                if !self.can_raise(seat) {
                    return Err("No more raises now".into());
                }
                self.level += RAISE;
                self.raises += 1;
                self.pay(seat, to_call + RAISE);
                self.log.push(format!("{who} raises to {}.", self.level));
            }
            HiLoMove::Fold => {
                self.live[seat] = false;
                self.log.push(format!("{who} folds."));
            }
            HiLoMove::Build { .. } => return Err("Not the time to build".into()),
        }
        self.acted[seat] = true;
        self.advance_turn();
        Ok(())
    }

    fn auto_build(&self, seat: usize) -> Build {
        let (low, dl, high, dh) = &self.best[seat];
        if *dl <= dh / 4.0 { low.clone() } else { high.clone() }
    }

    fn showdown(&mut self) {
        let n = self.n();
        let mut gains = vec![0; n];
        let live: Vec<usize> = (0..n).filter(|&s| self.live[s]).collect();
        let reveal = live.len() > 1;
        if live.len() == 1 {
            gains[live[0]] = self.pot;
            self.log.push(format!("{} takes the pot of {}.", self.players[live[0]], self.pot));
            // A replay moment: everyone else folded on a big pot (no card is revealed).
            if self.pot >= FOLD_MOMENT_POT {
                self.moments.push(&self.players[live[0]], format!("Pot of {} taken without a showdown", self.pot));
            }
        } else {
            for &s in &live {
                if self.builds[s].is_none() {
                    self.builds[s] = Some(self.auto_build(s));
                }
            }
            let side_best = |side: Side, target: f64| -> Vec<usize> {
                let dist = |s: usize| {
                    let b = self.builds[s].as_ref().expect("built");
                    if b.side == side { b.value.map(|v| (v - target).abs()) } else { None }
                };
                let best = live.iter().filter_map(|&s| dist(s)).fold(f64::INFINITY, f64::min);
                live.iter().copied().filter(|&s| dist(s).is_some_and(|d| (d - best).abs() < 1e-9)).collect()
            };
            let lows = side_best(Side::Low, LOW);
            let highs = side_best(Side::High, HIGH);
            let (low_share, high_share) = match (lows.is_empty(), highs.is_empty()) {
                (false, false) => (self.pot / 2, self.pot - self.pot / 2),
                (true, false) => (0, self.pot),
                (false, true) => (self.pot, 0),
                // Every equation was invalid: the pot is shared out.
                (true, true) => (0, 0),
            };
            let mut share = |winners: &[usize], amount: i64| {
                if winners.is_empty() {
                    return;
                }
                let each = amount / winners.len() as i64;
                for &w in winners {
                    gains[w] += each;
                }
                gains[winners[0]] += amount - each * winners.len() as i64;
            };
            share(&lows, low_share);
            share(&highs, high_share);
            if lows.is_empty() && highs.is_empty() {
                share(&live, self.pot);
            }
            // A replay moment: an exact 1 or 20, alone on its side, winning a big share (every equation is shown now).
            for (winners, target) in [(&lows, LOW), (&highs, HIGH)] {
                if let [w] = winners[..] {
                    let b = self.builds[w].as_ref().expect("built");
                    if b.value.is_some_and(|v| (v - target).abs() < 1e-9) && gains[w] >= EXACT_MOMENT_CHIPS {
                        let won = if gains[w] == self.pot {
                            format!("sweeps the pot of {}", self.pot)
                        } else {
                            format!("takes {} chips", gains[w])
                        };
                        self.moments.push(&self.players[w], format!("Exactly {target} {won}: {}", b.text));
                    }
                }
            }
            self.log.push(format!("Showdown: pot {} shared.", self.pot));
        }
        for (s, g) in gains.iter().enumerate() {
            self.chips[s] += g;
        }
        self.last = Some(Showdown {
            hand: self.hand + 1,
            cards: (0..n).map(|s| (reveal && self.live[s]).then_some(self.cards[s])).collect(),
            builds: (0..n).map(|s| if reveal && self.live[s] { self.builds[s].clone() } else { None }).collect(),
            gains: (0..n).map(|s| gains[s] - self.put[s]).collect(),
            pot: self.pot,
        });
        self.hand += 1;
        self.dealer = (self.dealer + 1) % n;
        self.start_hand();
    }

    fn after_build(&mut self) {
        if (0..self.n()).all(|s| !self.live[s] || self.builds[s].is_some()) {
            self.start_betting(Phase::Bet2);
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
    type State = HiLoState;
    type Move = HiLoMove;
    type View = HiLoView;

    fn info() -> GameInfo {
        GameInfo {
            id: "equation-hi-lo",
            kind: "main",
            difficulty: "medium",
            tags: &["numbers", "cards", "bluff"],
            min_players: 3,
            max_players: 6,
            name: "Equation Hi-Lo",
            short: "Build an equation from four number cards and three symbols, then bet on being closest to 1 or to 20.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<HiLoState, String> {
        if !(3..=6).contains(&players.len()) {
            return Err("Equation Hi-Lo needs 3–6 players".into());
        }
        let n = players.len();
        let mut s = HiLoState {
            players: players.to_vec(),
            rng: Rng::new(seed),
            chips: vec![CHIPS; n],
            hand: 0,
            dealer: 0,
            phase: Phase::Bet1,
            cards: vec![[0; 4]; n],
            live: vec![false; n],
            in_hand: vec![false; n],
            put: vec![0; n],
            put_round: vec![0; n],
            level: 0,
            raises: 0,
            cap: 0,
            acted: vec![false; n],
            turn: 0,
            builds: vec![None; n],
            best: Vec::new(),
            pot: 0,
            time_left_ms: BET_MS,
            log: Vec::new(),
            last: None,
            over: false,
            moments: Moments::default(),
        };
        s.start_hand();
        Ok(s)
    }

    fn apply_move(s: &mut HiLoState, player: &str, mv: HiLoMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.players.iter().position(|p| p == player).ok_or("Not in this game")?;
        if !s.live[seat] {
            return Err("You are out of this hand".into());
        }
        if s.phase == Phase::Build {
            let HiLoMove::Build { order, ops, side } = mv else { return Err("Build your equation first".into()) };
            if s.builds[seat].is_some() {
                return Err("You already built".into());
            }
            let mut o = order;
            o.sort();
            let distinct = ops[0] != ops[1] && ops[1] != ops[2] && ops[0] != ops[2];
            if o != [0, 1, 2, 3] || !distinct || ops.iter().any(|c| !SYMBOLS.contains(c)) {
                return Err("Use all four cards and three different symbols".into());
            }
            s.builds[seat] = Some(make_build(&s.cards[seat], order, ops, side));
            s.after_build();
            return Ok(());
        }
        if seat != s.turn {
            return Err("Not your turn".into());
        }
        s.bet_move(seat, mv)
    }

    fn tick(s: &mut HiLoState, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms > 0 {
            return;
        }
        if s.phase == Phase::Build {
            for seat in 0..s.n() {
                if s.live[seat] && s.builds[seat].is_none() {
                    s.builds[seat] = Some(s.auto_build(seat));
                }
            }
            s.after_build();
        } else {
            let seat = s.turn;
            let mv = if s.level > s.put_round[seat] { HiLoMove::Fold } else { HiLoMove::Call };
            let _ = s.bet_move(seat, mv);
        }
    }

    fn view_for(s: &HiLoState, player: &str) -> HiLoView {
        let me = s.players.iter().position(|p| p == player).unwrap_or(0);
        let betting = !s.over && s.phase != Phase::Build;
        HiLoView {
            me,
            hand: (s.hand + 1).min(HANDS),
            hands: HANDS,
            phase: match s.phase {
                _ if s.over => "over",
                Phase::Bet1 => "bet1",
                Phase::Build => "build",
                Phase::Bet2 => "bet2",
            },
            seats: (0..s.n())
                .map(|o| SeatView {
                    id: s.players[o].clone(),
                    chips: s.chips[o],
                    put: s.put[o],
                    in_hand: s.in_hand[o] && !s.over,
                    folded: s.in_hand[o] && !s.live[o],
                    up: (s.in_hand[o] && !s.over).then(|| [s.cards[o][0], s.cards[o][1]]),
                    built: s.builds[o].is_some(),
                })
                .collect(),
            cards: (s.in_hand[me] && !s.over).then_some(s.cards[me]),
            turn: betting.then_some(s.turn),
            to_call: if betting { s.level - s.put_round[me] } else { 0 },
            can_raise: betting && s.turn == me && s.can_raise(me),
            pot: s.pot,
            build: s.builds[me].clone(),
            time_left_ms: s.time_left_ms,
            phase_ms: if s.phase == Phase::Build { BUILD_MS } else { BET_MS },
            log: s.log[s.log.len().saturating_sub(10)..].to_vec(),
            last: s.last.clone(),
            over: s.over,
        }
    }

    fn is_over(s: &HiLoState) -> bool {
        s.over
    }

    fn results(s: &HiLoState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.players.iter().cloned().zip(s.chips.iter().copied()).collect();
        rank_points(&scores)
    }

    fn to_move(s: &HiLoState) -> Vec<String> {
        if s.over {
            return Vec::new();
        }
        if s.phase == Phase::Build {
            return (0..s.n()).filter(|&o| s.live[o] && s.builds[o].is_none()).map(|o| s.players[o].clone()).collect();
        }
        vec![s.players[s.turn].clone()]
    }

    fn bot_move(s: &HiLoState, player: &str, seed: u64) -> HiLoMove {
        let mut rng = Rng::new(seed);
        let seat = s.players.iter().position(|p| p == player).unwrap_or(0);
        let (low, dl, high, dh) = &s.best[seat];
        let (dl, dh) = (*dl, *dh);
        if s.phase == Phase::Build {
            let b = if dl <= dh / 4.0 { low } else { high };
            return HiLoMove::Build { order: b.order, ops: b.ops, side: b.side };
        }
        // Closeness on a common scale: Low distances count four times.
        let strength = (dl * 4.0).min(dh);
        let to_call = s.level - s.put_round[seat];
        let r = rng.next_f64();
        if strength < 0.5 && s.can_raise(seat) && r < 0.8 {
            HiLoMove::Raise
        } else if strength < 2.0 || to_call == 0 || (strength < 5.0 && to_call <= RAISE) || r < 0.15 {
            HiLoMove::Call
        } else {
            HiLoMove::Fold
        }
    }

    fn moments(s: &HiLoState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
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
    fn evaluates_with_precedence() {
        assert_eq!(evaluate(&[2, 3, 4, 5], &['+', '*', '-']), Some(9.0));
        assert_eq!(evaluate(&[8, 4, 2, 1], &['/', '+', '-']), Some(3.0));
        assert_eq!(evaluate(&[1, 2, 3, 4], &['/', '*', '+']), Some(5.5));
        assert_eq!(evaluate(&[5, 0, 1, 1], &['/', '+', '-']), None);
        assert_eq!(all_builds().len(), 24 * 24);
    }

    #[test]
    fn finds_exact_targets() {
        let (_, dl, high, dh) = best_builds(&[10, 10, 0, 3]);
        assert_eq!(dh, 0.0);
        assert_eq!(high.value, Some(20.0));
        assert!(dl < 0.5);
    }

    #[test]
    fn ante_and_betting_round() {
        let mut s = Rules::setup(&ids(3), 1).unwrap();
        assert_eq!(s.pot, 3);
        assert_eq!(s.chips, vec![19, 19, 19]);
        assert_eq!(s.turn, 1, "the seat after the dealer starts");
        Rules::apply_move(&mut s, "p2", HiLoMove::Raise).unwrap();
        Rules::apply_move(&mut s, "p3", HiLoMove::Call).unwrap();
        Rules::apply_move(&mut s, "p1", HiLoMove::Fold).unwrap();
        assert_eq!(s.phase, Phase::Build);
        assert_eq!(s.pot, 7);
        assert!(Rules::apply_move(&mut s, "p1", HiLoMove::Build { order: [0, 1, 2, 3], ops: ['+', '-', '*'], side: Side::Low }).is_err());
        assert!(Rules::apply_move(&mut s, "p2", HiLoMove::Build { order: [0, 1, 1, 3], ops: ['+', '-', '*'], side: Side::Low }).is_err());
        assert!(Rules::apply_move(&mut s, "p2", HiLoMove::Build { order: [0, 1, 2, 3], ops: ['+', '+', '*'], side: Side::Low }).is_err());
        Rules::apply_move(&mut s, "p2", HiLoMove::Build { order: [3, 2, 1, 0], ops: ['+', '-', '/'], side: Side::High }).unwrap();
        Rules::apply_move(&mut s, "p3", HiLoMove::Build { order: [0, 1, 2, 3], ops: ['*', '-', '+'], side: Side::Low }).unwrap();
        assert_eq!(s.phase, Phase::Bet2);
    }

    #[test]
    fn raises_are_capped() {
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        Rules::apply_move(&mut s, "p2", HiLoMove::Raise).unwrap();
        Rules::apply_move(&mut s, "p3", HiLoMove::Raise).unwrap();
        assert!(Rules::apply_move(&mut s, "p1", HiLoMove::Raise).is_err(), "two raises per round");
        Rules::apply_move(&mut s, "p1", HiLoMove::Call).unwrap();
        Rules::apply_move(&mut s, "p2", HiLoMove::Call).unwrap();
        assert_eq!(s.phase, Phase::Build);
        assert_eq!(s.put, vec![5, 5, 5]);
    }

    #[test]
    fn pot_splits_between_low_and_high() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        s.cards = vec![[10, 10, 0, 3], [1, 1, 1, 1], [2, 2, 2, 2]];
        for p in ["p2", "p3", "p1"] {
            Rules::apply_move(&mut s, p, HiLoMove::Call).unwrap();
        }
        // p1: 10 + 10 − 0 ÷ 3 = 20 (High); p2: 1 × 1 + 1 − 1 = 1 (Low); p3: 2 + 2 − 2 ÷ 2 = 3 (Low).
        Rules::apply_move(&mut s, "p1", HiLoMove::Build { order: [0, 1, 2, 3], ops: ['+', '-', '/'], side: Side::High }).unwrap();
        Rules::apply_move(&mut s, "p2", HiLoMove::Build { order: [0, 1, 2, 3], ops: ['*', '+', '-'], side: Side::Low }).unwrap();
        Rules::apply_move(&mut s, "p3", HiLoMove::Build { order: [0, 1, 2, 3], ops: ['+', '-', '/'], side: Side::Low }).unwrap();
        for p in ["p2", "p3", "p1"] {
            Rules::apply_move(&mut s, p, HiLoMove::Call).unwrap();
        }
        let last = s.last.as_ref().unwrap();
        assert_eq!(last.pot, 3);
        assert_eq!(last.gains, vec![1, 0, -1], "High takes 2, Low takes 1, each paid 1");
        assert_eq!(s.hand, 1);
    }

    #[test]
    fn last_player_standing_takes_the_pot() {
        let mut s = Rules::setup(&ids(3), 4).unwrap();
        Rules::apply_move(&mut s, "p2", HiLoMove::Raise).unwrap();
        Rules::apply_move(&mut s, "p3", HiLoMove::Fold).unwrap();
        Rules::apply_move(&mut s, "p1", HiLoMove::Fold).unwrap();
        // 19 − 2 + the pot of 5, then the next ante.
        assert_eq!(s.chips, vec![18, 21, 18]);
        assert!(s.last.as_ref().unwrap().cards.iter().all(Option::is_none), "nothing revealed");
    }

    #[test]
    fn view_hides_face_down_cards() {
        let s = Rules::setup(&ids(4), 5).unwrap();
        let v = serde_json::to_value(Rules::view_for(&s, "p1")).unwrap();
        assert_eq!(v["cards"].as_array().unwrap().len(), 4);
        assert_eq!(v["seats"][2]["up"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn timeouts_keep_it_moving() {
        let mut s = Rules::setup(&ids(3), 6).unwrap();
        for _ in 0..200 {
            Rules::tick(&mut s, BUILD_MS as u64);
        }
        assert!(s.over);
        assert_eq!(s.chips.iter().sum::<i64>(), 60);
    }

    #[test]
    fn build_move_reads_from_json() {
        let mv: HiLoMove =
            serde_json::from_value(serde_json::json!({ "type": "build", "order": [3, 1, 2, 0], "ops": ["/", "-", "*"], "side": "low" }))
                .unwrap();
        assert!(matches!(mv, HiLoMove::Build { order: [3, 1, 2, 0], ops: ['/', '-', '*'], side: Side::Low }));
    }

    #[test]
    fn thousand_bot_games() {
        for n in [3, 6] {
            bot_games_with::<Rules>(300, n, 4000);
        }
        bot_games_with::<Rules>(400, 4, 4000);
    }

    #[test]
    fn exact_hits_on_a_big_pot_and_a_big_pot_without_showdown_are_replay_moments() {
        let build = |s: &mut HiLoState, p: &str, ops: [char; 3], side: Side| {
            Rules::apply_move(s, p, HiLoMove::Build { order: [0, 1, 2, 3], ops, side }).unwrap();
        };
        // Bets as (seat, 'r'aise / 'c'all / 'f'old).
        let bet = |s: &mut HiLoState, moves: &[(&str, char)]| {
            for &(p, m) in moves {
                let mv = match m {
                    'r' => HiLoMove::Raise,
                    'f' => HiLoMove::Fold,
                    _ => HiLoMove::Call,
                };
                Rules::apply_move(s, p, mv).unwrap();
            }
        };
        // A small pot taken without a showdown is no moment.
        let mut s = Rules::setup(&ids(3), 4).unwrap();
        bet(&mut s, &[("p2", 'r'), ("p3", 'f'), ("p1", 'f')]);
        assert!(Rules::moments(&s, "p1").is_empty());
        // Four seats, two raises in both rounds: a pot of 36, split 18 / 18.
        let mut s = Rules::setup(&ids(4), 3).unwrap();
        s.cards = vec![[10, 10, 0, 3], [1, 1, 1, 1], [2, 2, 2, 2], [2, 2, 2, 2]];
        let round = [("p2", 'r'), ("p3", 'r'), ("p4", 'c'), ("p1", 'c'), ("p2", 'c')];
        bet(&mut s, &round);
        build(&mut s, "p1", ['+', '-', '/'], Side::High); // 10 + 10 − 0 ÷ 3 = 20
        build(&mut s, "p2", ['*', '+', '-'], Side::Low); // 1 × 1 + 1 − 1 = 1
        build(&mut s, "p3", ['+', '-', '/'], Side::High); // 3
        build(&mut s, "p4", ['+', '-', '/'], Side::Low); // 3
        bet(&mut s, &round);
        assert_eq!(s.last.as_ref().unwrap().pot, 36);
        let m = Rules::moments(&s, "p3");
        assert_eq!(m.len(), 2);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p2", "Exactly 1 takes 18 chips: 1 × 1 + 1 − 1"));
        assert_eq!((m[1].by.as_str(), m[1].text.as_str()), ("p1", "Exactly 20 takes 18 chips: 10 + 10 − 0 ÷ 3"));
        // Hand 2: a pot of 20 after the build; p3 raises again and everyone else folds.
        bet(&mut s, &[("p3", 'r'), ("p4", 'r'), ("p1", 'c'), ("p2", 'c'), ("p3", 'c')]);
        for p in ["p1", "p2", "p3", "p4"] {
            build(&mut s, p, ['+', '-', '*'], Side::High);
        }
        bet(&mut s, &[("p3", 'r'), ("p4", 'f'), ("p1", 'f'), ("p2", 'f')]);
        let m = Rules::moments(&s, "p1");
        assert_eq!(m.len(), 3);
        assert_eq!((m[2].by.as_str(), m[2].text.as_str()), ("p3", "Pot of 22 taken without a showdown"));
    }

    #[test]
    fn an_exact_hit_alone_on_a_big_pot_sweeps_it() {
        let mut s = Rules::setup(&ids(4), 3).unwrap();
        s.cards = vec![[10, 10, 0, 3], [1, 1, 1, 1], [2, 2, 2, 2], [2, 2, 2, 2]];
        // Everyone goes High: p1's exact 20 wins it all, nobody is Low.
        for _ in 0..2 {
            for (p, raise) in [("p2", true), ("p3", true), ("p4", false), ("p1", false), ("p2", false)] {
                Rules::apply_move(&mut s, p, if raise { HiLoMove::Raise } else { HiLoMove::Call }).unwrap();
            }
            if s.phase == Phase::Build {
                for (p, ops) in [("p1", ['+', '-', '/']), ("p2", ['*', '+', '-']), ("p3", ['+', '-', '/']), ("p4", ['+', '-', '/'])] {
                    Rules::apply_move(&mut s, p, HiLoMove::Build { order: [0, 1, 2, 3], ops, side: Side::High }).unwrap();
                }
            }
        }
        let m = Rules::moments(&s, "p4");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Exactly 20 sweeps the pot of 36: 10 + 10 − 0 ÷ 3"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 4_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

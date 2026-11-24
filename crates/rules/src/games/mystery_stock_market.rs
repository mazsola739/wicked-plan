//! Mystery Stock Market (main game). Rules page: docs/games/mystery-stock-market.md
//!
//! Seven made-up stocks, 24 rounds of trading. Each stock moves by its own
//! secret rule (steady, seesaw, crowd, …); players get clues and read the
//! public charts, demand and holder counts to work the rules out.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const STOCKS: usize = 7;
pub const ROUNDS: u32 = 24;
pub const START_PRICE: i64 = 10_000;
pub const FLOOR: i64 = 100;
pub const START_CASH: i64 = 100_000;
pub const REVEAL_MS: i64 = 20_000;
pub const PUBLIC_CLUES: [u32; 2] = [8, 16];
pub const NAMES: [(&str, &str); STOCKS] = [
    ("Lumen Labs", "LUM"),
    ("Cobalt Rail", "COB"),
    ("Pebble Foods", "PEB"),
    ("Vortex Media", "VOR"),
    ("Quill Paper", "QUI"),
    ("Ember Energy", "EMB"),
    ("Tidal Ships", "TID"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    Steady,
    Seesaw,
    Crowd,
    Contrarian,
    Shadow(usize),
    Mirror(usize),
    /// Crashes after this round.
    Bubble(u32),
    Dice,
    Lonely,
    Calendar,
}

pub const RULE_TEXTS: [(&str, &str); 10] = [
    ("Steady", "+5% every round"),
    ("Seesaw", "+20% after odd rounds, −15% after even rounds"),
    ("Crowd", "+4% per share bought by everyone this round, −4% per share sold (max ±40%)"),
    ("Contrarian", "the opposite of Crowd"),
    ("Shadow", "the same change as one other stock this round"),
    ("Mirror", "the opposite change of one other stock this round"),
    ("Bubble", "+12% a round until a secret round (12–18), then −60% once, then −5% a round"),
    ("Dice", "a random change from −25% to +25%"),
    ("Lonely", "−10% when more than half the players hold it, +12% otherwise"),
    ("Calendar", "+30% after every 4th round, −6% after the others"),
];

impl Rule {
    fn name(self) -> &'static str {
        match self {
            Rule::Steady => "Steady",
            Rule::Seesaw => "Seesaw",
            Rule::Crowd => "Crowd",
            Rule::Contrarian => "Contrarian",
            Rule::Shadow(_) => "Shadow",
            Rule::Mirror(_) => "Mirror",
            Rule::Bubble(_) => "Bubble",
            Rule::Dice => "Dice",
            Rule::Lonely => "Lonely",
            Rule::Calendar => "Calendar",
        }
    }

    /// The exact rule, with its hidden details.
    pub fn text(self) -> String {
        match self {
            Rule::Shadow(x) => format!("Shadow: moves like {}", NAMES[x].0),
            Rule::Mirror(x) => format!("Mirror: moves opposite to {}", NAMES[x].0),
            Rule::Bubble(b) => format!("Bubble: +12% a round, crashes −60% after round {b}, then −5% a round"),
            other => {
                let (name, text) = RULE_TEXTS.iter().find(|(n, _)| *n == other.name()).copied().unwrap_or(("", ""));
                format!("{name}: {text}")
            }
        }
    }

    /// Change in percent after `round` for a rule that does not follow another stock.
    fn own_pct(self, round: u32, net: i64, holders: usize, players: usize, dice: i64) -> i64 {
        match self {
            Rule::Steady => 5,
            Rule::Seesaw => {
                if round % 2 == 1 {
                    20
                } else {
                    -15
                }
            }
            Rule::Crowd => (4 * net).clamp(-40, 40),
            Rule::Contrarian => (-4 * net).clamp(-40, 40),
            Rule::Bubble(b) => {
                if round < b {
                    12
                } else if round == b {
                    -60
                } else {
                    -5
                }
            }
            Rule::Dice => dice,
            Rule::Lonely => {
                if holders * 2 > players {
                    -10
                } else {
                    12
                }
            }
            Rule::Calendar => {
                if round.is_multiple_of(4) {
                    30
                } else {
                    -6
                }
            }
            Rule::Shadow(_) | Rule::Mirror(_) => 0,
        }
    }
}

pub fn apply_pct(price: i64, pct: i64) -> i64 {
    ((price * (100 + pct) + 50) / 100).max(FLOOR)
}

const MILLION: i64 = 1_000_000;

/// 1376000 → "1,376,000".
fn money(v: i64) -> String {
    let digits = v.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Trade,
    Reveal,
    Over,
}

#[derive(Clone, Debug)]
pub struct Trader {
    pub id: String,
    pub cash: i64,
    pub shares: [i64; STOCKS],
    pub done: bool,
    pub clue: usize,
}

#[derive(Clone, Debug)]
pub struct MarketState {
    pub traders: Vec<Trader>,
    pub rules: [Rule; STOCKS],
    /// `prices[r]` after round r; `prices[0]` is the start.
    pub prices: Vec<[i64; STOCKS]>,
    pub net: Vec<[i64; STOCKS]>,
    pub holders: Vec<[usize; STOCKS]>,
    pub current_net: [i64; STOCKS],
    pub dice: Vec<[i64; STOCKS]>,
    pub public_clues: Vec<usize>,
    pub round: u32,
    pub phase: Phase,
    pub time_left_ms: i64,
    pub log: Vec<String>,
    pub rng: Rng,
    /// Replay moments: a portfolio up 30% in one round (only its owner sees it),
    /// the richest worth at the close when it is a million or more.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MarketMove {
    Trade { stock: usize, qty: i64 },
    Done,
    Close,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockView {
    pub name: &'static str,
    pub ticker: &'static str,
    pub price: i64,
    pub history: Vec<i64>,
    pub net: Vec<i64>,
    pub holders: Vec<usize>,
    pub rule: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraderView {
    pub id: String,
    pub done: bool,
    pub worth: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketView {
    pub phase: &'static str,
    pub round: u32,
    pub rounds: u32,
    pub stocks: Vec<StockView>,
    pub cash: i64,
    pub shares: [i64; STOCKS],
    pub worth: i64,
    pub done: bool,
    pub traders: Vec<TraderView>,
    pub clue: String,
    pub rule_texts: Vec<(&'static str, &'static str)>,
    pub log: Vec<String>,
    pub score: i64,
}

/// Points by rank: 1000 for first down to 100 for last.
fn rank_points(scores: &[(String, i64)]) -> Vec<PlayerResult> {
    let n = scores.len() as i64;
    rank_by_score(scores)
        .into_iter()
        .map(|mut r| {
            r.points = if n > 1 { 100 + 900 * (n - i64::from(r.rank)) / (n - 1) } else { 1000 };
            r
        })
        .collect()
}

impl MarketState {
    fn say(&mut self, line: String) {
        self.log.push(line);
        let extra = self.log.len().saturating_sub(10);
        self.log.drain(..extra);
    }

    pub fn price(&self, k: usize) -> i64 {
        self.prices.last().map_or(START_PRICE, |p| p[k])
    }

    pub fn worth(&self, i: usize) -> i64 {
        let t = &self.traders[i];
        t.cash + (0..STOCKS).map(|k| t.shares[k] * self.price(k)).sum::<i64>()
    }

    fn end_round(&mut self) {
        let r = self.round;
        let n = self.traders.len();
        let held: [usize; STOCKS] = std::array::from_fn(|k| self.traders.iter().filter(|t| t.shares[k] > 0).count());
        let mut pcts = [0i64; STOCKS];
        for k in 0..STOCKS {
            pcts[k] = self.rules[k].own_pct(r, self.current_net[k], held[k], n, self.dice[r as usize - 1][k]);
        }
        for k in 0..STOCKS {
            match self.rules[k] {
                Rule::Shadow(x) => pcts[k] = pcts[x],
                Rule::Mirror(x) => pcts[k] = -pcts[x],
                _ => {}
            }
        }
        let old = *self.prices.last().expect("prices");
        let new: [i64; STOCKS] = std::array::from_fn(|k| apply_pct(old[k], pcts[k]));
        let before: Vec<i64> = (0..n).map(|i| self.worth(i)).collect();
        self.prices.push(new);
        // A replay moment only its owner sees (holdings and cash are secret): worth up 30% in one round.
        for (i, &was) in before.iter().enumerate() {
            let gain = (self.worth(i) - was) * 100 / was.max(1);
            if gain >= 30 {
                let id = self.traders[i].id.clone();
                self.moments.push_for(&id, format!("Portfolio up {gain}% in one round"), &[&id]);
            }
        }
        self.net.push(self.current_net);
        self.holders.push(held);
        self.current_net = [0; STOCKS];
        for t in &mut self.traders {
            t.done = false;
        }
        let best = (0..STOCKS).max_by_key(|&k| new[k] * 1000 / old[k]).unwrap_or(0);
        let worst = (0..STOCKS).min_by_key(|&k| new[k] * 1000 / old[k]).unwrap_or(0);
        self.say(format!("Round {r}: best {} ({:+}%), worst {} ({:+}%).", NAMES[best].1, pcts[best], NAMES[worst].1, pcts[worst]));
        if PUBLIC_CLUES.contains(&r) {
            let hidden: Vec<usize> = (0..STOCKS).filter(|k| !self.public_clues.contains(k)).collect();
            if !hidden.is_empty() {
                let k = *self.rng.pick(&hidden);
                self.public_clues.push(k);
                let line = format!("Clue for everyone: {} follows {}.", NAMES[k].0, self.rules[k].text());
                self.say(line);
            }
        }
        if r >= ROUNDS {
            self.phase = Phase::Reveal;
            self.time_left_ms = REVEAL_MS;
            self.say("The market is closed. All rules revealed.".into());
            // A public replay moment (every worth is shown at the close): the richest player is a millionaire.
            let top = (0..n).rev().max_by_key(|&i| self.worth(i)).unwrap_or(0);
            let worth = self.worth(top);
            if worth >= MILLION {
                let id = self.traders[top].id.clone();
                self.moments.push(&id, format!("A millionaire at the close: {}", money(worth)));
            }
        } else {
            self.round += 1;
        }
    }

    /// Change in percent of stock `x` in completed round `r` (1-based), from the prices.
    fn realized(&self, x: usize, r: usize) -> f64 {
        let old = self.prices[r - 1][x] as f64;
        (self.prices[r][x] as f64 - old) * 100.0 / old
    }

    /// Whether `rule` fits the public history of stock `k`.
    pub fn fits(&self, k: usize, rule: Rule) -> bool {
        let n = self.traders.len();
        for r in 1..self.prices.len() {
            let old = self.prices[r - 1][k];
            let new = self.prices[r][k];
            let got = self.realized(k, r);
            let tol = 1.0 + 100.0 / old as f64;
            let ok = |want: f64| (got - want).abs() <= tol || (new == FLOOR && old as f64 * (100.0 + want) / 100.0 <= FLOOR as f64 + 1.0);
            let fine = match rule {
                Rule::Dice => got.abs() <= 25.0 + tol || new == FLOOR,
                Rule::Shadow(x) => x != k && ok(self.realized(x, r)),
                Rule::Mirror(x) => x != k && ok(-self.realized(x, r)),
                own => ok(own.own_pct(r as u32, self.net[r - 1][k], self.holders[r - 1][k], n, 0) as f64),
            };
            if !fine {
                return false;
            }
        }
        true
    }

    /// The rules a player could still believe for stock `k`.
    pub fn candidates(&self, k: usize, clue: Option<usize>) -> Vec<Rule> {
        if clue == Some(k) || self.public_clues.contains(&k) {
            return vec![self.rules[k]];
        }
        let mut all = vec![Rule::Steady, Rule::Seesaw, Rule::Crowd, Rule::Contrarian, Rule::Dice, Rule::Lonely, Rule::Calendar];
        all.extend((12..=18).map(Rule::Bubble));
        for x in (0..STOCKS).filter(|&x| x != k) {
            all.push(Rule::Shadow(x));
            all.push(Rule::Mirror(x));
        }
        let fit: Vec<Rule> = all.into_iter().filter(|&r| self.fits(k, r)).collect();
        if fit.is_empty() { vec![Rule::Dice] } else { fit }
    }

    /// Expected change of each stock next round, for a player with this clue.
    pub fn expected(&self, clue: Option<usize>) -> [f64; STOCKS] {
        let n = self.traders.len();
        let next = self.round;
        let held = self.holders.last().copied().unwrap_or([0; STOCKS]);
        let cands: Vec<Vec<Rule>> = (0..STOCKS).map(|k| self.candidates(k, clue)).collect();
        let own = |rule: Rule, k: usize| -> f64 {
            match rule {
                Rule::Crowd | Rule::Contrarian | Rule::Dice | Rule::Shadow(_) | Rule::Mirror(_) => 0.0,
                r => r.own_pct(next, 0, held[k], n, 0) as f64,
            }
        };
        let base: Vec<f64> = (0..STOCKS).map(|k| cands[k].iter().map(|&r| own(r, k)).sum::<f64>() / cands[k].len() as f64).collect();
        std::array::from_fn(|k| {
            cands[k]
                .iter()
                .map(|&r| match r {
                    Rule::Shadow(x) => base[x],
                    Rule::Mirror(x) => -base[x],
                    r => own(r, k),
                })
                .sum::<f64>()
                / cands[k].len() as f64
        })
    }
}

pub struct Rules;

impl Game for Rules {
    type State = MarketState;
    type Move = MarketMove;
    type View = MarketView;

    fn info() -> GameInfo {
        GameInfo {
            id: "mystery-stock-market",
            kind: "main",
            difficulty: "hard",
            tags: &["logic", "numbers"],
            min_players: 3,
            max_players: 8,
            name: "Mystery Stock Market",
            short: "Trade seven made-up stocks over 24 rounds and work out the secret rules that move their prices.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<MarketState, String> {
        let n = players.len();
        if !(3..=8).contains(&n) {
            return Err("Mystery Stock Market needs 3 to 8 players".into());
        }
        let mut rng = Rng::new(seed);
        let mut kinds: Vec<u8> = (0..10).collect();
        rng.shuffle(&mut kinds);
        kinds.truncate(STOCKS);
        // Shadow and Mirror follow a stock with one of the other rules.
        let primaries: Vec<usize> = (0..STOCKS).filter(|&k| kinds[k] != 4 && kinds[k] != 5).collect();
        let mut rules = [Rule::Steady; STOCKS];
        for k in 0..STOCKS {
            rules[k] = match kinds[k] {
                0 => Rule::Steady,
                1 => Rule::Seesaw,
                2 => Rule::Crowd,
                3 => Rule::Contrarian,
                4 => Rule::Shadow(*rng.pick(&primaries)),
                5 => Rule::Mirror(*rng.pick(&primaries)),
                6 => Rule::Bubble(rng.int(12, 18) as u32),
                7 => Rule::Dice,
                8 => Rule::Lonely,
                _ => Rule::Calendar,
            };
        }
        let dice = (0..ROUNDS).map(|_| std::array::from_fn(|_| rng.int(-25, 25))).collect();
        let mut clue_order: Vec<usize> = (0..STOCKS).collect();
        rng.shuffle(&mut clue_order);
        let traders = players
            .iter()
            .enumerate()
            .map(|(i, id)| Trader { id: id.clone(), cash: START_CASH, shares: [0; STOCKS], done: false, clue: clue_order[i % STOCKS] })
            .collect();
        Ok(MarketState {
            traders,
            rules,
            prices: vec![[START_PRICE; STOCKS]],
            net: Vec::new(),
            holders: Vec::new(),
            current_net: [0; STOCKS],
            dice,
            public_clues: Vec::new(),
            round: 1,
            phase: Phase::Trade,
            time_left_ms: 0,
            log: vec!["The market opens. Every stock costs 10,000.".into()],
            rng,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut MarketState, player: &str, mv: MarketMove) -> Result<(), String> {
        let i = s.traders.iter().position(|t| t.id == player).ok_or("Not in this game")?;
        match (s.phase, mv) {
            (Phase::Trade, _) if s.traders[i].done => Err("You are done this round".into()),
            (Phase::Trade, MarketMove::Trade { stock, qty }) => {
                if stock >= STOCKS || qty == 0 {
                    return Err("No such trade".into());
                }
                let price = s.price(stock);
                let t = &mut s.traders[i];
                if qty > 0 && qty.saturating_mul(price) > t.cash {
                    return Err("Not enough cash".into());
                }
                if qty < 0 && -qty > t.shares[stock] {
                    return Err("You do not have those shares".into());
                }
                t.cash -= qty * price;
                t.shares[stock] += qty;
                s.current_net[stock] += qty;
                Ok(())
            }
            (Phase::Trade, MarketMove::Done) => {
                s.traders[i].done = true;
                if s.traders.iter().all(|t| t.done) {
                    s.end_round();
                }
                Ok(())
            }
            (Phase::Reveal, MarketMove::Close) => {
                s.phase = Phase::Over;
                Ok(())
            }
            (Phase::Over, _) => Err("Game is over".into()),
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut MarketState, ms: u64) {
        if s.phase != Phase::Reveal {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.phase = Phase::Over;
        }
    }

    fn view_for(s: &MarketState, player: &str) -> MarketView {
        let me = s.traders.iter().position(|t| t.id == player).unwrap_or(0);
        let end = s.phase != Phase::Trade;
        let mine = &s.traders[me];
        MarketView {
            phase: match s.phase {
                Phase::Trade => "trade",
                Phase::Reveal => "reveal",
                Phase::Over => "over",
            },
            round: s.round,
            rounds: ROUNDS,
            stocks: (0..STOCKS)
                .map(|k| StockView {
                    name: NAMES[k].0,
                    ticker: NAMES[k].1,
                    price: s.price(k),
                    history: s.prices.iter().map(|p| p[k]).collect(),
                    net: s.net.iter().map(|p| p[k]).collect(),
                    holders: s.holders.iter().map(|p| p[k]).collect(),
                    rule: (end || mine.clue == k || s.public_clues.contains(&k)).then(|| s.rules[k].text()),
                })
                .collect(),
            cash: mine.cash,
            shares: mine.shares,
            worth: s.worth(me),
            done: mine.done,
            traders: (0..s.traders.len())
                .map(|i| TraderView { id: s.traders[i].id.clone(), done: s.traders[i].done, worth: end.then(|| s.worth(i)) })
                .collect(),
            clue: format!("{} follows {}", NAMES[mine.clue].0, s.rules[mine.clue].text()),
            rule_texts: RULE_TEXTS.to_vec(),
            log: s.log.clone(),
            score: s.worth(me),
        }
    }

    fn is_over(s: &MarketState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &MarketState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = (0..s.traders.len()).map(|i| (s.traders[i].id.clone(), s.worth(i))).collect();
        rank_points(&scores)
    }

    fn to_move(s: &MarketState) -> Vec<String> {
        if s.phase != Phase::Trade {
            return Vec::new();
        }
        s.traders.iter().filter(|t| !t.done).map(|t| t.id.clone()).collect()
    }

    fn moments(s: &MarketState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &MarketState, player: &str, _seed: u64) -> MarketMove {
        let Some(i) = s.traders.iter().position(|t| t.id == player) else { return MarketMove::Close };
        if s.phase != Phase::Trade {
            return MarketMove::Close;
        }
        let me = &s.traders[i];
        let exp = s.expected(Some(me.clue));
        if let Some(k) = (0..STOCKS).find(|&k| me.shares[k] > 0 && exp[k] < 0.0) {
            return MarketMove::Trade { stock: k, qty: -me.shares[k] };
        }
        let best = (0..STOCKS).max_by(|&a, &b| exp[a].total_cmp(&exp[b])).unwrap_or(0);
        let qty = me.cash * 9 / 10 / s.price(best);
        if exp[best] >= 3.0 && qty >= 1 {
            return MarketMove::Trade { stock: best, qty };
        }
        MarketMove::Done
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    fn all_done(s: &mut MarketState) {
        for p in players(s.traders.len()) {
            let _ = Rules::apply_move(s, &p, MarketMove::Done);
        }
    }

    #[test]
    fn trading_checks_cash_and_shares() {
        let mut s = Rules::setup(&players(3), 1).unwrap();
        assert!(Rules::apply_move(&mut s, "p1", MarketMove::Trade { stock: 0, qty: 11 }).is_err());
        Rules::apply_move(&mut s, "p1", MarketMove::Trade { stock: 0, qty: 10 }).unwrap();
        assert_eq!(s.traders[0].cash, 0);
        assert!(Rules::apply_move(&mut s, "p1", MarketMove::Trade { stock: 1, qty: -1 }).is_err());
        Rules::apply_move(&mut s, "p1", MarketMove::Trade { stock: 0, qty: -4 }).unwrap();
        assert_eq!(s.current_net[0], 6);
        assert_eq!(s.worth(0), START_CASH);
    }

    #[test]
    fn prices_follow_the_rules() {
        let mut s = Rules::setup(&players(3), 2).unwrap();
        s.rules = [Rule::Steady, Rule::Seesaw, Rule::Crowd, Rule::Shadow(1), Rule::Mirror(0), Rule::Lonely, Rule::Calendar];
        Rules::apply_move(&mut s, "p1", MarketMove::Trade { stock: 2, qty: 3 }).unwrap();
        Rules::apply_move(&mut s, "p2", MarketMove::Trade { stock: 5, qty: 1 }).unwrap();
        Rules::apply_move(&mut s, "p3", MarketMove::Trade { stock: 5, qty: 1 }).unwrap();
        all_done(&mut s);
        assert_eq!(s.prices[1], [10_500, 12_000, 11_200, 12_000, 9_500, 9_000, 9_400]);
        assert_eq!(s.round, 2);
        assert_eq!(apply_pct(105, -60), FLOOR);
    }

    #[test]
    fn a_round_waits_for_everyone_and_the_reveal_closes_by_itself() {
        let mut s = Rules::setup(&players(3), 4).unwrap();
        Rules::tick(&mut s, 600_000);
        assert_eq!((s.round, s.phase), (1, Phase::Trade));
        assert_eq!(Rules::to_move(&s).len(), 3);
        for _ in 0..ROUNDS {
            all_done(&mut s);
        }
        assert_eq!(s.phase, Phase::Reveal);
        Rules::tick(&mut s, REVEAL_MS as u64);
        assert_eq!(s.phase, Phase::Over);
    }

    #[test]
    fn deduction_keeps_the_true_rule() {
        for seed in 1..=20 {
            let mut s = Rules::setup(&players(4), seed).unwrap();
            for _ in 0..12 {
                for p in players(4) {
                    for _ in 0..5 {
                        let mv = Rules::bot_move(&s, &p, 1);
                        let done = matches!(mv, MarketMove::Done);
                        Rules::apply_move(&mut s, &p, mv).unwrap();
                        if done {
                            break;
                        }
                    }
                    let _ = Rules::apply_move(&mut s, &p, MarketMove::Done);
                }
            }
            for k in 0..STOCKS {
                let c = s.candidates(k, None);
                assert!(c.contains(&s.rules[k]) || s.rules[k] == Rule::Dice, "seed {seed} stock {k}: {:?} not in {c:?}", s.rules[k]);
            }
        }
    }

    #[test]
    fn rules_are_secret_until_the_end() {
        let mut s = Rules::setup(&players(3), 3).unwrap();
        let v = Rules::view_for(&s, "p1");
        assert_eq!(v.stocks.iter().filter(|k| k.rule.is_some()).count(), 1);
        for _ in 0..ROUNDS {
            all_done(&mut s);
        }
        assert_eq!(s.phase, Phase::Reveal);
        assert_eq!(Rules::view_for(&s, "p1").stocks.iter().filter(|k| k.rule.is_some()).count(), STOCKS);
        assert_eq!(s.public_clues.len(), 2);
    }

    #[test]
    fn bots_beat_doing_nothing_on_average() {
        let mut total = 0;
        for seed in 1..=30 {
            let mut s = Rules::setup(&players(3), seed).unwrap();
            while s.phase == Phase::Trade {
                for p in ["p1", "p2"] {
                    for _ in 0..5 {
                        let mv = Rules::bot_move(&s, p, 1);
                        if matches!(mv, MarketMove::Done) {
                            break;
                        }
                        Rules::apply_move(&mut s, p, mv).unwrap();
                    }
                }
                all_done(&mut s);
            }
            total += s.worth(0) + s.worth(1);
        }
        assert!(total > 2 * 30 * START_CASH, "bots ended with {total}");
    }

    #[test]
    fn bot_games() {
        bot_games_with::<Rules>(1000, 4, 6000);
        bot_games_with::<Rules>(200, 3, 6000);
    }

    #[test]
    fn a_big_round_is_a_moment_only_its_owner_sees() {
        let mut s = Rules::setup(&players(3), 2).unwrap();
        s.rules = [Rule::Steady, Rule::Seesaw, Rule::Crowd, Rule::Shadow(1), Rule::Mirror(0), Rule::Lonely, Rule::Calendar];
        // All cash into Crowd: 10 shares bought → +40%.
        Rules::apply_move(&mut s, "p1", MarketMove::Trade { stock: 2, qty: 10 }).unwrap();
        all_done(&mut s);
        let m = Rules::moments(&s, "p1");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Portfolio up 40% in one round"));
        assert!(Rules::moments(&s, "p2").is_empty(), "the others must not learn p1's holdings");
    }

    #[test]
    fn a_millionaire_at_the_close_is_a_public_moment() {
        let mut s = Rules::setup(&players(3), 3).unwrap();
        s.traders[1].cash = 1_376_000;
        for _ in 0..ROUNDS {
            assert!(Rules::moments(&s, "p1").is_empty());
            all_done(&mut s);
        }
        assert_eq!(s.phase, Phase::Reveal);
        let m = Rules::moments(&s, "p1");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p2", "A millionaire at the close: 1,376,000"));
        // Nobody near a million: no moment.
        let mut s = Rules::setup(&players(3), 3).unwrap();
        for _ in 0..ROUNDS {
            all_done(&mut s);
        }
        assert!(Rules::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 6000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 10 <= stats.games * 8, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

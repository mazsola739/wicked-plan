//! Big or Small (final duel, 2 players). Rules page: docs/games/big-or-small.md
//!
//! Both players hold the cards 1–9. Each round the dealer turns up a card;
//! both secretly play one of their cards and bet whether the opponent's card
//! is bigger, smaller or equal to the dealer's. Chips decide the winner.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const CARDS: u8 = 9;
pub const START_CHIPS: i64 = 10;
pub const MAX_STAKE: i64 = 3;
pub const TURN_MS: i64 = 45_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Guess {
    Bigger,
    Smaller,
    Equal,
}

impl Guess {
    fn of(card: u8, dealer: u8) -> Guess {
        match card.cmp(&dealer) {
            std::cmp::Ordering::Greater => Guess::Bigger,
            std::cmp::Ordering::Less => Guess::Smaller,
            std::cmp::Ordering::Equal => Guess::Equal,
        }
    }
    /// Chips won per staked chip when right.
    fn pays(self) -> i64 {
        if self == Guess::Equal { 2 } else { 1 }
    }
}

#[derive(Clone, Copy, Debug)]
struct Pick {
    card: u8,
    guess: Guess,
    stake: i64,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SideLog {
    pub card: u8,
    pub guess: Guess,
    pub stake: i64,
    pub right: bool,
    pub change: i64,
}

#[derive(Clone, Debug)]
struct RoundLog {
    dealer: u8,
    sides: [SideLog; 2],
}

#[derive(Clone, Debug)]
pub struct BigState {
    players: [String; 2],
    dealer: Vec<u8>,
    round: usize,
    hands: [Vec<u8>; 2],
    chips: [i64; 2],
    picks: [Option<Pick>; 2],
    history: Vec<RoundLog>,
    time_left_ms: i64,
    over: bool,
    /// Replay moments: an exact "equal" call, a six-chip swing, the lead stolen in the last round.
    pub moments: Moments,
}

impl BigState {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }
    fn dealer_card(&self) -> u8 {
        self.dealer[self.round.min(self.dealer.len() - 1)]
    }
    fn resolve(&mut self) {
        let (Some(a), Some(b)) = (self.picks[0], self.picks[1]) else { return };
        let dealer = self.dealer_card();
        let picks = [a, b];
        let sides: [SideLog; 2] = std::array::from_fn(|i| {
            let mine = picks[i];
            let theirs = picks[1 - i];
            let right = Guess::of(theirs.card, dealer) == mine.guess;
            let change = if right { mine.stake * mine.guess.pays() } else { -mine.stake };
            SideLog { card: mine.card, guess: mine.guess, stake: mine.stake, right, change }
        });
        let behind = [self.chips[0] < self.chips[1], self.chips[1] < self.chips[0]];
        for i in 0..2 {
            self.chips[i] += sides[i].change;
            self.hands[i].retain(|&c| c != picks[i].card);
        }
        // Replay moments (both cards are shown now): an exact call, a full-stake read while the
        // other full-stake read fails, and the lead stolen at the end.
        for i in 0..2 {
            let (me, them) = (sides[i], sides[1 - i]);
            let other = &self.players[1 - i];
            let text = if me.right && me.guess == Guess::Equal && me.stake > 0 {
                format!("Called {{{other}}}'s {} exactly (+{})", picks[1 - i].card, me.change)
            } else if me.right && me.stake == MAX_STAKE && !them.right && them.stake == MAX_STAKE {
                format!("Six-chip swing against {{{other}}}")
            } else {
                continue;
            };
            self.moments.push(&self.players[i], text);
        }
        for (i, was_behind) in behind.into_iter().enumerate() {
            if self.round + 1 == CARDS as usize && was_behind && self.chips[i] > self.chips[1 - i] {
                let text = format!("Lead stolen in the last round ({}–{})", self.chips[i], self.chips[1 - i]);
                self.moments.push(&self.players[i], text);
            }
        }
        self.history.push(RoundLog { dealer, sides });
        self.picks = [None, None];
        self.round += 1;
        self.time_left_ms = TURN_MS;
        if self.round >= CARDS as usize {
            self.over = true;
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BigMove {
    Play { card: u8, guess: Guess, stake: i64 },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogView {
    pub dealer: u8,
    pub mine: SideLog,
    pub theirs: SideLog,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BigView {
    pub round: usize,
    pub rounds: usize,
    pub dealer: u8,
    pub hand: Vec<u8>,
    pub their_hand: Vec<u8>,
    pub chips: i64,
    pub their_chips: i64,
    pub opponent: String,
    pub chosen: bool,
    pub they_chose: bool,
    pub max_stake: i64,
    pub time_left_ms: i64,
    pub turn_ms: i64,
    pub history: Vec<LogView>,
    pub over: bool,
}

pub struct BigOrSmall;

impl Game for BigOrSmall {
    type State = BigState;
    type Move = BigMove;
    type View = BigView;

    fn info() -> GameInfo {
        GameInfo {
            id: "big-or-small",
            kind: "final-duel",
            difficulty: "easy",
            tags: &["cards", "numbers"],
            min_players: 2,
            max_players: 2,
            name: "Big or Small",
            short: "Play a card in secret and bet whether your opponent's card beats the dealer's. Nine rounds of chips.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<BigState, String> {
        let [a, b] = players else { return Err("Big or Small is for 2 players".into()) };
        let mut rng = Rng::new(seed);
        let mut dealer: Vec<u8> = (1..=CARDS).collect();
        rng.shuffle(&mut dealer);
        Ok(BigState {
            players: [a.clone(), b.clone()],
            dealer,
            round: 0,
            hands: [(1..=CARDS).collect(), (1..=CARDS).collect()],
            chips: [START_CHIPS; 2],
            picks: [None, None],
            history: Vec::new(),
            time_left_ms: TURN_MS,
            over: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut BigState, player: &str, mv: BigMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let i = s.seat(player).ok_or("Not in this game")?;
        if s.picks[i].is_some() {
            return Err("You have already chosen".into());
        }
        let BigMove::Play { card, guess, stake } = mv;
        if !s.hands[i].contains(&card) {
            return Err("You do not hold that card".into());
        }
        let most = MAX_STAKE.min(s.chips[i]);
        let least = if s.chips[i] > 0 { 1 } else { 0 };
        if stake < least || stake > most {
            return Err(format!("Stake {least}–{most} chips"));
        }
        s.picks[i] = Some(Pick { card, guess, stake });
        s.resolve();
        Ok(())
    }

    fn tick(s: &mut BigState, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            for i in 0..2 {
                if s.picks[i].is_none() {
                    let p = s.players[i].clone();
                    let BigMove::Play { card, guess, stake } = Self::bot_move(s, &p, s.round as u64 + 1);
                    s.picks[i] = Some(Pick { card, guess, stake });
                }
            }
            s.resolve();
        }
    }

    fn view_for(s: &BigState, player: &str) -> BigView {
        let i = s.seat(player).unwrap_or(0);
        let o = 1 - i;
        BigView {
            round: (s.round + 1).min(CARDS as usize),
            rounds: CARDS as usize,
            dealer: s.dealer_card(),
            hand: s.hands[i].clone(),
            their_hand: s.hands[o].clone(),
            chips: s.chips[i],
            their_chips: s.chips[o],
            opponent: s.players[o].clone(),
            chosen: s.picks[i].is_some(),
            they_chose: s.picks[o].is_some(),
            max_stake: MAX_STAKE,
            time_left_ms: s.time_left_ms.max(0),
            turn_ms: TURN_MS,
            history: s.history.iter().map(|r| LogView { dealer: r.dealer, mine: r.sides[i], theirs: r.sides[o] }).collect(),
            over: s.over,
        }
    }

    fn is_over(s: &BigState) -> bool {
        s.over
    }

    fn results(s: &BigState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = (0..2).map(|i| (s.players[i].clone(), 100 + 20 * s.chips[i])).collect();
        rank_by_score(&scores)
    }

    fn to_move(s: &BigState) -> Vec<String> {
        if s.over {
            return Vec::new();
        }
        (0..2).filter(|&i| s.picks[i].is_none()).map(|i| s.players[i].clone()).collect()
    }

    fn bot_move(s: &BigState, player: &str, seed: u64) -> BigMove {
        let mut rng = Rng::new(seed);
        let i = s.seat(player).unwrap_or(0);
        let dealer = s.dealer_card();
        let classes = [Guess::Bigger, Guess::Smaller, Guess::Equal];
        // Expected chips per staked chip for each guess, if the opponent plays any remaining card.
        let ev = |hand: &[u8], g: Guess| {
            let n = hand.len().max(1) as f64;
            let p = hand.iter().filter(|&&c| Guess::of(c, dealer) == g).count() as f64 / n;
            p * g.pays() as f64 - (1.0 - p)
        };
        let theirs = &s.hands[1 - i];
        let guess = *classes.iter().max_by(|a, b| ev(theirs, **a).total_cmp(&ev(theirs, **b))).expect("classes");
        let best = ev(theirs, guess);
        let want = if best >= 0.6 {
            3
        } else if best >= 0.2 {
            2
        } else {
            1
        };
        let stake = if s.chips[i] > 0 { want.min(MAX_STAKE).min(s.chips[i]) } else { 0 };
        // Play a card from the class the opponent is least likely to bet on.
        let mine = &s.hands[i];
        let their_guess = *classes.iter().max_by(|a, b| ev(mine, **a).total_cmp(&ev(mine, **b))).expect("classes");
        let mut options: Vec<u8> = mine.iter().copied().filter(|&c| Guess::of(c, dealer) != their_guess).collect();
        if options.is_empty() {
            options = mine.clone();
        }
        let card = if options.is_empty() { 1 } else { *rng.pick(&options) };
        BigMove::Play { card, guess, stake }
    }

    fn moments(s: &BigState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub const READY: bool = true;
pub type Rules = BigOrSmall;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn new() -> BigState {
        BigOrSmall::setup(&["a".into(), "b".into()], 5).unwrap()
    }

    fn play(card: u8, guess: Guess, stake: i64) -> BigMove {
        BigMove::Play { card, guess, stake }
    }

    #[test]
    fn a_round_pays_the_right_bets() {
        let mut s = new();
        let d = s.dealer_card();
        // a plays the dealer's card; b guesses equal (pays double), a guesses bigger.
        let other = if d == 9 { 1 } else { 9 };
        BigOrSmall::apply_move(&mut s, "a", play(d, Guess::Bigger, 2)).unwrap();
        assert_eq!(BigOrSmall::to_move(&s), vec!["b".to_string()]);
        BigOrSmall::apply_move(&mut s, "b", play(other, Guess::Equal, 3)).unwrap();
        let a_right = other > d;
        assert_eq!(s.chips[0], START_CHIPS + if a_right { 2 } else { -2 });
        assert_eq!(s.chips[1], START_CHIPS + 6);
        assert_eq!(s.round, 1);
        assert!(!s.hands[0].contains(&d));
        assert!(!s.hands[1].contains(&other));
    }

    #[test]
    fn bad_moves_are_refused() {
        let mut s = new();
        assert!(BigOrSmall::apply_move(&mut s, "a", play(10, Guess::Bigger, 1)).is_err());
        assert!(BigOrSmall::apply_move(&mut s, "a", play(3, Guess::Bigger, 4)).is_err());
        assert!(BigOrSmall::apply_move(&mut s, "a", play(3, Guess::Bigger, 0)).is_err());
        assert!(BigOrSmall::apply_move(&mut s, "x", play(3, Guess::Bigger, 1)).is_err());
        BigOrSmall::apply_move(&mut s, "a", play(3, Guess::Bigger, 1)).unwrap();
        assert!(BigOrSmall::apply_move(&mut s, "a", play(4, Guess::Bigger, 1)).is_err());
    }

    #[test]
    fn the_secret_card_stays_hidden_until_both_chose() {
        let mut s = new();
        BigOrSmall::apply_move(&mut s, "a", play(3, Guess::Bigger, 1)).unwrap();
        let v = serde_json::to_value(BigOrSmall::view_for(&s, "b")).unwrap();
        assert_eq!(v["theirHand"].as_array().unwrap().len(), 9);
        assert_eq!(v["theyChose"], true);
        assert!(v["history"].as_array().unwrap().is_empty());
    }

    #[test]
    fn time_up_chooses_for_you() {
        let mut s = new();
        BigOrSmall::tick(&mut s, TURN_MS as u64);
        assert_eq!(s.round, 1);
        assert_eq!(s.history.len(), 1);
    }

    #[test]
    fn the_last_card_is_a_sure_bet_for_the_bot() {
        let mut s = new();
        for _ in 0..8 {
            BigOrSmall::tick(&mut s, TURN_MS as u64);
        }
        let BigMove::Play { guess, .. } = BigOrSmall::bot_move(&s, "a", 1);
        let last = s.hands[1][0];
        assert_eq!(guess, Guess::of(last, s.dealer_card()));
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<BigOrSmall>(1000, 2, 500);
    }

    #[test]
    fn an_exact_call_a_swing_and_a_last_round_steal_are_replay_moments() {
        let mut s = new();
        let d = s.dealer_card();
        let other = if d == 9 { 1 } else { 9 };
        // b plays the dealer's card and a calls it equal; b's bigger/smaller bet is no moment.
        BigOrSmall::apply_move(&mut s, "a", play(other, Guess::Equal, 3)).unwrap();
        BigOrSmall::apply_move(&mut s, "b", play(d, Guess::Bigger, 1)).unwrap();
        let m = BigOrSmall::moments(&s, "b");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.clone()), ("a", format!("Called {{b}}'s {d} exactly (+6)")));
        // Both stake 3: b reads a's 1 right, a's 9-call on b's 2 fails.
        let mut s = new();
        s.dealer[0] = 5;
        BigOrSmall::apply_move(&mut s, "a", play(1, Guess::Bigger, 3)).unwrap();
        BigOrSmall::apply_move(&mut s, "b", play(2, Guess::Smaller, 3)).unwrap();
        let m = BigOrSmall::moments(&s, "a");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("b", "Six-chip swing against {a}"));
        // A small stake is no swing.
        let mut s = new();
        s.dealer[0] = 5;
        BigOrSmall::apply_move(&mut s, "a", play(1, Guess::Bigger, 3)).unwrap();
        BigOrSmall::apply_move(&mut s, "b", play(2, Guess::Smaller, 2)).unwrap();
        assert!(BigOrSmall::moments(&s, "a").is_empty());
        // The last round: b is behind 9 to 11 and wins 3 while a loses 2.
        let mut s = new();
        s.round = 8;
        s.chips = [11, 9];
        s.hands = [vec![2], vec![5]];
        s.dealer[8] = 4;
        BigOrSmall::apply_move(&mut s, "a", play(2, Guess::Smaller, 2)).unwrap();
        BigOrSmall::apply_move(&mut s, "b", play(5, Guess::Smaller, 3)).unwrap();
        assert!(s.over);
        let m = BigOrSmall::moments(&s, "a");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("b", "Lead stolen in the last round (12–9)"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<BigOrSmall>(300, 2, 500);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

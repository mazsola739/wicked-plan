//! Colour Count Bluff (main game, 3–6 seats). Rules page: docs/games/colour-count-bluff.md
//!
//! Each player sees only their own 4 cards. Bet on how many cards of a colour
//! are on the table, raise the bet, or call it: the loser of a call loses a life.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const COLOURS: [&str; 4] = ["red", "blue", "green", "yellow"];
pub const PER_COLOUR: usize = 10;
pub const HAND: usize = 4;
pub const LIVES: i64 = 3;
pub const MAX_ROUNDS: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bet {
    pub seat: usize,
    pub count: usize,
    pub colour: usize,
}

impl Bet {
    fn beats(&self, other: &Bet) -> bool {
        self.count > other.count || (self.count == other.count && self.colour > other.colour)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Showdown {
    pub round: usize,
    pub hands: Vec<Vec<usize>>,
    pub bet: Bet,
    pub caller: usize,
    pub actual: usize,
    pub loser: usize,
}

#[derive(Clone, Debug)]
pub struct BluffState {
    pub players: Vec<String>,
    pub rng: Rng,
    pub lives: Vec<i64>,
    /// Rounds survived (counted when a round ends with the player still in).
    pub survived: Vec<i64>,
    pub hands: Vec<Vec<usize>>,
    pub round: usize,
    pub turn: usize,
    pub bet: Option<Bet>,
    pub log: Vec<String>,
    pub last: Option<Showdown>,
    pub over: bool,
    /// Replay moments: a bet well above the average that holds exactly, a bluff caught by 3 or more.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BluffMove {
    Bet { count: usize, colour: usize },
    Call,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BluffView {
    pub players: Vec<String>,
    pub me: usize,
    pub round: usize,
    pub max_rounds: usize,
    pub hand: Vec<usize>,
    pub lives: Vec<i64>,
    pub cards: Vec<usize>,
    pub total_cards: usize,
    pub bet: Option<Bet>,
    pub turn: Option<usize>,
    pub log: Vec<String>,
    pub last: Option<Showdown>,
    pub over: bool,
}

impl BluffState {
    fn alive(&self, seat: usize) -> bool {
        self.lives[seat] > 0
    }

    fn next_alive(&self, from: usize) -> usize {
        let n = self.players.len();
        (1..=n).map(|i| (from + i) % n).find(|&s| self.alive(s)).unwrap_or(from)
    }

    fn total_cards(&self) -> usize {
        self.hands.iter().map(Vec::len).sum()
    }

    fn deal(&mut self, starter: usize) {
        let mut deck: Vec<usize> = (0..COLOURS.len()).flat_map(|c| std::iter::repeat_n(c, PER_COLOUR)).collect();
        self.rng.shuffle(&mut deck);
        let mut cards = deck.into_iter();
        for seat in 0..self.players.len() {
            self.hands[seat] = if self.alive(seat) {
                let mut h: Vec<usize> = cards.by_ref().take(HAND).collect();
                h.sort();
                h
            } else {
                Vec::new()
            };
        }
        self.turn = if self.alive(starter) { starter } else { self.next_alive(starter) };
        self.bet = None;
        self.log.push(format!("Round {}: {} starts.", self.round + 1, self.players[self.turn]));
    }

    fn do_move(&mut self, seat: usize, mv: BluffMove) -> Result<(), String> {
        match mv {
            BluffMove::Bet { count, colour } => {
                let bet = Bet { seat, count, colour };
                if colour >= COLOURS.len() || count == 0 || count > self.total_cards() {
                    return Err("That bet is not possible".into());
                }
                if self.bet.is_some_and(|b| !bet.beats(&b)) {
                    return Err("Your bet must beat the last one".into());
                }
                self.log.push(format!("{} bets {count} {}.", self.players[seat], COLOURS[colour]));
                self.bet = Some(bet);
                self.turn = self.next_alive(seat);
            }
            BluffMove::Call => {
                let Some(bet) = self.bet else { return Err("There is no bet to call yet".into()) };
                let actual = self.hands.iter().flatten().filter(|&&c| c == bet.colour).count();
                let loser = if actual >= bet.count { seat } else { bet.seat };
                self.log.push(format!(
                    "{} calls: {actual} {} on the table. {} loses a life.",
                    self.players[seat], COLOURS[bet.colour], self.players[loser]
                ));
                self.lives[loser] -= 1;
                // Replay moments (every hand is shown now): a bet 2 or more above the table's average
                // that holds exactly, or a bluff caught 3 or more short.
                let colour = COLOURS[bet.colour];
                if actual == bet.count && bet.count * COLOURS.len() >= self.total_cards() + 2 * COLOURS.len() {
                    let text = format!("Exactly {actual} {colour}: {{{}}}'s call fails", self.players[seat]);
                    self.moments.push(&self.players[bet.seat], text);
                } else if bet.count >= actual + 3 {
                    let text = format!("{{{}}}'s {} {colour} caught: only {actual} there", self.players[bet.seat], bet.count);
                    self.moments.push(&self.players[seat], text);
                }
                self.last = Some(Showdown { round: self.round + 1, hands: self.hands.clone(), bet, caller: seat, actual, loser });
                for s in 0..self.players.len() {
                    if self.alive(s) {
                        self.survived[s] += 1;
                    }
                }
                self.round += 1;
                let left = (0..self.players.len()).filter(|&s| self.alive(s)).count();
                if left <= 1 || self.round >= MAX_ROUNDS {
                    self.over = true;
                    self.log.push("Game over.".into());
                } else {
                    self.deal(loser);
                }
            }
        }
        Ok(())
    }

    fn score(&self, seat: usize) -> i64 {
        100 * self.lives[seat] + self.survived[seat]
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

/// How many of `colour` a seat expects on the table from its own hand.
fn expected(s: &BluffState, seat: usize, colour: usize) -> f64 {
    let mine = s.hands[seat].iter().filter(|&&c| c == colour).count() as f64;
    mine + (s.total_cards() - s.hands[seat].len()) as f64 * 0.25
}

pub struct Rules;

impl Game for Rules {
    type State = BluffState;
    type Move = BluffMove;
    type View = BluffView;

    fn info() -> GameInfo {
        GameInfo {
            id: "colour-count-bluff",
            kind: "main",
            difficulty: "medium",
            tags: &["bluff", "cards"],
            min_players: 3,
            max_players: 6,
            name: "Colour Count Bluff",
            short: "Bet on how many cards of a colour are on the table, seeing only your own; raise or call the bluff.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<BluffState, String> {
        if !(3..=6).contains(&players.len()) {
            return Err("Colour Count Bluff needs 3–6 players".into());
        }
        let n = players.len();
        let mut s = BluffState {
            players: players.to_vec(),
            rng: Rng::new(seed),
            lives: vec![LIVES; n],
            survived: vec![0; n],
            hands: vec![Vec::new(); n],
            round: 0,
            turn: 0,
            bet: None,
            log: Vec::new(),
            last: None,
            over: false,
            moments: Moments::default(),
        };
        s.deal(0);
        Ok(s)
    }

    fn apply_move(s: &mut BluffState, player: &str, mv: BluffMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.players.iter().position(|p| p == player).ok_or("Not in this game")?;
        if seat != s.turn {
            return Err("Not your turn".into());
        }
        s.do_move(seat, mv)
    }

    fn view_for(s: &BluffState, player: &str) -> BluffView {
        let me = s.players.iter().position(|p| p == player).unwrap_or(0);
        BluffView {
            players: s.players.clone(),
            me,
            round: (s.round + 1).min(MAX_ROUNDS),
            max_rounds: MAX_ROUNDS,
            hand: s.hands[me].clone(),
            lives: s.lives.clone(),
            cards: s.hands.iter().map(Vec::len).collect(),
            total_cards: s.total_cards(),
            bet: s.bet,
            turn: (!s.over).then_some(s.turn),
            log: s.log[s.log.len().saturating_sub(10)..].to_vec(),
            last: s.last.clone(),
            over: s.over,
        }
    }

    fn is_over(s: &BluffState) -> bool {
        s.over
    }

    fn results(s: &BluffState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.players.iter().enumerate().map(|(i, p)| (p.clone(), s.score(i))).collect();
        rank_points(&scores)
    }

    fn to_move(s: &BluffState) -> Vec<String> {
        if s.over { Vec::new() } else { vec![s.players[s.turn].clone()] }
    }

    fn bot_move(s: &BluffState, player: &str, seed: u64) -> BluffMove {
        let mut rng = Rng::new(seed);
        let seat = s.players.iter().position(|p| p == player).unwrap_or(0);
        let noise = rng.next_f64() - 0.5;
        if let Some(b) = s.bet
            && b.count as f64 > expected(s, seat, b.colour) + 0.8 + noise
        {
            return BluffMove::Call;
        }
        // The smallest legal raise per colour; take the one with the most room.
        let best = (0..COLOURS.len())
            .map(|colour| {
                let count = match s.bet {
                    None => 1,
                    Some(b) if colour > b.colour => b.count,
                    Some(b) => b.count + 1,
                };
                let room = expected(s, seat, colour) - count as f64 + rng.next_f64() * 0.3;
                (colour, count.max(1), room)
            })
            .filter(|&(_, count, _)| count <= s.total_cards())
            .max_by(|a, b| a.2.total_cmp(&b.2));
        match best {
            Some((colour, count, room)) if s.bet.is_none() || room > -1.0 => BluffMove::Bet { count, colour },
            Some(_) | None if s.bet.is_some() => BluffMove::Call,
            _ => BluffMove::Bet { count: 1, colour: 0 },
        }
    }

    fn moments(s: &BluffState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn ids(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn deals_four_cards_each() {
        let s = Rules::setup(&ids(5), 1).unwrap();
        assert!(s.hands.iter().all(|h| h.len() == HAND));
        assert_eq!(s.turn, 0);
    }

    #[test]
    fn bets_must_go_up() {
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        assert!(Rules::apply_move(&mut s, "p1", BluffMove::Call).is_err(), "nothing to call");
        Rules::apply_move(&mut s, "p1", BluffMove::Bet { count: 2, colour: 1 }).unwrap();
        assert!(Rules::apply_move(&mut s, "p2", BluffMove::Bet { count: 2, colour: 0 }).is_err());
        assert!(Rules::apply_move(&mut s, "p2", BluffMove::Bet { count: 2, colour: 1 }).is_err());
        assert!(Rules::apply_move(&mut s, "p2", BluffMove::Bet { count: 13, colour: 3 }).is_err(), "only 12 cards");
        Rules::apply_move(&mut s, "p2", BluffMove::Bet { count: 2, colour: 2 }).unwrap();
        Rules::apply_move(&mut s, "p3", BluffMove::Bet { count: 3, colour: 0 }).unwrap();
        assert_eq!(s.turn, 0);
    }

    #[test]
    fn call_costs_the_wrong_side_a_life() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        s.hands = vec![vec![0, 0, 1, 2], vec![0, 3, 3, 3], vec![1, 1, 2, 2]];
        Rules::apply_move(&mut s, "p1", BluffMove::Bet { count: 3, colour: 0 }).unwrap();
        Rules::apply_move(&mut s, "p2", BluffMove::Call).unwrap();
        assert_eq!(s.lives, vec![3, 2, 3], "3 reds are there: the caller loses");
        assert_eq!(s.turn, 1, "the loser starts");
        s.hands = vec![vec![0, 0, 1, 2], vec![0, 3, 3, 3], vec![1, 1, 2, 2]];
        Rules::apply_move(&mut s, "p2", BluffMove::Bet { count: 4, colour: 3 }).unwrap();
        Rules::apply_move(&mut s, "p3", BluffMove::Call).unwrap();
        assert_eq!(s.lives, vec![3, 1, 3]);
        assert_eq!(s.last.as_ref().unwrap().actual, 3);
    }

    #[test]
    fn last_one_standing_wins() {
        let mut s = Rules::setup(&ids(3), 4).unwrap();
        s.lives = vec![1, 1, 0];
        s.deal(0);
        assert!(s.hands[2].is_empty());
        s.hands[0] = vec![1, 1, 1, 1];
        s.hands[1] = vec![1, 1, 1, 1];
        Rules::apply_move(&mut s, "p1", BluffMove::Bet { count: 1, colour: 0 }).unwrap();
        assert_eq!(s.turn, 1, "skips the player who is out");
        Rules::apply_move(&mut s, "p2", BluffMove::Call).unwrap();
        assert!(s.over);
        let r = Rules::results(&s);
        assert_eq!(r[1].rank, 1);
        assert_eq!(r[1].points, 1000);
    }

    #[test]
    fn no_timer_the_turn_waits_for_the_player() {
        let mut s = Rules::setup(&ids(4), 5).unwrap();
        Rules::tick(&mut s, 600_000);
        assert!(s.bet.is_none());
        assert_eq!((s.round, s.turn), (0, 0));
        Rules::apply_move(&mut s, "p1", BluffMove::Bet { count: 1, colour: 0 }).unwrap();
        Rules::tick(&mut s, 600_000);
        assert_eq!((s.round, s.turn), (0, 1));
    }

    #[test]
    fn view_shows_only_own_cards() {
        let s = Rules::setup(&ids(4), 6).unwrap();
        let v = serde_json::to_value(Rules::view_for(&s, "p3")).unwrap();
        assert_eq!(v["hand"].as_array().unwrap().len(), 4);
        assert!(v.get("hands").is_none());
    }

    #[test]
    fn an_exact_bet_and_a_caught_bluff_are_replay_moments() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        let hands = vec![vec![0, 0, 1, 2], vec![0, 3, 0, 3], vec![1, 1, 0, 2]];
        // 5 reds of 12 cards: 2 above the average of 3, and exactly right.
        s.hands = hands.clone();
        Rules::apply_move(&mut s, "p1", BluffMove::Bet { count: 5, colour: 0 }).unwrap();
        Rules::apply_move(&mut s, "p2", BluffMove::Call).unwrap();
        let m = Rules::moments(&s, "p3");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Exactly 5 red: {p2}'s call fails"));
        // p2 starts and bets 5 green with 2 there: p3 calls it.
        s.hands = hands.clone();
        Rules::apply_move(&mut s, "p2", BluffMove::Bet { count: 5, colour: 2 }).unwrap();
        Rules::apply_move(&mut s, "p3", BluffMove::Call).unwrap();
        let m = Rules::moments(&s, "p1");
        assert_eq!(m.len(), 2);
        assert_eq!((m[1].by.as_str(), m[1].text.as_str()), ("p3", "{p2}'s 5 green caught: only 2 there"));
        // A bet 1 above the average that holds exactly, or a bluff 2 short, is no moment.
        s.hands = vec![vec![0, 0, 1, 2], vec![0, 3, 1, 3], vec![1, 1, 0, 2]];
        Rules::apply_move(&mut s, "p2", BluffMove::Bet { count: 4, colour: 0 }).unwrap();
        Rules::apply_move(&mut s, "p3", BluffMove::Call).unwrap();
        s.hands = hands;
        Rules::apply_move(&mut s, "p3", BluffMove::Bet { count: 4, colour: 2 }).unwrap();
        Rules::apply_move(&mut s, "p1", BluffMove::Call).unwrap();
        assert_eq!(s.round, 4);
        assert_eq!(Rules::moments(&s, "p1").len(), 2);
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 4000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }

    #[test]
    fn thousand_bot_games() {
        for n in [3, 6] {
            bot_games_with::<Rules>(500, n, 4000);
        }
    }
}

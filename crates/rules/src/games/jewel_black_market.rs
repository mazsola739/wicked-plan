//! Jewel Black Market (main game, 4–8 seats). Rules page: docs/games/jewel-black-market.md
//!
//! Every seat holds one hidden card (jewel or bomb), the buyer holds 3, the
//! discard 1. Two swap rounds (forward, then reverse); a jewel scores its value
//! times the number of same-type jewels in the buyer's space, a bomb costs.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 3;
pub const TURN_MS: i64 = 20_000;
pub const BUYER: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Diamond,
    Gold,
    Silver,
    Bronze,
    Bomb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Card {
    pub kind: Kind,
    pub value: i64,
}

impl Card {
    fn name(&self) -> String {
        let kind = match self.kind {
            Kind::Diamond => "Diamond",
            Kind::Gold => "Gold",
            Kind::Silver => "Silver",
            Kind::Bronze => "Bronze",
            Kind::Bomb => "Bomb",
        };
        format!("{kind} {}", self.value)
    }
}

pub fn deck() -> Vec<Card> {
    use Kind::*;
    [(Diamond, 5), (Diamond, 6), (Gold, 4), (Gold, 5), (Silver, 2), (Silver, 3), (Silver, 4), (Bronze, 1), (Bronze, 1), (Bronze, 2), (Bronze, 2), (Bomb, -6), (Bomb, -4)]
        .into_iter()
        .map(|(kind, value)| Card { kind, value })
        .collect()
}

/// What a card scores against this buyer's space.
pub fn card_points(card: Card, buyer: &[Card]) -> i64 {
    if card.kind == Kind::Bomb { card.value } else { card.value * buyer.iter().filter(|b| b.kind == card.kind).count() as i64 }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reveal {
    pub round: usize,
    pub cards: Vec<Card>,
    pub buyer: Vec<Card>,
    pub discard: Card,
    pub points: Vec<i64>,
}

#[derive(Clone, Debug)]
pub struct JewelState {
    pub players: Vec<String>,
    pub rng: Rng,
    pub round: usize,
    pub hands: Vec<Card>,
    pub buyer: Vec<Card>,
    pub discard: Card,
    /// Seats in turn order for this round: forward, then reverse.
    pub queue: Vec<usize>,
    pub pos: usize,
    pub time_left_ms: i64,
    pub scores: Vec<i64>,
    /// Per seat: the public log of this round plus what that seat saw.
    pub logs: Vec<Vec<String>>,
    pub last: Option<Reveal>,
    pub over: bool,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum JewelMove {
    Keep,
    SwapPlayer { seat: usize },
    SwapBuyer { slot: usize },
    SwapDiscard,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JewelView {
    pub players: Vec<String>,
    pub me: usize,
    pub round: usize,
    pub rounds: usize,
    /// 1 = forward swap round, 2 = reverse.
    pub swap_round: usize,
    pub turn: Option<usize>,
    pub my_card: Card,
    pub time_left_ms: i64,
    pub turn_ms: i64,
    pub scores: Vec<i64>,
    pub log: Vec<String>,
    pub last: Option<Reveal>,
    pub over: bool,
}

impl JewelState {
    fn deal(&mut self) {
        let mut cards = deck();
        self.rng.shuffle(&mut cards);
        let n = self.players.len();
        self.hands = cards[..n].to_vec();
        self.buyer = cards[n..n + BUYER].to_vec();
        self.discard = cards[n + BUYER];
        let first = self.round % n;
        let forward: Vec<usize> = (0..n).map(|i| (first + i) % n).collect();
        self.queue = forward.iter().chain(forward.iter().rev()).copied().collect();
        self.pos = 0;
        self.time_left_ms = TURN_MS;
        self.logs = vec![vec![format!("Round {} dealt.", self.round + 1)]; n];
    }

    fn name(&self, seat: usize) -> &str {
        &self.players[seat]
    }

    fn public(&mut self, line: String) {
        for log in &mut self.logs {
            log.push(line.clone());
        }
    }

    fn turn(&self) -> Option<usize> {
        (!self.over).then(|| self.queue[self.pos])
    }

    fn do_move(&mut self, seat: usize, mv: JewelMove) -> Result<(), String> {
        let bomb = self.hands[seat].kind == Kind::Bomb;
        let who = self.name(seat).to_string();
        match mv {
            JewelMove::Keep if bomb => return Err("A bomb must be passed to a player".into()),
            JewelMove::Keep => self.public(format!("{who} kept their card.")),
            JewelMove::SwapPlayer { seat: other } => {
                if other >= self.players.len() || other == seat {
                    return Err("Pick another player".into());
                }
                self.hands.swap(seat, other);
                let them = self.name(other).to_string();
                self.public(format!("{who} swapped with {them}."));
                let (mine, theirs) = (self.hands[seat].name(), self.hands[other].name());
                self.logs[seat].push(format!("You got {mine} from {them}."));
                self.logs[other].push(format!("You got {theirs} from {who}."));
            }
            JewelMove::SwapBuyer { .. } | JewelMove::SwapDiscard if bomb => return Err("A bomb must be passed to a player".into()),
            JewelMove::SwapBuyer { slot } => {
                if slot >= BUYER {
                    return Err("No such buyer card".into());
                }
                std::mem::swap(&mut self.hands[seat], &mut self.buyer[slot]);
                self.public(format!("{who} swapped with buyer card {}.", slot + 1));
                let got = self.hands[seat].name();
                self.logs[seat].push(format!("You got {got} from the buyer."));
            }
            JewelMove::SwapDiscard => {
                std::mem::swap(&mut self.hands[seat], &mut self.discard);
                self.public(format!("{who} swapped with the discard."));
                let got = self.hands[seat].name();
                self.logs[seat].push(format!("You got {got} from the discard."));
            }
        }
        self.advance();
        Ok(())
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.time_left_ms = TURN_MS;
        if self.pos < self.queue.len() {
            return;
        }
        let points: Vec<i64> = self.hands.iter().map(|&c| card_points(c, &self.buyer)).collect();
        for (s, p) in self.scores.iter_mut().zip(&points) {
            *s += p;
        }
        self.last = Some(Reveal { round: self.round + 1, cards: self.hands.clone(), buyer: self.buyer.clone(), discard: self.discard, points });
        self.round += 1;
        if self.round >= ROUNDS {
            self.over = true;
            self.time_left_ms = 0;
        } else {
            self.deal();
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
    type State = JewelState;
    type Move = JewelMove;
    type View = JewelView;

    fn info() -> GameInfo {
        GameInfo {
            id: "jewel-black-market",
            kind: "main",
            difficulty: "medium",
            min_players: 4,
            max_players: 8,
            name: "Jewel Black Market",
            short: "One hidden card each; swap with players, the buyer or the discard. Match the buyer's jewels, dodge the bombs.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<JewelState, String> {
        if !(4..=8).contains(&players.len()) {
            return Err("Jewel Black Market needs 4–8 players".into());
        }
        let n = players.len();
        let mut s = JewelState {
            players: players.to_vec(),
            rng: Rng::new(seed),
            round: 0,
            hands: Vec::new(),
            buyer: Vec::new(),
            discard: deck()[0],
            queue: Vec::new(),
            pos: 0,
            time_left_ms: TURN_MS,
            scores: vec![0; n],
            logs: Vec::new(),
            last: None,
            over: false,
        };
        s.deal();
        Ok(s)
    }

    fn apply_move(s: &mut JewelState, player: &str, mv: JewelMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.players.iter().position(|p| p == player).ok_or("Not in this game")?;
        if s.turn() != Some(seat) {
            return Err("Not your turn".into());
        }
        s.do_move(seat, mv)
    }

    fn tick(s: &mut JewelState, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms > 0 {
            return;
        }
        let seat = s.queue[s.pos];
        let mv = if s.hands[seat].kind == Kind::Bomb {
            JewelMove::SwapPlayer { seat: (seat + 1) % s.players.len() }
        } else {
            JewelMove::Keep
        };
        let _ = s.do_move(seat, mv);
    }

    fn view_for(s: &JewelState, player: &str) -> JewelView {
        let me = s.players.iter().position(|p| p == player).unwrap_or(0);
        let log = &s.logs[me];
        JewelView {
            players: s.players.clone(),
            me,
            round: (s.round + 1).min(ROUNDS),
            rounds: ROUNDS,
            swap_round: if s.pos < s.players.len() { 1 } else { 2 },
            turn: s.turn(),
            my_card: s.hands[me],
            time_left_ms: s.time_left_ms,
            turn_ms: TURN_MS,
            scores: s.scores.clone(),
            log: log[log.len().saturating_sub(12)..].to_vec(),
            last: s.last.clone(),
            over: s.over,
        }
    }

    fn is_over(s: &JewelState) -> bool {
        s.over
    }

    fn results(s: &JewelState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.players.iter().cloned().zip(s.scores.iter().copied()).collect();
        rank_points(&scores)
    }

    fn to_move(s: &JewelState) -> Vec<String> {
        s.turn().map(|t| vec![s.players[t].clone()]).unwrap_or_default()
    }

    fn bot_move(s: &JewelState, player: &str, seed: u64) -> JewelMove {
        let mut rng = Rng::new(seed);
        let n = s.players.len();
        let seat = s.players.iter().position(|p| p == player).unwrap_or(0);
        let mut other = rng.index(n - 1);
        if other >= seat {
            other += 1;
        }
        let card = s.hands[seat];
        if card.kind == Kind::Bomb {
            return JewelMove::SwapPlayer { seat: other };
        }
        let keep = if card.value >= 4 { 0.75 } else { 0.25 };
        let r = rng.next_f64();
        if r < keep {
            JewelMove::Keep
        } else if r < keep + (1.0 - keep) * 0.5 {
            JewelMove::SwapBuyer { slot: rng.index(BUYER) }
        } else if r < keep + (1.0 - keep) * 0.8 {
            JewelMove::SwapPlayer { seat: other }
        } else {
            JewelMove::SwapDiscard
        }
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

    fn card(kind: Kind, value: i64) -> Card {
        Card { kind, value }
    }

    #[test]
    fn deck_has_thirteen_cards() {
        let d = deck();
        assert_eq!(d.len(), 13);
        assert_eq!(d.iter().filter(|c| c.kind == Kind::Bomb).count(), 2);
        assert_eq!(d.iter().filter(|c| c.kind == Kind::Bronze).count(), 4);
    }

    #[test]
    fn jewel_scores_value_times_matches() {
        let buyer = [card(Kind::Gold, 4), card(Kind::Gold, 5), card(Kind::Silver, 2)];
        assert_eq!(card_points(card(Kind::Gold, 5), &buyer), 10);
        assert_eq!(card_points(card(Kind::Silver, 3), &buyer), 3);
        assert_eq!(card_points(card(Kind::Diamond, 6), &buyer), 0);
        assert_eq!(card_points(card(Kind::Bomb, -6), &buyer), -6);
    }

    #[test]
    fn turns_go_forward_then_back() {
        let s = Rules::setup(&ids(4), 1).unwrap();
        assert_eq!(s.queue, vec![0, 1, 2, 3, 3, 2, 1, 0]);
    }

    #[test]
    fn bomb_must_go_to_a_player() {
        let mut s = Rules::setup(&ids(4), 2).unwrap();
        s.hands[0] = card(Kind::Bomb, -4);
        assert!(Rules::apply_move(&mut s, "p1", JewelMove::Keep).is_err());
        assert!(Rules::apply_move(&mut s, "p1", JewelMove::SwapBuyer { slot: 0 }).is_err());
        assert!(Rules::apply_move(&mut s, "p1", JewelMove::SwapDiscard).is_err());
        assert!(Rules::apply_move(&mut s, "p2", JewelMove::Keep).is_err(), "not p2's turn");
        Rules::apply_move(&mut s, "p1", JewelMove::SwapPlayer { seat: 2 }).unwrap();
        assert_eq!(s.hands[2], card(Kind::Bomb, -4));
        assert!(Rules::view_for(&s, "p3").log.iter().any(|l| l.contains("Bomb -4")));
        assert!(!Rules::view_for(&s, "p2").log.iter().any(|l| l.contains("Bomb")));
    }

    #[test]
    fn timeout_keeps_or_passes_the_bomb() {
        let mut s = Rules::setup(&ids(4), 3).unwrap();
        s.hands[0] = card(Kind::Bomb, -6);
        Rules::tick(&mut s, TURN_MS as u64);
        assert_eq!(s.hands[1], card(Kind::Bomb, -6));
        assert_eq!(s.pos, 1);
    }

    #[test]
    fn round_scores_and_reveals() {
        let mut s = Rules::setup(&ids(4), 4).unwrap();
        s.hands = vec![card(Kind::Gold, 5), card(Kind::Bomb, -6), card(Kind::Silver, 2), card(Kind::Bronze, 1)];
        s.buyer = vec![card(Kind::Gold, 4), card(Kind::Silver, 3), card(Kind::Silver, 4)];
        s.pos = s.queue.len() - 1;
        s.advance();
        assert_eq!(s.scores, vec![5, -6, 4, 0]);
        assert_eq!(s.last.as_ref().unwrap().round, 1);
        assert_eq!(s.round, 1);
        assert_eq!(s.queue[0], 1, "the start moves on");
    }

    #[test]
    fn view_hides_other_cards() {
        let s = Rules::setup(&ids(5), 5).unwrap();
        let json = serde_json::to_value(Rules::view_for(&s, "p2")).unwrap();
        assert!(json.get("hands").is_none());
        assert!(json.get("buyer").is_none());
        assert_eq!(json["me"], 1);
    }

    #[test]
    fn thousand_bot_games() {
        for n in [4, 8] {
            bot_games_with::<Rules>(500, n, 2000);
        }
    }
}

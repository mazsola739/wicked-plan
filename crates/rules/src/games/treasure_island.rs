//! Treasure Island (Main game, 3–6 players). Rules page: docs/games/treasure-island.md
//!
//! A treasure on a 9×9 island. Clue boxes show the direction to it. Everyone
//! has one private box; five more are auctioned with secret bids. Then everyone
//! digs once: closeness plus the coins kept is the score.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const SIZE: i32 = 9;
pub const START_COINS: i64 = 100;
pub const AUCTIONS: usize = 5;
pub const DIG_POINTS: [i64; 4] = [500, 300, 150, 50];

pub type Square = (i32, i32);

/// The arrow from a box towards the treasure: the sign of the step in x and y.
pub fn arrow(from: Square, to: Square) -> (i32, i32) {
    ((to.0 - from.0).signum(), (to.1 - from.1).signum())
}

pub fn distance(a: Square, b: Square) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

pub fn dig_points(dig: Square, treasure: Square) -> i64 {
    DIG_POINTS.get(distance(dig, treasure) as usize).copied().unwrap_or(0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Auction,
    Dig,
    Over,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Auction {
    pub bids: Vec<i64>,
    pub winners: Vec<usize>,
    pub price: i64,
}

#[derive(Clone, Debug)]
pub struct State {
    pub players: Vec<String>,
    pub treasure: Square,
    pub starters: Vec<Square>,
    pub boxes: Vec<Square>,
    pub coins: Vec<i64>,
    pub phase: Phase,
    pub round: usize,
    pub bids: Vec<Option<i64>>,
    pub history: Vec<Auction>,
    pub digs: Vec<Option<Square>>,
    /// Replay moments: a dig right on the treasure with only the private clue, or as the only one.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Bid { amount: i64 },
    Dig { x: i32, y: i32 },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoxView {
    pub x: i32,
    pub y: i32,
    /// The auction it is sold in; None for your private box.
    pub auction: Option<usize>,
    /// Only when you opened it (or at the end).
    pub arrow: Option<(i32, i32)>,
    pub opened_by: Vec<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: Vec<String>,
    pub you: Option<usize>,
    pub size: i32,
    pub phase: Phase,
    pub round: usize,
    pub auctions: usize,
    pub boxes: Vec<BoxView>,
    /// Squares that still fit your arrows.
    pub possible: Vec<Square>,
    pub coins: Vec<i64>,
    pub bid_in: Vec<bool>,
    pub your_bid: Option<i64>,
    pub history: Vec<Auction>,
    pub your_dig: Option<Square>,
    pub dug: Vec<bool>,
    /// Only at the end.
    pub treasure: Option<Square>,
    pub digs: Option<Vec<Option<Square>>>,
    pub scores: Option<Vec<i64>>,
}

impl State {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }

    /// The boxes `seat` has opened: its own, plus the auctions it won.
    pub fn known(&self, seat: usize) -> Vec<Square> {
        let mut out = vec![self.starters[seat]];
        for (i, a) in self.history.iter().enumerate() {
            if a.winners.contains(&seat) {
                out.push(self.boxes[i]);
            }
        }
        out
    }

    /// Squares that fit every arrow `seat` has seen, and are not under a box it knows of.
    pub fn possible(&self, seat: usize) -> Vec<Square> {
        let known = self.known(seat);
        let arrows: Vec<(Square, (i32, i32))> = known.iter().map(|&b| (b, arrow(b, self.treasure))).collect();
        let mut out = Vec::new();
        for y in 0..SIZE {
            for x in 0..SIZE {
                let sq = (x, y);
                if self.boxes.contains(&sq) || sq == self.starters[seat] {
                    continue;
                }
                if arrows.iter().all(|&(b, a)| arrow(b, sq) == a) {
                    out.push(sq);
                }
            }
        }
        out
    }

    pub fn scores(&self) -> Vec<i64> {
        (0..self.players.len()).map(|k| self.coins[k] + self.digs[k].map_or(0, |d| dig_points(d, self.treasure))).collect()
    }

    fn settle_auction(&mut self) {
        let bids: Vec<i64> = self.bids.iter().map(|b| b.unwrap_or(0)).collect();
        let price = *bids.iter().max().unwrap_or(&0);
        let winners: Vec<usize> = if price > 0 { (0..bids.len()).filter(|&k| bids[k] == price).collect() } else { Vec::new() };
        for &w in &winners {
            self.coins[w] -= price;
        }
        self.history.push(Auction { bids, winners, price });
        self.round += 1;
        self.bids = vec![None; self.players.len()];
        if self.round >= AUCTIONS {
            self.phase = Phase::Dig;
        }
    }

    fn finish(&mut self) {
        self.phase = Phase::Over;
        // Replay moments: everything is shown at the end, so they are public.
        let exact: Vec<usize> = (0..self.players.len()).filter(|&k| self.digs[k] == Some(self.treasure)).collect();
        for &k in &exact {
            let text = if self.known(k).len() == 1 {
                "Treasure dug up with just the private clue (+500)"
            } else if exact.len() == 1 {
                "The only dig right on the treasure (+500)"
            } else {
                continue;
            };
            self.moments.push(&self.players[k].clone(), text);
        }
    }
}

/// The square with the highest expected dig points over `possible`, and those points summed.
fn best_square(possible: &[Square]) -> (Square, i64) {
    let mut best = ((SIZE / 2, SIZE / 2), -1);
    // The best square is never outside the box around the possible squares.
    let xs = || possible.iter().map(|p| p.0);
    let ys = || possible.iter().map(|p| p.1);
    let (x0, x1) = (xs().min().unwrap_or(0), xs().max().unwrap_or(SIZE - 1));
    let (y0, y1) = (ys().min().unwrap_or(0), ys().max().unwrap_or(SIZE - 1));
    for y in y0..=y1 {
        for x in x0..=x1 {
            let v: i64 = possible.iter().map(|&t| dig_points((x, y), t)).sum();
            if v > best.1 {
                best = ((x, y), v);
            }
        }
    }
    best
}

pub fn best_dig(possible: &[Square]) -> Square {
    best_square(possible).0
}

/// How many dig points opening a box at `b` is expected to add, for a player who
/// still thinks the treasure is on one of `possible`.
pub fn clue_value(possible: &[Square], b: Square) -> f64 {
    if possible.is_empty() {
        return 0.0;
    }
    let mut groups: Vec<((i32, i32), Vec<Square>)> = Vec::new();
    for &sq in possible {
        let a = arrow(b, sq);
        match groups.iter_mut().find(|(g, _)| *g == a) {
            Some((_, list)) => list.push(sq),
            None => groups.push((a, vec![sq])),
        }
    }
    let n = possible.len() as f64;
    let after: i64 = groups.iter().map(|(_, g)| best_square(g).1).sum();
    (after - best_square(possible).1) as f64 / n
}

pub struct TreasureIsland;

impl Game for TreasureIsland {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "treasure-island",
            kind: "main",
            difficulty: "hard",
            tags: &["bidding", "logic"],
            min_players: 3,
            max_players: 6,
            name: "Treasure Island",
            short: "Bid in secret for clue boxes whose arrows point to the treasure, then dig once.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if !(3..=6).contains(&players.len()) {
            return Err("Treasure Island needs 3–6 players".into());
        }
        let mut rng = Rng::new(seed);
        let mut squares: Vec<Square> = (0..SIZE * SIZE).map(|i| (i % SIZE, i / SIZE)).collect();
        rng.shuffle(&mut squares);
        let treasure = squares[0];
        let boxes = squares[1..=AUCTIONS].to_vec();
        let starters = squares[AUCTIONS + 1..AUCTIONS + 1 + players.len()].to_vec();
        let n = players.len();
        Ok(State {
            players: players.to_vec(),
            treasure,
            starters,
            boxes,
            coins: vec![START_COINS; n],
            phase: Phase::Auction,
            round: 0,
            bids: vec![None; n],
            history: Vec::new(),
            digs: vec![None; n],
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        let seat = s.seat(player).ok_or("Not in this game")?;
        match (s.phase, mv) {
            (Phase::Auction, Move::Bid { amount }) => {
                if s.bids[seat].is_some() {
                    return Err("You already bid".into());
                }
                if amount < 0 || amount > s.coins[seat] {
                    return Err("You don't have that many coins".into());
                }
                s.bids[seat] = Some(amount);
                if s.bids.iter().all(Option::is_some) {
                    s.settle_auction();
                }
            }
            (Phase::Dig, Move::Dig { x, y }) => {
                if s.digs[seat].is_some() {
                    return Err("You already chose".into());
                }
                if !(0..SIZE).contains(&x) || !(0..SIZE).contains(&y) {
                    return Err("That is not on the island".into());
                }
                s.digs[seat] = Some((x, y));
                if s.digs.iter().all(Option::is_some) {
                    s.finish();
                }
            }
            (Phase::Over, _) => return Err("Game is over".into()),
            _ => return Err("Not now".into()),
        }
        Ok(())
    }

    fn view_for(s: &State, player: &str) -> View {
        let you = s.seat(player);
        let over = s.phase == Phase::Over;
        let mut boxes = Vec::new();
        if let Some(k) = you {
            let b = s.starters[k];
            boxes.push(BoxView { x: b.0, y: b.1, auction: None, arrow: Some(arrow(b, s.treasure)), opened_by: vec![k] });
        }
        for (i, &b) in s.boxes.iter().enumerate() {
            let opened_by = s.history.get(i).map(|a| a.winners.clone()).unwrap_or_default();
            let seen = over || you.is_some_and(|k| opened_by.contains(&k));
            boxes.push(BoxView { x: b.0, y: b.1, auction: Some(i), arrow: seen.then(|| arrow(b, s.treasure)), opened_by });
        }
        View {
            players: s.players.clone(),
            you,
            size: SIZE,
            phase: s.phase,
            round: (s.round + 1).min(AUCTIONS),
            auctions: AUCTIONS,
            boxes,
            possible: you.map(|k| s.possible(k)).unwrap_or_default(),
            coins: s.coins.clone(),
            bid_in: s.bids.iter().map(Option::is_some).collect(),
            your_bid: you.and_then(|k| s.bids[k]),
            history: s.history.clone(),
            your_dig: you.and_then(|k| s.digs[k]),
            dug: s.digs.iter().map(Option::is_some).collect(),
            treasure: over.then_some(s.treasure),
            digs: over.then(|| s.digs.clone()),
            scores: over.then(|| s.scores()),
        }
    }

    fn is_over(s: &State) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        let scores = s.scores();
        let named: Vec<(String, i64)> = s.players.iter().cloned().zip(scores).collect();
        rank_by_score(&named)
    }

    fn to_move(s: &State) -> Vec<String> {
        let waiting: Vec<bool> = match s.phase {
            Phase::Auction => s.bids.iter().map(Option::is_none).collect(),
            Phase::Dig => s.digs.iter().map(Option::is_none).collect(),
            Phase::Over => return Vec::new(),
        };
        s.players.iter().zip(waiting).filter(|(_, w)| *w).map(|(p, _)| p.clone()).collect()
    }

    fn bot_move(s: &State, player: &str, seed: u64) -> Move {
        let seat = s.seat(player).unwrap_or(0);
        let possible = s.possible(seat);
        if s.phase != Phase::Auction {
            let (x, y) = best_dig(&possible);
            return Move::Dig { x, y };
        }
        let mut rng = Rng::new(seed);
        let b = s.boxes[s.round.min(AUCTIONS - 1)];
        // Bid part of what the clue is worth in dig points (paying it all would gain nothing),
        // and keep coins for the auctions still to come.
        let left = (AUCTIONS - s.round.min(AUCTIONS - 1)) as f64;
        let cap = s.coins[seat] as f64 * 1.5 / left;
        let amount = (clue_value(&possible, b) * (0.25 + 0.25 * rng.next_f64())).min(cap).round() as i64;
        Move::Bid { amount: amount.clamp(0, s.coins[seat]) }
    }

    fn moments(s: &State, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub const READY: bool = true;
pub type Rules = TreasureIsland;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn arrows_point_at_the_treasure() {
        assert_eq!(arrow((0, 0), (5, 5)), (1, 1));
        assert_eq!(arrow((4, 4), (4, 0)), (0, -1));
        assert_eq!(arrow((8, 2), (1, 2)), (-1, 0));
        assert_eq!(dig_points((3, 3), (3, 3)), 500);
        assert_eq!(dig_points((3, 3), (4, 4)), 300);
        assert_eq!(dig_points((0, 0), (3, 1)), 50);
        assert_eq!(dig_points((0, 0), (8, 8)), 0);
    }

    #[test]
    fn the_treasure_always_fits_and_boxes_are_apart() {
        for seed in 0..100 {
            let s = TreasureIsland::setup(&players(6), seed).unwrap();
            let mut all = s.boxes.clone();
            all.extend(&s.starters);
            all.push(s.treasure);
            let mut sorted = all.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), all.len());
            for k in 0..6 {
                assert!(s.possible(k).contains(&s.treasure));
            }
        }
    }

    #[test]
    fn highest_bid_pays_and_ties_share() {
        let mut s = TreasureIsland::setup(&players(3), 1).unwrap();
        TreasureIsland::apply_move(&mut s, "p1", Move::Bid { amount: 30 }).unwrap();
        assert!(TreasureIsland::apply_move(&mut s, "p1", Move::Bid { amount: 5 }).is_err());
        assert!(TreasureIsland::apply_move(&mut s, "p2", Move::Bid { amount: 101 }).is_err());
        TreasureIsland::apply_move(&mut s, "p2", Move::Bid { amount: 30 }).unwrap();
        TreasureIsland::apply_move(&mut s, "p3", Move::Bid { amount: 10 }).unwrap();
        assert_eq!(s.history[0].winners, vec![0, 1]);
        assert_eq!(s.coins, vec![70, 70, 100]);
        let v1 = TreasureIsland::view_for(&s, "p1");
        let v3 = TreasureIsland::view_for(&s, "p3");
        assert!(v1.boxes[1].arrow.is_some());
        assert!(v3.boxes[1].arrow.is_none());
        assert!(v3.treasure.is_none());
        // Only your own private box is in your view.
        assert_eq!(v3.boxes.iter().filter(|b| b.auction.is_none()).count(), 1);
        assert_eq!((v3.boxes[0].x, v3.boxes[0].y), s.starters[2]);
        // The auction waits for every bid, however long it takes.
        TreasureIsland::tick(&mut s, 600_000);
        assert_eq!(s.round, 1);
        // Everyone bids 0: the box stays shut.
        for p in players(3) {
            TreasureIsland::apply_move(&mut s, &p, Move::Bid { amount: 0 }).unwrap();
        }
        assert!(s.history[1].winners.is_empty());
        assert_eq!(s.round, 2);
    }

    #[test]
    fn dig_scores_closeness_plus_coins() {
        let mut s = TreasureIsland::setup(&players(3), 7).unwrap();
        for _ in 0..AUCTIONS {
            for p in players(3) {
                TreasureIsland::apply_move(&mut s, &p, Move::Bid { amount: 0 }).unwrap();
            }
        }
        assert_eq!(s.phase, Phase::Dig);
        let (tx, ty) = s.treasure;
        TreasureIsland::apply_move(&mut s, "p1", Move::Dig { x: tx, y: ty }).unwrap();
        for p in ["p2", "p3"] {
            TreasureIsland::apply_move(&mut s, p, Move::Dig { x: (tx + 4) % SIZE, y: ty }).unwrap();
        }
        assert_eq!(s.phase, Phase::Over);
        let r = TreasureIsland::results(&s);
        assert_eq!(r[0].points, 600);
        assert_eq!(r[0].rank, 1);
    }

    #[test]
    fn bots_dig_close_to_the_treasure() {
        let mut total = 0;
        for seed in 0..50 {
            let mut s = TreasureIsland::setup(&players(4), seed).unwrap();
            while s.phase != Phase::Over {
                let p = TreasureIsland::to_move(&s)[0].clone();
                let mv = TreasureIsland::bot_move(&s, &p, seed);
                TreasureIsland::apply_move(&mut s, &p, mv).unwrap();
            }
            total += s.digs.iter().map(|d| dig_points(d.unwrap(), s.treasure)).sum::<i64>();
        }
        // A random dig averages well under 100 points.
        assert!(total / 200 >= 150, "average dig {}", total / 200);
    }

    #[test]
    fn thousand_bot_games() {
        for seats in 3..=6 {
            bot_games_with::<TreasureIsland>(1000, seats, 3000);
        }
    }

    /// Every auction ends with `bids`; then the seats dig at `digs`.
    fn play(bids: [[i64; 3]; AUCTIONS], digs: impl Fn(&State) -> [Square; 3]) -> State {
        let mut s = TreasureIsland::setup(&players(3), 3).unwrap();
        for round in bids {
            for (k, amount) in round.into_iter().enumerate() {
                TreasureIsland::apply_move(&mut s, &format!("p{}", k + 1), Move::Bid { amount }).unwrap();
            }
        }
        for (k, (x, y)) in digs(&s).into_iter().enumerate() {
            TreasureIsland::apply_move(&mut s, &format!("p{}", k + 1), Move::Dig { x, y }).unwrap();
        }
        assert_eq!(s.phase, Phase::Over);
        s
    }

    #[test]
    fn digging_right_on_the_treasure_can_be_a_replay_moment() {
        let away = |s: &State| ((s.treasure.0 + 4) % SIZE, s.treasure.1);
        // p1 wins every box and is the only one on the treasure.
        let s = play([[5, 0, 0]; AUCTIONS], |s| [s.treasure, away(s), away(s)]);
        let m = TreasureIsland::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "The only dig right on the treasure (+500)"));
        // p1 and p3 both hit it, so only p3, who never won a box, gets a moment.
        let s = play([[5, 0, 0]; AUCTIONS], |s| [s.treasure, away(s), s.treasure]);
        let m = TreasureIsland::moments(&s, "p1");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p3", "Treasure dug up with just the private clue (+500)"));
        // Two hits with boxes bought: no moment.
        let s = play([[5, 5, 0]; AUCTIONS], |s| [s.treasure, s.treasure, away(s)]);
        assert!(s.moments.is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<TreasureIsland>(300, 4, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

//! Animal Tile Auction (main game, 3–6 seats). Rules page: docs/games/animal-tile-auction.md
//!
//! A shared 5×5 grid; each auction offers one animal tile. Secret bids, the
//! single highest bidder pays and places it. Everyone scores their own secret
//! three-animal pattern (any rotation) on the final grid.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const SIZE: i32 = 5;
pub const CELLS: usize = 25;
pub const ANIMALS: [&str; 4] = ["fox", "owl", "frog", "bear"];
pub const PER_ANIMAL: usize = 7;
pub const START_TILES: usize = 5;
pub const AUCTIONS: usize = 12;
pub const CHIPS: i64 = 15;
pub const BID_MS: i64 = 15_000;
pub const PLACE_MS: i64 = 15_000;
pub const FULL: i64 = 25;
pub const PER_MATCH: i64 = 5;

/// One cell of a pattern: offset and animal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Part {
    pub x: i32,
    pub y: i32,
    pub animal: usize,
}

pub type Pattern = Vec<Part>;

const SHAPES: [[(i32, i32); 3]; 2] = [[(0, 0), (1, 0), (2, 0)], [(0, 0), (1, 0), (0, 1)]];

/// Every placement of a pattern on the grid: (cell index, animal) per part.
pub fn placements(p: &Pattern) -> Vec<Vec<(usize, usize)>> {
    let mut out: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut parts = p.clone();
    for _ in 0..4 {
        // Shift the turned pattern so its smallest offsets are 0.
        let (mx, my) = (parts.iter().map(|q| q.x).min().unwrap_or(0), parts.iter().map(|q| q.y).min().unwrap_or(0));
        for q in &mut parts {
            (q.x, q.y) = (q.x - mx, q.y - my);
        }
        for ay in 0..SIZE {
            for ax in 0..SIZE {
                let cells: Option<Vec<(usize, usize)>> = parts
                    .iter()
                    .map(|q| {
                        let (x, y) = (q.x + ax, q.y + ay);
                        ((0..SIZE).contains(&x) && (0..SIZE).contains(&y)).then_some(((y * SIZE + x) as usize, q.animal))
                    })
                    .collect();
                if let Some(mut c) = cells {
                    c.sort();
                    if !out.contains(&c) {
                        out.push(c);
                    }
                }
            }
        }
        for q in &mut parts {
            (q.x, q.y) = (-q.y, q.x);
        }
    }
    out
}

/// (full matches, most matching cells in one placement).
pub fn matches(grid: &[Option<usize>], p: &Pattern) -> (usize, usize) {
    matches_in(grid, &placements(p))
}

fn matches_in(grid: &[Option<usize>], places: &[Vec<(usize, usize)>]) -> (usize, usize) {
    let mut full = 0;
    let mut best = 0;
    for pl in places {
        let m = pl.iter().filter(|&&(c, a)| grid[c] == Some(a)).count();
        if m == pl.len() {
            full += 1;
        }
        best = best.max(m);
    }
    (full, best)
}

pub fn pattern_score(grid: &[Option<usize>], p: &Pattern) -> i64 {
    score_in(grid, &placements(p))
}

fn score_in(grid: &[Option<usize>], places: &[Vec<(usize, usize)>]) -> i64 {
    let (full, best) = matches_in(grid, places);
    FULL * full as i64 + PER_MATCH * best as i64
}

/// How promising the grid is for a pattern: placements with no wrong animal, weighted by matches.
fn potential(grid: &[Option<usize>], places: &[Vec<(usize, usize)>]) -> f64 {
    places
        .iter()
        .map(|pl| {
            if pl.iter().any(|&(c, a)| grid[c].is_some_and(|g| g != a)) {
                return 0.0;
            }
            match pl.iter().filter(|&&(c, _)| grid[c].is_some()).count() {
                3 => 40.0,
                2 => 6.0,
                1 => 1.0,
                _ => 0.1,
            }
        })
        .sum()
}

/// The best empty cell for this tile and how much it raises the potential.
fn best_cell(grid: &[Option<usize>], places: &[Vec<(usize, usize)>], tile: usize) -> (usize, f64) {
    let now = potential(grid, places);
    let mut g = grid.to_vec();
    let mut best = (usize::MAX, f64::NEG_INFINITY);
    for c in 0..CELLS {
        if grid[c].is_some() {
            continue;
        }
        g[c] = Some(tile);
        let gain = potential(&g, places) - now;
        g[c] = None;
        if gain > best.1 {
            best = (c, gain);
        }
    }
    best
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Bid,
    Place(usize),
}

#[derive(Clone, Debug)]
pub struct TileState {
    pub players: Vec<String>,
    pub grid: Vec<Option<usize>>,
    pub deck: Vec<usize>,
    pub patterns: Vec<Pattern>,
    /// Every placement of each seat's pattern (fixed for the game).
    pub places: Vec<Vec<Vec<(usize, usize)>>>,
    pub chips: Vec<i64>,
    pub auction: usize,
    pub phase: Phase,
    pub bids: Vec<Option<i64>>,
    pub time_left_ms: i64,
    pub log: Vec<String>,
    pub over: bool,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TileMove {
    Bid { chips: i64 },
    Place { cell: usize },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Final {
    pub patterns: Vec<Pattern>,
    pub scores: Vec<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileView {
    pub players: Vec<String>,
    pub me: usize,
    pub auction: usize,
    pub auctions: usize,
    /// "bid" | "place" | "over"
    pub phase: &'static str,
    pub grid: Vec<Option<usize>>,
    pub tile: Option<usize>,
    pub pattern: Pattern,
    pub chips: Vec<i64>,
    pub my_bid: Option<i64>,
    pub bid_done: Vec<bool>,
    pub placer: Option<usize>,
    pub time_left_ms: i64,
    pub phase_ms: i64,
    /// Your pattern score on the grid as it is now.
    pub pattern_score: i64,
    pub log: Vec<String>,
    pub result: Option<Final>,
}

impl TileState {
    fn tile(&self) -> usize {
        self.deck[self.auction]
    }

    fn score(&self, seat: usize) -> i64 {
        score_in(&self.grid, &self.places[seat]) + self.chips[seat]
    }

    fn next_auction(&mut self) {
        self.auction += 1;
        self.phase = Phase::Bid;
        self.bids = vec![None; self.players.len()];
        self.time_left_ms = BID_MS;
        if self.auction >= AUCTIONS || self.grid.iter().all(Option::is_some) {
            self.over = true;
            self.time_left_ms = 0;
            self.log.push("Game over: patterns are scored.".into());
        }
    }

    fn resolve_bids(&mut self) {
        let bid = |s: usize| self.bids[s].unwrap_or(0);
        let top = (0..self.players.len()).map(bid).max().unwrap_or(0);
        let best: Vec<usize> = (0..self.players.len()).filter(|&s| bid(s) == top).collect();
        let animal = ANIMALS[self.tile()];
        if best.len() == 1 {
            let w = best[0];
            self.chips[w] -= top;
            self.log.push(format!("{} wins the {animal} for {top}.", self.players[w]));
            self.phase = Phase::Place(w);
            self.time_left_ms = PLACE_MS;
        } else {
            self.log.push(format!("Tie at {top}: the {animal} is lost."));
            self.next_auction();
        }
    }

    fn place(&mut self, seat: usize, cell: usize) -> Result<(), String> {
        if cell >= CELLS || self.grid[cell].is_some() {
            return Err("Pick an empty cell".into());
        }
        self.grid[cell] = Some(self.tile());
        self.log.push(format!("{} places the {} at {}.", self.players[seat], ANIMALS[self.tile()], cell_name(cell)));
        self.next_auction();
        Ok(())
    }
}

/// "A1" … "E5": column letter, row number.
pub fn cell_name(cell: usize) -> String {
    format!("{}{}", (b'A' + (cell % 5) as u8) as char, cell / 5 + 1)
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
    type State = TileState;
    type Move = TileMove;
    type View = TileView;

    fn info() -> GameInfo {
        GameInfo {
            id: "animal-tile-auction",
            kind: "main",
            min_players: 3,
            max_players: 6,
            name: "Animal Tile Auction",
            short: "Bid chips for animal tiles and place them on a shared 5×5 grid to build your secret pattern.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<TileState, String> {
        if !(3..=6).contains(&players.len()) {
            return Err("Animal Tile Auction needs 3–6 players".into());
        }
        let n = players.len();
        let mut rng = Rng::new(seed);
        let mut deck: Vec<usize> = (0..ANIMALS.len()).flat_map(|a| std::iter::repeat_n(a, PER_ANIMAL)).collect();
        rng.shuffle(&mut deck);
        let mut grid = vec![None; CELLS];
        let mut cells: Vec<usize> = (0..CELLS).collect();
        rng.shuffle(&mut cells);
        for &c in &cells[..START_TILES] {
            grid[c] = deck.pop();
        }
        let mut patterns: Vec<Pattern> = Vec::new();
        while patterns.len() < n {
            let shape = SHAPES[rng.index(SHAPES.len())];
            let p: Pattern = shape.iter().map(|&(x, y)| Part { x, y, animal: rng.index(ANIMALS.len()) }).collect();
            if !patterns.contains(&p) {
                patterns.push(p);
            }
        }
        Ok(TileState {
            players: players.to_vec(),
            grid,
            deck,
            places: patterns.iter().map(placements).collect(),
            patterns,
            chips: vec![CHIPS; n],
            auction: 0,
            phase: Phase::Bid,
            bids: vec![None; n],
            time_left_ms: BID_MS,
            log: Vec::new(),
            over: false,
        })
    }

    fn apply_move(s: &mut TileState, player: &str, mv: TileMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.players.iter().position(|p| p == player).ok_or("Not in this game")?;
        match (mv, s.phase) {
            (TileMove::Bid { chips }, Phase::Bid) => {
                if s.bids[seat].is_some() {
                    return Err("You already bid".into());
                }
                if chips < 0 || chips > s.chips[seat] {
                    return Err("You do not have that many chips".into());
                }
                s.bids[seat] = Some(chips);
                if s.bids.iter().all(Option::is_some) {
                    s.resolve_bids();
                }
                Ok(())
            }
            (TileMove::Place { cell }, Phase::Place(w)) if w == seat => s.place(seat, cell),
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut TileState, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms > 0 {
            return;
        }
        match s.phase {
            Phase::Bid => {
                for b in &mut s.bids {
                    b.get_or_insert(0);
                }
                s.resolve_bids();
            }
            Phase::Place(w) => {
                let (cell, _) = best_cell(&s.grid, &s.places[w], s.tile());
                let _ = s.place(w, cell);
            }
        }
    }

    fn view_for(s: &TileState, player: &str) -> TileView {
        let me = s.players.iter().position(|p| p == player).unwrap_or(0);
        TileView {
            players: s.players.clone(),
            me,
            auction: (s.auction + 1).min(AUCTIONS),
            auctions: AUCTIONS,
            phase: match s.phase {
                _ if s.over => "over",
                Phase::Bid => "bid",
                Phase::Place(_) => "place",
            },
            grid: s.grid.clone(),
            tile: (!s.over).then(|| s.tile()),
            pattern: s.patterns[me].clone(),
            chips: s.chips.clone(),
            my_bid: s.bids[me],
            bid_done: s.bids.iter().map(Option::is_some).collect(),
            placer: match s.phase {
                Phase::Place(w) if !s.over => Some(w),
                _ => None,
            },
            time_left_ms: s.time_left_ms,
            phase_ms: if s.phase == Phase::Bid { BID_MS } else { PLACE_MS },
            pattern_score: score_in(&s.grid, &s.places[me]),
            log: s.log[s.log.len().saturating_sub(10)..].to_vec(),
            result: s.over.then(|| Final { patterns: s.patterns.clone(), scores: (0..s.players.len()).map(|i| s.score(i)).collect() }),
        }
    }

    fn is_over(s: &TileState) -> bool {
        s.over
    }

    fn results(s: &TileState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.players.iter().enumerate().map(|(i, p)| (p.clone(), s.score(i))).collect();
        rank_points(&scores)
    }

    fn to_move(s: &TileState) -> Vec<String> {
        if s.over {
            return Vec::new();
        }
        match s.phase {
            Phase::Bid => (0..s.players.len()).filter(|&i| s.bids[i].is_none()).map(|i| s.players[i].clone()).collect(),
            Phase::Place(w) => vec![s.players[w].clone()],
        }
    }

    fn bot_move(s: &TileState, player: &str, seed: u64) -> TileMove {
        let mut rng = Rng::new(seed);
        let seat = s.players.iter().position(|p| p == player).unwrap_or(0);
        let (cell, gain) = best_cell(&s.grid, &s.places[seat], s.tile());
        if let Phase::Place(_) = s.phase {
            return TileMove::Place { cell };
        }
        let chips = s.chips[seat];
        let left = (AUCTIONS - s.auction) as f64;
        let pace = chips as f64 / left * 2.0;
        let want = (gain / 5.0).min(pace + 1.0) * (0.7 + 0.6 * rng.next_f64());
        TileMove::Bid { chips: (want.round() as i64).clamp(0, chips) }
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

    fn line(a: usize, b: usize, c: usize) -> Pattern {
        vec![Part { x: 0, y: 0, animal: a }, Part { x: 1, y: 0, animal: b }, Part { x: 2, y: 0, animal: c }]
    }

    #[test]
    fn placements_cover_rotations() {
        // A line of 3 on 5×5: 3×5 across + 5×3 down = 30, each also reversed (other animal order).
        assert_eq!(placements(&line(0, 1, 2)).len(), 60);
        // An L: 4 rotations × 16 spots.
        let l = vec![Part { x: 0, y: 0, animal: 0 }, Part { x: 1, y: 0, animal: 0 }, Part { x: 0, y: 1, animal: 0 }];
        assert_eq!(placements(&l).len(), 64);
    }

    #[test]
    fn scores_full_and_partial_matches() {
        let mut grid = vec![None; CELLS];
        grid[0] = Some(0);
        grid[1] = Some(1);
        assert_eq!(matches(&grid, &line(0, 1, 2)), (0, 2));
        grid[2] = Some(2);
        assert_eq!(matches(&grid, &line(0, 1, 2)), (1, 3));
        // Turned: read downwards it also matches.
        let mut g2 = vec![None; CELLS];
        g2[22] = Some(0);
        g2[17] = Some(1);
        g2[12] = Some(2);
        assert_eq!(pattern_score(&g2, &line(0, 1, 2)), FULL + 3 * PER_MATCH);
    }

    #[test]
    fn highest_bid_wins_and_places() {
        let mut s = Rules::setup(&ids(3), 1).unwrap();
        Rules::apply_move(&mut s, "p1", TileMove::Bid { chips: 4 }).unwrap();
        assert!(Rules::apply_move(&mut s, "p1", TileMove::Bid { chips: 1 }).is_err());
        assert!(Rules::apply_move(&mut s, "p2", TileMove::Bid { chips: 16 }).is_err());
        Rules::apply_move(&mut s, "p2", TileMove::Bid { chips: 2 }).unwrap();
        Rules::apply_move(&mut s, "p3", TileMove::Bid { chips: 0 }).unwrap();
        assert_eq!(s.phase, Phase::Place(0));
        assert_eq!(s.chips, vec![11, 15, 15]);
        assert!(Rules::apply_move(&mut s, "p2", TileMove::Place { cell: 0 }).is_err());
        let empty = (0..CELLS).find(|&c| s.grid[c].is_none()).unwrap();
        let full = (0..CELLS).find(|&c| s.grid[c].is_some()).unwrap();
        assert!(Rules::apply_move(&mut s, "p1", TileMove::Place { cell: full }).is_err());
        let tile = s.tile();
        Rules::apply_move(&mut s, "p1", TileMove::Place { cell: empty }).unwrap();
        assert_eq!(s.grid[empty], Some(tile));
        assert_eq!(s.auction, 1);
    }

    #[test]
    fn tie_loses_the_tile() {
        let mut s = Rules::setup(&ids(3), 2).unwrap();
        let filled = s.grid.iter().filter(|c| c.is_some()).count();
        Rules::tick(&mut s, BID_MS as u64);
        assert_eq!(s.auction, 1);
        assert_eq!(s.grid.iter().filter(|c| c.is_some()).count(), filled);
    }

    #[test]
    fn timeout_places_for_the_winner() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        Rules::apply_move(&mut s, "p2", TileMove::Bid { chips: 1 }).unwrap();
        Rules::tick(&mut s, BID_MS as u64);
        assert_eq!(s.phase, Phase::Place(1));
        Rules::tick(&mut s, PLACE_MS as u64);
        assert_eq!(s.grid.iter().filter(|c| c.is_some()).count(), START_TILES + 1);
    }

    #[test]
    fn view_hides_other_patterns() {
        let s = Rules::setup(&ids(4), 4).unwrap();
        let v = serde_json::to_value(Rules::view_for(&s, "p2")).unwrap();
        assert!(v["result"].is_null());
        assert_eq!(v["pattern"].as_array().unwrap().len(), 3);
        assert!(v.get("patterns").is_none());
    }

    #[test]
    fn thousand_bot_games() {
        for n in [3, 6] {
            bot_games_with::<Rules>(300, n, 3000);
        }
        bot_games_with::<Rules>(400, 4, 3000);
    }

    #[test]
    fn bots_build_their_patterns() {
        let ids = ids(4);
        let (mut full, mut games, mut chips) = (0, 0, 0);
        for seed in 1..60 {
            let mut s = Rules::setup(&ids, seed).unwrap();
            let mut k = 0u64;
            while !s.over {
                for p in Rules::to_move(&s) {
                    k += 1;
                    let mv = Rules::bot_move(&s, &p, k * 31 + seed);
                    let _ = Rules::apply_move(&mut s, &p, mv);
                }
            }
            for i in 0..4 {
                full += matches(&s.grid, &s.patterns[i]).0;
                chips += s.chips[i];
            }
            games += 4;
        }
        // Measured: about 1.3 full patterns and 7 chips left per bot.
        assert!(full as f64 / games as f64 > 0.8);
        assert!(chips as f64 / games as f64 > 2.0, "bots pace their chips");
    }
}

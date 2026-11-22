//! Find the Differences (exile game, a race). Rules page: docs/games/find-the-differences.md
//!
//! Two generated pictures differ in a few shapes; tap the differences before
//! the rivals find theirs. Rivals (the seats after the first) are played by the
//! rules with seeded timing.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 3;
pub const DIFFS: [usize; ROUNDS] = [5, 6, 7];
pub const ROUND_MS: i64 = 60_000;
pub const REVEAL_MS: i64 = 4_000;
pub const FIND_POINTS: i64 = 30;
pub const MISS_POINTS: i64 = -10;
pub const RACE_BONUS: [i64; 3] = [100, 50, 25];
pub const SIZE: i32 = 1000;
pub const KINDS: u8 = 6;
pub const COLOURS: u8 = 8;
const MARGIN: i32 = 25;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shape {
    pub id: usize,
    /// 0 circle, 1 square, 2 triangle, 3 star, 4 diamond, 5 hexagon.
    pub kind: u8,
    pub colour: u8,
    pub x: i32,
    pub y: i32,
    pub r: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Picture {
    pub ground: u8,
    pub left: Vec<Shape>,
    pub right: Vec<Shape>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Spot {
    pub x: i32,
    pub y: i32,
    pub r: i32,
}

/// A pair of pictures with `diffs` differences, and where they are.
pub fn generate(rng: &mut Rng, diffs: usize) -> (Picture, Vec<Spot>) {
    let mut slots: Vec<(i32, i32)> = (0..16).map(|i| (125 + 250 * (i % 4), 125 + 250 * (i / 4))).collect();
    rng.shuffle(&mut slots);
    let count = rng.int(13, 15) as usize;
    let left: Vec<Shape> = slots[..count]
        .iter()
        .enumerate()
        .map(|(id, &(x, y))| Shape {
            id,
            kind: rng.int(0, i64::from(KINDS) - 1) as u8,
            colour: rng.int(0, i64::from(COLOURS) - 1) as u8,
            x: x + rng.int(-35, 35) as i32,
            y: y + rng.int(-35, 35) as i32,
            r: rng.int(55, 85) as i32,
        })
        .collect();
    let mut changed: Vec<usize> = (0..count).collect();
    rng.shuffle(&mut changed);
    changed.truncate(diffs);
    let mut right = left.clone();
    for &id in &changed {
        let s = &mut right[id];
        match rng.int(0, 3) {
            0 => s.colour = (s.colour + rng.int(1, i64::from(COLOURS) - 1) as u8) % COLOURS,
            1 => s.kind = (s.kind + rng.int(1, i64::from(KINDS) - 1) as u8) % KINDS,
            2 => s.r = if s.r >= 70 { s.r * 3 / 5 } else { s.r * 3 / 2 },
            _ => s.r = 0,
        }
    }
    right.retain(|s| s.r > 0);
    let spots = changed.iter().map(|&id| Spot { x: left[id].x, y: left[id].y, r: left[id].r.max(right_r(&right, id)) }).collect();
    (Picture { ground: rng.int(0, 3) as u8, left, right }, spots)
}

fn right_r(right: &[Shape], id: usize) -> i32 {
    right.iter().find(|s| s.id == id).map_or(0, |s| s.r)
}

/// The solver: every shape that differs between the pictures (sorted by id).
pub fn solve(p: &Picture) -> Vec<usize> {
    p.left.iter().filter(|l| p.right.iter().find(|r| r.id == l.id) != Some(*l)).map(|l| l.id).collect()
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    pub points: i64,
    /// First seat: (spot index, time) of each find. Rivals: planned find times.
    pub finds: Vec<(usize, i64)>,
    pub misses: i64,
}

impl Seat {
    fn found(&self, now: i64) -> usize {
        self.finds.iter().filter(|f| f.1 <= now).count()
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub seats: Vec<Seat>,
    pub rng: Rng,
    pub round: usize,
    pub picture: Picture,
    pub spots: Vec<Spot>,
    pub elapsed_ms: i64,
    pub revealing: bool,
    pub reveal_left_ms: i64,
    pub over: bool,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Tap { x: i32, y: i32 },
    Next,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub found: usize,
    pub points: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round: usize,
    pub rounds: usize,
    pub time_left_ms: i64,
    pub round_ms: i64,
    pub picture: Picture,
    pub diffs: usize,
    pub found: Vec<Spot>,
    pub misses: i64,
    /// Every difference: only in the reveal and after the game.
    pub all: Option<Vec<Spot>>,
    pub seats: Vec<SeatView>,
    pub you: usize,
    pub revealing: bool,
    pub reveal_left_ms: i64,
    pub over: bool,
}

impl State {
    fn start_round(&mut self) {
        let (picture, spots) = generate(&mut self.rng, DIFFS[self.round]);
        self.picture = picture;
        self.spots = spots;
        self.elapsed_ms = 0;
        self.revealing = false;
        let n = self.spots.len();
        for i in 0..self.seats.len() {
            self.seats[i].misses = 0;
            self.seats[i].finds = if i == 0 {
                Vec::new()
            } else {
                let speed = 0.8 + self.rng.next_f64() * 0.6;
                let mut at = 0;
                let mut plan = Vec::new();
                for k in 0..n {
                    at += (self.rng.int(4_000, 12_000) as f64 * speed) as i64;
                    if self.rng.next_f64() < 0.92 {
                        plan.push((k, at));
                    }
                }
                plan
            };
        }
    }

    fn live_points(&self, i: usize) -> i64 {
        let s = &self.seats[i];
        if self.revealing || self.over {
            s.points
        } else {
            s.points + FIND_POINTS * s.found(self.elapsed_ms) as i64 + MISS_POINTS * s.misses
        }
    }

    fn finish_time(&self, i: usize) -> Option<i64> {
        let s = &self.seats[i];
        (s.found(ROUND_MS) == self.spots.len()).then(|| s.finds.iter().map(|f| f.1).max().unwrap_or(0))
    }

    fn end_round(&mut self) {
        self.elapsed_ms = ROUND_MS;
        let times: Vec<Option<i64>> = (0..self.seats.len()).map(|i| self.finish_time(i)).collect();
        for i in 0..self.seats.len() {
            let mut gain = FIND_POINTS * self.seats[i].found(ROUND_MS) as i64 + MISS_POINTS * self.seats[i].misses;
            if let Some(t) = times[i] {
                let ahead = times.iter().flatten().filter(|&&o| o < t).count();
                gain += RACE_BONUS.get(ahead).copied().unwrap_or(0);
            }
            self.seats[i].points += gain;
        }
        self.revealing = true;
        self.reveal_left_ms = REVEAL_MS;
    }

    fn next_round(&mut self) {
        self.round += 1;
        if self.round >= ROUNDS {
            self.over = true;
            self.revealing = false;
        } else {
            self.start_round();
        }
    }

    fn first_done(&self) -> bool {
        self.seats[0].finds.len() == self.spots.len()
    }

    fn hit(&self, x: i32, y: i32) -> Option<usize> {
        // A tap on the right picture counts the same as on the left one.
        self.spots.iter().enumerate().find_map(|(k, sp)| {
            let (dx, dy) = (i64::from(x - sp.x), i64::from(y - sp.y));
            let reach = i64::from(sp.r + MARGIN);
            (dx * dx + dy * dy <= reach * reach && !self.seats[0].finds.iter().any(|f| f.0 == k)).then_some(k)
        })
    }
}

pub struct FindTheDifferences;

impl Game for FindTheDifferences {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "find-the-differences",
            kind: "exile",
            min_players: 1,
            max_players: 4,
            name: "Find the Differences",
            short: "Two pictures, a handful of differences: tap them all before the rivals do.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if players.is_empty() || players.len() > 4 {
            return Err("Find the Differences needs 1–4 players".into());
        }
        let seats = players.iter().map(|id| Seat { id: id.clone(), points: 0, finds: Vec::new(), misses: 0 }).collect();
        let mut s = State {
            seats,
            rng: Rng::new(seed),
            round: 0,
            picture: Picture { ground: 0, left: Vec::new(), right: Vec::new() },
            spots: Vec::new(),
            elapsed_ms: 0,
            revealing: false,
            reveal_left_ms: 0,
            over: false,
        };
        s.start_round();
        Ok(s)
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        if player != s.seats[0].id {
            return Err("Rivals play by themselves".into());
        }
        match mv {
            Move::Tap { x, y } if !s.revealing => {
                if !(0..=SIZE).contains(&x) || !(0..=SIZE).contains(&y) {
                    return Err("Tap inside the picture".into());
                }
                match s.hit(x, y) {
                    Some(k) => {
                        let at = s.elapsed_ms;
                        s.seats[0].finds.push((k, at));
                        if s.first_done() {
                            s.end_round();
                        }
                    }
                    None => s.seats[0].misses += 1,
                }
                Ok(())
            }
            Move::Next if s.revealing => {
                s.next_round();
                Ok(())
            }
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut State, ms: u64) {
        if s.over {
            return;
        }
        if s.revealing {
            s.reveal_left_ms -= ms as i64;
            if s.reveal_left_ms <= 0 {
                s.next_round();
            }
        } else {
            s.elapsed_ms += ms as i64;
            if s.elapsed_ms >= ROUND_MS {
                s.end_round();
            }
        }
    }

    fn view_for(s: &State, player: &str) -> View {
        let me = s.seats.iter().position(|x| x.id == player).unwrap_or(0);
        let now = s.elapsed_ms;
        let found = if me == 0 { s.seats[0].finds.iter().map(|f| s.spots[f.0]).collect() } else { Vec::new() };
        View {
            round: (s.round + 1).min(ROUNDS),
            rounds: ROUNDS,
            time_left_ms: (ROUND_MS - now).max(0),
            round_ms: ROUND_MS,
            picture: s.picture.clone(),
            diffs: s.spots.len(),
            found,
            misses: s.seats[me].misses,
            all: (s.revealing || s.over).then(|| s.spots.clone()),
            seats: (0..s.seats.len())
                .map(|i| SeatView { id: s.seats[i].id.clone(), found: s.seats[i].found(now), points: s.live_points(i) })
                .collect(),
            you: me,
            revealing: s.revealing,
            reveal_left_ms: s.reveal_left_ms.max(0),
            over: s.over,
        }
    }

    fn is_over(s: &State) -> bool {
        s.over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        rank_by_score(&s.seats.iter().map(|x| (x.id.clone(), x.points)).collect::<Vec<_>>())
    }

    fn to_move(s: &State) -> Vec<String> {
        if s.over || s.revealing { Vec::new() } else { vec![s.seats[0].id.clone()] }
    }

    fn bot_move(s: &State, _player: &str, seed: u64) -> Move {
        if s.revealing {
            return Move::Next;
        }
        let mut rng = Rng::new(seed);
        let open: Vec<&Spot> = s.spots.iter().enumerate().filter(|(k, _)| !s.seats[0].finds.iter().any(|f| f.0 == *k)).map(|(_, sp)| sp).collect();
        if !open.is_empty() && rng.next_f64() < 0.7 {
            let sp = rng.pick(&open);
            Move::Tap { x: sp.x, y: sp.y }
        } else {
            Move::Tap { x: rng.int(0, i64::from(SIZE)) as i32, y: rng.int(0, i64::from(SIZE)) as i32 }
        }
    }
}

pub const READY: bool = true;
pub type Rules = FindTheDifferences;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn the_solver_finds_exactly_the_generated_differences() {
        for seed in 1..=300 {
            let mut rng = Rng::new(seed);
            let diffs = 5 + (seed % 3) as usize;
            let (p, spots) = generate(&mut rng, diffs);
            let found = solve(&p);
            assert_eq!(found.len(), diffs, "seed {seed}");
            let mut at: Vec<(i32, i32)> = found.iter().map(|&id| (p.left[id].x, p.left[id].y)).collect();
            let mut want: Vec<(i32, i32)> = spots.iter().map(|s| (s.x, s.y)).collect();
            at.sort();
            want.sort();
            assert_eq!(at, want);
            // Shapes stay apart: every shape is in its own slot.
            assert!(p.left.iter().all(|s| s.x - s.r > -30 && s.x + s.r < SIZE + 30));
        }
    }

    #[test]
    fn taps_find_differences_once_and_misses_cost() {
        let mut s = FindTheDifferences::setup(&players(2), 1).unwrap();
        let sp = s.spots[0];
        FindTheDifferences::apply_move(&mut s, "p1", Move::Tap { x: sp.x + sp.r, y: sp.y }).unwrap();
        assert_eq!(s.seats[0].finds.len(), 1);
        FindTheDifferences::apply_move(&mut s, "p1", Move::Tap { x: sp.x, y: sp.y }).unwrap();
        assert_eq!(s.seats[0].finds.len(), 1);
        assert_eq!(s.seats[0].misses, 1);
        let v = FindTheDifferences::view_for(&s, "p1");
        assert_eq!(v.found.len(), 1);
        assert!(v.all.is_none());
        assert_eq!(v.seats[0].points, FIND_POINTS + MISS_POINTS);
        assert!(FindTheDifferences::apply_move(&mut s, "p1", Move::Tap { x: -5, y: 0 }).is_err());
        assert!(FindTheDifferences::apply_move(&mut s, "p2", Move::Next).is_err());
    }

    #[test]
    fn finding_all_first_ends_the_round_with_the_bonus() {
        let mut s = FindTheDifferences::setup(&players(4), 2).unwrap();
        for sp in s.spots.clone() {
            FindTheDifferences::apply_move(&mut s, "p1", Move::Tap { x: sp.x, y: sp.y }).unwrap();
        }
        assert!(s.revealing);
        assert_eq!(s.seats[0].points, FIND_POINTS * DIFFS[0] as i64 + RACE_BONUS[0]);
        assert_eq!(FindTheDifferences::view_for(&s, "p1").all.unwrap().len(), DIFFS[0]);
        // The rivals' whole minute was played out.
        assert!(s.seats[1..].iter().all(|r| r.points >= FIND_POINTS * r.found(ROUND_MS) as i64));
        FindTheDifferences::apply_move(&mut s, "p1", Move::Next).unwrap();
        assert_eq!(s.spots.len(), DIFFS[1]);
    }

    #[test]
    fn rivals_find_with_time() {
        let mut s = FindTheDifferences::setup(&players(3), 3).unwrap();
        FindTheDifferences::tick(&mut s, 1_000);
        assert!(FindTheDifferences::view_for(&s, "p1").seats[1..].iter().all(|r| r.found == 0));
        FindTheDifferences::tick(&mut s, 40_000);
        assert!(FindTheDifferences::view_for(&s, "p1").seats[1..].iter().any(|r| r.found >= 2));
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<FindTheDifferences>(1000, 1, 400);
        bot_games_with::<FindTheDifferences>(300, 4, 400);
    }
}

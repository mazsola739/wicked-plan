//! Metal Cast Puzzle (prison, solo). Rules page: docs/games/metal-cast-puzzle.md
//!
//! Rings on a metal loop: ring 1 always slides; ring k slides only when ring
//! k − 1 is on and every ring below it is off. Take every ring off.
//! States are bit masks (bit 0 = ring 1); the solver walks all 2ⁿ states.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult};
use crate::rng::Rng;

pub const RINGS: [usize; 3] = [4, 5, 6];
pub const MIN_DISTANCE: [u32; 3] = [0, 12, 24];
pub const LEVEL_POINTS: [i64; 3] = [150, 250, 400];

/// Can ring `ring` (0-based) slide in state `mask`?
pub fn can_toggle(mask: u32, ring: usize) -> bool {
    ring == 0 || (mask >> (ring - 1)) & 1 == 1 && mask & ((1 << (ring - 1)) - 1) == 0
}

/// The fewest moves from every state to "all off", by breadth-first search.
pub fn distances(n: usize) -> Vec<u32> {
    let mut dist = vec![u32::MAX; 1 << n];
    dist[0] = 0;
    let mut queue = VecDeque::from([0u32]);
    while let Some(m) = queue.pop_front() {
        for r in 0..n {
            let next = m ^ (1 << r);
            if can_toggle(m, r) && dist[next as usize] == u32::MAX {
                dist[next as usize] = dist[m as usize] + 1;
                queue.push_back(next);
            }
        }
    }
    dist
}

#[derive(Clone, Debug)]
pub struct CastState {
    pub player: String,
    pub level: usize,
    pub start: u32,
    pub mask: u32,
    pub best: u32,
    pub moves: u32,
    pub solved: Vec<bool>,
    pub score: i64,
    pub over: bool,
    /// Replay moments: a mixed level (5 or 6 rings) solved in the fewest moves.
    pub moments: Moments,
    seed: u64,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CastMove {
    /// 1-based ring number.
    Toggle { ring: usize },
    Reset,
    Skip,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CastView {
    pub level: usize,
    pub levels: usize,
    pub rings: Vec<bool>,
    pub moves: u32,
    pub best: u32,
    pub solved: Vec<bool>,
    pub score: i64,
    pub over: bool,
}

impl CastState {
    pub fn n(&self) -> usize {
        RINGS[self.level.min(RINGS.len() - 1)]
    }

    fn load_level(&mut self) {
        let n = self.n();
        let dist = distances(n);
        let start = if self.level == 0 {
            (1 << n) - 1
        } else {
            let mut rng = Rng::new(self.seed.wrapping_add(self.level as u64 * 7_777_777));
            let far: Vec<u32> = (0..1u32 << n).filter(|&m| dist[m as usize] >= MIN_DISTANCE[self.level]).collect();
            *rng.pick(&far)
        };
        self.start = start;
        self.mask = start;
        self.best = dist[start as usize];
        self.moves = 0;
    }

    fn next_level(&mut self, solved: bool) {
        if solved {
            let base = LEVEL_POINTS[self.level];
            let moves = i64::from(self.moves.max(self.best).max(1));
            self.score += (base * i64::from(self.best) + moves / 2) / moves;
            if self.level > 0 && self.moves == self.best {
                let text = format!("{} rings off in {} moves, the fewest possible", self.n(), self.moves);
                self.moments.push(&self.player.clone(), text);
            }
        }
        self.solved.push(solved);
        self.level += 1;
        if self.level >= RINGS.len() {
            self.over = true;
        } else {
            self.load_level();
        }
    }
}

pub struct Rules;

impl Game for Rules {
    type State = CastState;
    type Move = CastMove;
    type View = CastView;

    fn info() -> GameInfo {
        GameInfo {
            id: "metal-cast-puzzle",
            kind: "prison",
            difficulty: "easy",
            tags: &["logic"],
            min_players: 1,
            max_players: 1,
            name: "Metal Cast Puzzle",
            short: "Slide every ring off the metal loop; a ring moves only when the ring below lets it.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<CastState, String> {
        let [player] = players else { return Err("Metal Cast Puzzle is a solo game".into()) };
        let mut s = CastState {
            player: player.clone(),
            level: 0,
            start: 0,
            mask: 0,
            best: 0,
            moves: 0,
            solved: Vec::new(),
            score: 0,
            over: false,
            moments: Moments::default(),
            seed,
        };
        s.load_level();
        Ok(s)
    }

    fn apply_move(s: &mut CastState, player: &str, mv: CastMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        if player != s.player {
            return Err("Not your game".into());
        }
        match mv {
            CastMove::Toggle { ring } => {
                if ring == 0 || ring > s.n() {
                    return Err("No such ring".into());
                }
                if !can_toggle(s.mask, ring - 1) {
                    return Err(format!("Ring {ring} is stuck: ring {} must be on and every ring below it off", ring - 1));
                }
                s.mask ^= 1 << (ring - 1);
                s.moves += 1;
                if s.mask == 0 {
                    s.next_level(true);
                }
            }
            CastMove::Reset => s.mask = s.start,
            CastMove::Skip => s.next_level(false),
        }
        Ok(())
    }

    fn view_for(s: &CastState, _player: &str) -> CastView {
        CastView {
            level: (s.level + 1).min(RINGS.len()),
            levels: RINGS.len(),
            rings: (0..s.n()).map(|r| (s.mask >> r) & 1 == 1).collect(),
            moves: s.moves,
            best: s.best,
            solved: s.solved.clone(),
            score: s.score,
            over: s.over,
        }
    }

    fn is_over(s: &CastState) -> bool {
        s.over
    }

    fn results(s: &CastState) -> Vec<PlayerResult> {
        vec![PlayerResult { player_id: s.player.clone(), rank: 1, points: s.score }]
    }

    fn moments(s: &CastState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &CastState, _player: &str, seed: u64) -> CastMove {
        let n = s.n();
        let dist = distances(n);
        let legal: Vec<usize> = (0..n).filter(|&r| can_toggle(s.mask, r)).collect();
        let mut rng = Rng::new(seed);
        let ring = if rng.next_f64() < 0.1 {
            *rng.pick(&legal)
        } else {
            *legal.iter().min_by_key(|&&r| dist[(s.mask ^ (1 << r)) as usize]).expect("ring 1 always moves")
        };
        CastMove::Toggle { ring: ring + 1 }
    }
}

pub const READY: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games;

    const P: &str = "p1";

    #[test]
    fn known_shortest_solutions() {
        // The classic counts for all rings on: 1, 2, 5, 10, 21, 42.
        for (n, want) in [(1, 1), (2, 2), (3, 5), (4, 10), (5, 21), (6, 42)] {
            assert_eq!(distances(n)[(1 << n) - 1], want, "n={n}");
        }
        // Every state can be solved.
        assert!(distances(6).iter().all(|&d| d != u32::MAX));
    }

    #[test]
    fn only_free_rings_move() {
        let mut s = Rules::setup(&[P.into()], 1).unwrap();
        // All four on: ring 1 and ring 2 move; ring 3 and 4 are stuck.
        assert!(Rules::apply_move(&mut s, P, CastMove::Toggle { ring: 3 }).is_err());
        assert!(Rules::apply_move(&mut s, P, CastMove::Toggle { ring: 4 }).is_err());
        assert_eq!(s.moves, 0);
        Rules::apply_move(&mut s, P, CastMove::Toggle { ring: 2 }).unwrap();
        assert_eq!(s.mask, 0b1101);
        Rules::apply_move(&mut s, P, CastMove::Reset).unwrap();
        assert_eq!((s.mask, s.moves), (0b1111, 1));
    }

    #[test]
    fn levels_start_far_enough_and_score() {
        for seed in 0..50 {
            let mut s = Rules::setup(&[P.into()], seed).unwrap();
            assert_eq!(s.best, 10);
            // Play the shortest path.
            let dist = distances(4);
            while s.level == 0 {
                let r = (0..4).find(|&r| can_toggle(s.mask, r) && dist[(s.mask ^ (1 << r)) as usize] < dist[s.mask as usize]).unwrap();
                Rules::apply_move(&mut s, P, CastMove::Toggle { ring: r + 1 }).unwrap();
            }
            assert_eq!(s.solved, vec![true]);
            assert_eq!(s.score, 150);
            assert!(s.best >= MIN_DISTANCE[1]);
            // No level timer: a long wait changes nothing.
            Rules::tick(&mut s, 3_600_000);
            assert_eq!((s.level, s.moves), (1, 0));
            Rules::apply_move(&mut s, P, CastMove::Skip).unwrap();
            assert!(s.best >= MIN_DISTANCE[2] && s.n() == 6);
        }
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<Rules>(1000, 5000);
    }

    /// Plays the shortest path, plus `detour` wasted moves (ring 1 off and on again) first.
    fn solve(s: &mut CastState, detour: bool) {
        if detour {
            Rules::apply_move(s, P, CastMove::Toggle { ring: 1 }).unwrap();
            Rules::apply_move(s, P, CastMove::Toggle { ring: 1 }).unwrap();
        }
        let (level, dist) = (s.level, distances(s.n()));
        while s.level == level {
            let r = (0..s.n()).find(|&r| can_toggle(s.mask, r) && dist[(s.mask ^ (1 << r)) as usize] < dist[s.mask as usize]).unwrap();
            Rules::apply_move(s, P, CastMove::Toggle { ring: r + 1 }).unwrap();
        }
    }

    #[test]
    fn a_mixed_level_in_the_fewest_moves_is_a_replay_moment() {
        let mut s = Rules::setup(&[P.into()], 5).unwrap();
        // Level 1 (all rings on, always the same) is no moment, even when perfect.
        solve(&mut s, false);
        assert!(Rules::moments(&s, P).is_empty());
        let best = s.best;
        solve(&mut s, false);
        let m = Rules::moments(&s, P);
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), (P, format!("5 rings off in {best} moves, the fewest possible").as_str()));
        // Two wasted moves: no moment.
        solve(&mut s, true);
        assert_eq!(s.solved, vec![true, true, true]);
        assert_eq!(Rules::moments(&s, P).len(), 1);
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 1, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 10 <= stats.games * 8, "too common: {stats:?}");
        assert!(stats.most <= 2, "too many in one game: {stats:?}");
    }
}

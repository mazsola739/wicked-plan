//! Knight's Tour (Hidden, solo). Rules page: docs/games/knights-tour.md
//!
//! Three boards: 5×5, 6×6, 7×7. Put the knight on any square, then move it
//! like a chess knight (an L: two squares one way, one square sideways) until
//! it has visited every square once. Stuck? Start the board again. All three
//! boards share one 15-minute clock.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult};
use crate::rng::Rng;

pub const BOARDS: [usize; 3] = [5, 6, 7];
pub const BOARD_POINTS: [i64; 3] = [200, 300, 500];
pub const TIME_MS: i64 = 15 * 60_000;

const JUMPS: [(i32, i32); 8] = [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)];

#[derive(Clone, Debug)]
pub struct KnightState {
    pub player: String,
    pub board: usize,
    pub path: Vec<(usize, usize)>,
    pub restarts: u32,
    pub solved: usize,
    pub score: i64,
    pub time_left_ms: i64,
    pub over: bool,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum KnightMove {
    Step { x: usize, y: usize },
    Restart,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnightView {
    pub board: usize,
    pub boards: usize,
    pub size: usize,
    pub path: Vec<(usize, usize)>,
    /// Squares the knight may jump to now (all squares before the first step).
    pub targets: Vec<(usize, usize)>,
    pub stuck: bool,
    pub restarts: u32,
    pub solved: usize,
    pub score: i64,
    pub time_left_ms: i64,
    pub total_time_ms: i64,
    pub over: bool,
}

pub fn jumps_from(size: usize, (x, y): (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
    JUMPS.iter().filter_map(move |&(dx, dy)| {
        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
        (nx >= 0 && ny >= 0 && (nx as usize) < size && (ny as usize) < size).then_some((nx as usize, ny as usize))
    })
}

impl KnightState {
    pub fn size(&self) -> usize {
        BOARDS[self.board.min(BOARDS.len() - 1)]
    }

    pub fn targets(&self) -> Vec<(usize, usize)> {
        let n = self.size();
        match self.path.last() {
            None => (0..n).flat_map(|y| (0..n).map(move |x| (x, y))).collect(),
            Some(&at) => jumps_from(n, at).filter(|sq| !self.path.contains(sq)).collect(),
        }
    }

    fn next_board(&mut self) {
        self.score += BOARD_POINTS[self.board];
        self.solved += 1;
        self.board += 1;
        self.path.clear();
        if self.board >= BOARDS.len() {
            self.over = true;
            // Time left is worth 1 point a second.
            self.score += self.time_left_ms / 1000;
        }
    }
}

/// Warnsdorff's rule: jump to the square with the fewest onward jumps.
pub fn warnsdorff(size: usize, start: (usize, usize), rng: &mut Rng) -> Vec<(usize, usize)> {
    let mut path = vec![start];
    while path.len() < size * size {
        let at = *path.last().expect("path");
        let onward = |sq: (usize, usize), path: &[(usize, usize)]| jumps_from(size, sq).filter(|s| !path.contains(s) && *s != sq).count();
        let mut options: Vec<((usize, usize), usize)> =
            jumps_from(size, at).filter(|sq| !path.contains(sq)).map(|sq| (sq, onward(sq, &path))).collect();
        if options.is_empty() {
            break;
        }
        rng.shuffle(&mut options);
        let best = options.iter().min_by_key(|(_, n)| *n).expect("options").0;
        path.push(best);
    }
    path
}

pub struct KnightsTour;

impl Game for KnightsTour {
    type State = KnightState;
    type Move = KnightMove;
    type View = KnightView;

    fn info() -> GameInfo {
        GameInfo {
            id: "knights-tour",
            kind: "hidden",
            min_players: 1,
            max_players: 1,
            name: "Knight's Tour",
            short: "Visit every square with the knight, never the same one twice. Three boards, 15 minutes.",
        }
    }

    fn setup(players: &[String], _seed: u64) -> Result<KnightState, String> {
        let [player] = players else { return Err("Knight's Tour is a solo game".into()) };
        Ok(KnightState {
            player: player.clone(),
            board: 0,
            path: Vec::new(),
            restarts: 0,
            solved: 0,
            score: 0,
            time_left_ms: TIME_MS,
            over: false,
        })
    }

    fn apply_move(s: &mut KnightState, player: &str, mv: KnightMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        if player != s.player {
            return Err("Not your game".into());
        }
        match mv {
            KnightMove::Restart => {
                if !s.path.is_empty() {
                    s.path.clear();
                    s.restarts += 1;
                }
            }
            KnightMove::Step { x, y } => {
                if !s.targets().contains(&(x, y)) {
                    return Err("The knight cannot go there".into());
                }
                s.path.push((x, y));
                if s.path.len() == s.size() * s.size() {
                    s.next_board();
                }
            }
        }
        Ok(())
    }

    fn tick(s: &mut KnightState, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.time_left_ms = 0;
            s.over = true;
        }
    }

    fn view_for(s: &KnightState, _player: &str) -> KnightView {
        let targets = if s.over { Vec::new() } else { s.targets() };
        KnightView {
            board: (s.board + 1).min(BOARDS.len()),
            boards: BOARDS.len(),
            size: s.size(),
            stuck: !s.over && !s.path.is_empty() && targets.is_empty(),
            path: s.path.clone(),
            targets,
            restarts: s.restarts,
            solved: s.solved,
            score: s.score,
            time_left_ms: s.time_left_ms,
            total_time_ms: TIME_MS,
            over: s.over,
        }
    }

    fn is_over(s: &KnightState) -> bool {
        s.over
    }

    fn results(s: &KnightState) -> Vec<PlayerResult> {
        vec![PlayerResult { player_id: s.player.clone(), rank: 1, points: s.score }]
    }

    fn bot_move(s: &KnightState, _player: &str, seed: u64) -> KnightMove {
        let mut rng = Rng::new(seed);
        let n = s.size();
        let targets = s.targets();
        if targets.is_empty() {
            return KnightMove::Restart;
        }
        if s.path.is_empty() {
            // Corners work well with Warnsdorff's rule.
            let c = *rng.pick(&[(0, 0), (n - 1, 0), (0, n - 1), (n - 1, n - 1)]);
            return KnightMove::Step { x: c.0, y: c.1 };
        }
        let onward = |sq: (usize, usize)| jumps_from(n, sq).filter(|t| !s.path.contains(t) && *t != sq).count();
        let mut options = targets;
        rng.shuffle(&mut options);
        let best = options.into_iter().min_by_key(|&sq| onward(sq)).expect("targets");
        KnightMove::Step { x: best.0, y: best.1 }
    }
}

pub const READY: bool = true;
pub type Rules = KnightsTour;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games;

    const P: &str = "p1";

    #[test]
    fn every_board_has_a_tour() {
        for (i, &n) in BOARDS.iter().enumerate() {
            let found = (0..200).any(|seed| warnsdorff(n, (0, 0), &mut Rng::new(seed)).len() == n * n);
            assert!(found, "no tour found on board {i}");
        }
    }

    #[test]
    fn only_knight_jumps_to_new_squares() {
        let mut s = KnightsTour::setup(&[P.into()], 1).unwrap();
        KnightsTour::apply_move(&mut s, P, KnightMove::Step { x: 0, y: 0 }).unwrap();
        assert!(KnightsTour::apply_move(&mut s, P, KnightMove::Step { x: 1, y: 1 }).is_err());
        KnightsTour::apply_move(&mut s, P, KnightMove::Step { x: 1, y: 2 }).unwrap();
        assert!(KnightsTour::apply_move(&mut s, P, KnightMove::Step { x: 0, y: 0 }).is_err());
        KnightsTour::apply_move(&mut s, P, KnightMove::Restart).unwrap();
        assert!(s.path.is_empty());
        assert_eq!(s.restarts, 1);
    }

    #[test]
    fn a_full_tour_moves_to_the_next_board() {
        let mut s = KnightsTour::setup(&[P.into()], 1).unwrap();
        let tour = (0..200).map(|seed| warnsdorff(5, (0, 0), &mut Rng::new(seed))).find(|t| t.len() == 25).unwrap();
        for (x, y) in tour {
            KnightsTour::apply_move(&mut s, P, KnightMove::Step { x, y }).unwrap();
        }
        assert_eq!(s.board, 1);
        assert_eq!(s.score, BOARD_POINTS[0]);
        assert_eq!(KnightsTour::view_for(&s, P).size, 6);
    }

    #[test]
    fn time_up_ends_the_game() {
        let mut s = KnightsTour::setup(&[P.into()], 1).unwrap();
        KnightsTour::tick(&mut s, TIME_MS as u64);
        assert!(s.over);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<KnightsTour>(1000, 5000);
    }
}

//! Symmetry Puzzle (exile, a race for 1–4). Rules page: docs/games/symmetry-puzzle.md
//!
//! A coloured figure should be symmetric about the axis (or axes), but parts
//! are missing. Paint the missing mirror cells. Seat 0 solves; the other seats
//! are bots that finish at seeded times (`Race`, from Piece Puzzle).

use serde::{Deserialize, Serialize};

use super::piece_puzzle::{Race, RaceView};
use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult};
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Axis {
    Vertical,
    Horizontal,
    Diagonal,
    Both,
}

pub const SIZES: [usize; 3] = [6, 7, 8];
pub const AXES: [Axis; 3] = [Axis::Vertical, Axis::Diagonal, Axis::Both];
pub const COLOURS: [u8; 3] = [2, 3, 3];
pub const ROUND_MS: [i64; 3] = [90_000, 2 * 60_000, 3 * 60_000];

/// The cell and its mirror images (the cell itself first; may repeat on an axis).
pub fn orbit(n: usize, axis: Axis, cell: usize) -> Vec<usize> {
    let (r, c) = (cell / n, cell % n);
    let at = |r: usize, c: usize| r * n + c;
    let mut out = vec![cell];
    match axis {
        Axis::Vertical => out.push(at(r, n - 1 - c)),
        Axis::Horizontal => out.push(at(n - 1 - r, c)),
        Axis::Diagonal => out.push(at(c, r)),
        Axis::Both => out.extend([at(r, n - 1 - c), at(n - 1 - r, c), at(n - 1 - r, n - 1 - c)]),
    }
    out
}

pub fn is_symmetric(n: usize, axis: Axis, cells: &[u8]) -> bool {
    (0..n * n).all(|i| orbit(n, axis, i).iter().all(|&j| cells[j] == cells[i]))
}

#[derive(Clone, Debug)]
pub struct Figure {
    pub size: usize,
    pub axis: Axis,
    pub colours: u8,
    pub solution: Vec<u8>,
    pub given: Vec<bool>,
}

/// A symmetric figure, then for every coloured group of mirror cells some
/// (never all) are hidden. At least a quarter of the coloured cells go missing.
pub fn generate(round: usize, rng: &mut Rng) -> Figure {
    let (n, axis, colours) = (SIZES[round], AXES[round], COLOURS[round]);
    loop {
        let mut solution = vec![0u8; n * n];
        let mut given = vec![false; n * n];
        for i in 0..n * n {
            let mut group = orbit(n, axis, i);
            group.sort_unstable();
            group.dedup();
            if group[0] != i || rng.next_f64() >= 0.42 {
                continue;
            }
            let colour = rng.int(1, i64::from(colours)) as u8;
            for &j in &group {
                solution[j] = colour;
            }
            if group.len() == 1 {
                // A cell on the axis is its own mirror: shown.
                given[i] = true;
                continue;
            }
            rng.shuffle(&mut group);
            let keep = rng.int(1, group.len() as i64 - 1) as usize;
            for &j in &group[..keep] {
                given[j] = true;
            }
        }
        let coloured = solution.iter().filter(|&&c| c > 0).count();
        let missing = (0..n * n).filter(|&i| solution[i] > 0 && !given[i]).count();
        if coloured >= n * n / 4 && missing * 4 >= coloured {
            return Figure { size: n, axis, colours, solution, given };
        }
    }
}

#[derive(Clone, Debug)]
pub struct SymmetryState {
    pub race: Race,
    pub figure: Figure,
    pub cells: Vec<u8>,
    seed: u64,
    /// Rounds the solver finished in first place.
    firsts: usize,
    /// Replay moments: first by less than 2 s; first in every round.
    pub moments: Moments,
}

impl SymmetryState {
    /// Seat 0 just solved a round: its replay moments, if any (every finish time is public now).
    fn note_solve(&mut self) {
        let Some(last) = self.race.last.as_ref() else { return };
        let (Some(t), Some(1)) = (last[0].time_ms, last[0].place) else { return };
        let by = self.race.players[0].clone();
        let next = last[1..].iter().filter_map(|f| f.time_ms.map(|o| (o - t, f.id.clone()))).filter(|(d, _)| *d > 0).min();
        if let Some((d, other)) = next.filter(|(d, _)| *d < 2_000) {
            let tenths = (d + 99) / 100;
            self.moments.push(&by, format!("Photo finish: {{{other}}} beaten by {}.{} s", tenths / 10, tenths % 10));
        }
        self.firsts += 1;
        if self.firsts == self.race.round_ms.len() {
            self.moments.push(&by, "Clean sweep: first in all three rounds");
        }
    }

    fn load(&mut self) {
        if !self.race.over {
            self.figure = generate(self.race.round, &mut Rng::new(self.seed.wrapping_add(self.race.round as u64 * 1_000_003)));
            let f = &self.figure;
            self.cells = (0..f.solution.len()).map(|i| if f.given[i] { f.solution[i] } else { 0 }).collect();
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SymmetryMove {
    Paint { cell: usize, colour: u8 },
    GiveUp,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymmetryView {
    #[serde(flatten)]
    pub race: RaceView,
    pub size: usize,
    pub axis: Axis,
    pub colours: u8,
    pub cells: Vec<u8>,
    pub given: Vec<bool>,
    pub over: bool,
}

pub struct Rules;

impl Game for Rules {
    type State = SymmetryState;
    type Move = SymmetryMove;
    type View = SymmetryView;

    fn info() -> GameInfo {
        GameInfo {
            id: "symmetry-puzzle",
            kind: "exile",
            difficulty: "medium",
            tags: &["logic", "speed"],
            min_players: 1,
            max_players: 4,
            name: "Symmetry Puzzle",
            short: "Paint the missing half so the figure mirrors itself across the axis, faster than the others.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<SymmetryState, String> {
        let race = Race::new(players, seed, &ROUND_MS)?;
        let mut s =
            SymmetryState { race, figure: generate(0, &mut Rng::new(seed)), cells: Vec::new(), seed, firsts: 0, moments: Moments::default() };
        s.load();
        Ok(s)
    }

    fn apply_move(s: &mut SymmetryState, player: &str, mv: SymmetryMove) -> Result<(), String> {
        s.race.check_move(player)?;
        match mv {
            SymmetryMove::Paint { cell, colour } => {
                if cell >= s.cells.len() || colour > s.figure.colours {
                    return Err("Illegal move".into());
                }
                if s.figure.given[cell] {
                    return Err("That cell is given".into());
                }
                s.cells[cell] = colour;
                if s.cells == s.figure.solution {
                    s.race.solver_done(true);
                    s.note_solve();
                    s.load();
                }
            }
            SymmetryMove::GiveUp => {
                s.race.solver_done(false);
                s.load();
            }
        }
        Ok(())
    }

    fn tick(s: &mut SymmetryState, ms: u64) {
        if s.race.tick(ms) {
            s.load();
        }
    }

    fn view_for(s: &SymmetryState, _player: &str) -> SymmetryView {
        SymmetryView {
            race: s.race.view(),
            size: s.figure.size,
            axis: s.figure.axis,
            colours: s.figure.colours,
            cells: s.cells.clone(),
            given: s.figure.given.clone(),
            over: s.race.over,
        }
    }

    fn is_over(s: &SymmetryState) -> bool {
        s.race.over
    }

    fn results(s: &SymmetryState) -> Vec<PlayerResult> {
        s.race.results()
    }

    fn to_move(s: &SymmetryState) -> Vec<String> {
        s.race.to_move()
    }

    fn bot_move(s: &SymmetryState, _player: &str, seed: u64) -> SymmetryMove {
        let mut rng = Rng::new(seed);
        match (0..s.cells.len()).find(|&i| s.cells[i] != s.figure.solution[i]) {
            // Now and then a slip that it has to fix later.
            Some(cell) if rng.next_f64() < 0.1 => SymmetryMove::Paint { cell, colour: rng.int(0, i64::from(s.figure.colours)) as u8 },
            Some(cell) => SymmetryMove::Paint { cell, colour: s.figure.solution[cell] },
            None => SymmetryMove::GiveUp,
        }
    }

    fn moments(s: &SymmetryState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub const READY: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::{bot_games, bot_games_with};

    #[test]
    fn figures_are_symmetric_and_deducible() {
        for seed in 0..200 {
            for round in 0..3 {
                let f = generate(round, &mut Rng::new(seed));
                let n = f.size;
                assert!(is_symmetric(n, f.axis, &f.solution));
                // Every coloured cell has a given cell among its mirror images, and
                // the given cells alone are not symmetric yet.
                for i in 0..n * n {
                    if f.solution[i] > 0 {
                        assert!(orbit(n, f.axis, i).iter().any(|&j| f.given[j]), "seed {seed} round {round}");
                    } else {
                        assert!(!f.given[i]);
                    }
                }
                let shown: Vec<u8> = (0..n * n).map(|i| if f.given[i] { f.solution[i] } else { 0 }).collect();
                assert!(!is_symmetric(n, f.axis, &shown));
            }
        }
    }

    #[test]
    fn orbits() {
        assert_eq!(orbit(6, Axis::Vertical, 0), vec![0, 5]);
        assert_eq!(orbit(7, Axis::Diagonal, 1), vec![1, 7]);
        assert_eq!(orbit(8, Axis::Both, 0), vec![0, 7, 56, 63]);
    }

    #[test]
    fn painting_completes_the_round() {
        let players = vec!["p1".to_string(), "p2".to_string()];
        let mut s = Rules::setup(&players, 5).unwrap();
        let given = s.figure.given.iter().position(|&g| g).unwrap();
        assert!(Rules::apply_move(&mut s, "p1", SymmetryMove::Paint { cell: given, colour: 0 }).is_err());
        assert!(Rules::apply_move(&mut s, "p1", SymmetryMove::Paint { cell: 0, colour: 9 }).is_err());
        let sol = s.figure.solution.clone();
        // An extra cell: symmetric or not, it is not the figure.
        let extra = (0..sol.len()).find(|&i| sol[i] == 0).unwrap();
        Rules::apply_move(&mut s, "p1", SymmetryMove::Paint { cell: extra, colour: 1 }).unwrap();
        for (i, &colour) in sol.iter().enumerate() {
            if !s.figure.given[i] && colour > 0 {
                Rules::apply_move(&mut s, "p1", SymmetryMove::Paint { cell: i, colour }).unwrap();
            }
        }
        assert_eq!(s.race.round, 0);
        Rules::apply_move(&mut s, "p1", SymmetryMove::Paint { cell: extra, colour: 0 }).unwrap();
        assert_eq!(s.race.round, 1);
        assert!(s.race.scores[0] >= 140);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<Rules>(1000, 5000);
        bot_games_with::<Rules>(1000, 4, 5000);
    }

    fn solve(s: &mut SymmetryState) {
        let round = s.race.round;
        while s.race.round == round {
            let cell = (0..s.cells.len()).find(|&i| s.cells[i] != s.figure.solution[i]).unwrap();
            Rules::apply_move(s, "p1", SymmetryMove::Paint { cell, colour: s.figure.solution[cell] }).unwrap();
        }
    }

    #[test]
    fn a_photo_finish_and_a_clean_sweep_are_replay_moments() {
        let players: Vec<String> = (1..=4).map(|i| format!("p{i}")).collect();
        let mut s = Rules::setup(&players, 7).unwrap();
        // Find when the first racer finishes (to 0.1 s), then solve 1.5 s before it.
        let mut probe = s.clone();
        while probe.race.view().racers.iter().all(|r| !r.done) {
            Rules::tick(&mut probe, 100);
        }
        let first = probe.race.view().racers.iter().find(|r| r.done).unwrap().id.clone();
        Rules::tick(&mut s, (probe.race.elapsed_ms - 1_500) as u64);
        solve(&mut s);
        let m = Rules::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].by, "p1");
        assert_eq!(m[0].text, format!("Photo finish: {{{first}}} beaten by 1.5 s"));
        // Solved at once: first again, with no racer close, so no moment until the sweep.
        solve(&mut s);
        assert_eq!(Rules::moments(&s, "p1").len(), 1);
        solve(&mut s);
        let m = Rules::moments(&s, "p1");
        assert_eq!(m.len(), 2);
        assert_eq!(m[1].text, "Clean sweep: first in all three rounds");
    }

    #[test]
    fn a_second_place_is_no_sweep() {
        let players: Vec<String> = (1..=4).map(|i| format!("p{i}")).collect();
        let mut s = Rules::setup(&players, 7).unwrap();
        Rules::tick(&mut s, 85_000);
        solve(&mut s);
        solve(&mut s);
        solve(&mut s);
        assert!(s.race.over);
        assert!(Rules::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

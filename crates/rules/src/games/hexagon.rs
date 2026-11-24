//! Hexagon (Final duel, played solo here). Rules page: docs/games/hexagon.md
//!
//! 19 numbered cells form a hexagon. You may study the numbers for 30 seconds,
//! then they turn face down (only the letters A–S remain). Five rounds follow:
//! each shows a target; tap three cells in a straight line, next to each other,
//! whose numbers add up to the target.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult};
use crate::rng::Rng;

pub const STUDY_MS: i64 = 30_000;
pub const ROUNDS: usize = 5;

/// Axial coordinates (q, r) of the 19 cells, top row first, left to right.
pub fn cells() -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for r in -2..=2 {
        for q in (-2).max(-2 - r)..=2.min(2 - r) {
            out.push((q, r));
        }
    }
    out
}

/// Every three cells in a straight line, next to each other (sorted indices).
pub fn lines() -> Vec<[usize; 3]> {
    let cs = cells();
    let at = |q: i32, r: i32| cs.iter().position(|&c| c == (q, r));
    let mut out = Vec::new();
    for (i, &(q, r)) in cs.iter().enumerate() {
        for (dq, dr) in [(1, 0), (0, 1), (1, -1)] {
            if let (Some(j), Some(k)) = (at(q + dq, r + dr), at(q + 2 * dq, r + 2 * dr)) {
                let mut l = [i, j, k];
                l.sort();
                out.push(l);
            }
        }
    }
    out
}

#[derive(Clone, Debug)]
pub struct HexRound {
    pub target: i64,
    pub line: [usize; 3],
    pub given: Option<Option<[usize; 3]>>,
    pub correct: bool,
    pub points: i64,
}

#[derive(Clone, Debug)]
pub struct HexState {
    pub player: String,
    pub numbers: Vec<i64>,
    pub studying: bool,
    pub rounds: Vec<HexRound>,
    pub current: usize,
    pub time_left_ms: i64,
    pub over: bool,
    /// Replay moments: all five lines right.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HexMove {
    /// Ends the study time early.
    Ready,
    Answer {
        cells: [usize; 3],
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HexHistory {
    pub target: i64,
    pub given: Option<[usize; 3]>,
    pub solution: [usize; 3],
    pub correct: bool,
    pub points: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HexView {
    /// Numbers only while studying and after the game; null otherwise.
    pub numbers: Option<Vec<i64>>,
    pub studying: bool,
    pub round: usize,
    pub rounds: usize,
    pub target: Option<i64>,
    pub time_left_ms: i64,
    pub phase_time_ms: i64,
    pub history: Vec<HexHistory>,
    pub score: i64,
    pub over: bool,
}

impl HexState {
    fn is_correct(&self, given: &[usize; 3], target: i64) -> bool {
        let mut g = *given;
        g.sort();
        lines().contains(&g) && g.iter().map(|&i| self.numbers[i]).sum::<i64>() == target
    }

    fn finish_round(&mut self, given: Option<[usize; 3]>) {
        let correct = given.is_some_and(|g| self.is_correct(&g, self.rounds[self.current].target));
        let points = if correct { 100 } else { 0 };
        let round = &mut self.rounds[self.current];
        round.given = Some(given);
        round.correct = correct;
        round.points = points;
        self.current += 1;
        self.over = self.current >= self.rounds.len();
    }

    pub fn score(&self) -> i64 {
        self.rounds.iter().map(|r| r.points).sum()
    }
}

pub struct Hexagon;

impl Game for Hexagon {
    type State = HexState;
    type Move = HexMove;
    type View = HexView;

    fn info() -> GameInfo {
        GameInfo {
            id: "hexagon",
            kind: "final-duel",
            difficulty: "medium",
            tags: &["memory", "numbers"],
            min_players: 1,
            max_players: 1,
            name: "Hexagon",
            short: "Memorise 19 numbers, then find lines of three that add up to the target, blind.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<HexState, String> {
        let [player] = players else { return Err("Hexagon is a solo game here".into()) };
        let mut rng = Rng::new(seed);
        let numbers: Vec<i64> = (0..cells().len()).map(|_| rng.int(1, 9)).collect();
        let mut all = lines();
        rng.shuffle(&mut all);
        // Different targets each round, where possible.
        let mut rounds: Vec<HexRound> = Vec::new();
        for line in all {
            let target = line.iter().map(|&i| numbers[i]).sum();
            if rounds.len() < ROUNDS && rounds.iter().all(|r| r.target != target) {
                rounds.push(HexRound { target, line, given: None, correct: false, points: 0 });
            }
        }
        let mut more = lines().into_iter().cycle();
        while rounds.len() < ROUNDS {
            let line = more.next().expect("lines");
            let target = line.iter().map(|&i| numbers[i]).sum();
            rounds.push(HexRound { target, line, given: None, correct: false, points: 0 });
        }
        Ok(HexState {
            player: player.clone(),
            numbers,
            studying: true,
            rounds,
            current: 0,
            time_left_ms: STUDY_MS,
            over: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut HexState, player: &str, mv: HexMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        if player != s.player {
            return Err("Not your game".into());
        }
        match mv {
            HexMove::Ready if s.studying => {
                s.studying = false;
                s.time_left_ms = 0;
                Ok(())
            }
            HexMove::Answer { cells: given } if !s.studying => {
                if given.iter().any(|&i| i >= s.numbers.len()) || given[0] == given[1] || given[1] == given[2] || given[0] == given[2] {
                    return Err("Pick three different cells".into());
                }
                s.finish_round(Some(given));
                // A replay moment, after the last round: every line right.
                if s.over && s.rounds.iter().all(|r| r.correct) {
                    s.moments.push(&s.player, "All five lines from memory");
                }
                Ok(())
            }
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut HexState, ms: u64) {
        if s.over || !s.studying {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.studying = false;
            s.time_left_ms = 0;
        }
    }

    fn view_for(s: &HexState, _player: &str) -> HexView {
        HexView {
            numbers: (s.studying || s.over).then(|| s.numbers.clone()),
            studying: s.studying,
            round: (s.current + 1).min(ROUNDS),
            rounds: ROUNDS,
            target: (!s.studying && !s.over).then(|| s.rounds[s.current].target),
            time_left_ms: s.time_left_ms,
            phase_time_ms: STUDY_MS,
            history: s
                .rounds
                .iter()
                .filter_map(|r| {
                    r.given.map(|given| HexHistory { target: r.target, given, solution: r.line, correct: r.correct, points: r.points })
                })
                .collect(),
            score: s.score(),
            over: s.over,
        }
    }

    fn is_over(s: &HexState) -> bool {
        s.over
    }

    fn results(s: &HexState) -> Vec<PlayerResult> {
        vec![PlayerResult { player_id: s.player.clone(), rank: 1, points: s.score() }]
    }

    fn bot_move(s: &HexState, _player: &str, seed: u64) -> HexMove {
        let mut rng = Rng::new(seed);
        if s.studying {
            return HexMove::Ready;
        }
        let line = s.rounds[s.current].line;
        // Remembers the board 60% of the time, otherwise guesses a line.
        if rng.next_f64() < 0.6 { HexMove::Answer { cells: line } } else { HexMove::Answer { cells: *rng.pick(&lines()) } }
    }

    fn moments(s: &HexState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub const READY: bool = true;
pub type Rules = Hexagon;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games;

    const P: &str = "p1";

    #[test]
    fn board_has_19_cells_and_all_lines() {
        assert_eq!(cells().len(), 19);
        // Per direction: rows of 3,4,5,4,3 cells give 1+2+3+2+1 = 9 lines.
        assert_eq!(lines().len(), 27);
    }

    #[test]
    fn numbers_hide_after_studying() {
        let mut s = Hexagon::setup(&[P.into()], 1).unwrap();
        assert!(Hexagon::view_for(&s, P).numbers.is_some());
        assert!(Hexagon::view_for(&s, P).target.is_none());
        Hexagon::apply_move(&mut s, P, HexMove::Ready).unwrap();
        let v = Hexagon::view_for(&s, P);
        assert!(v.numbers.is_none());
        assert!(v.target.is_some());
        assert!(serde_json::to_string(&v).unwrap().find("\"line\"").is_none());
    }

    #[test]
    fn study_time_runs_out_by_itself() {
        let mut s = Hexagon::setup(&[P.into()], 2).unwrap();
        Hexagon::tick(&mut s, STUDY_MS as u64);
        assert!(!s.studying);
        // No round clock: the target waits for an answer.
        Hexagon::tick(&mut s, 600_000);
        assert_eq!(s.current, 0);
        assert!(!s.over);
    }

    #[test]
    fn scores_a_line_in_any_order_but_not_a_bent_one() {
        let mut s = Hexagon::setup(&[P.into()], 3).unwrap();
        Hexagon::apply_move(&mut s, P, HexMove::Ready).unwrap();
        let [a, b, c] = s.rounds[0].line;
        Hexagon::apply_move(&mut s, P, HexMove::Answer { cells: [c, a, b] }).unwrap();
        assert!(s.rounds[0].correct);
        // A, B, E are neighbours but not in a straight line.
        assert!(!s.is_correct(&[0, 1, 4], s.numbers[0] + s.numbers[1] + s.numbers[4]));
        assert!(Hexagon::apply_move(&mut s, P, HexMove::Answer { cells: [1, 1, 2] }).is_err());
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<Hexagon>(1000, 200);
    }

    #[test]
    fn a_perfect_run_is_a_replay_moment() {
        let right = |s: &mut HexState| {
            let line = s.rounds[s.current].line;
            Hexagon::apply_move(s, P, HexMove::Answer { cells: line }).unwrap();
        };
        // Five right answers, each worth a flat 100.
        let mut s = Hexagon::setup(&[P.into()], 4).unwrap();
        Hexagon::apply_move(&mut s, P, HexMove::Ready).unwrap();
        for _ in 0..4 {
            right(&mut s);
            assert!(Hexagon::moments(&s, P).is_empty(), "only the end makes moments");
        }
        right(&mut s);
        assert_eq!(s.score(), 500);
        let m = Hexagon::moments(&s, P);
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), (P, "All five lines from memory"));
        // A wrong first answer (A, B, E is never a straight line): nothing.
        let mut s = Hexagon::setup(&[P.into()], 4).unwrap();
        Hexagon::apply_move(&mut s, P, HexMove::Ready).unwrap();
        Hexagon::apply_move(&mut s, P, HexMove::Answer { cells: [0, 1, 4] }).unwrap();
        for _ in 0..4 {
            right(&mut s);
        }
        assert!(s.over);
        assert_eq!(s.score(), 400);
        assert!(Hexagon::moments(&s, P).is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Hexagon>(300, 1, 200);
        // One moment only (a perfect run): about 1 bot game in 10.
        assert!(stats.with_any * 20 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 1, "too many in one game: {stats:?}");
    }
}

//! Weight Balance Puzzle (Team mini game, 1–2 players). Rules page: docs/games/weight-balance-puzzle.md
//!
//! Five blocks with different secret weights 1–9 g. The team shares 12
//! weighings on a balance scale (blocks plus counterweights on each pan), then
//! every player writes down the five weights in secret.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult};
use crate::rng::Rng;

pub const BLOCKS: usize = 5;
pub const MAX_WEIGHT: u32 = 9;
pub const WEIGHINGS: u32 = 12;
pub const MAX_EXTRA: u32 = 20;
pub const TURN_MS: i64 = 40_000;
pub const ANSWER_MS: i64 = 120_000;
pub const BLOCK_POINTS: i64 = 150;
pub const SPARE_POINTS: i64 = 50;
/// The bot judges a weighing on at most this many of the weight sets still possible.
const SAMPLE: usize = 120;
/// … and tries this many of its candidate weighings (cheap enough for slow phones).
const LOADS_TRIED: usize = 80;

pub type Weights = [u32; BLOCKS];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Load {
    pub left: Vec<usize>,
    pub right: Vec<usize>,
    pub left_extra: u32,
    pub right_extra: u32,
}

impl Load {
    /// 1 = left heavier, -1 = right heavier, 0 = balanced.
    pub fn outcome(&self, w: &Weights) -> i8 {
        let l: u32 = self.left.iter().map(|&i| w[i]).sum::<u32>() + self.left_extra;
        let r: u32 = self.right.iter().map(|&i| w[i]).sum::<u32>() + self.right_extra;
        (l.cmp(&r)) as i8
    }

    /// The load as left-minus-right factors per block and counterweight difference.
    fn factors(&self) -> ([i32; BLOCKS], i32) {
        let mut f = [0; BLOCKS];
        for &i in &self.left {
            f[i] += 1;
        }
        for &i in &self.right {
            f[i] -= 1;
        }
        (f, self.left_extra as i32 - self.right_extra as i32)
    }

    fn check(&self) -> Result<(), String> {
        let mut seen = [false; BLOCKS];
        for &i in self.left.iter().chain(&self.right) {
            if i >= BLOCKS || seen[i] {
                return Err("Each block can be on one pan once".into());
            }
            seen[i] = true;
        }
        if self.left_extra > MAX_EXTRA || self.right_extra > MAX_EXTRA {
            return Err("Counterweights go up to 20 g".into());
        }
        if self.left.is_empty() && self.right.is_empty() {
            return Err("Put a block on the scale".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Weighing {
    pub seat: usize,
    /// None when the turn ran out.
    pub load: Option<Load>,
    pub result: i8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Weigh,
    Answer,
    Over,
}

#[derive(Clone, Debug)]
pub struct State {
    pub players: Vec<String>,
    pub weights: Weights,
    pub phase: Phase,
    pub turn: usize,
    pub log: Vec<Weighing>,
    pub answers: Vec<Option<Weights>>,
    pub time_left_ms: i64,
    /// Every weight set that fits the log so far (public knowledge, kept for the bot).
    pub possible: Vec<Weights>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Weigh {
        #[serde(flatten)]
        load: Load,
    },
    Answer,
    Submit {
        weights: Weights,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: Vec<String>,
    pub you: Option<usize>,
    pub phase: Phase,
    pub turn: usize,
    pub your_turn: bool,
    pub weighings_left: u32,
    pub weighings: u32,
    pub log: Vec<Weighing>,
    pub max_weight: u32,
    pub max_extra: u32,
    pub your_answer: Option<Weights>,
    pub answered: Vec<bool>,
    pub time_left_ms: i64,
    pub phase_ms: i64,
    pub score: i64,
    /// Only at the end.
    pub weights: Option<Weights>,
    pub answers: Option<Vec<Option<Weights>>>,
}

/// The scale's result for load factors `f`, counterweight difference `d` and weights `w`.
fn tilt(f: &[i32; BLOCKS], d: i32, w: &Weights) -> i8 {
    let diff = d + f[0] * w[0] as i32 + f[1] * w[1] as i32 + f[2] * w[2] as i32 + f[3] * w[3] as i32 + f[4] * w[4] as i32;
    diff.signum() as i8
}

static ALL_SETS: std::sync::OnceLock<Vec<Weights>> = std::sync::OnceLock::new();

/// Every set of five different weights 1–9.
pub fn all_weight_sets() -> Vec<Weights> {
    let mut out = Vec::new();
    let mut cur = [0u32; BLOCKS];
    fn rec(k: usize, cur: &mut Weights, used: &mut [bool; 10], out: &mut Vec<Weights>) {
        if k == BLOCKS {
            out.push(*cur);
            return;
        }
        for v in 1..=MAX_WEIGHT {
            if !used[v as usize] {
                used[v as usize] = true;
                cur[k] = v;
                rec(k + 1, cur, used, out);
                used[v as usize] = false;
            }
        }
    }
    rec(0, &mut cur, &mut [false; 10], &mut out);
    out
}

/// The weighings the bot considers.
fn candidate_loads() -> Vec<Load> {
    let mut out = Vec::new();
    for i in 0..BLOCKS {
        for r in 2..MAX_WEIGHT {
            out.push(Load { left: vec![i], right: vec![], left_extra: 0, right_extra: r });
        }
        for j in 0..BLOCKS {
            if i != j {
                for r in 0..6 {
                    out.push(Load { left: vec![i], right: vec![j], left_extra: 0, right_extra: r });
                }
            }
            if i < j {
                for r in 5..=15 {
                    out.push(Load { left: vec![i, j], right: vec![], left_extra: 0, right_extra: r });
                }
            }
        }
    }
    out
}

/// The load whose worst result leaves the fewest weight sets.
pub fn best_load(possible: &[Weights], seed: u64) -> Load {
    let mut rng = Rng::new(seed);
    let step = possible.len().div_ceil(SAMPLE).max(1);
    let offset = rng.index(step);
    let sample: Vec<&Weights> = possible.iter().skip(offset).step_by(step).collect();
    let mut loads = candidate_loads();
    rng.shuffle(&mut loads);
    let mut best: Option<(f64, Load)> = None;
    for load in loads.into_iter().take(LOADS_TRIED) {
        let (f, d) = load.factors();
        let mut counts = [0usize; 3];
        for w in &sample {
            counts[(tilt(&f, d, w) + 1) as usize] += 1;
        }
        let worst = *counts.iter().max().unwrap_or(&0) as f64 + rng.next_f64() * 0.5;
        if best.as_ref().is_none_or(|(b, _)| worst < *b) {
            best = Some((worst, load));
        }
    }
    best.map(|(_, l)| l).unwrap_or(Load { left: vec![0], right: vec![], left_extra: 0, right_extra: 5 })
}

impl State {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }

    pub fn weighings_left(&self) -> u32 {
        WEIGHINGS.saturating_sub(self.log.len() as u32)
    }

    fn next_turn(&mut self) {
        self.turn = (self.turn + 1) % self.players.len();
        self.time_left_ms = TURN_MS;
        if self.weighings_left() == 0 {
            self.start_answers();
        }
    }

    fn start_answers(&mut self) {
        self.phase = Phase::Answer;
        self.time_left_ms = ANSWER_MS;
    }

    fn weigh(&mut self, seat: usize, load: Option<Load>) {
        let result = load.as_ref().map_or(0, |l| l.outcome(&self.weights));
        if let Some(l) = &load {
            let (f, d) = l.factors();
            self.possible.retain(|w| tilt(&f, d, w) == result);
        }
        self.log.push(Weighing { seat, load, result });
        self.next_turn();
    }

    fn end(&mut self) {
        self.phase = Phase::Over;
        self.time_left_ms = 0;
    }

    pub fn score(&self) -> i64 {
        if self.phase != Phase::Over {
            return 0;
        }
        let right = (0..BLOCKS).filter(|&b| self.answers.iter().all(|a| a.is_some_and(|a| a[b] == self.weights[b]))).count();
        let mut pts = right as i64 * BLOCK_POINTS;
        if right == BLOCKS {
            pts += i64::from(self.weighings_left()) * SPARE_POINTS;
        }
        pts
    }
}

pub struct WeightBalancePuzzle;

impl Game for WeightBalancePuzzle {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "weight-balance-puzzle",
            kind: "mini-team",
            difficulty: "hard",
            min_players: 1,
            max_players: 2,
            name: "Weight Balance Puzzle",
            short: "Share 12 weighings on one scale to find the weights of five blocks.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if !(1..=2).contains(&players.len()) {
            return Err("Weight Balance Puzzle is for 1–2 players".into());
        }
        let mut rng = Rng::new(seed);
        let mut pool: Vec<u32> = (1..=MAX_WEIGHT).collect();
        rng.shuffle(&mut pool);
        let weights = [pool[0], pool[1], pool[2], pool[3], pool[4]];
        Ok(State {
            players: players.to_vec(),
            weights,
            phase: Phase::Weigh,
            turn: 0,
            log: Vec::new(),
            answers: vec![None; players.len()],
            time_left_ms: TURN_MS,
            possible: ALL_SETS.get_or_init(all_weight_sets).clone(),
        })
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        let seat = s.seat(player).ok_or("Not in this game")?;
        match (s.phase, mv) {
            (Phase::Weigh, Move::Weigh { load }) => {
                if seat != s.turn {
                    return Err("Not your turn".into());
                }
                load.check()?;
                s.weigh(seat, Some(load));
            }
            (Phase::Weigh, Move::Answer) => {
                if seat != s.turn {
                    return Err("Not your turn".into());
                }
                s.start_answers();
            }
            (Phase::Answer, Move::Submit { weights }) => {
                if s.answers[seat].is_some() {
                    return Err("Already answered".into());
                }
                if weights.iter().any(|w| !(1..=MAX_WEIGHT).contains(w)) {
                    return Err("Weights are 1–9".into());
                }
                s.answers[seat] = Some(weights);
                if s.answers.iter().all(Option::is_some) {
                    s.end();
                }
            }
            (Phase::Over, _) => return Err("Game is over".into()),
            _ => return Err("Not now".into()),
        }
        Ok(())
    }

    fn tick(s: &mut State, ms: u64) {
        if s.phase == Phase::Over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms > 0 {
            return;
        }
        match s.phase {
            Phase::Weigh => s.weigh(s.turn, None),
            _ => s.end(),
        }
    }

    fn view_for(s: &State, player: &str) -> View {
        let you = s.seat(player);
        let over = s.phase == Phase::Over;
        View {
            players: s.players.clone(),
            you,
            phase: s.phase,
            turn: s.turn,
            your_turn: s.phase == Phase::Weigh && you == Some(s.turn),
            weighings_left: s.weighings_left(),
            weighings: WEIGHINGS,
            log: s.log.clone(),
            max_weight: MAX_WEIGHT,
            max_extra: MAX_EXTRA,
            your_answer: you.and_then(|k| s.answers[k]),
            answered: s.answers.iter().map(Option::is_some).collect(),
            time_left_ms: s.time_left_ms,
            phase_ms: if s.phase == Phase::Answer { ANSWER_MS } else { TURN_MS },
            score: s.score(),
            weights: over.then_some(s.weights),
            answers: over.then(|| s.answers.clone()),
        }
    }

    fn is_over(s: &State) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        let pts = s.score();
        s.players.iter().map(|p| PlayerResult { player_id: p.clone(), rank: 1, points: pts }).collect()
    }

    fn to_move(s: &State) -> Vec<String> {
        match s.phase {
            Phase::Weigh => vec![s.players[s.turn].clone()],
            Phase::Answer => (0..s.players.len()).filter(|&k| s.answers[k].is_none()).map(|k| s.players[k].clone()).collect(),
            Phase::Over => Vec::new(),
        }
    }

    fn bot_move(s: &State, _player: &str, seed: u64) -> Move {
        match s.phase {
            Phase::Weigh if s.possible.len() <= 1 => Move::Answer,
            Phase::Weigh => Move::Weigh { load: best_load(&s.possible, seed) },
            _ => {
                let mut rng = Rng::new(seed);
                let weights = if s.possible.is_empty() { [1, 2, 3, 4, 5] } else { *rng.pick(&s.possible) };
                Move::Submit { weights }
            }
        }
    }
}

pub const READY: bool = true;
pub type Rules = WeightBalancePuzzle;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn duo() -> Vec<String> {
        vec!["p1".into(), "p2".into()]
    }

    #[test]
    fn weight_sets_are_all_different_numbers() {
        let all = all_weight_sets();
        assert_eq!(all.len(), 9 * 8 * 7 * 6 * 5);
        let s = WeightBalancePuzzle::setup(&duo(), 3).unwrap();
        assert!(all.contains(&s.weights));
    }

    #[test]
    fn moves_arrive_as_json() {
        let mv: Move = serde_json::from_value(serde_json::json!({
            "type": "weigh", "left": [0, 2], "right": [1], "leftExtra": 0, "rightExtra": 4
        }))
        .unwrap();
        assert!(matches!(mv, Move::Weigh { load } if load.left == vec![0, 2] && load.right_extra == 4));
    }

    #[test]
    fn the_scale_compares_pans() {
        let w = [3, 1, 9, 4, 7];
        let load = |left: Vec<usize>, right: Vec<usize>, le, re| Load { left, right, left_extra: le, right_extra: re };
        assert_eq!(load(vec![0], vec![1], 0, 0).outcome(&w), 1);
        assert_eq!(load(vec![0, 1], vec![3], 0, 0).outcome(&w), 0);
        assert_eq!(load(vec![2], vec![], 0, 10).outcome(&w), -1);
        assert_eq!(load(vec![], vec![4], 7, 0).outcome(&w), 0);
        assert!(load(vec![0], vec![0], 0, 0).check().is_err());
        assert!(load(vec![], vec![], 3, 0).check().is_err());
        assert!(load(vec![5], vec![], 0, 0).check().is_err());
    }

    #[test]
    fn turns_alternate_and_time_out_loses_a_weighing() {
        let mut s = WeightBalancePuzzle::setup(&duo(), 1).unwrap();
        let load = Load { left: vec![0], right: vec![1], left_extra: 0, right_extra: 0 };
        assert!(WeightBalancePuzzle::apply_move(&mut s, "p2", Move::Weigh { load: load.clone() }).is_err());
        WeightBalancePuzzle::apply_move(&mut s, "p1", Move::Weigh { load }).unwrap();
        assert_eq!(s.turn, 1);
        WeightBalancePuzzle::tick(&mut s, TURN_MS as u64);
        assert_eq!(s.turn, 0);
        assert_eq!(s.weighings_left(), WEIGHINGS - 2);
        assert!(s.log[1].load.is_none());
    }

    #[test]
    fn every_block_must_be_right_for_everyone() {
        let mut s = WeightBalancePuzzle::setup(&duo(), 2).unwrap();
        WeightBalancePuzzle::apply_move(&mut s, "p1", Move::Answer).unwrap();
        let right = s.weights;
        let mut wrong = right;
        wrong[0] = if right[0] == 1 { 2 } else { 1 };
        WeightBalancePuzzle::apply_move(&mut s, "p1", Move::Submit { weights: right }).unwrap();
        assert!(WeightBalancePuzzle::apply_move(&mut s, "p1", Move::Submit { weights: right }).is_err());
        WeightBalancePuzzle::apply_move(&mut s, "p2", Move::Submit { weights: wrong }).unwrap();
        assert_eq!(s.phase, Phase::Over);
        assert_eq!(s.score(), 4 * BLOCK_POINTS);
        let r = WeightBalancePuzzle::results(&s);
        assert!(r.iter().all(|r| r.rank == 1 && r.points == 4 * BLOCK_POINTS));
    }

    #[test]
    fn answers_stay_secret_until_the_end() {
        let mut s = WeightBalancePuzzle::setup(&duo(), 4).unwrap();
        WeightBalancePuzzle::apply_move(&mut s, "p1", Move::Answer).unwrap();
        WeightBalancePuzzle::apply_move(&mut s, "p2", Move::Submit { weights: [1, 2, 3, 4, 5] }).unwrap();
        let v = WeightBalancePuzzle::view_for(&s, "p1");
        assert!(v.weights.is_none() && v.answers.is_none() && v.your_answer.is_none());
        assert_eq!(v.answered, vec![false, true]);
    }

    #[test]
    fn bots_find_the_weights_within_the_weighings() {
        let mut solved = 0;
        for seed in 0..30 {
            let mut s = WeightBalancePuzzle::setup(&duo(), seed).unwrap();
            let mut n = 0;
            while s.phase != Phase::Over {
                let p = WeightBalancePuzzle::to_move(&s)[0].clone();
                let mv = WeightBalancePuzzle::bot_move(&s, &p, seed * 31 + n);
                WeightBalancePuzzle::apply_move(&mut s, &p, mv).unwrap();
                n += 1;
            }
            if s.score() >= BLOCKS as i64 * BLOCK_POINTS {
                solved += 1;
            }
        }
        assert!(solved >= 27, "solved only {solved} of 30");
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<WeightBalancePuzzle>(1000, 1, 3000);
        bot_games_with::<WeightBalancePuzzle>(1000, 2, 3000);
    }
}

//! Hidden Rules Race (main game). Rules page: docs/games/hidden-rules-race.md
//!
//! Race a cube over a 7×7 board of five colours to the centre. Each colour has
//! a secret movement rule (step one, step two, backwards, turn right, turn
//! left); everyone picks a direction at once and learns from the results.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const SIZE: i32 = 7;
pub const GOAL: (i32, i32) = (3, 3);
pub const MAX_ROUNDS: u32 = 30;
pub const ROUND_MS: i64 = 20_000;
pub const REVEAL_MS: i64 = 15_000;
pub const CLUE_ROUNDS: [u32; 3] = [6, 12, 18];
pub const COLOURS: [&str; 5] = ["red", "blue", "green", "yellow", "purple"];
const STARTS: [(i32, i32); 8] = [(0, 0), (6, 0), (6, 6), (0, 6), (3, 0), (6, 3), (3, 6), (0, 3)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Dir {
    Up,
    Right,
    Down,
    Left,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::Up, Dir::Right, Dir::Down, Dir::Left];
    fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Right => (1, 0),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
        }
    }
    fn turned(self, quarters: usize) -> Dir {
        let i = Dir::ALL.iter().position(|&d| d == self).expect("dir");
        Dir::ALL[(i + quarters) % 4]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Effect {
    StepOne,
    StepTwo,
    Backwards,
    TurnRight,
    TurnLeft,
}

impl Effect {
    pub const ALL: [Effect; 5] = [Effect::StepOne, Effect::StepTwo, Effect::Backwards, Effect::TurnRight, Effect::TurnLeft];
    pub fn text(self) -> &'static str {
        match self {
            Effect::StepOne => "step one",
            Effect::StepTwo => "step two",
            Effect::Backwards => "backwards",
            Effect::TurnRight => "turn right",
            Effect::TurnLeft => "turn left",
        }
    }
}

/// Where a piece ends up: off the board means it stays.
pub fn moved(e: Effect, from: (i32, i32), d: Dir) -> (i32, i32) {
    let (dir, steps) = match e {
        Effect::StepOne => (d, 1),
        Effect::StepTwo => (d, 2),
        Effect::Backwards => (d.turned(2), 1),
        Effect::TurnRight => (d.turned(1), 1),
        Effect::TurnLeft => (d.turned(3), 1),
    };
    let (dx, dy) = dir.delta();
    let to = (from.0 + dx * steps, from.1 + dy * steps);
    if (0..SIZE).contains(&to.0) && (0..SIZE).contains(&to.1) { to } else { from }
}

pub fn distance(p: (i32, i32)) -> i64 {
    i64::from((p.0 - GOAL.0).abs() + (p.1 - GOAL.1).abs())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Choose,
    Reveal,
    Over,
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub id: String,
    pub pos: (i32, i32),
    pub choice: Option<Option<Dir>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub player: String,
    pub colour: usize,
    pub dir: Option<Dir>,
    pub from: (i32, i32),
    pub to: (i32, i32),
}

#[derive(Clone, Debug)]
pub struct HrrState {
    pub board: Vec<usize>,
    /// The secret rule of each colour.
    pub rules: [Effect; 5],
    pub pieces: Vec<Piece>,
    pub round: u32,
    pub phase: Phase,
    pub time_left_ms: i64,
    pub revealed: Vec<usize>,
    pub history: Vec<Vec<Step>>,
    pub rng: Rng,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HrrMove {
    Go { dir: Dir },
    Close,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceView {
    pub id: String,
    pub pos: (i32, i32),
    pub chosen: bool,
    pub distance: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownRule {
    pub colour: usize,
    pub rule: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HrrView {
    pub phase: &'static str,
    pub size: i32,
    pub goal: (i32, i32),
    pub colours: [&'static str; 5],
    pub rule_names: Vec<&'static str>,
    pub board: Vec<usize>,
    pub pieces: Vec<PieceView>,
    pub round: u32,
    pub max_rounds: u32,
    pub time_left_ms: i64,
    pub phase_time_ms: i64,
    pub known: Vec<KnownRule>,
    /// Newest round first, at most 12 rounds.
    pub history: Vec<Vec<Step>>,
    pub my_choice: Option<Dir>,
    pub score: i64,
}

/// Points by rank: 1000 for first down to 100 for last.
fn rank_points(scores: &[(String, i64)]) -> Vec<PlayerResult> {
    let n = scores.len() as i64;
    rank_by_score(scores)
        .into_iter()
        .map(|mut r| {
            r.points = if n > 1 { 100 + 900 * (n - i64::from(r.rank)) / (n - 1) } else { 1000 };
            r
        })
        .collect()
}

impl HrrState {
    fn colour_at(&self, p: (i32, i32)) -> usize {
        self.board[(p.1 * SIZE + p.0) as usize]
    }

    fn resolve(&mut self) {
        let mut steps = Vec::new();
        for i in 0..self.pieces.len() {
            let from = self.pieces[i].pos;
            let colour = self.colour_at(from);
            let dir = self.pieces[i].choice.flatten();
            let to = if from == GOAL { from } else { dir.map_or(from, |d| moved(self.rules[colour], from, d)) };
            self.pieces[i].pos = to;
            self.pieces[i].choice = None;
            steps.push(Step { player: self.pieces[i].id.clone(), colour, dir, from, to });
        }
        self.history.push(steps);
        if CLUE_ROUNDS.contains(&self.round) {
            let hidden: Vec<usize> = (0..5).filter(|c| !self.revealed.contains(c)).collect();
            if !hidden.is_empty() {
                let c = *self.rng.pick(&hidden);
                self.revealed.push(c);
            }
        }
        if self.round >= MAX_ROUNDS || self.pieces.iter().any(|p| p.pos == GOAL) {
            self.phase = Phase::Reveal;
            self.time_left_ms = REVEAL_MS;
        } else {
            self.round += 1;
            self.time_left_ms = ROUND_MS;
        }
    }
}

/// For every colour, the rules that fit the public history and clues.
pub fn candidates(s: &HrrState) -> Vec<Vec<Effect>> {
    let mut out: Vec<Vec<Effect>> = (0..5)
        .map(|c| if s.revealed.contains(&c) || s.phase != Phase::Choose { vec![s.rules[c]] } else { Effect::ALL.to_vec() })
        .collect();
    for step in s.history.iter().flatten() {
        if let Some(d) = step.dir
            && step.from != GOAL
        {
            out[step.colour].retain(|&e| moved(e, step.from, d) == step.to);
        }
    }
    // Each rule is on exactly one colour.
    loop {
        let fixed: Vec<(usize, Effect)> = (0..5).filter(|&c| out[c].len() == 1).map(|c| (c, out[c][0])).collect();
        let mut changed = false;
        for (c, set) in out.iter_mut().enumerate() {
            if set.len() > 1 {
                let before = set.len();
                set.retain(|e| !fixed.iter().any(|&(fc, fe)| fc != c && fe == *e));
                changed |= set.len() != before;
            }
        }
        if !changed {
            break;
        }
    }
    for set in &mut out {
        if set.is_empty() {
            *set = Effect::ALL.to_vec();
        }
    }
    out
}

pub struct Rules;

impl Game for Rules {
    type State = HrrState;
    type Move = HrrMove;
    type View = HrrView;

    fn info() -> GameInfo {
        GameInfo {
            id: "hidden-rules-race",
            kind: "main",
            difficulty: "hard",
            min_players: 4,
            max_players: 8,
            name: "Hidden Rules Race",
            short: "Race a cube to the centre of a coloured board while you work out each colour's secret movement rule.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<HrrState, String> {
        if players.len() < 4 || players.len() > 8 {
            return Err("Hidden Rules Race needs 4 to 8 players".into());
        }
        let mut rng = Rng::new(seed);
        let mut board: Vec<usize> = (0..(SIZE * SIZE) as usize).map(|i| i % 5).collect();
        rng.shuffle(&mut board);
        let mut rules = Effect::ALL;
        rng.shuffle(&mut rules);
        let pieces = players.iter().zip(STARTS).map(|(id, pos)| Piece { id: id.clone(), pos, choice: None }).collect();
        Ok(HrrState {
            board,
            rules,
            pieces,
            round: 1,
            phase: Phase::Choose,
            time_left_ms: ROUND_MS,
            revealed: Vec::new(),
            history: Vec::new(),
            rng,
        })
    }

    fn apply_move(s: &mut HrrState, player: &str, mv: HrrMove) -> Result<(), String> {
        let i = s.pieces.iter().position(|p| p.id == player).ok_or("Not in this game")?;
        match (s.phase, mv) {
            (Phase::Choose, HrrMove::Go { dir }) => {
                if s.pieces[i].choice.is_some() {
                    return Err("You have chosen this round".into());
                }
                s.pieces[i].choice = Some(Some(dir));
                if s.pieces.iter().all(|p| p.choice.is_some()) {
                    s.resolve();
                }
                Ok(())
            }
            (Phase::Reveal, HrrMove::Close) => {
                s.phase = Phase::Over;
                s.time_left_ms = 0;
                Ok(())
            }
            (Phase::Over, _) => Err("Game is over".into()),
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut HrrState, ms: u64) {
        if s.phase == Phase::Over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms > 0 {
            return;
        }
        match s.phase {
            Phase::Choose => {
                for p in &mut s.pieces {
                    p.choice.get_or_insert(None);
                }
                s.resolve();
            }
            _ => {
                s.phase = Phase::Over;
                s.time_left_ms = 0;
            }
        }
    }

    fn view_for(s: &HrrState, player: &str) -> HrrView {
        let all = s.phase != Phase::Choose;
        let known = (0..5).filter(|c| all || s.revealed.contains(c)).map(|c| KnownRule { colour: c, rule: s.rules[c].text() }).collect();
        let me = s.pieces.iter().find(|p| p.id == player);
        HrrView {
            phase: match s.phase {
                Phase::Choose => "choose",
                Phase::Reveal => "reveal",
                Phase::Over => "over",
            },
            size: SIZE,
            goal: GOAL,
            colours: COLOURS,
            rule_names: Effect::ALL.iter().map(|e| e.text()).collect(),
            board: s.board.clone(),
            pieces: s
                .pieces
                .iter()
                .map(|p| PieceView { id: p.id.clone(), pos: p.pos, chosen: p.choice.is_some(), distance: distance(p.pos) })
                .collect(),
            round: s.round,
            max_rounds: MAX_ROUNDS,
            time_left_ms: s.time_left_ms,
            phase_time_ms: if s.phase == Phase::Choose { ROUND_MS } else { REVEAL_MS },
            known,
            history: s.history.iter().rev().take(12).cloned().collect(),
            my_choice: me.and_then(|p| p.choice.flatten()),
            score: me.map_or(0, |p| (12 - distance(p.pos)) * 10),
        }
    }

    fn is_over(s: &HrrState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &HrrState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.pieces.iter().map(|p| (p.id.clone(), -distance(p.pos))).collect();
        rank_points(&scores)
    }

    fn to_move(s: &HrrState) -> Vec<String> {
        if s.phase != Phase::Choose {
            return Vec::new();
        }
        s.pieces.iter().filter(|p| p.choice.is_none()).map(|p| p.id.clone()).collect()
    }

    fn bot_move(s: &HrrState, player: &str, seed: u64) -> HrrMove {
        let mut rng = Rng::new(seed);
        let Some(me) = s.pieces.iter().find(|p| p.id == player) else { return HrrMove::Go { dir: Dir::Up } };
        if s.phase != Phase::Choose {
            return HrrMove::Close;
        }
        let options = &candidates(s)[s.colour_at(me.pos)];
        let expected = |d: Dir| options.iter().map(|&e| distance(moved(e, me.pos, d)) as f64).sum::<f64>() / options.len() as f64;
        let best = Dir::ALL.iter().map(|&d| expected(d)).fold(f64::MAX, f64::min);
        let good: Vec<Dir> = Dir::ALL.into_iter().filter(|&d| expected(d) <= best + 1e-9).collect();
        HrrMove::Go { dir: *rng.pick(&good) }
    }
}

pub const READY: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn movement_rules_and_walls() {
        assert_eq!(moved(Effect::StepTwo, (0, 0), Dir::Right), (2, 0));
        assert_eq!(moved(Effect::Backwards, (2, 2), Dir::Up), (2, 3));
        assert_eq!(moved(Effect::TurnRight, (2, 2), Dir::Up), (3, 2));
        assert_eq!(moved(Effect::TurnLeft, (2, 2), Dir::Up), (1, 2));
        assert_eq!(moved(Effect::StepOne, (0, 0), Dir::Up), (0, 0));
    }

    #[test]
    fn rules_stay_secret_until_the_end() {
        let mut s = Rules::setup(&players(4), 1).unwrap();
        let v = serde_json::to_value(Rules::view_for(&s, "p1")).unwrap();
        assert!(v["known"].as_array().unwrap().is_empty());
        for p in players(4) {
            Rules::apply_move(&mut s, &p, HrrMove::Go { dir: Dir::Down }).unwrap();
        }
        assert_eq!(s.round, 2);
        assert_eq!(s.history.len(), 1);
        s.round = MAX_ROUNDS;
        Rules::tick(&mut s, ROUND_MS as u64);
        assert_eq!(s.phase, Phase::Reveal);
        assert_eq!(Rules::view_for(&s, "p1").known.len(), 5);
        Rules::apply_move(&mut s, "p2", HrrMove::Close).unwrap();
        assert!(Rules::is_over(&s));
    }

    #[test]
    fn deduction_narrows_to_the_truth() {
        let mut s = Rules::setup(&players(4), 2).unwrap();
        for _ in 0..10 {
            for p in players(4) {
                let mv = Rules::bot_move(&s, &p, 7);
                Rules::apply_move(&mut s, &p, mv).unwrap();
            }
            if s.phase != Phase::Choose {
                break;
            }
        }
        let c = candidates(&s);
        for (colour, set) in c.iter().enumerate() {
            assert!(set.contains(&s.rules[colour]));
        }
    }

    #[test]
    fn bots_reach_the_goal_usually() {
        let mut finished = 0;
        for seed in 1..=50 {
            let mut s = Rules::setup(&players(4), seed).unwrap();
            while s.phase == Phase::Choose {
                for p in players(4) {
                    let mv = Rules::bot_move(&s, &p, seed);
                    let _ = Rules::apply_move(&mut s, &p, mv);
                }
            }
            finished += usize::from(s.pieces.iter().any(|p| p.pos == GOAL));
        }
        assert!(finished >= 40, "only {finished} of 50 races reached the goal");
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(1000, 4, 3000);
        bot_games_with::<Rules>(200, 8, 6000);
    }
}

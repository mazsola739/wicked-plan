//! Rules Race (main game). Rules page: docs/games/rules-race.md
//!
//! A race to square 18 with a die of 1, 2, 3, 3, prison and escape. Everyone
//! writes a secret personal rule (condition + effect) that fires on other
//! players' rolls; group rules can be switched by vote every third round.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const TRACK: i64 = 18;
pub const MAX_ROUNDS: u32 = 15;
pub const VOTE_EVERY: u32 = 3;

pub const BUMP: usize = 0;
pub const STRICT: usize = 1;
pub const EXACT: usize = 2;
pub const BIG_THREE: usize = 3;
pub const GROUP_RULES: [&str; 4] = [
    "Bump: landing on a square with other pieces sends them back 2.",
    "Strict prison: only an escape roll gets you out of prison.",
    "Exact finish: overshooting the goal bounces you back.",
    "Big three: a 3 moves 4.",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Cond {
    RollsOne,
    RollsThree,
    RollsPrison,
    LandsOnFour,
    PassesMe,
}

impl Cond {
    pub const ALL: [Cond; 5] = [Cond::RollsOne, Cond::RollsThree, Cond::RollsPrison, Cond::LandsOnFour, Cond::PassesMe];
    pub fn text(self) -> &'static str {
        match self {
            Cond::RollsOne => "another player rolls a 1",
            Cond::RollsThree => "another player rolls a 3",
            Cond::RollsPrison => "another player rolls prison",
            Cond::LandsOnFour => "another player lands on a multiple of 4",
            Cond::PassesMe => "another player passes me",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Effect {
    MeForward2,
    ThemBack2,
    ThemPrison,
    SwapIfAhead,
    MeEscapeCard,
}

impl Effect {
    pub const ALL: [Effect; 5] = [Effect::MeForward2, Effect::ThemBack2, Effect::ThemPrison, Effect::SwapIfAhead, Effect::MeEscapeCard];
    pub fn text(self) -> &'static str {
        match self {
            Effect::MeForward2 => "I move forward 2",
            Effect::ThemBack2 => "they move back 2",
            Effect::ThemPrison => "they go to prison",
            Effect::SwapIfAhead => "I swap places with them if they are ahead",
            Effect::MeEscapeCard => "I get an escape card",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Face {
    One,
    Two,
    Three,
    Prison,
    Escape,
}

pub const FACES: [Face; 6] = [Face::One, Face::Two, Face::Three, Face::Three, Face::Prison, Face::Escape];

impl Face {
    fn text(self) -> &'static str {
        match self {
            Face::One => "1",
            Face::Two => "2",
            Face::Three => "3",
            Face::Prison => "prison",
            Face::Escape => "escape",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Write,
    Turn,
    Vote,
    Over,
}

#[derive(Clone, Debug)]
pub struct Racer {
    pub id: String,
    pub pos: i64,
    /// Turns served, while in prison.
    pub prison: Option<u32>,
    pub escape: bool,
    pub rule: Option<(Cond, Effect)>,
    pub revealed: bool,
    pub vote: Option<Option<usize>>,
}

#[derive(Clone, Debug)]
pub struct RaceState {
    pub racers: Vec<Racer>,
    pub phase: Phase,
    pub turn: usize,
    pub round: u32,
    pub group: [bool; 4],
    pub rng: Rng,
    pub last_roll: Option<(usize, Face)>,
    pub log: Vec<String>,
    /// Replay moments: a rule that pushes a racer off the goal, swaps 6+ squares or wins the race.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RaceMove {
    Write { condition: Cond, effect: Effect },
    Roll,
    Vote { rule: Option<usize> },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleView {
    pub condition: Cond,
    pub effect: Effect,
    pub text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RacerView {
    pub id: String,
    pub pos: i64,
    pub in_prison: bool,
    pub escape: bool,
    pub rule: Option<RuleView>,
    pub written: bool,
    pub voted: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub id: String,
    pub text: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupView {
    pub text: &'static str,
    pub on: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceView {
    pub phase: &'static str,
    pub track: i64,
    pub round: u32,
    pub max_rounds: u32,
    pub racers: Vec<RacerView>,
    pub current: Option<String>,
    pub last_roll: Option<String>,
    pub group: Vec<GroupView>,
    pub conditions: Vec<Choice>,
    pub effects: Vec<Choice>,
    pub log: Vec<String>,
    pub score: i64,
}

fn rule_text(c: Cond, e: Effect) -> String {
    format!("When {}, {}.", c.text(), e.text())
}

fn id_of<T: Serialize>(v: T) -> String {
    serde_json::to_value(v).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default()
}

/// Points by rank: 1000 for first down to 100 for last.
pub fn rank_points(scores: &[(String, i64)]) -> Vec<PlayerResult> {
    let n = scores.len() as i64;
    rank_by_score(scores)
        .into_iter()
        .map(|mut r| {
            r.points = if n > 1 { 100 + 900 * (n - i64::from(r.rank)) / (n - 1) } else { 1000 };
            r
        })
        .collect()
}

impl RaceState {
    fn say(&mut self, line: String) {
        self.log.push(line);
        let extra = self.log.len().saturating_sub(12);
        self.log.drain(..extra);
    }

    fn name(&self, i: usize) -> String {
        self.racers[i].id.clone()
    }

    fn index(&self, player: &str) -> Option<usize> {
        self.racers.iter().position(|r| r.id == player)
    }

    /// Moves a piece forward, with Exact finish and Bump.
    fn advance(&mut self, i: usize, steps: i64) {
        let mut to = self.racers[i].pos + steps;
        if to > TRACK {
            to = if self.group[EXACT] { TRACK - (to - TRACK) } else { TRACK };
        }
        self.racers[i].pos = to;
        if self.group[BUMP] && to > 0 && to < TRACK {
            for j in 0..self.racers.len() {
                if j != i && self.racers[j].pos == to && self.racers[j].prison.is_none() {
                    self.racers[j].pos = (to - 2).max(0);
                    let line = format!("{} bumps {} back to {}.", self.name(i), self.name(j), self.racers[j].pos);
                    self.say(line);
                }
            }
        }
    }

    fn send_to_prison(&mut self, i: usize) {
        if self.racers[i].prison.is_some() {
            return;
        }
        if self.racers[i].escape {
            self.racers[i].escape = false;
            let line = format!("{} uses an escape card and stays out of prison.", self.name(i));
            self.say(line);
        } else {
            self.racers[i].prison = Some(0);
            let line = format!("{} goes to prison.", self.name(i));
            self.say(line);
        }
    }

    fn roll(&mut self) {
        let i = self.turn;
        let face = FACES[self.rng.index(FACES.len())];
        self.last_roll = Some((i, face));
        let before: Vec<i64> = self.racers.iter().map(|r| r.pos).collect();
        let line = format!("{} rolls {}.", self.name(i), face.text());
        self.say(line);
        let mut steps = 0;
        if let Some(served) = self.racers[i].prison {
            if face == Face::Escape {
                self.racers[i].prison = None;
                steps = 2;
                let line = format!("{} escapes from prison.", self.name(i));
                self.say(line);
            } else if !self.group[STRICT] && served + 1 >= 2 {
                self.racers[i].prison = None;
                let line = format!("{} has served the time and is free.", self.name(i));
                self.say(line);
            } else {
                self.racers[i].prison = Some(served + 1);
            }
        } else {
            match face {
                Face::One => steps = 1,
                Face::Two => steps = 2,
                Face::Three => steps = if self.group[BIG_THREE] { 4 } else { 3 },
                Face::Prison => self.send_to_prison(i),
                Face::Escape if self.racers[i].escape => steps = 1,
                Face::Escape => {
                    self.racers[i].escape = true;
                    let line = format!("{} gets an escape card.", self.name(i));
                    self.say(line);
                }
            }
        }
        if steps > 0 {
            self.advance(i, steps);
        }
        let landed = steps > 0 && self.racers[i].pos > 0 && self.racers[i].pos % 4 == 0;
        let n = self.racers.len();
        for k in 1..n {
            let j = (i + k) % n;
            let Some((cond, effect)) = self.racers[j].rule else { continue };
            let fires = match cond {
                Cond::RollsOne => face == Face::One,
                Cond::RollsThree => face == Face::Three,
                Cond::RollsPrison => face == Face::Prison,
                Cond::LandsOnFour => landed,
                Cond::PassesMe => before[i] <= before[j] && self.racers[i].pos > before[j],
            };
            if fires {
                self.fire(j, i, effect);
            }
        }
        if self.racers.iter().any(|r| r.pos >= TRACK) {
            // Replay moment: a rule that carries its owner onto the goal on someone else's roll.
            if let Some(w) = self.racers.iter().position(|r| r.pos >= TRACK).filter(|&w| w != i) {
                let roller = self.name(i);
                self.moments.push(&self.name(w), format!("Reaches the goal on {{{roller}}}'s roll"));
            }
            self.phase = Phase::Over;
            return;
        }
        self.next_turn();
    }

    fn fire(&mut self, owner: usize, them: usize, effect: Effect) {
        let first = !self.racers[owner].revealed;
        self.racers[owner].revealed = true;
        let (c, e) = self.racers[owner].rule.expect("rule");
        let line = if first {
            format!("{}'s rule fires: {}", self.name(owner), rule_text(c, e))
        } else {
            format!("{}'s rule fires again.", self.name(owner))
        };
        self.say(line);
        let (mine, theirs) = (self.racers[owner].pos, self.racers[them].pos);
        match effect {
            Effect::MeForward2 => self.advance(owner, 2),
            Effect::ThemBack2 => self.racers[them].pos = (self.racers[them].pos - 2).max(0),
            Effect::ThemPrison => self.send_to_prison(them),
            Effect::SwapIfAhead => {
                if self.racers[them].pos > self.racers[owner].pos {
                    let p = self.racers[them].pos;
                    self.racers[them].pos = self.racers[owner].pos;
                    self.racers[owner].pos = p;
                }
            }
            Effect::MeEscapeCard => self.racers[owner].escape = true,
        }
        // Replay moments (a rule that fires is shown to everyone): a rule pushing a racer back off
        // the goal, or a swap that jumps 6 squares or more (one onto the goal is a win, see `roll`).
        let (by, target) = (self.name(owner), self.name(them));
        if effect == Effect::ThemBack2 && theirs >= TRACK {
            self.moments.push(&by, format!("Rule pushes {{{target}}} back off the goal"));
        } else if effect == Effect::SwapIfAhead && theirs >= mine + 6 && theirs < TRACK {
            self.moments.push(&by, format!("Swap with {{{target}}}: from {mine} to {theirs}"));
        }
    }

    fn next_turn(&mut self) {
        self.turn += 1;
        if self.turn < self.racers.len() {
            return;
        }
        self.turn = 0;
        if self.round >= MAX_ROUNDS {
            self.phase = Phase::Over;
            return;
        }
        if self.round.is_multiple_of(VOTE_EVERY) {
            self.phase = Phase::Vote;
            for r in &mut self.racers {
                r.vote = None;
            }
        }
        self.round += 1;
    }

    fn finish_write(&mut self) {
        self.phase = Phase::Turn;
        self.turn = 0;
        self.say("Everyone has a secret rule. The race is on!".into());
    }

    fn finish_vote(&mut self) {
        let mut tally = [0usize; 5];
        let mut parts = Vec::new();
        for r in &self.racers {
            let v = r.vote.flatten();
            tally[v.unwrap_or(4)] += 1;
            parts.push(format!("{}: {}", r.id, v.map_or("keep".to_string(), |k| format!("switch {}", k + 1))));
        }
        self.say(format!("Votes: {}.", parts.join(", ")));
        let best = *tally.iter().max().expect("tally");
        let winners: Vec<usize> = (0..5).filter(|&k| tally[k] == best).collect();
        if let [k] = winners[..]
            && k < 4
        {
            self.group[k] = !self.group[k];
            let line = format!("Group rule {} is now {}.", k + 1, if self.group[k] { "on" } else { "off" });
            self.say(line);
        } else {
            self.say("The group rules stay as they are.".into());
        }
        self.phase = Phase::Turn;
        self.turn = 0;
    }
}

pub struct Rules;

impl Game for Rules {
    type State = RaceState;
    type Move = RaceMove;
    type View = RaceView;

    fn info() -> GameInfo {
        GameInfo {
            id: "rules-race",
            kind: "main",
            difficulty: "hard",
            tags: &["strategy-territory"],
            min_players: 4,
            max_players: 8,
            name: "Rules Race",
            short: "Race to the goal with a die of numbers, prison and escape; write a secret rule and vote on the group rules.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<RaceState, String> {
        if players.len() < 4 || players.len() > 8 {
            return Err("Rules Race needs 4 to 8 players".into());
        }
        let racers = players
            .iter()
            .map(|id| Racer { id: id.clone(), pos: 0, prison: None, escape: false, rule: None, revealed: false, vote: None })
            .collect();
        Ok(RaceState {
            racers,
            phase: Phase::Write,
            turn: 0,
            round: 1,
            group: [true, false, false, false],
            rng: Rng::new(seed),
            last_roll: None,
            log: vec!["Write your secret rule.".into()],
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut RaceState, player: &str, mv: RaceMove) -> Result<(), String> {
        let i = s.index(player).ok_or("Not in this game")?;
        match (s.phase, mv) {
            (Phase::Write, RaceMove::Write { condition, effect }) => {
                if s.racers[i].rule.is_some() {
                    return Err("You have written your rule".into());
                }
                s.racers[i].rule = Some((condition, effect));
                if s.racers.iter().all(|r| r.rule.is_some()) {
                    s.finish_write();
                }
                Ok(())
            }
            (Phase::Turn, RaceMove::Roll) => {
                if s.turn != i {
                    return Err("Not your turn".into());
                }
                s.roll();
                Ok(())
            }
            (Phase::Vote, RaceMove::Vote { rule }) => {
                if rule.is_some_and(|k| k >= GROUP_RULES.len()) {
                    return Err("No such group rule".into());
                }
                if s.racers[i].vote.is_some() {
                    return Err("You have voted".into());
                }
                s.racers[i].vote = Some(rule);
                if s.racers.iter().all(|r| r.vote.is_some()) {
                    s.finish_vote();
                }
                Ok(())
            }
            (Phase::Over, _) => Err("Game is over".into()),
            _ => Err("Not now".into()),
        }
    }

    fn view_for(s: &RaceState, player: &str) -> RaceView {
        let racers = s
            .racers
            .iter()
            .map(|r| RacerView {
                id: r.id.clone(),
                pos: r.pos,
                in_prison: r.prison.is_some(),
                escape: r.escape,
                rule: r.rule.filter(|_| r.revealed || r.id == player || s.phase == Phase::Over).map(|(c, e)| RuleView {
                    condition: c,
                    effect: e,
                    text: rule_text(c, e),
                }),
                written: r.rule.is_some(),
                voted: r.vote.is_some(),
            })
            .collect();
        RaceView {
            phase: match s.phase {
                Phase::Write => "write",
                Phase::Turn => "turn",
                Phase::Vote => "vote",
                Phase::Over => "over",
            },
            track: TRACK,
            round: s.round,
            max_rounds: MAX_ROUNDS,
            racers,
            current: (s.phase == Phase::Turn).then(|| s.racers[s.turn].id.clone()),
            last_roll: s.last_roll.map(|(i, f)| format!("{} rolled {}", s.racers[i].id, f.text())),
            group: GROUP_RULES.iter().zip(s.group).map(|(&text, on)| GroupView { text, on }).collect(),
            conditions: Cond::ALL.iter().map(|&c| Choice { id: id_of(c), text: c.text() }).collect(),
            effects: Effect::ALL.iter().map(|&e| Choice { id: id_of(e), text: e.text() }).collect(),
            log: s.log.clone(),
            score: s.index(player).map_or(0, |i| s.racers[i].pos),
        }
    }

    fn is_over(s: &RaceState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &RaceState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.racers.iter().map(|r| (r.id.clone(), r.pos)).collect();
        rank_points(&scores)
    }

    fn to_move(s: &RaceState) -> Vec<String> {
        match s.phase {
            Phase::Write => s.racers.iter().filter(|r| r.rule.is_none()).map(|r| r.id.clone()).collect(),
            Phase::Turn => vec![s.racers[s.turn].id.clone()],
            Phase::Vote => s.racers.iter().filter(|r| r.vote.is_none()).map(|r| r.id.clone()).collect(),
            Phase::Over => Vec::new(),
        }
    }

    fn moments(s: &RaceState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &RaceState, player: &str, seed: u64) -> RaceMove {
        let mut rng = Rng::new(seed);
        match s.phase {
            Phase::Write => {
                let effects =
                    [Effect::ThemBack2, Effect::ThemBack2, Effect::ThemPrison, Effect::ThemPrison, Effect::MeForward2, Effect::SwapIfAhead];
                RaceMove::Write { condition: *rng.pick(&Cond::ALL), effect: *rng.pick(&effects) }
            }
            Phase::Vote => {
                let me = s.index(player).map_or(0, |i| s.racers[i].pos);
                let lead = s.racers.iter().map(|r| r.pos).max().unwrap_or(0);
                let rule = if me >= lead {
                    None
                } else if lead >= TRACK - 4 && !s.group[EXACT] {
                    Some(EXACT)
                } else if !s.group[BUMP] {
                    Some(BUMP)
                } else {
                    None
                };
                RaceMove::Vote { rule }
            }
            _ => RaceMove::Roll,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    fn write_all(s: &mut RaceState, c: Cond, e: Effect) {
        for p in players(s.racers.len()) {
            Rules::apply_move(s, &p, RaceMove::Write { condition: c, effect: e }).unwrap();
        }
    }

    #[test]
    fn rules_stay_secret_until_they_fire() {
        let mut s = Rules::setup(&players(4), 1).unwrap();
        write_all(&mut s, Cond::RollsOne, Effect::ThemBack2);
        let v = Rules::view_for(&s, "p1");
        assert!(v.racers[0].rule.is_some());
        assert!(v.racers[1].rule.is_none());
        assert_eq!(v.phase, "turn");
        // p1 rolls a 1: everyone else's rule fires and pushes p1 back.
        s.racers[0].pos = 5;
        s.fire(1, 0, Effect::ThemBack2);
        assert_eq!(s.racers[0].pos, 3);
        assert!(Rules::view_for(&s, "p1").racers[1].rule.is_some());
    }

    #[test]
    fn only_the_current_player_rolls() {
        let mut s = Rules::setup(&players(4), 2).unwrap();
        write_all(&mut s, Cond::PassesMe, Effect::MeEscapeCard);
        assert!(Rules::apply_move(&mut s, "p2", RaceMove::Roll).is_err());
        Rules::apply_move(&mut s, "p1", RaceMove::Roll).unwrap();
        assert_eq!(Rules::to_move(&s), vec!["p2".to_string()]);
    }

    #[test]
    fn bump_and_exact_finish() {
        let mut s = Rules::setup(&players(4), 3).unwrap();
        s.racers[1].pos = 5;
        s.racers[0].pos = 3;
        s.advance(0, 2);
        assert_eq!(s.racers[1].pos, 3);
        s.group[EXACT] = true;
        s.racers[0].pos = 17;
        s.advance(0, 3);
        assert_eq!(s.racers[0].pos, 16);
    }

    #[test]
    fn escape_card_saves_from_prison() {
        let mut s = Rules::setup(&players(4), 4).unwrap();
        s.racers[2].escape = true;
        s.send_to_prison(2);
        assert!(s.racers[2].prison.is_none() && !s.racers[2].escape);
        s.send_to_prison(2);
        assert_eq!(s.racers[2].prison, Some(0));
    }

    #[test]
    fn a_vote_switches_a_group_rule_and_a_tie_does_not() {
        let mut s = Rules::setup(&players(4), 5).unwrap();
        s.phase = Phase::Vote;
        for (p, v) in [("p1", Some(EXACT)), ("p2", Some(EXACT)), ("p3", None), ("p4", Some(BUMP))] {
            Rules::apply_move(&mut s, p, RaceMove::Vote { rule: v }).unwrap();
        }
        assert!(s.group[EXACT]);
        s.phase = Phase::Vote;
        for r in &mut s.racers {
            r.vote = None;
        }
        for (p, v) in [("p1", Some(BIG_THREE)), ("p2", Some(BIG_THREE)), ("p3", None), ("p4", None)] {
            Rules::apply_move(&mut s, p, RaceMove::Vote { rule: v }).unwrap();
        }
        assert!(!s.group[BIG_THREE]);
    }

    #[test]
    fn no_timer_the_game_waits_for_every_seat() {
        let mut s = Rules::setup(&players(5), 6).unwrap();
        Rules::tick(&mut s, 600_000);
        assert_eq!(s.phase, Phase::Write);
        assert_eq!(Rules::to_move(&s).len(), 5);
        write_all(&mut s, Cond::RollsOne, Effect::ThemBack2);
        Rules::tick(&mut s, 600_000);
        assert_eq!(s.turn, 0);
        assert!(s.last_roll.is_none());
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(1000, 4, 3000);
        bot_games_with::<Rules>(200, 8, 6000);
    }

    #[test]
    fn rules_that_stop_a_finish_swap_far_or_win_are_replay_moments() {
        let mut s = Rules::setup(&players(4), 7).unwrap();
        write_all(&mut s, Cond::RollsPrison, Effect::MeEscapeCard);
        s.racers[0].pos = TRACK;
        s.fire(1, 0, Effect::ThemBack2);
        s.racers[0].pos = 12;
        s.racers[2].pos = 3;
        s.fire(2, 0, Effect::SwapIfAhead);
        // A short swap is no moment.
        s.racers[3].pos = 10;
        s.fire(3, 0, Effect::SwapIfAhead);
        let m = Rules::moments(&s, "p4");
        let texts: Vec<(&str, &str)> = m.iter().map(|m| (m.by.as_str(), m.text.as_str())).collect();
        assert_eq!(texts, vec![("p2", "Rule pushes {p1} back off the goal"), ("p3", "Swap with {p1}: from 3 to 12")]);
        // p1 rolls a 1 and p2's rule carries p2 onto the goal.
        let mut s = Rules::setup(&players(4), 7).unwrap();
        write_all(&mut s, Cond::RollsPrison, Effect::MeEscapeCard);
        s.racers[1].rule = Some((Cond::RollsOne, Effect::MeForward2));
        s.racers[1].pos = TRACK - 2;
        s.rng = Rng::new((0..1000).find(|&k| Rng::new(k).index(FACES.len()) == 0).unwrap());
        Rules::apply_move(&mut s, "p1", RaceMove::Roll).unwrap();
        assert!(Rules::is_over(&s));
        let m = Rules::moments(&s, "p1");
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p2", "Reaches the goal on {p1}'s roll"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 3000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        // 15 rounds of four rolls: a long game may hold a few more.
        assert!(stats.most <= 5, "too many in one game: {stats:?}");
    }
}

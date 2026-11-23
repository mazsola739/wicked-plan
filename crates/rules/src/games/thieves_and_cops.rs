//! Thieves and Cops (main game). Rules page: docs/games/thieves-and-cops.md
//!
//! Thieves collect hidden pieces on a 17-station metro map; the police hunt
//! them with only occasional sightings. One officer is secretly corrupt.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: u32 = 12;
pub const MOVE_MS: i64 = 20_000;
pub const UNMASK_MS: i64 = 20_000;
pub const REVEAL_MS: i64 = 15_000;
pub const PIECES: usize = 7;
pub const TO_WIN: u32 = 5;
pub const SIGHTINGS: [u32; 5] = [2, 4, 6, 8, 10];

/// Name, x, y (on a 600 × 600 map).
pub const STATIONS: [(&str, i32, i32); 17] = [
    ("Harbour", 60, 300),
    ("Market", 175, 300),
    ("Central", 300, 300),
    ("Museum", 425, 300),
    ("Airport", 540, 300),
    ("North Gate", 300, 60),
    ("Library", 300, 175),
    ("Casino", 420, 180),
    ("Stadium", 300, 425),
    ("South Pier", 300, 540),
    ("Old Town", 120, 120),
    ("Park", 180, 180),
    ("Tower", 480, 120),
    ("Factory", 480, 480),
    ("Docks", 120, 480),
    ("Bank", 180, 420),
    ("Zoo", 420, 420),
];

/// Stretches of line, with the line's index: ring, red, blue, green, purple.
pub const EDGES: [(usize, usize, usize); 24] = [
    (10, 5, 0),
    (5, 12, 0),
    (12, 4, 0),
    (4, 13, 0),
    (13, 9, 0),
    (9, 14, 0),
    (14, 0, 0),
    (0, 10, 0),
    (0, 1, 1),
    (1, 2, 1),
    (2, 3, 1),
    (3, 4, 1),
    (5, 6, 2),
    (6, 2, 2),
    (2, 8, 2),
    (8, 9, 2),
    (10, 11, 3),
    (11, 6, 3),
    (6, 7, 3),
    (7, 12, 3),
    (14, 15, 4),
    (15, 8, 4),
    (8, 16, 4),
    (16, 13, 4),
];

const RING: [usize; 8] = [10, 5, 12, 4, 13, 9, 14, 0];
const POLICE_STARTS: [usize; 7] = [2, 6, 8, 1, 3, 11, 16];

pub fn neighbours(s: usize) -> Vec<usize> {
    EDGES.iter().filter_map(|&(a, b, _)| if a == s { Some(b) } else if b == s { Some(a) } else { None }).collect()
}

/// Stops from `from` to every station.
pub fn distances(from: usize) -> Vec<u32> {
    let mut d = vec![u32::MAX; STATIONS.len()];
    d[from] = 0;
    let mut queue = vec![from];
    let mut i = 0;
    while i < queue.len() {
        let s = queue[i];
        i += 1;
        for n in neighbours(s) {
            if d[n] == u32::MAX {
                d[n] = d[s] + 1;
                queue.push(n);
            }
        }
    }
    d
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Thief,
    Cop,
    Corrupt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Move,
    Unmask,
    Reveal,
    Over,
}

#[derive(Clone, Debug)]
pub struct Agent {
    pub id: String,
    pub role: Role,
    pub pos: usize,
    pub caught: bool,
    pub kept: u32,
    pub catches: u32,
    pub choice: Option<usize>,
    pub accuse: Option<usize>,
    /// Station at the end of each round.
    pub trail: Vec<usize>,
    /// Last public sighting of a thief: (round, station).
    pub seen: Option<(u32, usize)>,
}

#[derive(Clone, Debug)]
pub struct TcState {
    pub agents: Vec<Agent>,
    pub loot: Vec<usize>,
    /// (round, station) of every piece taken.
    pub taken: Vec<(u32, usize)>,
    /// Rounds with a general sighting.
    pub sighted: Vec<u32>,
    pub round: u32,
    pub phase: Phase,
    pub time_left_ms: i64,
    pub unmasked: bool,
    pub log: Vec<String>,
    pub rng: Rng,
    /// Replay moments: a catch; the piece that wins it for the thieves in round 11 or 12.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TcMove {
    Go { to: usize },
    Accuse { target: usize },
    Close,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StationView {
    pub name: &'static str,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentView {
    pub index: usize,
    pub id: String,
    /// "thief", "police", or "corrupt" when you may know it.
    pub role: &'static str,
    pub pos: Option<usize>,
    pub caught: bool,
    pub kept: u32,
    pub catches: u32,
    pub seen: Option<(u32, usize)>,
    pub done: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TcView {
    pub phase: &'static str,
    pub round: u32,
    pub rounds: u32,
    pub time_left_ms: i64,
    pub phase_time_ms: i64,
    pub me: usize,
    pub role: Role,
    pub stations: Vec<StationView>,
    pub edges: Vec<(usize, usize, usize)>,
    pub agents: Vec<AgentView>,
    pub loot: Option<Vec<usize>>,
    pub taken: Vec<(u32, usize)>,
    pub kept_total: u32,
    pub to_win: u32,
    pub options: Vec<usize>,
    pub chosen: Option<usize>,
    pub can_accuse: bool,
    pub accused: Option<usize>,
    pub unmasked: bool,
    pub winner: Option<&'static str>,
    pub log: Vec<String>,
    pub score: i64,
}

impl TcState {
    fn say(&mut self, line: String) {
        self.log.push(line);
        let extra = self.log.len().saturating_sub(14);
        self.log.drain(..extra);
    }

    fn thief_side(&self, i: usize) -> bool {
        self.agents[i].role != Role::Cop
    }

    fn police(&self, i: usize) -> bool {
        self.agents[i].role != Role::Thief
    }

    fn movers(&self) -> Vec<usize> {
        (0..self.agents.len()).filter(|&i| !self.agents[i].caught).collect()
    }

    pub fn kept_total(&self) -> u32 {
        self.agents.iter().map(|a| a.kept).sum()
    }

    pub fn thieves_win(&self) -> bool {
        self.kept_total() >= TO_WIN
    }

    fn resolve_moves(&mut self) {
        let n = self.agents.len();
        let won_before = self.thieves_win();
        let mut last_take = None;
        let from: Vec<usize> = self.agents.iter().map(|a| a.pos).collect();
        for a in &mut self.agents {
            if !a.caught
                && let Some(to) = a.choice.take()
            {
                a.pos = to;
            }
            a.choice = None;
        }
        for t in 0..n {
            if self.agents[t].role != Role::Thief || self.agents[t].caught {
                continue;
            }
            let catcher = (0..n).find(|&c| {
                self.agents[c].role == Role::Cop
                    && (self.agents[c].pos == self.agents[t].pos || (from[c] == self.agents[t].pos && self.agents[c].pos == from[t]))
            });
            if let Some(c) = catcher {
                let thief = &mut self.agents[t];
                thief.caught = true;
                let lost = thief.kept;
                thief.kept = 0;
                self.agents[c].catches += 1;
                let line = format!(
                    "{} catches {} at {}{}!",
                    self.agents[c].id,
                    self.agents[t].id,
                    STATIONS[self.agents[t].pos].0,
                    if lost > 0 { format!(" ({lost} pieces lost)") } else { String::new() }
                );
                self.say(line);
                // A replay moment: every catch (the log tells everyone anyway).
                let lost = match lost {
                    0 => String::new(),
                    1 => ": 1 piece lost".into(),
                    n => format!(": {n} pieces lost"),
                };
                let text = format!("{{{}}} caught at {}{lost}", self.agents[t].id, STATIONS[self.agents[t].pos].0);
                self.moments.push(&self.agents[c].id.clone(), text);
                continue;
            }
            let at = self.agents[t].pos;
            if let Some(k) = self.loot.iter().position(|&s| s == at) {
                self.loot.remove(k);
                self.agents[t].kept += 1;
                self.taken.push((self.round, at));
                self.agents[t].seen = Some((self.round, at));
                let line = format!("A piece was taken at {}.", STATIONS[at].0);
                self.say(line);
                last_take = Some((t, at));
            }
        }
        // A replay moment: the piece that wins it for the thieves at the last moment, in round 11
        // or 12 (where a piece was taken, and by whom, is public).
        if !won_before
            && self.thieves_win()
            && self.round + 1 >= ROUNDS
            && let Some((t, at)) = last_take
        {
            let text = format!("Winning piece taken at {} in round {}", STATIONS[at].0, self.round);
            self.moments.push(&self.agents[t].id.clone(), text);
        }
        for a in &mut self.agents {
            a.trail.push(a.pos);
        }
        if SIGHTINGS.contains(&self.round) {
            self.sighted.push(self.round);
            let mut parts = Vec::new();
            for t in 0..n {
                if self.agents[t].role == Role::Thief && !self.agents[t].caught {
                    self.agents[t].seen = Some((self.round, self.agents[t].pos));
                    parts.push(format!("{} at {}", self.agents[t].id, STATIONS[self.agents[t].pos].0));
                }
            }
            if !parts.is_empty() {
                self.say(format!("Sighting: {}.", parts.join(", ")));
            }
        }
        let all_caught = self.agents.iter().all(|a| a.role != Role::Thief || a.caught);
        if self.thieves_win() || all_caught || self.round >= ROUNDS {
            let line = if self.thieves_win() { "The thieves have 5 pieces!" } else { "The chase is over." };
            self.say(line.into());
            self.say("Unmask: name the officer you think is corrupt.".into());
            self.phase = Phase::Unmask;
            self.time_left_ms = UNMASK_MS;
        } else {
            self.round += 1;
            self.time_left_ms = MOVE_MS;
        }
    }

    fn resolve_unmask(&mut self) {
        let n = self.agents.len();
        let mut tally = vec![0usize; n];
        for i in 0..n {
            if let Some(t) = self.agents[i].accuse {
                tally[t] += 1;
            }
        }
        let best = tally.iter().copied().max().unwrap_or(0);
        let top: Vec<usize> = (0..n).filter(|&i| tally[i] == best).collect();
        let corrupt = (0..n).find(|&i| self.agents[i].role == Role::Corrupt);
        self.unmasked = best > 0 && top.len() == 1 && Some(top[0]) == corrupt;
        if let Some(c) = corrupt {
            let line = if self.unmasked {
                format!("{} is unmasked as the corrupt officer!", self.agents[c].id)
            } else {
                format!("The corrupt officer, {}, got away with it.", self.agents[c].id)
            };
            self.say(line);
        }
        self.phase = Phase::Reveal;
        self.time_left_ms = REVEAL_MS;
    }

    pub fn score(&self, i: usize) -> i64 {
        let a = &self.agents[i];
        let mut pts = if self.thief_side(i) == self.thieves_win() { 500 } else { 200 };
        match a.role {
            Role::Thief => pts += 50 * i64::from(a.kept),
            Role::Cop => pts += 200 * i64::from(a.catches) + if self.unmasked { 100 } else { 0 },
            Role::Corrupt => pts -= if self.unmasked { 300 } else { 0 },
        }
        pts
    }

    /// Stations where a free thief could be, from public facts only.
    fn possible(&self, t: usize) -> Vec<usize> {
        let (since, near): (u32, Vec<usize>) = match self.agents[t].seen {
            Some((r, s)) => (self.round - r, vec![s]),
            None => (self.round, RING.to_vec()),
        };
        let honest: Vec<usize> = self.agents.iter().filter(|a| a.role == Role::Cop).map(|a| a.pos).collect();
        let dist: Vec<Vec<u32>> = near.iter().map(|&s| distances(s)).collect();
        (0..STATIONS.len()).filter(|&s| dist.iter().any(|d| d[s] <= since) && !honest.contains(&s)).collect()
    }
}

pub struct Rules;

impl Game for Rules {
    type State = TcState;
    type Move = TcMove;
    type View = TcView;

    fn info() -> GameInfo {
        GameInfo {
            id: "thieves-and-cops",
            kind: "main",
            difficulty: "hard",
            tags: &["board", "secret-role", "alliance"],
            min_players: 4,
            max_players: 8,
            name: "Thieves and Cops",
            short: "Thieves collect hidden pieces on a metro map while the police hunt them; one officer is secretly corrupt.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<TcState, String> {
        let n = players.len();
        if !(4..=8).contains(&n) {
            return Err("Thieves and Cops needs 4 to 8 players".into());
        }
        let mut rng = Rng::new(seed);
        let thieves = match n {
            4 | 5 => 1,
            6 | 7 => 2,
            _ => 3,
        };
        let mut roles = vec![Role::Thief; thieves];
        roles.push(Role::Corrupt);
        roles.resize(n, Role::Cop);
        rng.shuffle(&mut roles);
        let mut ring = RING.to_vec();
        rng.shuffle(&mut ring);
        let mut police = POLICE_STARTS.to_vec();
        rng.shuffle(&mut police[..3]);
        let (mut ti, mut pi) = (0, 0);
        let agents: Vec<Agent> = players
            .iter()
            .zip(roles)
            .map(|(id, role)| {
                let pos = if role == Role::Thief {
                    ti += 1;
                    ring[ti - 1]
                } else {
                    pi += 1;
                    police[pi - 1]
                };
                Agent { id: id.clone(), role, pos, caught: false, kept: 0, catches: 0, choice: None, accuse: None, trail: Vec::new(), seen: None }
            })
            .collect();
        let starts: Vec<usize> = agents.iter().map(|a| a.pos).collect();
        let mut free: Vec<usize> = (0..STATIONS.len()).filter(|s| !starts.contains(s)).collect();
        rng.shuffle(&mut free);
        let loot = free[..PIECES].to_vec();
        Ok(TcState {
            agents,
            loot,
            taken: Vec::new(),
            sighted: Vec::new(),
            round: 1,
            phase: Phase::Move,
            time_left_ms: MOVE_MS,
            unmasked: false,
            log: vec!["Round 1: everyone moves at once.".into()],
            rng,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut TcState, player: &str, mv: TcMove) -> Result<(), String> {
        let i = s.agents.iter().position(|a| a.id == player).ok_or("Not in this game")?;
        match (s.phase, mv) {
            (Phase::Move, TcMove::Go { to }) => {
                if s.agents[i].caught {
                    return Err("You have been caught".into());
                }
                if s.agents[i].choice.is_some() {
                    return Err("You have moved this round".into());
                }
                if to != s.agents[i].pos && !neighbours(s.agents[i].pos).contains(&to) {
                    return Err("Move one stop along a line, or stay".into());
                }
                s.agents[i].choice = Some(to);
                if s.movers().iter().all(|&a| s.agents[a].choice.is_some()) {
                    s.resolve_moves();
                }
                Ok(())
            }
            (Phase::Unmask, TcMove::Accuse { target }) => {
                if !s.police(i) {
                    return Err("Only the police vote".into());
                }
                if target >= s.agents.len() || target == i || !s.police(target) {
                    return Err("Name another officer".into());
                }
                if s.agents[i].accuse.is_some() {
                    return Err("You have voted".into());
                }
                s.agents[i].accuse = Some(target);
                if (0..s.agents.len()).all(|a| !s.police(a) || s.agents[a].accuse.is_some()) {
                    s.resolve_unmask();
                }
                Ok(())
            }
            (Phase::Reveal, TcMove::Close) => {
                s.phase = Phase::Over;
                Ok(())
            }
            (Phase::Over, _) => Err("Game is over".into()),
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut TcState, ms: u64) {
        if s.phase == Phase::Over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms > 0 {
            return;
        }
        match s.phase {
            Phase::Move => s.resolve_moves(),
            Phase::Unmask => s.resolve_unmask(),
            _ => s.phase = Phase::Over,
        }
    }

    fn view_for(s: &TcState, player: &str) -> TcView {
        let me = s.agents.iter().position(|a| a.id == player).unwrap_or(0);
        let end = matches!(s.phase, Phase::Reveal | Phase::Over);
        let insider = s.thief_side(me) || end;
        let agents = s
            .agents
            .iter()
            .enumerate()
            .map(|(i, a)| AgentView {
                index: i,
                id: a.id.clone(),
                role: match a.role {
                    Role::Thief => "thief",
                    Role::Corrupt if insider || i == me => "corrupt",
                    _ => "police",
                },
                pos: (a.role != Role::Thief || insider || a.caught).then_some(a.pos),
                caught: a.caught,
                kept: a.kept,
                catches: a.catches,
                seen: a.seen,
                done: match s.phase {
                    Phase::Move => a.choice.is_some() || a.caught,
                    Phase::Unmask => a.accuse.is_some() || a.role == Role::Thief,
                    _ => true,
                },
            })
            .collect();
        let mine = &s.agents[me];
        let mut options = Vec::new();
        if s.phase == Phase::Move && !mine.caught && mine.choice.is_none() {
            options.push(mine.pos);
            options.extend(neighbours(mine.pos));
        }
        TcView {
            phase: match s.phase {
                Phase::Move => "move",
                Phase::Unmask => "unmask",
                Phase::Reveal => "reveal",
                Phase::Over => "over",
            },
            round: s.round,
            rounds: ROUNDS,
            time_left_ms: s.time_left_ms,
            phase_time_ms: match s.phase {
                Phase::Move => MOVE_MS,
                Phase::Unmask => UNMASK_MS,
                _ => REVEAL_MS,
            },
            me,
            role: mine.role,
            stations: STATIONS.iter().map(|&(name, x, y)| StationView { name, x, y }).collect(),
            edges: EDGES.to_vec(),
            agents,
            loot: insider.then(|| s.loot.clone()),
            taken: s.taken.clone(),
            kept_total: s.kept_total(),
            to_win: TO_WIN,
            options,
            chosen: mine.choice,
            can_accuse: s.phase == Phase::Unmask && s.police(me) && mine.accuse.is_none(),
            accused: mine.accuse,
            unmasked: s.unmasked,
            winner: end.then(|| if s.thieves_win() { "thieves" } else { "police" }),
            log: s.log.clone(),
            score: if end { s.score(me) } else { 0 },
        }
    }

    fn is_over(s: &TcState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &TcState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = (0..s.agents.len()).map(|i| (s.agents[i].id.clone(), s.score(i))).collect();
        rank_by_score(&scores)
    }

    fn to_move(s: &TcState) -> Vec<String> {
        let open = |i: usize| match s.phase {
            Phase::Move => !s.agents[i].caught && s.agents[i].choice.is_none(),
            Phase::Unmask => s.police(i) && s.agents[i].accuse.is_none(),
            _ => false,
        };
        (0..s.agents.len()).filter(|&i| open(i)).map(|i| s.agents[i].id.clone()).collect()
    }

    fn bot_move(s: &TcState, player: &str, seed: u64) -> TcMove {
        let mut rng = Rng::new(seed);
        let Some(i) = s.agents.iter().position(|a| a.id == player) else { return TcMove::Close };
        let me = &s.agents[i];
        match s.phase {
            Phase::Move => {
                let mut options = vec![me.pos];
                options.extend(neighbours(me.pos));
                let honest: Vec<usize> = s.agents.iter().filter(|a| a.role == Role::Cop).map(|a| a.pos).collect();
                let cost: Vec<f64> = match me.role {
                    Role::Thief => options
                        .iter()
                        .map(|&o| {
                            let d = distances(o);
                            let danger = honest.iter().filter(|&&c| d[c] <= 1).count() + honest.iter().filter(|&&c| c == o).count();
                            let to_loot = s.loot.iter().map(|&l| d[l]).min().unwrap_or(0);
                            f64::from(to_loot) + 10.0 * danger as f64
                        })
                        .collect(),
                    Role::Cop => {
                        let mut spots = Vec::new();
                        for t in 0..s.agents.len() {
                            if s.agents[t].role == Role::Thief && !s.agents[t].caught {
                                spots.extend(s.possible(t));
                            }
                        }
                        let mine = distances(me.pos);
                        let target = spots.iter().copied().min_by_key(|&p| (mine[p], (p * 7 + i) % 17));
                        options
                            .iter()
                            .map(|&o| target.map_or(0.0, |t| f64::from(distances(o)[t])) + if o == me.pos { 0.5 } else { 0.0 })
                            .collect()
                    }
                    Role::Corrupt => {
                        let thieves: Vec<usize> = s.agents.iter().filter(|a| a.role == Role::Thief).map(|a| a.pos).collect();
                        options.iter().map(|o| if thieves.contains(o) { 10.0 } else { 0.0 }).collect()
                    }
                };
                let best = cost.iter().copied().fold(f64::MAX, f64::min);
                let good: Vec<usize> = options.iter().zip(&cost).filter(|(_, c)| **c <= best + 1e-9).map(|(&o, _)| o).collect();
                TcMove::Go { to: *rng.pick(&good) }
            }
            Phase::Unmask => {
                let others: Vec<usize> = (0..s.agents.len()).filter(|&k| k != i && s.police(k)).collect();
                if others.is_empty() {
                    return TcMove::Close;
                }
                if me.role == Role::Corrupt {
                    let honest: Vec<usize> = others.iter().copied().filter(|&k| s.agents[k].role == Role::Cop).collect();
                    return TcMove::Accuse { target: *rng.pick(if honest.is_empty() { &others } else { &honest }) };
                }
                // Average distance to the thieves at the general sightings.
                let far = |k: usize| -> f64 {
                    let mut sum = 0.0;
                    let mut count = 0.0;
                    for &r in &s.sighted {
                        let Some(&at) = s.agents[k].trail.get(r as usize - 1) else { continue };
                        let d = distances(at);
                        let near = s
                            .agents
                            .iter()
                            .filter(|a| a.role == Role::Thief)
                            .filter_map(|a| a.trail.get(r as usize - 1).map(|&p| d[p]))
                            .min()
                            .unwrap_or(0);
                        sum += f64::from(near);
                        count += 1.0;
                    }
                    if count > 0.0 { sum / count } else { 0.0 }
                };
                let best = others.iter().map(|&k| far(k)).fold(f64::MIN, f64::max);
                let suspects: Vec<usize> = others.iter().copied().filter(|&k| far(k) >= best - 1e-9).collect();
                TcMove::Accuse { target: *rng.pick(&suspects) }
            }
            _ => TcMove::Close,
        }
    }

    fn moments(s: &TcState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
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

    fn find(s: &TcState, role: Role) -> usize {
        s.agents.iter().position(|a| a.role == role).unwrap()
    }

    #[test]
    fn the_map_is_connected() {
        let d = distances(0);
        assert!(d.iter().all(|&x| x < 10));
        assert_eq!(neighbours(2).len(), 4);
    }

    #[test]
    fn honest_police_do_not_see_the_thief_or_the_corrupt() {
        let s = Rules::setup(&players(4), 1).unwrap();
        let c = find(&s, Role::Cop);
        let t = find(&s, Role::Thief);
        let v = Rules::view_for(&s, &s.agents[c].id);
        assert!(v.agents[t].pos.is_none());
        assert!(v.loot.is_none());
        assert!(v.agents.iter().all(|a| a.role != "corrupt"));
        let v = Rules::view_for(&s, &s.agents[t].id);
        assert!(v.agents.iter().any(|a| a.role == "corrupt"));
        assert_eq!(v.loot.unwrap().len(), PIECES);
    }

    #[test]
    fn an_honest_cop_catches_and_the_corrupt_does_not() {
        let mut s = Rules::setup(&players(4), 2).unwrap();
        let t = find(&s, Role::Thief);
        let k = find(&s, Role::Corrupt);
        let c = find(&s, Role::Cop);
        s.agents[t].pos = 1;
        s.agents[t].kept = 2;
        s.agents[k].pos = 0;
        s.agents[c].pos = 3;
        for a in 0..4 {
            let to = if a == t {
                0
            } else if a == c {
                2
            } else {
                s.agents[a].pos
            };
            s.agents[a].choice = Some(to);
        }
        s.resolve_moves();
        assert!(!s.agents[t].caught, "the corrupt officer lets the thief go");
        s.agents[c].pos = 2;
        s.agents[t].pos = 1;
        for a in 0..4 {
            s.agents[a].choice = Some(if a == c { 1 } else { s.agents[a].pos });
        }
        s.resolve_moves();
        assert!(s.agents[t].caught);
        assert_eq!(s.agents[t].kept, 0);
        assert_eq!(s.agents[c].catches, 1);
    }

    #[test]
    fn moves_must_follow_a_line() {
        let mut s = Rules::setup(&players(4), 3).unwrap();
        let p = s.agents[0].id.clone();
        let far = (0..STATIONS.len()).find(|&x| distances(s.agents[0].pos)[x] > 1).unwrap();
        assert!(Rules::apply_move(&mut s, &p, TcMove::Go { to: far }).is_err());
        let near = neighbours(s.agents[0].pos)[0];
        assert!(Rules::apply_move(&mut s, &p, TcMove::Go { to: near }).is_ok());
    }

    #[test]
    fn both_sides_win_sometimes() {
        for n in [4, 5, 6, 7, 8] {
            let games = 300;
            let mut thieves = 0;
            for seed in 1..=games {
                let mut s = Rules::setup(&players(n), seed).unwrap();
                let mut guard = 0;
                while s.phase == Phase::Move || s.phase == Phase::Unmask {
                    for p in Rules::to_move(&s) {
                        let mv = Rules::bot_move(&s, &p, seed * 13 + guard);
                        let _ = Rules::apply_move(&mut s, &p, mv);
                    }
                    guard += 1;
                    assert!(guard < 200);
                }
                thieves += usize::from(s.thieves_win());
            }
            eprintln!("{n} players: thieves won {thieves} of {games}");
            assert!(thieves > games as usize / 5 && thieves < games as usize * 4 / 5, "{n} players: thieves won {thieves} of {games}");
        }
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(1000, 4, 3000);
        bot_games_with::<Rules>(200, 8, 6000);
    }

    #[test]
    fn a_catch_is_a_replay_moment() {
        let mut s = Rules::setup(&players(4), 2).unwrap();
        let t = find(&s, Role::Thief);
        let k = find(&s, Role::Corrupt);
        let c = find(&s, Role::Cop);
        // The corrupt officer lands on the thief: no catch, no moment.
        s.loot = vec![16];
        s.agents[t].pos = 1;
        s.agents[t].kept = 2;
        s.agents[k].pos = 0;
        s.agents[c].pos = 3;
        for a in 0..4 {
            s.agents[a].choice = Some(if a == t { 0 } else { s.agents[a].pos });
        }
        s.resolve_moves();
        assert!(s.moments.is_empty());
        // An honest officer passes the thief on the line: caught at Market with 2 pieces.
        s.agents[c].pos = 2;
        s.agents[t].pos = 1;
        for a in 0..4 {
            s.agents[a].choice = Some(match a {
                _ if a == c => 1,
                _ if a == t => 2,
                _ => s.agents[a].pos,
            });
        }
        s.resolve_moves();
        let m = Rules::moments(&s, "p1");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].by, s.agents[c].id);
        assert_eq!(m[0].text, format!("{{{}}} caught at Central: 2 pieces lost", s.agents[t].id));
    }

    /// The thief (4 pieces kept) steps from Harbour onto a piece at Market in `round`.
    fn last_piece(round: u32) -> TcState {
        let mut s = Rules::setup(&players(4), 2).unwrap();
        let t = find(&s, Role::Thief);
        s.round = round;
        s.agents[t].pos = 0;
        s.agents[t].kept = 4;
        s.loot = vec![1, 16];
        for a in 0..4 {
            if a != t {
                s.agents[a].pos = 13;
            }
            s.agents[a].choice = Some(if a == t { 1 } else { s.agents[a].pos });
        }
        s.resolve_moves();
        assert!(s.thieves_win());
        s
    }

    #[test]
    fn a_last_minute_win_is_a_replay_moment() {
        let s = last_piece(11);
        let m = Rules::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].by, s.agents[find(&s, Role::Thief)].id);
        assert_eq!(m[0].text, "Winning piece taken at Market in round 11");
        // An early win is no moment.
        assert!(last_piece(6).moments.is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
        let stats = crate::game::testing::moment_stats::<Rules>(300, 8, 5_000);
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

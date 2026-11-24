//! Virus Game (main game, hidden roles). Rules page: docs/games/virus-game.md
//!
//! Terrorists infect whoever they meet; the city (officer, journalist,
//! researchers, citizens) meets, reports and votes people into quarantine.
//! Four rounds of contact → test → report → vote.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: u32 = 4;
pub const REVEAL_MS: i64 = 15_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Terrorist,
    Officer,
    Journalist,
    Researcher,
    Citizen,
}

impl Role {
    pub fn text(self) -> &'static str {
        match self {
            Role::Terrorist => "Terrorist: whoever you meet, or who meets you, is infected.",
            Role::Officer => "Officer: you learn whether the person you meet is a terrorist.",
            Role::Journalist => "Journalist: you learn whom the person you meet met, and if they got infected.",
            Role::Researcher => "Researcher: the person you meet is vaccinated (cured and immune).",
            Role::Citizen => "Citizen: find the terrorists and stay healthy.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mission {
    StayHealthy,
    MeetTwice,
    CatchVoter,
    NeverAccused,
    InfectOfficer,
    StayFree,
}

impl Mission {
    pub fn text(self) -> &'static str {
        match self {
            Mission::StayHealthy => "Stay healthy to the end.",
            Mission::MeetTwice => "Meet the same player in two rounds.",
            Mission::CatchVoter => "Vote for a terrorist who goes into quarantine.",
            Mission::NeverAccused => "Never be accused.",
            Mission::InfectOfficer => "The officer is infected at the end.",
            Mission::StayFree => "Never be quarantined.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Contact,
    Report,
    Vote,
    Reveal,
    Over,
}

#[derive(Clone, Debug)]
pub struct Agent {
    pub id: String,
    pub number: u32,
    pub role: Role,
    pub infected: bool,
    pub immune: bool,
    pub quarantined: bool,
    pub mission: Mission,
    pub contact: Option<Option<usize>>,
    pub report: Option<Option<usize>>,
    pub vote: Option<Option<usize>>,
    /// Whom this player met, round by round.
    pub met: Vec<usize>,
    /// Private findings (officer, journalist), as text.
    pub notes: Vec<String>,
    /// Officer: terrorists found. Journalist: players under suspicion.
    pub suspects: Vec<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub round: u32,
    pub by: usize,
    pub target: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteRec {
    pub round: u32,
    pub by: usize,
    pub target: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct VirusState {
    pub agents: Vec<Agent>,
    pub round: u32,
    pub phase: Phase,
    pub time_left_ms: i64,
    pub reports: Vec<Report>,
    pub votes: Vec<VoteRec>,
    /// Quarantined players and whether they were terrorists, in order.
    pub revealed: Vec<(usize, bool)>,
    pub new_infections: Vec<u32>,
    pub log: Vec<String>,
    /// Replay moments: the last terrorist quarantined; more than half the city newly infected
    /// in one round (only the terrorists see that one).
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum VirusMove {
    Contact { target: usize },
    Report { target: Option<usize> },
    Vote { target: Option<usize> },
    Close,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentView {
    pub index: usize,
    pub id: String,
    pub number: u32,
    pub quarantined: bool,
    /// Side shown on quarantine: true = terrorist.
    pub revealed: Option<bool>,
    pub done: bool,
    /// Known role: your own, your partners' if you are a terrorist, everyone's at the end.
    pub role: Option<Role>,
    pub infected: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VirusView {
    pub phase: &'static str,
    pub round: u32,
    pub rounds: u32,
    pub me: usize,
    pub role: Role,
    pub role_text: &'static str,
    pub mission: &'static str,
    pub infected: bool,
    pub immune: bool,
    pub quarantined: bool,
    pub agents: Vec<AgentView>,
    pub reports: Vec<Report>,
    pub votes: Vec<VoteRec>,
    pub new_infections: Vec<u32>,
    pub notes: Vec<String>,
    pub log: Vec<String>,
    /// Whether you have acted in this phase.
    pub acted: bool,
    pub winner: Option<&'static str>,
    pub score: i64,
}

impl VirusState {
    fn say(&mut self, line: String) {
        self.log.push(line);
        let extra = self.log.len().saturating_sub(14);
        self.log.drain(..extra);
    }

    fn num(&self, i: usize) -> String {
        format!("#{}", self.agents[i].number)
    }

    fn active(&self) -> Vec<usize> {
        (0..self.agents.len()).filter(|&i| !self.agents[i].quarantined).collect()
    }

    fn resolve_contacts(&mut self) {
        let n = self.agents.len();
        let active = self.active();
        let mut targets: Vec<Option<usize>> = vec![None; n];
        for &a in &active {
            targets[a] = self.agents[a].contact.flatten().filter(|t| active.contains(t) && *t != a);
        }
        let before: Vec<bool> = self.agents.iter().map(|a| a.infected).collect();
        for (a, &target) in targets.iter().enumerate() {
            if let (Role::Researcher, Some(t)) = (self.agents[a].role, target) {
                let target = &mut self.agents[t];
                target.immune = true;
                if target.role != Role::Terrorist {
                    target.infected = false;
                }
            }
        }
        for (a, &target) in targets.iter().enumerate() {
            let Some(b) = target else { continue };
            for (x, y) in [(a, b), (b, a)] {
                if self.agents[x].role == Role::Terrorist && self.agents[y].role != Role::Terrorist && !self.agents[y].immune {
                    self.agents[y].infected = true;
                }
            }
        }
        let newly: Vec<bool> = (0..n).map(|i| self.agents[i].infected && !before[i] && self.agents[i].role != Role::Terrorist).collect();
        for a in 0..n {
            let Some(b) = targets[a] else { continue };
            self.agents[a].met.push(b);
            match self.agents[a].role {
                Role::Officer => {
                    let t = self.agents[b].role == Role::Terrorist;
                    let note = format!("Round {}: {} is {}.", self.round, self.num(b), if t { "a TERRORIST" } else { "not a terrorist" });
                    self.agents[a].notes.push(note);
                    if t && !self.agents[a].suspects.contains(&b) {
                        self.agents[a].suspects.push(b);
                    }
                }
                Role::Journalist => {
                    let note = match targets[b] {
                        Some(c) => {
                            if newly[c] && c != a && !self.agents[a].suspects.contains(&b) {
                                self.agents[a].suspects.push(b);
                            }
                            format!(
                                "Round {}: {} met {}, who was {}.",
                                self.round,
                                self.num(b),
                                self.num(c),
                                if newly[c] { "newly infected" } else { "not newly infected" }
                            )
                        }
                        None => format!("Round {}: {} met nobody.", self.round, self.num(b)),
                    };
                    self.agents[a].notes.push(note);
                }
                _ => {}
            }
        }
        let count = newly.iter().filter(|&&x| x).count() as u32;
        self.new_infections.push(count);
        // A replay moment for the terrorists only (crediting anyone would reveal a role): more
        // than half the city newly infected in one round. Each terrorist sees it as its own, since
        // whose contacts did it is secret even from a partner.
        let city = self.agents.iter().filter(|a| a.role != Role::Terrorist).count() as u32;
        if count * 2 > city {
            for t in (0..n).filter(|&t| self.agents[t].role == Role::Terrorist) {
                let id = self.agents[t].id.clone();
                self.moments.push_for(&id, format!("{count} new infections in one round"), &[&id]);
            }
        }
        self.say(format!("Round {}: everyone has met someone. New infections: {count}.", self.round));
        for agent in &mut self.agents {
            agent.contact = None;
        }
        self.phase = Phase::Report;
    }

    fn resolve_reports(&mut self) {
        let mut lines = Vec::new();
        for i in 0..self.agents.len() {
            if let Some(Some(t)) = self.agents[i].report {
                self.reports.push(Report { round: self.round, by: i, target: t });
                lines.push(format!("{} accuses {}", self.num(i), self.num(t)));
            }
            self.agents[i].report = None;
        }
        self.say(if lines.is_empty() { "Nobody accuses anyone.".into() } else { format!("Reports: {}.", lines.join("; ")) });
        self.phase = Phase::Vote;
    }

    fn resolve_votes(&mut self) {
        let n = self.agents.len();
        let mut tally = vec![0usize; n];
        let mut parts = Vec::new();
        for i in 0..n {
            let target = self.agents[i].vote.flatten().filter(|&t| t < n && !self.agents[t].quarantined);
            self.votes.push(VoteRec { round: self.round, by: i, target });
            if let Some(t) = target {
                tally[t] += 1;
            }
            parts.push(format!("{}→{}", self.num(i), target.map_or("nobody".into(), |t| self.num(t))));
            self.agents[i].vote = None;
        }
        self.say(format!("Votes: {}.", parts.join(", ")));
        let best = tally.iter().copied().max().unwrap_or(0);
        let top: Vec<usize> = (0..n).filter(|&i| tally[i] == best).collect();
        if best >= 2
            && let [q] = top[..]
        {
            self.agents[q].quarantined = true;
            let terrorist = self.agents[q].role == Role::Terrorist;
            self.revealed.push((q, terrorist));
            let line = format!("{} goes into quarantine and is {}.", self.num(q), if terrorist { "a TERRORIST" } else { "not a terrorist" });
            self.say(line);
            // A replay moment: the last terrorist quarantined (reports, votes and the side are
            // public). By the first player who accused them, or else the first who voted for them.
            if terrorist && self.agents.iter().all(|a| a.role != Role::Terrorist || a.quarantined) {
                let accuser = self.reports.iter().find(|r| r.target == q).map(|r| r.by);
                let voter = self.votes.iter().find(|v| v.round == self.round && v.target == Some(q)).map(|v| v.by);
                if let Some(by) = accuser.or(voter) {
                    let text = format!("{{{}}} quarantined: the last terrorist caught", self.agents[q].id);
                    self.moments.push(&self.agents[by].id.clone(), text);
                }
            }
        } else {
            self.say("Nobody goes into quarantine.".into());
        }
        let all_caught = self.agents.iter().all(|a| a.role != Role::Terrorist || a.quarantined);
        if all_caught || self.round >= ROUNDS {
            self.phase = Phase::Reveal;
            self.time_left_ms = REVEAL_MS;
            let line = format!("The game is over: {}.", if self.terrorists_win() { "the terrorists win" } else { "the city wins" });
            self.say(line);
        } else {
            self.round += 1;
            self.phase = Phase::Contact;
        }
    }

    pub fn terrorists_win(&self) -> bool {
        let city: Vec<&Agent> = self.agents.iter().filter(|a| a.role != Role::Terrorist).collect();
        let ill = city.iter().filter(|a| a.infected).count();
        ill * 2 >= city.len()
    }

    fn mission_done(&self, i: usize) -> bool {
        let a = &self.agents[i];
        match a.mission {
            Mission::StayHealthy => !a.infected,
            Mission::MeetTwice => a.met.iter().enumerate().any(|(k, m)| a.met[..k].contains(m)),
            Mission::CatchVoter => self.votes.iter().any(|v| v.by == i && v.target.is_some_and(|t| self.revealed.contains(&(t, true)))),
            Mission::NeverAccused => !self.reports.iter().any(|r| r.target == i),
            Mission::InfectOfficer => self.agents.iter().any(|o| o.role == Role::Officer && o.infected),
            Mission::StayFree => !a.quarantined,
        }
    }

    pub fn score(&self, i: usize) -> i64 {
        let a = &self.agents[i];
        let won = (a.role == Role::Terrorist) == self.terrorists_win();
        let mut pts = if won { 500 } else { 200 };
        if self.mission_done(i) {
            pts += 150;
        }
        if a.role != Role::Terrorist && !a.infected {
            pts += 100;
        }
        pts
    }

    /// How far a reporter is trusted, from public facts only.
    pub fn credibility(&self, by: usize) -> f64 {
        let mut c = 1.0;
        for r in self.reports.iter().filter(|r| r.by == by) {
            match self.revealed.iter().find(|(q, _)| *q == r.target) {
                Some((_, true)) => c += 1.0,
                Some((_, false)) => return 0.0,
                None => {}
            }
        }
        c
    }

    /// Public suspicion of a player who is still free.
    pub fn suspicion(&self, t: usize) -> f64 {
        if self.agents[t].quarantined {
            return 0.0;
        }
        self.reports.iter().filter(|r| r.target == t).map(|r| self.credibility(r.by)).sum()
    }
}

pub struct Rules;

impl Game for Rules {
    type State = VirusState;
    type Move = VirusMove;
    type View = VirusView;

    fn info() -> GameInfo {
        GameInfo {
            id: "virus-game",
            kind: "main",
            difficulty: "hard",
            tags: &["secret-role", "bluff", "alliance"],
            min_players: 6,
            max_players: 10,
            name: "Virus Game",
            short: "Hidden roles: terrorists spread a virus; the officer, journalist, researchers and citizens must quarantine them.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<VirusState, String> {
        let n = players.len();
        if !(6..=10).contains(&n) {
            return Err("Virus Game needs 6 to 10 players".into());
        }
        let mut rng = Rng::new(seed);
        let terrorists = if n >= 9 { 3 } else { 2 };
        let researchers = if n >= 8 { 2 } else { 1 };
        let mut roles = vec![Role::Terrorist; terrorists];
        roles.extend([Role::Officer, Role::Journalist]);
        roles.extend(vec![Role::Researcher; researchers]);
        roles.resize(n, Role::Citizen);
        rng.shuffle(&mut roles);
        let mut numbers: Vec<u32> = (1..=n as u32).collect();
        rng.shuffle(&mut numbers);
        let city = [Mission::StayHealthy, Mission::MeetTwice, Mission::CatchVoter, Mission::NeverAccused];
        let terror = [Mission::InfectOfficer, Mission::StayFree];
        let agents = players
            .iter()
            .zip(roles)
            .zip(numbers)
            .map(|((id, role), number)| Agent {
                id: id.clone(),
                number,
                role,
                infected: false,
                immune: false,
                quarantined: false,
                mission: if role == Role::Terrorist { *rng.pick(&terror) } else { *rng.pick(&city) },
                contact: None,
                report: None,
                vote: None,
                met: Vec::new(),
                notes: Vec::new(),
                suspects: Vec::new(),
            })
            .collect();
        Ok(VirusState {
            agents,
            round: 1,
            phase: Phase::Contact,
            time_left_ms: 0,
            reports: Vec::new(),
            votes: Vec::new(),
            revealed: Vec::new(),
            new_infections: Vec::new(),
            log: vec!["Round 1: choose whom to meet.".into()],
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut VirusState, player: &str, mv: VirusMove) -> Result<(), String> {
        let i = s.agents.iter().position(|a| a.id == player).ok_or("Not in this game")?;
        let n = s.agents.len();
        match (s.phase, mv) {
            (Phase::Contact, VirusMove::Contact { target }) => {
                if s.agents[i].quarantined {
                    return Err("You are in quarantine".into());
                }
                if target >= n || target == i || s.agents[target].quarantined {
                    return Err("You cannot meet that player".into());
                }
                if s.agents[i].contact.is_some() {
                    return Err("You have chosen".into());
                }
                s.agents[i].contact = Some(Some(target));
                if s.active().iter().all(|&a| s.agents[a].contact.is_some()) {
                    s.resolve_contacts();
                }
                Ok(())
            }
            (Phase::Report, VirusMove::Report { target }) => {
                if target.is_some_and(|t| t >= n || t == i) {
                    return Err("You cannot accuse that player".into());
                }
                if s.agents[i].report.is_some() {
                    return Err("You have reported".into());
                }
                s.agents[i].report = Some(target);
                if s.agents.iter().all(|a| a.report.is_some()) {
                    s.resolve_reports();
                }
                Ok(())
            }
            (Phase::Vote, VirusMove::Vote { target }) => {
                if target.is_some_and(|t| t >= n || s.agents[t].quarantined) {
                    return Err("You cannot vote for that player".into());
                }
                if s.agents[i].vote.is_some() {
                    return Err("You have voted".into());
                }
                s.agents[i].vote = Some(target);
                if s.agents.iter().all(|a| a.vote.is_some()) {
                    s.resolve_votes();
                }
                Ok(())
            }
            (Phase::Reveal, VirusMove::Close) => {
                s.phase = Phase::Over;
                Ok(())
            }
            (Phase::Over, _) => Err("Game is over".into()),
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut VirusState, ms: u64) {
        if s.phase != Phase::Reveal {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.phase = Phase::Over;
        }
    }

    fn view_for(s: &VirusState, player: &str) -> VirusView {
        let me = s.agents.iter().position(|a| a.id == player).unwrap_or(0);
        let mine = &s.agents[me];
        let end = matches!(s.phase, Phase::Reveal | Phase::Over);
        let agents = s
            .agents
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let partner = mine.role == Role::Terrorist && a.role == Role::Terrorist;
                AgentView {
                    index: i,
                    id: a.id.clone(),
                    number: a.number,
                    quarantined: a.quarantined,
                    revealed: s.revealed.iter().find(|(q, _)| *q == i).map(|&(_, t)| t),
                    done: match s.phase {
                        Phase::Contact => a.contact.is_some() || a.quarantined,
                        Phase::Report => a.report.is_some(),
                        Phase::Vote => a.vote.is_some(),
                        _ => true,
                    },
                    role: (end || i == me || partner).then_some(a.role),
                    infected: (end || i == me).then_some(a.infected),
                }
            })
            .collect();
        VirusView {
            phase: match s.phase {
                Phase::Contact => "contact",
                Phase::Report => "report",
                Phase::Vote => "vote",
                Phase::Reveal => "reveal",
                Phase::Over => "over",
            },
            round: s.round,
            rounds: ROUNDS,
            me,
            role: mine.role,
            role_text: mine.role.text(),
            mission: mine.mission.text(),
            infected: mine.infected,
            immune: mine.immune,
            quarantined: mine.quarantined,
            agents,
            reports: s.reports.clone(),
            votes: s.votes.clone(),
            new_infections: s.new_infections.clone(),
            notes: mine.notes.clone(),
            log: s.log.clone(),
            acted: match s.phase {
                Phase::Contact => mine.contact.is_some(),
                Phase::Report => mine.report.is_some(),
                Phase::Vote => mine.vote.is_some(),
                _ => true,
            },
            winner: end.then(|| if s.terrorists_win() { "terrorists" } else { "city" }),
            score: if end { s.score(me) } else { 0 },
        }
    }

    fn is_over(s: &VirusState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &VirusState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = (0..s.agents.len()).map(|i| (s.agents[i].id.clone(), s.score(i))).collect();
        rank_by_score(&scores)
    }

    fn to_move(s: &VirusState) -> Vec<String> {
        let open = |a: &Agent| match s.phase {
            Phase::Contact => a.contact.is_none() && !a.quarantined,
            Phase::Report => a.report.is_none(),
            Phase::Vote => a.vote.is_none(),
            _ => false,
        };
        s.agents.iter().filter(|a| open(a)).map(|a| a.id.clone()).collect()
    }

    fn bot_move(s: &VirusState, player: &str, seed: u64) -> VirusMove {
        let mut rng = Rng::new(seed);
        let Some(i) = s.agents.iter().position(|a| a.id == player) else { return VirusMove::Close };
        let me = &s.agents[i];
        let free: Vec<usize> = s.active().into_iter().filter(|&t| t != i).collect();
        let terrorist = |t: usize| s.agents[t].role == Role::Terrorist;
        match s.phase {
            Phase::Contact => {
                let fresh = |t: &usize| !me.met.contains(t);
                let preferred: Vec<usize> = match me.role {
                    Role::Terrorist => free.iter().copied().filter(|&t| !terrorist(t) && fresh(&t)).collect(),
                    Role::Citizen => free.iter().copied().filter(|&t| s.suspicion(t) < 0.5).collect(),
                    _ => free.iter().copied().filter(fresh).collect(),
                };
                let pool = if preferred.is_empty() { &free } else { &preferred };
                if pool.is_empty() {
                    return VirusMove::Close;
                }
                VirusMove::Contact { target: *rng.pick(pool) }
            }
            Phase::Report => {
                let open: Vec<usize> = me.suspects.iter().copied().filter(|&t| !s.agents[t].quarantined).collect();
                let target = match me.role {
                    Role::Officer | Role::Journalist => open.first().copied(),
                    Role::Terrorist if rng.next_f64() < 0.6 => {
                        let accusers: Vec<usize> =
                            s.reports.iter().filter(|r| terrorist(r.target) && !terrorist(r.by)).map(|r| r.by).collect();
                        let city: Vec<usize> = free.iter().copied().filter(|&t| !terrorist(t)).collect();
                        accusers.last().copied().or_else(|| (!city.is_empty()).then(|| *rng.pick(&city)))
                    }
                    _ => None,
                };
                VirusMove::Report { target }
            }
            Phase::Vote => {
                let candidates: Vec<usize> = (0..s.agents.len()).filter(|&t| t != i && !s.agents[t].quarantined).collect();
                let most = |pool: &[usize]| {
                    pool.iter().copied().filter(|&t| s.suspicion(t) >= 0.5).max_by(|&a, &b| s.suspicion(a).total_cmp(&s.suspicion(b)))
                };
                let target = match me.role {
                    Role::Officer | Role::Journalist => {
                        me.suspects.iter().copied().find(|&t| !s.agents[t].quarantined).or_else(|| most(&candidates))
                    }
                    Role::Terrorist => {
                        let city: Vec<usize> = candidates.iter().copied().filter(|&t| !terrorist(t)).collect();
                        most(&city).or_else(|| (!city.is_empty() && candidates.iter().any(|&t| terrorist(t) && s.suspicion(t) > 0.0)).then(|| *rng.pick(&city)))
                    }
                    _ => most(&candidates),
                };
                VirusMove::Vote { target }
            }
            _ => VirusMove::Close,
        }
    }

    fn moments(s: &VirusState, player: &str) -> Vec<Moment> {
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

    fn find(s: &VirusState, role: Role) -> usize {
        s.agents.iter().position(|a| a.role == role).unwrap()
    }

    #[test]
    fn roles_by_player_count() {
        for (n, t, r) in [(6, 2, 1), (8, 2, 2), (10, 3, 2)] {
            let s = Rules::setup(&players(n), 1).unwrap();
            assert_eq!(s.agents.iter().filter(|a| a.role == Role::Terrorist).count(), t);
            assert_eq!(s.agents.iter().filter(|a| a.role == Role::Researcher).count(), r);
        }
    }

    #[test]
    fn views_hide_roles() {
        let s = Rules::setup(&players(6), 2).unwrap();
        let c = find(&s, Role::Citizen);
        let v = Rules::view_for(&s, &s.agents[c].id);
        assert_eq!(v.agents.iter().filter(|a| a.role.is_some()).count(), 1);
        let t = find(&s, Role::Terrorist);
        let v = Rules::view_for(&s, &s.agents[t].id);
        assert_eq!(v.agents.iter().filter(|a| a.role == Some(Role::Terrorist)).count(), 2);
    }

    #[test]
    fn meeting_a_terrorist_infects_unless_vaccinated() {
        let mut s = Rules::setup(&players(6), 3).unwrap();
        let t = find(&s, Role::Terrorist);
        let c = find(&s, Role::Citizen);
        let o = find(&s, Role::Officer);
        let r = find(&s, Role::Researcher);
        for a in 0..6 {
            s.agents[a].contact = Some(Some(if a == t {
                c
            } else if a == r {
                o
            } else if a == o {
                t
            } else {
                r
            }));
        }
        s.resolve_contacts();
        assert!(s.agents[c].infected);
        assert!(!s.agents[o].infected, "vaccinated first");
        assert!(s.agents[o].notes[0].contains("TERRORIST"));
        assert_eq!(s.phase, Phase::Report);
    }

    #[test]
    fn a_vote_needs_two_and_no_tie() {
        let mut s = Rules::setup(&players(6), 4).unwrap();
        s.phase = Phase::Vote;
        let t = find(&s, Role::Terrorist);
        for (k, p) in players(6).iter().enumerate() {
            let target = if k < 3 && k != t { Some(t) } else { None };
            Rules::apply_move(&mut s, p, VirusMove::Vote { target }).unwrap();
        }
        assert!(s.agents[t].quarantined);
        assert_eq!(s.revealed, vec![(t, true)]);
        assert!(Rules::apply_move(&mut s, "p1", VirusMove::Contact { target: t }).is_err());
    }

    #[test]
    fn both_sides_win_sometimes() {
        for n in [6, 8, 10] {
            let mut terror = 0;
            let games = 300;
            for seed in 1..=games {
                let mut s = Rules::setup(&players(n), seed).unwrap();
                let mut guard = 0;
                while !matches!(s.phase, Phase::Reveal | Phase::Over) {
                    for p in Rules::to_move(&s) {
                        let mv = Rules::bot_move(&s, &p, seed * 31 + guard);
                        let _ = Rules::apply_move(&mut s, &p, mv);
                    }
                    guard += 1;
                    assert!(guard < 100);
                }
                terror += usize::from(s.terrorists_win());
            }
            eprintln!("{n} players: terrorists won {terror} of {games}");
            assert!(terror > games as usize / 5 && terror < games as usize * 4 / 5, "{n} players: terrorists won {terror} of {games}");
        }
    }

    #[test]
    fn no_timer_the_game_waits_for_every_contact() {
        let mut s = Rules::setup(&players(6), 5).unwrap();
        Rules::tick(&mut s, 600_000);
        assert_eq!(s.phase, Phase::Contact);
        assert_eq!(Rules::to_move(&s).len(), 6);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(1000, 6, 3000);
        bot_games_with::<Rules>(200, 10, 6000);
    }

    #[test]
    fn the_last_terrorist_quarantined_is_a_public_replay_moment() {
        let mut s = Rules::setup(&players(6), 4).unwrap();
        let ts: Vec<usize> = (0..6).filter(|&k| s.agents[k].role == Role::Terrorist).collect();
        s.agents[ts[0]].quarantined = true;
        s.revealed.push((ts[0], true));
        s.phase = Phase::Report;
        let o = find(&s, Role::Officer);
        for (k, p) in players(6).iter().enumerate() {
            Rules::apply_move(&mut s, p, VirusMove::Report { target: (k == o).then_some(ts[1]) }).unwrap();
        }
        for (k, p) in players(6).iter().enumerate() {
            Rules::apply_move(&mut s, p, VirusMove::Vote { target: (!ts.contains(&k)).then_some(ts[1]) }).unwrap();
        }
        assert!(s.agents[ts[1]].quarantined);
        let c = find(&s, Role::Citizen);
        let m = Rules::moments(&s, &s.agents[c].id);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].by, s.agents[o].id);
        assert_eq!(m[0].text, format!("{{{}}} quarantined: the last terrorist caught", s.agents[ts[1]].id));
    }

    #[test]
    fn a_big_outbreak_is_a_replay_moment_for_the_terrorists_only() {
        let mut s = Rules::setup(&players(6), 3).unwrap();
        let ts: Vec<usize> = (0..6).filter(|&k| s.agents[k].role == Role::Terrorist).collect();
        let (o, j, r, c) = (find(&s, Role::Officer), find(&s, Role::Journalist), find(&s, Role::Researcher), find(&s, Role::Citizen));
        // ts[0] infects the officer and the citizen who meets it, ts[1] the journalist and the
        // researcher who meets it: 4 of 4. Each terrorist sees it as its own moment, not its partner's.
        for (a, b) in [(ts[0], o), (ts[1], j), (c, ts[0]), (r, ts[1]), (o, ts[0]), (j, ts[1])] {
            s.agents[a].contact = Some(Some(b));
        }
        s.resolve_contacts();
        assert_eq!(s.new_infections, vec![4]);
        for t in &ts {
            let m = Rules::moments(&s, &s.agents[*t].id);
            assert_eq!(m.len(), 1);
            assert_eq!(m[0].by, s.agents[*t].id);
            assert_eq!(m[0].text, "4 new infections in one round");
        }
        for k in [o, j, r, c] {
            assert!(Rules::moments(&s, &s.agents[k].id).is_empty());
        }
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 6, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}

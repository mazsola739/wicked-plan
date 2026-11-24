//! Either Or (main game, 3–8 seats). Rules page: docs/games/either-or.md
//!
//! Ten two-way questions scored by majority or minority, with four surprise
//! mini rounds in between. Every round is a secret, simultaneous pick.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const QUESTIONS: usize = 10;

const PROMPTS: [(&str, &str, &str); 16] = [
    ("Which would you rather keep?", "Sunrise", "Sunset"),
    ("Pick a drink for a long night.", "Coffee", "Tea"),
    ("Your dream holiday?", "Mountains", "Seaside"),
    ("The better pet?", "Cat", "Dog"),
    ("Pick a superpower.", "Flying", "Invisibility"),
    ("Where do you sleep better?", "City", "Countryside"),
    ("Which is worse?", "Too hot", "Too cold"),
    ("Pick a breakfast.", "Sweet", "Savoury"),
    ("Your getaway car?", "Fast", "Quiet"),
    ("Which would you trust?", "A map", "Your gut"),
    ("Pick a seat.", "Window", "Aisle"),
    ("Better time to think?", "Early morning", "Late night"),
    ("How do you win?", "Plan ahead", "Improvise"),
    ("Pick a prize.", "Cash now", "Double next week"),
    ("Which tool?", "Rope", "Torch"),
    ("Escape through the…", "Roof", "Tunnel"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Majority,
    Minority,
    LowestUnique,
    ShareGrab,
    Finger,
    LoneWolf,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Round {
    pub kind: Kind,
    pub prompt: String,
    /// Answer labels for two-way rounds; empty otherwise.
    pub options: Vec<String>,
    /// How often bots lean to option 0 (hidden).
    #[serde(skip)]
    pub lean: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub number: usize,
    pub kind: Kind,
    pub prompt: String,
    pub options: Vec<String>,
    pub picks: Vec<i64>,
    pub gains: Vec<i64>,
    pub note: String,
}

#[derive(Clone, Debug)]
pub struct EitherState {
    pub players: Vec<String>,
    pub rounds: Vec<Round>,
    pub current: usize,
    pub picks: Vec<Option<i64>>,
    pub scores: Vec<i64>,
    pub last: Option<Outcome>,
    pub over: bool,
    /// Replay moments: a lone grab, and the lead stolen in the last round.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EitherMove {
    Pick { value: i64 },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EitherView {
    pub players: Vec<String>,
    pub me: usize,
    pub number: usize,
    pub total: usize,
    pub round: Option<Round>,
    pub my_pick: Option<i64>,
    pub done: Vec<bool>,
    pub scores: Vec<i64>,
    pub last: Option<Outcome>,
    pub over: bool,
}

fn two_way(kind: Kind, (prompt, a, b): (&str, &str, &str), lean: f64) -> Round {
    Round { kind, prompt: prompt.into(), options: vec![a.into(), b.into()], lean }
}

fn mini(kind: Kind) -> Round {
    let prompt = match kind {
        Kind::LowestUnique => "Lowest Unique: pick 1–5. The lowest number nobody else picked scores +3.",
        Kind::ShareGrab => "Share or Grab: all share, +2 each. One grabber takes +5. Two or more grabbers, −1 each.",
        Kind::Finger => "Point the Finger: name a player. The most-named loses 3; who named them scores +1.",
        _ => "",
    };
    let options = if kind == Kind::ShareGrab { vec!["Share".into(), "Grab".into()] } else { Vec::new() };
    Round { kind, prompt: prompt.into(), options, lean: 0.7 }
}

impl EitherState {
    fn valid(&self, seat: usize, value: i64) -> bool {
        match self.rounds[self.current].kind {
            Kind::LowestUnique => (1..=5).contains(&value),
            Kind::Finger => value >= 0 && (value as usize) < self.players.len() && value as usize != seat,
            _ => value == 0 || value == 1,
        }
    }

    fn resolve(&mut self) {
        let n = self.players.len();
        let picks: Vec<i64> = self.picks.iter().map(|p| p.unwrap_or(0)).collect();
        let round = &self.rounds[self.current];
        let mut gains = vec![0; n];
        let side = |v: i64| picks.iter().filter(|&&p| p == v).count();
        let note = match round.kind {
            Kind::Majority | Kind::Minority => {
                let (a, b) = (side(0), side(1));
                let win = match round.kind {
                    Kind::Majority if a != b => Some(if a > b { 0 } else { 1 }),
                    Kind::Minority if a != b && a.min(b) > 0 => Some(if a < b { 0 } else { 1 }),
                    _ => None,
                };
                let bonus = if round.kind == Kind::Majority { 1 } else { 2 };
                match win {
                    Some(w) => {
                        for (g, &p) in gains.iter_mut().zip(&picks) {
                            if p == w {
                                *g = bonus;
                            }
                        }
                        format!("{} wins: +{bonus} each.", round.options[w as usize])
                    }
                    None => "No winning side.".into(),
                }
            }
            Kind::LoneWolf => {
                let lone: Vec<usize> = (0..n).filter(|&s| side(picks[s]) == 1).collect();
                for &s in &lone {
                    gains[s] = 4;
                }
                if lone.is_empty() { "Nobody stood alone.".into() } else { "A lone wolf scores +4.".into() }
            }
            Kind::LowestUnique => match (1..=5).find(|&v| side(v) == 1) {
                Some(v) => {
                    let s = picks.iter().position(|&p| p == v).unwrap_or(0);
                    gains[s] = 3;
                    format!("{v} was the lowest unique number: +3.")
                }
                None => "No unique number.".into(),
            },
            Kind::ShareGrab => {
                let grabs = side(1);
                for (g, &p) in gains.iter_mut().zip(&picks) {
                    *g = match (grabs, p) {
                        (0, _) => 2,
                        (1, 1) => 5,
                        (1, _) => 0,
                        (_, 1) => -1,
                        _ => 0,
                    };
                }
                match grabs {
                    0 => "Everyone shared: +2 each.".into(),
                    1 => "One grabber took +5.".into(),
                    _ => "Too many grabbers: −1 each.".into(),
                }
            }
            Kind::Finger => {
                let named: Vec<usize> = (0..n).map(|s| side(s as i64)).collect();
                let top = named.iter().copied().max().unwrap_or(0);
                let tops: Vec<usize> = (0..n).filter(|&s| named[s] == top).collect();
                if tops.len() == 1 {
                    let t = tops[0];
                    gains[t] -= 3;
                    for s in 0..n {
                        if picks[s] == t as i64 {
                            gains[s] += 1;
                        }
                    }
                    format!("{} was named most: −3.", self.players[t])
                } else {
                    "A tie for most named: nothing happens.".into()
                }
            }
        };
        let top_before = self.scores.iter().copied().max().unwrap_or(0);
        for (s, g) in self.scores.iter_mut().zip(&gains) {
            *s += g;
        }
        // Replay moments (every pick is shown after the round anyway).
        if round.kind == Kind::ShareGrab && side(1) == 1 {
            let g = picks.iter().position(|&p| p == 1).unwrap_or(0);
            self.moments.push(&self.players[g], "Lone grab while the rest shared (+5)");
        }
        let top = self.scores.iter().copied().max().unwrap_or(0);
        let leaders: Vec<usize> = (0..n).filter(|&i| self.scores[i] == top).collect();
        if self.current + 1 == self.rounds.len() && leaders.len() == 1 && self.scores[leaders[0]] - gains[leaders[0]] < top_before {
            self.moments.push(&self.players[leaders[0]], "Lead stolen in the very last round");
        }
        self.last = Some(Outcome {
            number: self.current + 1,
            kind: round.kind,
            prompt: round.prompt.clone(),
            options: round.options.clone(),
            picks,
            gains,
            note,
        });
        self.current += 1;
        self.picks = vec![None; n];
        if self.current >= self.rounds.len() {
            self.over = true;
        }
    }
}

/// Ranks from scores; points 1000 for first down to 100 for last.
pub fn rank_points(scores: &[(String, i64)]) -> Vec<PlayerResult> {
    let n = scores.len() as i64;
    rank_by_score(scores)
        .into_iter()
        .map(|mut r| {
            r.points = 100 + 900 * (n - i64::from(r.rank)) / (n - 1).max(1);
            r
        })
        .collect()
}

pub struct Rules;

impl Game for Rules {
    type State = EitherState;
    type Move = EitherMove;
    type View = EitherView;

    fn info() -> GameInfo {
        GameInfo {
            id: "either-or",
            kind: "main",
            difficulty: "hard",
            tags: &["alliance", "words"],
            min_players: 3,
            max_players: 8,
            name: "Either Or",
            short: "Ten two-way choices scored by majority or minority, with four surprise mini rounds.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<EitherState, String> {
        if !(3..=8).contains(&players.len()) {
            return Err("Either Or needs 3–8 players".into());
        }
        let n = players.len();
        let mut rng = Rng::new(seed);
        let mut prompts = PROMPTS.to_vec();
        rng.shuffle(&mut prompts);
        let mut rounds: Vec<Round> = prompts[..QUESTIONS]
            .iter()
            .map(|&p| {
                let kind = if rng.next_f64() < 0.5 { Kind::Majority } else { Kind::Minority };
                let lean = 0.25 + 0.5 * rng.next_f64();
                two_way(kind, p, if lean < 0.5 { lean - 0.1 } else { lean + 0.1 })
            })
            .collect();
        let mut minis = vec![
            mini(Kind::LowestUnique),
            mini(Kind::ShareGrab),
            mini(Kind::Finger),
            two_way(Kind::LoneWolf, prompts[QUESTIONS], 0.5 + 0.2 * rng.next_f64()),
        ];
        rng.shuffle(&mut minis);
        // Four distinct gaps after questions 1–9, inserted from the back.
        let mut gaps: Vec<usize> = (1..QUESTIONS).collect();
        rng.shuffle(&mut gaps);
        let mut gaps = gaps[..minis.len()].to_vec();
        gaps.sort_unstable_by(|a, b| b.cmp(a));
        for (gap, m) in gaps.into_iter().zip(minis) {
            rounds.insert(gap, m);
        }
        Ok(EitherState {
            players: players.to_vec(),
            rounds,
            current: 0,
            picks: vec![None; n],
            scores: vec![0; n],
            last: None,
            over: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut EitherState, player: &str, mv: EitherMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.players.iter().position(|p| p == player).ok_or("Not in this game")?;
        let EitherMove::Pick { value } = mv;
        if s.picks[seat].is_some() {
            return Err("You already picked".into());
        }
        if !s.valid(seat, value) {
            return Err("Not a valid pick".into());
        }
        s.picks[seat] = Some(value);
        if s.picks.iter().all(Option::is_some) {
            s.resolve();
        }
        Ok(())
    }

    fn view_for(s: &EitherState, player: &str) -> EitherView {
        let me = s.players.iter().position(|p| p == player).unwrap_or(0);
        EitherView {
            players: s.players.clone(),
            me,
            number: (s.current + 1).min(s.rounds.len()),
            total: s.rounds.len(),
            round: (!s.over).then(|| s.rounds[s.current].clone()),
            my_pick: s.picks[me],
            done: s.picks.iter().map(Option::is_some).collect(),
            scores: s.scores.clone(),
            last: s.last.clone(),
            over: s.over,
        }
    }

    fn is_over(s: &EitherState) -> bool {
        s.over
    }

    fn results(s: &EitherState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.players.iter().cloned().zip(s.scores.iter().copied()).collect();
        rank_points(&scores)
    }

    fn to_move(s: &EitherState) -> Vec<String> {
        if s.over {
            return Vec::new();
        }
        (0..s.players.len()).filter(|&i| s.picks[i].is_none()).map(|i| s.players[i].clone()).collect()
    }

    fn bot_move(s: &EitherState, player: &str, seed: u64) -> EitherMove {
        let mut rng = Rng::new(seed);
        let seat = s.players.iter().position(|p| p == player).unwrap_or(0);
        let n = s.players.len();
        let round = &s.rounds[s.current.min(s.rounds.len() - 1)];
        let lean = |against: f64, rng: &mut Rng| {
            let towards = if round.lean >= 0.5 { 0 } else { 1 };
            let p = (round.lean - 0.5).abs() + 0.5;
            let follow = rng.next_f64() < p * (1.0 - against) + (1.0 - p) * against;
            if follow { towards } else { 1 - towards }
        };
        let value = match round.kind {
            Kind::Majority => lean(0.0, &mut rng),
            Kind::Minority => lean(0.45, &mut rng),
            Kind::LoneWolf => lean(0.35, &mut rng),
            Kind::ShareGrab => i64::from(rng.next_f64() < 0.3),
            Kind::LowestUnique => *rng.pick(&[1, 1, 1, 2, 2, 2, 3, 3, 4, 5]),
            Kind::Finger => {
                let others: Vec<usize> = (0..n).filter(|&o| o != seat).collect();
                let leader = others.iter().copied().max_by_key(|&o| (s.scores[o], o)).unwrap_or(others[0]);
                (if rng.next_f64() < 0.6 { leader } else { *rng.pick(&others) }) as i64
            }
        };
        EitherMove::Pick { value }
    }

    fn moments(s: &EitherState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn ids(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    /// Puts the round at `current` to this kind and plays these picks.
    fn play(s: &mut EitherState, kind: Kind, picks: &[i64]) -> Vec<i64> {
        s.rounds[s.current].kind = kind;
        let before = s.scores.clone();
        for (i, &v) in picks.iter().enumerate() {
            Rules::apply_move(s, &format!("p{}", i + 1), EitherMove::Pick { value: v }).unwrap();
        }
        s.scores.iter().zip(&before).map(|(a, b)| a - b).collect()
    }

    #[test]
    fn fourteen_rounds_with_four_minis_not_first() {
        for seed in 1..50 {
            let s = Rules::setup(&ids(4), seed).unwrap();
            assert_eq!(s.rounds.len(), 14);
            assert!(matches!(s.rounds[0].kind, Kind::Majority | Kind::Minority));
            let minis: Vec<Kind> =
                s.rounds.iter().map(|r| r.kind).filter(|k| !matches!(k, Kind::Majority | Kind::Minority)).collect();
            assert_eq!(minis.len(), 4);
            assert!(s.rounds.iter().filter(|r| !r.options.is_empty()).all(|r| r.options.len() == 2));
        }
    }

    #[test]
    fn majority_and_minority() {
        let mut s = Rules::setup(&ids(4), 1).unwrap();
        assert_eq!(play(&mut s, Kind::Majority, &[0, 0, 0, 1]), vec![1, 1, 1, 0]);
        assert_eq!(play(&mut s, Kind::Minority, &[0, 0, 0, 1]), vec![0, 0, 0, 2]);
        assert_eq!(play(&mut s, Kind::Majority, &[0, 0, 1, 1]), vec![0, 0, 0, 0]);
        assert_eq!(play(&mut s, Kind::Minority, &[1, 1, 1, 1]), vec![0, 0, 0, 0]);
    }

    #[test]
    fn mini_rounds_score() {
        let mut s = Rules::setup(&ids(4), 2).unwrap();
        assert_eq!(play(&mut s, Kind::LowestUnique, &[1, 1, 2, 3]), vec![0, 0, 3, 0]);
        assert_eq!(play(&mut s, Kind::ShareGrab, &[0, 0, 0, 0]), vec![2, 2, 2, 2]);
        assert_eq!(play(&mut s, Kind::ShareGrab, &[0, 1, 0, 0]), vec![0, 5, 0, 0]);
        assert_eq!(play(&mut s, Kind::ShareGrab, &[1, 1, 0, 0]), vec![-1, -1, 0, 0]);
        assert_eq!(play(&mut s, Kind::Finger, &[3, 3, 0, 1]), vec![1, 1, 0, -3]);
        assert_eq!(play(&mut s, Kind::Finger, &[3, 3, 3, 2]), vec![1, 1, 1, -3]);
        assert_eq!(play(&mut s, Kind::LoneWolf, &[0, 0, 0, 1]), vec![0, 0, 0, 4]);
    }

    #[test]
    fn bad_picks_are_refused() {
        let mut s = Rules::setup(&ids(3), 3).unwrap();
        s.rounds[0].kind = Kind::Finger;
        assert!(Rules::apply_move(&mut s, "p1", EitherMove::Pick { value: 0 }).is_err(), "not yourself");
        assert!(Rules::apply_move(&mut s, "p1", EitherMove::Pick { value: 3 }).is_err());
        Rules::apply_move(&mut s, "p1", EitherMove::Pick { value: 2 }).unwrap();
        assert!(Rules::apply_move(&mut s, "p1", EitherMove::Pick { value: 1 }).is_err(), "only once");
    }

    #[test]
    fn no_timer_and_the_lean_stays_hidden() {
        let mut s = Rules::setup(&ids(3), 4).unwrap();
        let v = serde_json::to_string(&Rules::view_for(&s, "p1")).unwrap();
        assert!(!v.contains("lean"));
        Rules::tick(&mut s, 600_000);
        assert_eq!(s.current, 0);
        assert!(s.last.is_none());
        assert_eq!(Rules::to_move(&s).len(), 3);
    }

    #[test]
    fn thousand_bot_games() {
        for n in [3, 8] {
            bot_games_with::<Rules>(500, n, 3000);
        }
    }

    #[test]
    fn a_lone_grab_and_a_last_round_steal_are_replay_moments() {
        let mut s = Rules::setup(&ids(4), 2).unwrap();
        play(&mut s, Kind::ShareGrab, &[0, 1, 0, 0]);
        let m = Rules::moments(&s, "p3");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p2", "Lone grab while the rest shared (+5)"));
        // Two grabbers: no moment.
        play(&mut s, Kind::ShareGrab, &[1, 1, 0, 0]);
        assert_eq!(Rules::moments(&s, "p1").len(), 1);
        // Last round: p4 trails p1 by one, then takes the lead alone.
        s.current = s.rounds.len() - 1;
        s.scores = vec![5, 0, 0, 4];
        play(&mut s, Kind::Minority, &[0, 0, 0, 1]);
        let m = Rules::moments(&s, "p1");
        assert_eq!(m.len(), 2);
        assert_eq!((m[1].by.as_str(), m[1].text.as_str()), ("p4", "Lead stolen in the very last round"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 3_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 2, "too many in one game: {stats:?}");
    }
}

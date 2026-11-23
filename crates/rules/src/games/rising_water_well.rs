//! Rising Water Well (hidden, solo). Rules page: docs/games/rising-water-well.md
//!
//! Four picture riddles, one after the other, before the well fills
//! (10 minutes; a wrong answer costs 30 seconds). Every riddle is made from
//! the seed; the view carries only what is drawn, never the answer.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult};
use crate::rng::Rng;

pub const TOTAL_MS: i64 = 10 * 60_000;
pub const WRONG_MS: i64 = 30_000;
pub const RIDDLES: usize = 4;
pub const RIDDLE_POINTS: i64 = 125;
pub const ESCAPE_POINTS: i64 = 200;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stone {
    /// 0 circle, 1 triangle, 2 square, 3 star.
    pub shape: u8,
    /// 0 red, 1 blue, 2 gold.
    pub colour: u8,
    /// Position in a 100 × 100 box.
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolSum {
    /// 0 star, 1 moon, 2 drop.
    pub symbols: Vec<u8>,
    /// None on the sum to work out.
    pub total: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Riddle {
    /// How many stones of `shape` (and `colour`, when given)?
    Stones {
        stones: Vec<Stone>,
        shape: u8,
        colour: Option<u8>,
    },
    /// Four groups of marks; how many come next?
    Scratches {
        groups: Vec<i64>,
    },
    /// `circles` circles = 1 square; 1 triangle = 1 square + `extra` circles.
    Scales {
        circles: i64,
        extra: i64,
    },
    Symbols {
        sums: Vec<SymbolSum>,
    },
    /// Angles (degrees clockwise from the top) as seen in the water.
    Reflection {
        hour_angle: i64,
        minute_angle: i64,
    },
}

/// One riddle of the given kind (0–4) and its answer.
pub fn make_riddle(kind: usize, rng: &mut Rng) -> (Riddle, i64) {
    match kind {
        0 => {
            let mut slots: Vec<(i64, i64)> = (0..20).map(|i| (i % 5, i / 5)).collect();
            rng.shuffle(&mut slots);
            let stones: Vec<Stone> = slots[..rng.int(13, 17) as usize]
                .iter()
                .map(|&(cx, cy)| Stone {
                    shape: rng.int(0, 3) as u8,
                    colour: rng.int(0, 2) as u8,
                    x: 10 + cx * 20 + rng.int(-3, 3),
                    y: 12 + cy * 25 + rng.int(-3, 3),
                })
                .collect();
            let pick = rng.pick(&stones).clone();
            let colour = (rng.next_f64() < 0.6).then_some(pick.colour);
            let answer = stones.iter().filter(|s| s.shape == pick.shape && colour.is_none_or(|c| s.colour == c)).count() as i64;
            (Riddle::Stones { stones, shape: pick.shape, colour }, answer)
        }
        1 => {
            let mut groups: Vec<i64> = match rng.int(0, 2) {
                0 => {
                    let (a, d) = (rng.int(1, 5), rng.int(2, 5));
                    (0..5).map(|i| a + d * i).collect()
                }
                1 => {
                    let a = rng.int(1, 6);
                    (0..5).map(|i| a + i * (i + 1) / 2).collect()
                }
                _ => {
                    let a = rng.int(1, 2);
                    (0..5).map(|i| a << i).collect()
                }
            };
            let answer = groups.pop().expect("five groups");
            (Riddle::Scratches { groups }, answer)
        }
        2 => {
            let (circles, extra) = (rng.int(2, 4), rng.int(1, 3));
            (Riddle::Scales { circles, extra }, circles + extra)
        }
        3 => {
            let mut values = [0i64; 3];
            for v in &mut values {
                *v = rng.int(1, 9);
            }
            // Which picture stands for which unknown.
            let mut sym = [0u8, 1, 2];
            rng.shuffle(&mut sym);
            let [a, b, c] = values;
            let sum = |symbols: Vec<u8>, total: Option<i64>| SymbolSum { symbols, total };
            let sums = vec![
                sum(vec![sym[0], sym[0]], Some(2 * a)),
                sum(vec![sym[0], sym[1]], Some(a + b)),
                sum(vec![sym[1], sym[2], sym[2]], Some(b + 2 * c)),
                sum(vec![sym[2], sym[0], sym[1]], None),
            ];
            (Riddle::Symbols { sums }, a + b + c)
        }
        _ => {
            let hour = rng.int(1, 12);
            // Mirrored in a level water surface: angle θ is seen at 180° − θ.
            let hour_angle = (180 - 30 * hour).rem_euclid(360);
            (Riddle::Reflection { hour_angle, minute_angle: 180 }, hour)
        }
    }
}

#[derive(Clone, Debug)]
pub struct WellState {
    pub player: String,
    pub riddles: Vec<(Riddle, i64)>,
    pub index: usize,
    pub wrong_tries: u32,
    pub last_wrong: Option<i64>,
    pub score: i64,
    pub time_left_ms: i64,
    pub escaped: bool,
    pub over: bool,
    /// Replay moments: an escape without a wrong answer, or one in the last minute.
    pub moments: Moments,
}

/// `ms` as m:ss.
fn clock(ms: i64) -> String {
    let secs = ms.max(0) / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WellMove {
    Answer { value: i64 },
    GiveUp,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WellView {
    pub riddle: Option<Riddle>,
    pub index: usize,
    pub riddles: usize,
    pub solved: usize,
    pub wrong_tries: u32,
    pub last_wrong: Option<i64>,
    pub score: i64,
    pub time_left_ms: i64,
    pub total_ms: i64,
    pub escaped: bool,
    pub over: bool,
}

pub struct Rules;

impl Game for Rules {
    type State = WellState;
    type Move = WellMove;
    type View = WellView;

    fn info() -> GameInfo {
        GameInfo {
            id: "rising-water-well",
            kind: "hidden",
            difficulty: "hard",
            tags: &["numbers", "logic"],
            min_players: 1,
            max_players: 1,
            name: "Rising Water Well",
            short: "Solve four picture riddles on the wall of a well before the water reaches the top.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<WellState, String> {
        let [player] = players else { return Err("Rising Water Well is a solo game".into()) };
        let mut rng = Rng::new(seed);
        let mut kinds = [0usize, 1, 2, 3, 4];
        rng.shuffle(&mut kinds);
        Ok(WellState {
            player: player.clone(),
            riddles: kinds[..RIDDLES].iter().map(|&k| make_riddle(k, &mut rng)).collect(),
            index: 0,
            wrong_tries: 0,
            last_wrong: None,
            score: 0,
            time_left_ms: TOTAL_MS,
            escaped: false,
            over: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut WellState, player: &str, mv: WellMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        if player != s.player {
            return Err("Not your game".into());
        }
        match mv {
            WellMove::Answer { value } => {
                if !(0..=99).contains(&value) {
                    return Err("Answers are 0 to 99".into());
                }
                if value == s.riddles[s.index].1 {
                    s.score += RIDDLE_POINTS;
                    s.index += 1;
                    s.last_wrong = None;
                    if s.index == RIDDLES {
                        s.escaped = true;
                        s.over = true;
                        s.score += ESCAPE_POINTS + s.time_left_ms / 2000;
                        // Replay moment: a flawless escape, or one with the water at the top.
                        let left = clock(s.time_left_ms);
                        if s.wrong_tries == 0 {
                            s.moments.push(&s.player.clone(), format!("Flawless escape, {left} to spare"));
                        } else if s.time_left_ms < 60_000 {
                            s.moments.push(&s.player.clone(), format!("Out of the well with {left} left"));
                        }
                    }
                } else {
                    s.wrong_tries += 1;
                    s.last_wrong = Some(value);
                    Self::tick(s, WRONG_MS as u64);
                }
            }
            WellMove::GiveUp => s.over = true,
        }
        Ok(())
    }

    fn tick(s: &mut WellState, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.time_left_ms = 0;
            s.over = true;
        }
    }

    fn view_for(s: &WellState, _player: &str) -> WellView {
        WellView {
            riddle: (!s.over).then(|| s.riddles[s.index].0.clone()),
            index: (s.index + 1).min(RIDDLES),
            riddles: RIDDLES,
            solved: s.index,
            wrong_tries: s.wrong_tries,
            last_wrong: s.last_wrong,
            score: s.score,
            time_left_ms: s.time_left_ms,
            total_ms: TOTAL_MS,
            escaped: s.escaped,
            over: s.over,
        }
    }

    fn is_over(s: &WellState) -> bool {
        s.over
    }

    fn results(s: &WellState) -> Vec<PlayerResult> {
        vec![PlayerResult { player_id: s.player.clone(), rank: 1, points: s.score }]
    }

    fn moments(s: &WellState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &WellState, _player: &str, seed: u64) -> WellMove {
        let answer = s.riddles[s.index.min(RIDDLES - 1)].1;
        let mut rng = Rng::new(seed);
        WellMove::Answer { value: if rng.next_f64() < 0.8 { answer } else { answer + 1 } }
    }
}

pub const READY: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games;

    const P: &str = "p1";

    #[test]
    fn riddles_have_fair_answers() {
        for seed in 0..500 {
            let mut rng = Rng::new(seed);
            for kind in 0..5 {
                let (riddle, answer) = make_riddle(kind, &mut rng);
                assert!((0..=99).contains(&answer));
                match riddle {
                    Riddle::Stones { stones, shape, colour } => {
                        assert!(answer >= 1, "the asked stone is in the pile");
                        assert!(stones.iter().all(|s| (0..=100).contains(&s.x) && (0..=100).contains(&s.y)));
                        assert!(stones.iter().any(|s| s.shape == shape && colour.is_none_or(|c| c == s.colour)));
                    }
                    Riddle::Scratches { groups } => {
                        assert_eq!(groups.len(), 4);
                        assert!(answer <= 32 && groups.iter().all(|&g| g >= 1));
                    }
                    Riddle::Scales { circles, extra } => assert_eq!(answer, circles + extra),
                    Riddle::Symbols { sums } => {
                        // The three totals pin down every symbol: the answer follows from them.
                        let t: Vec<i64> = sums[..3].iter().map(|s| s.total.unwrap()).collect();
                        let a = t[0] / 2;
                        let b = t[1] - a;
                        let c = (t[2] - b) / 2;
                        assert_eq!(answer, a + b + c);
                        assert!(sums[3].total.is_none());
                    }
                    Riddle::Reflection { hour_angle, .. } => {
                        // Mirroring twice gives the real hour back.
                        assert_eq!((180 - hour_angle).rem_euclid(360), (30 * answer) % 360);
                    }
                }
            }
        }
    }

    #[test]
    fn answers_open_locks_and_wrong_ones_cost_time() {
        let mut s = Rules::setup(&[P.into()], 7).unwrap();
        let wrong = (s.riddles[0].1 + 1) % 100;
        Rules::apply_move(&mut s, P, WellMove::Answer { value: wrong }).unwrap();
        assert_eq!((s.time_left_ms, s.wrong_tries, s.index), (TOTAL_MS - WRONG_MS, 1, 0));
        assert!(Rules::apply_move(&mut s, P, WellMove::Answer { value: 100 }).is_err());
        assert!(serde_json::to_string(&Rules::view_for(&s, P)).unwrap().contains("\"kind\""));
        for i in 0..RIDDLES {
            let v = s.riddles[i].1;
            Rules::apply_move(&mut s, P, WellMove::Answer { value: v }).unwrap();
        }
        assert!(s.escaped && s.over);
        assert_eq!(s.score, 4 * RIDDLE_POINTS + ESCAPE_POINTS + (TOTAL_MS - WRONG_MS) / 2000);
    }

    #[test]
    fn the_well_fills() {
        let mut s = Rules::setup(&[P.into()], 8).unwrap();
        let v = s.riddles[0].1;
        Rules::apply_move(&mut s, P, WellMove::Answer { value: v }).unwrap();
        Rules::tick(&mut s, TOTAL_MS as u64);
        assert!(s.over && !s.escaped);
        assert_eq!(s.score, RIDDLE_POINTS);
        assert!(Rules::view_for(&s, P).riddle.is_none());
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<Rules>(1000, 2000);
    }

    #[test]
    fn a_flawless_or_last_minute_escape_is_a_replay_moment() {
        let solve = |s: &mut WellState| {
            for i in 0..RIDDLES {
                let v = s.riddles[i].1;
                Rules::apply_move(s, P, WellMove::Answer { value: v }).unwrap();
            }
        };
        let mut s = Rules::setup(&[P.into()], 7).unwrap();
        Rules::tick(&mut s, 75_000);
        solve(&mut s);
        let m = Rules::moments(&s, P);
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), (P, "Flawless escape, 8:45 to spare"));
        // One wrong answer and the water near the top.
        let mut s = Rules::setup(&[P.into()], 7).unwrap();
        let wrong = (s.riddles[0].1 + 1) % 100;
        Rules::apply_move(&mut s, P, WellMove::Answer { value: wrong }).unwrap();
        Rules::tick(&mut s, (TOTAL_MS - WRONG_MS - 42_000) as u64);
        solve(&mut s);
        assert_eq!(Rules::moments(&s, P)[0].text, "Out of the well with 0:42 left");
        // A plain escape is no moment.
        let mut s = Rules::setup(&[P.into()], 7).unwrap();
        Rules::apply_move(&mut s, P, WellMove::Answer { value: wrong }).unwrap();
        solve(&mut s);
        assert!(s.escaped && Rules::moments(&s, P).is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 1, 2000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 1, "one escape per game: {stats:?}");
    }
}

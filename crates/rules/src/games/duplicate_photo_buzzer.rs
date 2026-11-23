//! Duplicate Picture Buzzer (team game, a race here). Rules page: docs/games/duplicate-photo-buzzer.md
//!
//! Generated pictures flash one after another; buzz when one repeats. Rivals
//! (the seats after the first) are played by the rules with seeded reaction
//! times, so the race has human-like speeds.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 6;
pub const REVEAL_MS: i64 = 2_500;
pub const WIN_POINTS: i64 = 100;
pub const SPEED_POINTS: i64 = 60;
pub const WRONG_POINTS: i64 = -30;

/// How long a picture stays in round `round` (0-based).
pub fn frame_ms(round: usize) -> i64 {
    1_400 - 100 * round as i64
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mark {
    /// 0 circle, 1 square, 2 triangle, 3 star, 4 diamond, 5 hexagon.
    pub kind: u8,
    pub colour: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Picture {
    pub ground: u8,
    pub layout: u8,
    pub marks: [Mark; 3],
}

fn random_picture(rng: &mut Rng) -> Picture {
    let mut mark = || Mark { kind: rng.int(0, 5) as u8, colour: rng.int(0, 7) as u8 };
    let marks = [mark(), mark(), mark()];
    Picture { ground: rng.int(0, 3) as u8, layout: rng.int(0, 3) as u8, marks }
}

/// One shape of `p` in another colour or kind.
fn near_copy(p: &Picture, rng: &mut Rng) -> Picture {
    let mut q = *p;
    let m = &mut q.marks[rng.index(3)];
    if rng.next_f64() < 0.5 {
        m.colour = (m.colour + rng.int(1, 7) as u8) % 8;
    } else {
        m.kind = (m.kind + rng.int(1, 5) as u8) % 6;
    }
    q
}

#[derive(Clone, Debug)]
pub struct Stream {
    pub frames: Vec<Picture>,
    pub original: usize,
    /// The repeat; one more fresh picture follows it.
    pub repeat: usize,
}

/// A stream of 7–14 pictures where exactly one repeats an earlier one.
pub fn generate(rng: &mut Rng) -> Stream {
    let len = rng.int(7, 14) as usize;
    let repeat = len - 2;
    let mut frames: Vec<Picture> = Vec::new();
    let fresh = |frames: &Vec<Picture>, rng: &mut Rng, lure: bool| loop {
        let p = if lure && frames.len() >= 2 { near_copy(rng.pick(frames), rng) } else { random_picture(rng) };
        if !frames.contains(&p) {
            return p;
        }
    };
    for i in 0..repeat {
        let lure = i >= 2 && rng.next_f64() < 0.35;
        let p = fresh(&frames, rng, lure);
        frames.push(p);
    }
    let original = rng.int(0, repeat as i64 - 2) as usize;
    frames.push(frames[original]);
    let tail = fresh(&frames, rng, false);
    frames.push(tail);
    Stream { frames, original, repeat }
}

#[derive(Clone, Debug)]
pub struct Seat {
    pub id: String,
    pub points: i64,
    /// This round: Some(true) a right buzz, Some(false) a wrong one.
    pub buzz: Option<bool>,
    /// Rivals: the planned buzz (time, right) for this round.
    pub plan: Option<(i64, bool)>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub seats: Vec<Seat>,
    pub rng: Rng,
    pub round: usize,
    pub stream: Stream,
    pub elapsed_ms: i64,
    pub revealing: bool,
    pub reveal_left_ms: i64,
    pub winner: Option<usize>,
    pub over: bool,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Buzz,
    Next,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub points: i64,
    pub buzz: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reveal {
    pub picture: Picture,
    pub first: usize,
    pub second: usize,
    pub winner: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round: usize,
    pub rounds: usize,
    /// The picture on screen now (none in the reveal).
    pub picture: Option<Picture>,
    pub number: usize,
    pub frame_ms: i64,
    pub frame_left_ms: i64,
    pub locked: bool,
    pub seats: Vec<SeatView>,
    pub you: usize,
    pub reveal: Option<Reveal>,
    pub reveal_left_ms: i64,
    pub over: bool,
}

impl State {
    fn frame(&self) -> i64 {
        frame_ms(self.round)
    }

    fn window_start(&self) -> i64 {
        self.stream.repeat as i64 * self.frame()
    }

    fn end_time(&self) -> i64 {
        self.window_start() + 2 * self.frame()
    }

    fn start_round(&mut self) {
        self.stream = generate(&mut self.rng);
        self.elapsed_ms = 0;
        self.revealing = false;
        self.winner = None;
        let (start, end, frame, repeat) = (self.window_start(), self.end_time(), self.frame(), self.stream.repeat as i64);
        for i in 0..self.seats.len() {
            self.seats[i].buzz = None;
            let roll = self.rng.next_f64();
            let react = self.rng.int(700, 2_000);
            self.seats[i].plan = if i == 0 || (0.10..0.40).contains(&roll) {
                None
            } else if roll < 0.10 {
                Some((self.rng.int(0, repeat - 1) * frame + react.min(frame - 1), false))
            } else {
                // Too slow for the two pictures: no buzz at all.
                (start + react < end).then_some((start + react, true))
            };
        }
    }

    fn win(&mut self, seat: usize, at: i64) {
        let window = 2 * self.frame();
        let late = (at - self.window_start()).clamp(0, window);
        self.seats[seat].points += WIN_POINTS + SPEED_POINTS * (window - late) / window;
        self.seats[seat].buzz = Some(true);
        self.winner = Some(seat);
        self.elapsed_ms = at;
        self.reveal();
    }

    fn wrong(&mut self, seat: usize) {
        self.seats[seat].points += WRONG_POINTS;
        self.seats[seat].buzz = Some(false);
    }

    fn reveal(&mut self) {
        self.revealing = true;
        self.reveal_left_ms = REVEAL_MS;
    }

    fn next_round(&mut self) {
        self.round += 1;
        if self.round >= ROUNDS {
            self.over = true;
            self.revealing = false;
        } else {
            self.start_round();
        }
    }

    fn in_window(&self, t: i64) -> bool {
        t >= self.window_start() && t < self.end_time()
    }

    fn current(&self) -> usize {
        ((self.elapsed_ms / self.frame()) as usize).min(self.stream.frames.len() - 1)
    }
}

pub struct DuplicatePictureBuzzer;

impl Game for DuplicatePictureBuzzer {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "duplicate-photo-buzzer",
            kind: "mini-team",
            difficulty: "easy",
            min_players: 1,
            max_players: 4,
            name: "Duplicate Picture Buzzer",
            short: "Pictures flash by; buzz the moment one repeats, before the rivals do.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if players.is_empty() || players.len() > 4 {
            return Err("Duplicate Picture Buzzer needs 1–4 players".into());
        }
        let mut rng = Rng::new(seed);
        let stream = generate(&mut rng);
        let seats = players.iter().map(|id| Seat { id: id.clone(), points: 0, buzz: None, plan: None }).collect();
        let mut s =
            State { seats, rng, round: 0, stream, elapsed_ms: 0, revealing: false, reveal_left_ms: 0, winner: None, over: false };
        s.start_round();
        Ok(s)
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        if player != s.seats[0].id {
            return Err("Rivals play by themselves".into());
        }
        match mv {
            Move::Buzz if !s.revealing => {
                if s.seats[0].buzz.is_some() {
                    return Err("You are out of this round".into());
                }
                let now = s.elapsed_ms;
                if s.in_window(now) {
                    s.win(0, now);
                } else {
                    s.wrong(0);
                }
                Ok(())
            }
            Move::Next if s.revealing => {
                s.next_round();
                Ok(())
            }
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut State, ms: u64) {
        if s.over {
            return;
        }
        if s.revealing {
            s.reveal_left_ms -= ms as i64;
            if s.reveal_left_ms <= 0 {
                s.next_round();
            }
            return;
        }
        let target = (s.elapsed_ms + ms as i64).min(s.end_time());
        let mut due: Vec<(i64, usize, bool)> = (1..s.seats.len())
            .filter_map(|i| s.seats[i].plan.filter(|&(t, _)| t > s.elapsed_ms && t <= target).map(|(t, right)| (t, i, right)))
            .collect();
        due.sort();
        for (t, i, right) in due {
            if right {
                s.win(i, t);
                return;
            }
            s.wrong(i);
        }
        s.elapsed_ms = target;
        if s.elapsed_ms >= s.end_time() {
            s.reveal();
        }
    }

    fn view_for(s: &State, player: &str) -> View {
        let me = s.seats.iter().position(|x| x.id == player).unwrap_or(0);
        let watching = !s.revealing && !s.over;
        let shown = s.revealing || s.over;
        View {
            round: (s.round + 1).min(ROUNDS),
            rounds: ROUNDS,
            picture: watching.then(|| s.stream.frames[s.current()]),
            number: s.current() + 1,
            frame_ms: s.frame(),
            frame_left_ms: s.frame() - s.elapsed_ms % s.frame(),
            locked: s.seats[me].buzz.is_some(),
            seats: s.seats.iter().map(|x| SeatView { id: x.id.clone(), points: x.points, buzz: x.buzz }).collect(),
            you: me,
            reveal: shown.then(|| Reveal {
                picture: s.stream.frames[s.stream.repeat],
                first: s.stream.original + 1,
                second: s.stream.repeat + 1,
                winner: s.winner.map(|w| s.seats[w].id.clone()),
            }),
            reveal_left_ms: s.reveal_left_ms.max(0),
            over: s.over,
        }
    }

    fn is_over(s: &State) -> bool {
        s.over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        rank_by_score(&s.seats.iter().map(|x| (x.id.clone(), x.points)).collect::<Vec<_>>())
    }

    fn to_move(s: &State) -> Vec<String> {
        if s.over || s.revealing || s.seats[0].buzz.is_some() { Vec::new() } else { vec![s.seats[0].id.clone()] }
    }

    /// Buzzes on the repeat 80% of the time; otherwise sends a move that is
    /// refused while watching (keeps watching).
    fn bot_move(s: &State, _player: &str, seed: u64) -> Move {
        let mut rng = Rng::new(seed);
        if !s.revealing && s.in_window(s.elapsed_ms) && rng.next_f64() < 0.8 { Move::Buzz } else { Move::Next }
    }
}

pub const READY: bool = true;
pub type Rules = DuplicatePictureBuzzer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn exactly_one_picture_repeats() {
        for seed in 1..=500 {
            let st = generate(&mut Rng::new(seed));
            let f = &st.frames;
            assert!((7..=14).contains(&f.len()));
            assert_eq!(f[st.original], f[st.repeat]);
            assert!(st.original + 2 <= st.repeat);
            let pairs = (0..f.len()).flat_map(|i| (i + 1..f.len()).map(move |j| (i, j))).filter(|&(i, j)| f[i] == f[j]).count();
            assert_eq!(pairs, 1, "seed {seed}");
        }
    }

    #[test]
    fn a_buzz_on_the_repeat_wins_and_a_wrong_one_locks_out() {
        let mut s = DuplicatePictureBuzzer::setup(&players(1), 1).unwrap();
        DuplicatePictureBuzzer::apply_move(&mut s, "p1", Move::Buzz).unwrap();
        assert_eq!(s.seats[0].points, WRONG_POINTS);
        assert!(DuplicatePictureBuzzer::view_for(&s, "p1").locked);
        assert!(DuplicatePictureBuzzer::apply_move(&mut s, "p1", Move::Buzz).is_err());
        DuplicatePictureBuzzer::tick(&mut s, 60_000);
        assert!(s.revealing);
        assert!(DuplicatePictureBuzzer::view_for(&s, "p1").reveal.unwrap().winner.is_none());
        DuplicatePictureBuzzer::apply_move(&mut s, "p1", Move::Next).unwrap();
        let start = s.window_start() as u64;
        DuplicatePictureBuzzer::tick(&mut s, start);
        DuplicatePictureBuzzer::apply_move(&mut s, "p1", Move::Buzz).unwrap();
        assert_eq!(s.seats[0].points, WRONG_POINTS + WIN_POINTS + SPEED_POINTS);
        assert_eq!(DuplicatePictureBuzzer::view_for(&s, "p1").reveal.unwrap().winner.as_deref(), Some("p1"));
    }

    #[test]
    fn the_view_shows_only_the_current_picture() {
        let mut s = DuplicatePictureBuzzer::setup(&players(2), 4).unwrap();
        let v = DuplicatePictureBuzzer::view_for(&s, "p1");
        assert_eq!(v.picture, Some(s.stream.frames[0]));
        assert!(v.reveal.is_none());
        DuplicatePictureBuzzer::tick(&mut s, frame_ms(0) as u64);
        assert_eq!(DuplicatePictureBuzzer::view_for(&s, "p1").number, 2);
    }

    #[test]
    fn rivals_buzz_with_their_reaction_time() {
        let mut wins = 0;
        for seed in 1..=200 {
            let mut s = DuplicatePictureBuzzer::setup(&players(4), seed).unwrap();
            DuplicatePictureBuzzer::tick(&mut s, 60_000);
            assert!(s.revealing);
            if let Some(w) = s.winner {
                wins += 1;
                assert!(w > 0);
                let (t, right) = s.seats[w].plan.unwrap();
                assert!(right && s.in_window(t));
            }
        }
        assert!(wins > 100, "rivals won {wins} of 200 rounds");
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<DuplicatePictureBuzzer>(1000, 1, 1500);
        bot_games_with::<DuplicatePictureBuzzer>(300, 4, 1500);
    }
}

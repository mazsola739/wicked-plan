//! Colour Mancala (Main game, 2–4 players). Rules page: docs/games/colour-mancala.md
//!
//! A ring of pits: every player owns 5 pits and a store. Stones are red, green
//! or blue; you sow a pit's stones starting with the colour you choose. Your
//! score is the lowest of your three colour totals in your store.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const PITS: usize = 5;
pub const STONES_PER_PIT: u32 = 4;
pub const MAX_TURNS_EACH: u32 = 40;
/// The smallest capture that is a replay moment.
pub const BIG_CAPTURE: u32 = 10;
/// Holes per player: the pits plus the store.
const W: usize = PITS + 1;

/// Stones of each colour: red, green, blue.
pub type Stones = [u32; 3];

fn total(s: &Stones) -> u32 {
    s.iter().sum()
}

pub fn points(store: &Stones) -> i64 {
    50 * i64::from(*store.iter().min().unwrap_or(&0)) + i64::from(total(store))
}

fn store_of(seat: usize) -> usize {
    seat * W + PITS
}

fn is_store(hole: usize) -> bool {
    hole % W == PITS
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastMove {
    pub seat: usize,
    pub hole: usize,
    pub first: usize,
    /// Every hole a stone was dropped into, in order.
    pub path: Vec<usize>,
    pub captured: u32,
    pub extra: bool,
}

#[derive(Clone, Debug)]
pub struct State {
    pub players: Vec<String>,
    /// The ring: seat k owns holes k*6 … k*6+4, its store is k*6+5.
    pub holes: Vec<Stones>,
    pub turn: usize,
    pub turns: u32,
    pub last: Option<LastMove>,
    pub over: bool,
    /// Replay moments: a capture of 10 or more stones.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Sow { pit: usize, first: usize },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: Vec<String>,
    pub you: Option<usize>,
    pub pits: usize,
    pub holes: Vec<Stones>,
    pub turn: usize,
    pub your_turn: bool,
    pub turns_left: u32,
    pub points: Vec<i64>,
    pub last: Option<LastMove>,
    pub over: bool,
}

/// Sows `seat`'s pit; returns what happened. The holes are left as they are on an error.
pub fn sow(holes: &mut [Stones], seat: usize, pit: usize, first: usize) -> Result<LastMove, String> {
    let n = holes.len() / W;
    if seat >= n || pit >= PITS || first >= 3 {
        return Err("Illegal move".into());
    }
    let start = seat * W + pit;
    let stones = holes[start];
    if total(&stones) == 0 {
        return Err("That pit is empty".into());
    }
    holes[start] = [0; 3];
    let mut at = start;
    let mut path = Vec::new();
    for k in 0..3 {
        let colour = (first + k) % 3;
        for _ in 0..stones[colour] {
            loop {
                at = (at + 1) % holes.len();
                if !is_store(at) || at == store_of(seat) {
                    break;
                }
            }
            holes[at][colour] += 1;
            path.push(at);
        }
    }
    let extra = at == store_of(seat);
    let mut captured = 0;
    if !is_store(at) && at / W == seat && total(&holes[at]) == 1 {
        let facing = ((seat + 1) % n) * W + (PITS - 1 - at % W);
        if total(&holes[facing]) > 0 {
            let (mine, theirs) = (holes[at], holes[facing]);
            for (c, stones) in holes[store_of(seat)].iter_mut().enumerate() {
                *stones += mine[c] + theirs[c];
            }
            captured = total(&holes[at]) + total(&holes[facing]);
            holes[at] = [0; 3];
            holes[facing] = [0; 3];
        }
    }
    Ok(LastMove { seat, hole: start, first, path, captured, extra })
}

fn row_empty(holes: &[Stones], seat: usize) -> bool {
    (0..PITS).all(|p| total(&holes[seat * W + p]) == 0)
}

impl State {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }

    fn finish(&mut self) {
        for seat in 0..self.players.len() {
            let store = store_of(seat);
            for p in 0..PITS {
                let h = seat * W + p;
                for c in 0..3 {
                    self.holes[store][c] += self.holes[h][c];
                }
                self.holes[h] = [0; 3];
            }
        }
        self.over = true;
    }

    fn play(&mut self, seat: usize, pit: usize, first: usize) -> Result<(), String> {
        let last = sow(&mut self.holes, seat, pit, first)?;
        // A replay moment: a big capture (the board is public).
        if last.captured >= BIG_CAPTURE {
            let victim = &self.players[(seat + 1) % self.players.len()];
            self.moments.push(&self.players[seat], format!("{} stones captured from {{{victim}}}", last.captured));
        }
        let extra = last.extra;
        self.last = Some(last);
        self.turns += 1;
        if !extra {
            self.turn = (self.turn + 1) % self.players.len();
        }
        if self.turns >= MAX_TURNS_EACH * self.players.len() as u32 || row_empty(&self.holes, self.turn) {
            self.finish();
        }
        Ok(())
    }
}

/// The bot's choice for `seat`: the pit and first colour that raise its points most.
pub fn best_move(holes: &[Stones], seat: usize, seed: u64) -> (usize, usize) {
    let mut rng = Rng::new(seed);
    let before = points(&holes[store_of(seat)]);
    let mut best = (0, 0);
    let mut best_value = f64::MIN;
    for pit in 0..PITS {
        for first in 0..3 {
            let mut h = holes.to_vec();
            let Ok(last) = sow(&mut h, seat, pit, first) else { continue };
            let gain = (points(&h[store_of(seat)]) - before) as f64;
            let value = gain + if last.extra { 30.0 } else { 0.0 } + f64::from(last.captured) * 3.0 + rng.next_f64() * 6.0;
            if value > best_value {
                best_value = value;
                best = (pit, first);
            }
        }
    }
    best
}

pub struct ColourMancala;

impl Game for ColourMancala {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "colour-mancala",
            kind: "main",
            difficulty: "hard",
            tags: &["board", "numbers"],
            min_players: 2,
            max_players: 4,
            name: "Colour Mancala",
            short: "Round mancala with stones in three colours; only your weakest colour counts.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if !(2..=4).contains(&players.len()) {
            return Err("Colour Mancala needs 2–4 players".into());
        }
        let mut rng = Rng::new(seed);
        // One random row pattern, copied to every player.
        let row: Vec<Stones> = (0..PITS)
            .map(|_| {
                let mut s = [0; 3];
                for _ in 0..STONES_PER_PIT {
                    s[rng.index(3)] += 1;
                }
                s
            })
            .collect();
        let mut holes = Vec::new();
        for _ in players {
            holes.extend(row.iter().copied());
            holes.push([0; 3]);
        }
        Ok(State {
            players: players.to_vec(),
            holes,
            turn: 0,
            turns: 0,
            last: None,
            over: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.seat(player).ok_or("Not in this game")?;
        if seat != s.turn {
            return Err("Not your turn".into());
        }
        let Move::Sow { pit, first } = mv;
        s.play(seat, pit, first)
    }

    fn view_for(s: &State, player: &str) -> View {
        let you = s.seat(player);
        let max = MAX_TURNS_EACH * s.players.len() as u32;
        View {
            players: s.players.clone(),
            you,
            pits: PITS,
            holes: s.holes.clone(),
            turn: s.turn,
            your_turn: !s.over && you == Some(s.turn),
            turns_left: max.saturating_sub(s.turns),
            points: (0..s.players.len()).map(|k| points(&s.holes[store_of(k)])).collect(),
            last: s.last.clone(),
            over: s.over,
        }
    }

    fn is_over(s: &State) -> bool {
        s.over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> =
            s.players.iter().enumerate().map(|(k, p)| (p.clone(), points(&s.holes[store_of(k)]))).collect();
        rank_by_score(&scores)
    }

    fn to_move(s: &State) -> Vec<String> {
        if s.over { Vec::new() } else { vec![s.players[s.turn].clone()] }
    }

    fn bot_move(s: &State, player: &str, seed: u64) -> Move {
        let seat = s.seat(player).unwrap_or(0);
        let (pit, first) = best_move(&s.holes, seat, seed);
        Move::Sow { pit, first }
    }

    fn moments(s: &State, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub const READY: bool = true;
pub type Rules = ColourMancala;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    fn empty(n: usize) -> Vec<Stones> {
        vec![[0; 3]; n * W]
    }

    #[test]
    fn sowing_starts_with_the_chosen_colour_and_skips_other_stores() {
        let mut h = empty(2);
        h[4] = [1, 1, 1];
        let last = sow(&mut h, 0, 4, 2).unwrap();
        // Hole 5 is my store (blue first), 6 and 7 the next player's pits.
        assert_eq!(last.path, vec![5, 6, 7]);
        assert_eq!(h[5], [0, 0, 1]);
        assert_eq!(h[6], [1, 0, 0]);
        assert_eq!(h[7], [0, 1, 0]);
        // Seat 1 sowing past seat 0's store skips it.
        let mut h = empty(2);
        h[10] = [0, 3, 0];
        sow(&mut h, 1, 4, 0).unwrap();
        assert_eq!(h[11], [0, 1, 0]);
        assert_eq!(h[0], [0, 1, 0]);
        assert_eq!(h[1], [0, 1, 0]);
        let mut h = empty(2);
        h[4] = [0, 0, 7];
        let last = sow(&mut h, 0, 4, 0).unwrap();
        assert_eq!(h[11], [0, 0, 0], "skips the other store");
        // The last stone lands in my empty pit 0 and captures the facing pit (hole 10).
        assert_eq!(*last.path.last().unwrap(), 0);
        assert_eq!(last.captured, 2);
        assert_eq!(h[5], [0, 0, 3]);
    }

    #[test]
    fn last_stone_in_own_store_gives_another_turn() {
        let mut s = ColourMancala::setup(&players(2), 1).unwrap();
        s.holes = empty(2);
        s.holes[3] = [2, 0, 0];
        s.holes[8] = [1, 0, 0];
        ColourMancala::apply_move(&mut s, "p1", Move::Sow { pit: 3, first: 0 }).unwrap();
        assert_eq!(s.turn, 0);
        assert!(s.last.as_ref().unwrap().extra);
    }

    #[test]
    fn landing_in_an_own_empty_pit_captures_the_facing_pit() {
        let mut h = empty(2);
        h[0] = [1, 0, 0];
        // Pit 1 of seat 0 faces pit 3 of seat 1 (hole 6 + 3 = 9).
        h[9] = [0, 2, 1];
        let last = sow(&mut h, 0, 0, 0).unwrap();
        assert_eq!(last.captured, 4);
        assert_eq!(h[5], [1, 2, 1]);
        assert_eq!(h[1], [0; 3]);
        assert_eq!(h[9], [0; 3]);
    }

    #[test]
    fn a_big_capture_is_a_replay_moment() {
        let mut s = ColourMancala::setup(&players(3), 1).unwrap();
        s.holes = empty(3);
        s.holes[0] = [1, 0, 0];
        s.holes[W] = [0, 1, 0];
        // Pit 1 of p1 faces pit 3 of p2.
        s.holes[W + 3] = [4, 3, 2];
        ColourMancala::apply_move(&mut s, "p1", Move::Sow { pit: 0, first: 0 }).unwrap();
        let m = ColourMancala::moments(&s, "p3");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "10 stones captured from {p2}"));
        // A smaller capture is no moment.
        s.holes[2 * W] = [1, 0, 0];
        s.holes[3] = [3, 3, 2];
        ColourMancala::apply_move(&mut s, "p2", Move::Sow { pit: 0, first: 0 }).unwrap();
        ColourMancala::apply_move(&mut s, "p3", Move::Sow { pit: 0, first: 0 }).unwrap();
        assert_eq!(s.last.as_ref().unwrap().captured, 9);
        assert_eq!(ColourMancala::moments(&s, "p1").len(), 1);
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<ColourMancala>(300, 4, 5000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }

    #[test]
    fn score_is_the_weakest_colour() {
        assert_eq!(points(&[5, 9, 2]), 50 * 2 + 16);
        assert!(points(&[4, 4, 4]) > points(&[1, 10, 10]));
    }

    #[test]
    fn rows_start_equal_and_an_empty_row_ends_the_game() {
        let mut s = ColourMancala::setup(&players(3), 5).unwrap();
        assert_eq!(s.holes[0..PITS], s.holes[W..W + PITS]);
        assert!(ColourMancala::apply_move(&mut s, "p2", Move::Sow { pit: 0, first: 0 }).is_err());
        s.holes = empty(3);
        s.holes[4] = [1, 0, 0];
        s.holes[W] = [0, 3, 0];
        ColourMancala::apply_move(&mut s, "p1", Move::Sow { pit: 4, first: 0 }).unwrap();
        // p1 moves again but its row is empty: the game ends, p2's pit is swept to its store.
        assert!(s.over);
        assert_eq!(s.holes[store_of(1)], [0, 3, 0]);
        assert_eq!(ColourMancala::results(&s).len(), 3);
    }

    #[test]
    fn a_turn_waits_for_its_player() {
        let mut s = ColourMancala::setup(&players(2), 2).unwrap();
        ColourMancala::tick(&mut s, 600_000);
        assert_eq!(s.turns, 0);
        assert_eq!(ColourMancala::to_move(&s), vec!["p1".to_string()]);
    }

    #[test]
    fn bot_beats_a_first_pit_player_mostly() {
        let mut wins = 0;
        for seed in 0..40 {
            let mut s = ColourMancala::setup(&players(2), seed).unwrap();
            while !s.over {
                let mv = if s.turn == 0 {
                    ColourMancala::bot_move(&s, "p1", seed)
                } else {
                    Move::Sow { pit: (0..PITS).find(|&p| total(&s.holes[W + p]) > 0).unwrap(), first: 0 }
                };
                let p = s.players[s.turn].clone();
                ColourMancala::apply_move(&mut s, &p, mv).unwrap();
            }
            let r = ColourMancala::results(&s);
            if r[0].rank == 1 {
                wins += 1;
            }
        }
        assert!(wins >= 28, "bot won only {wins} of 40");
    }

    #[test]
    fn thousand_bot_games() {
        for seats in 2..=4 {
            bot_games_with::<ColourMancala>(1000, seats, 5000);
        }
    }
}

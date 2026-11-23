//! The contract every game follows. The page and the bots only talk to it.
//!
//! `State` holds every secret; `View` is the only thing the page receives for a
//! player. Rules are pure: no clock, no I/O, no global randomness (time arrives
//! through `tick`, randomness through the seed). Today the rules run in the
//! browser; keeping secrets out of the views means a server can referee later.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::rng::Rng;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResult {
    pub player_id: String,
    /// 1 is best; tied players share a rank.
    pub rank: u32,
    pub points: i64,
}

/// Catalogue entry the game picker shows (docs/games.md).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub id: &'static str,
    /// main | mini-solo | mini-team | prison | hidden | exile | final-duel
    pub kind: &'static str,
    /// easy | medium | hard (docs/games.md); the picker colours the card by it.
    pub difficulty: &'static str,
    pub min_players: u32,
    pub max_players: u32,
    pub name: &'static str,
    pub short: &'static str,
}

impl GameInfo {
    /// How many seats the page fills (the player plus bots): the minimum, or 4
    /// when the game allows more, never above the maximum.
    pub fn seats(&self) -> u32 {
        self.min_players.max(4.min(self.max_players))
    }
}

pub trait Game {
    type State: Clone + Send + 'static;
    type Move: DeserializeOwned;
    type View: Serialize;

    fn info() -> GameInfo;
    /// `players[0]` is the human when there is one; the rest may be bots.
    fn setup(players: &[String], seed: u64) -> Result<Self::State, String>;
    /// Leaves the state untouched when it returns an error.
    fn apply_move(state: &mut Self::State, player: &str, mv: Self::Move) -> Result<(), String>;
    /// Timers. `ms` is the time since the last tick.
    fn tick(_state: &mut Self::State, _ms: u64) {}
    fn view_for(state: &Self::State, player: &str) -> Self::View;
    fn is_over(state: &Self::State) -> bool;
    fn results(state: &Self::State) -> Vec<PlayerResult>;
    /// Players who may move now (several at once for secret bids). Bots act
    /// only when they are listed here. Solo games may keep the default.
    fn to_move(_state: &Self::State) -> Vec<String> {
        Vec::new()
    }
    /// For bots and the "1,000 bot games" tests. Should be a sensible move.
    fn bot_move(state: &Self::State, player: &str, seed: u64) -> Self::Move;
}

/// A running game behind JSON, so the page can hold any game the same way.
pub trait Session: Send {
    fn game_id(&self) -> &'static str;
    fn apply(&mut self, player: &str, mv: Value) -> Result<(), String>;
    fn tick(&mut self, ms: u64);
    fn view(&self, player: &str) -> Value;
    fn is_over(&self) -> bool;
    fn results(&self) -> Vec<PlayerResult>;
    fn to_move(&self) -> Vec<String>;
    /// Lets `player` make its bot move.
    fn bot_act(&mut self, player: &str, seed: u64) -> Result<(), String>;
}

pub struct Typed<G: Game> {
    pub state: G::State,
}

impl<G: Game + Send + 'static> Session for Typed<G> {
    fn game_id(&self) -> &'static str {
        G::info().id
    }
    fn apply(&mut self, player: &str, mv: Value) -> Result<(), String> {
        let mv: G::Move = serde_json::from_value(mv).map_err(|e| format!("Bad move: {e}"))?;
        G::apply_move(&mut self.state, player, mv)
    }
    fn tick(&mut self, ms: u64) {
        G::tick(&mut self.state, ms);
    }
    fn view(&self, player: &str) -> Value {
        serde_json::to_value(G::view_for(&self.state, player)).unwrap_or(Value::Null)
    }
    fn is_over(&self) -> bool {
        G::is_over(&self.state)
    }
    fn results(&self) -> Vec<PlayerResult> {
        G::results(&self.state)
    }
    fn to_move(&self) -> Vec<String> {
        G::to_move(&self.state)
    }
    fn bot_act(&mut self, player: &str, seed: u64) -> Result<(), String> {
        let mv = G::bot_move(&self.state, player, seed);
        G::apply_move(&mut self.state, player, mv)
    }
}

/// A session with bots in some seats: on every tick, each bot that may move
/// "thinks" for a short while (0.7–1.8 s of game time), then moves.
pub struct WithBots {
    inner: Box<dyn Session>,
    bots: Vec<(String, i64)>,
    rng: Rng,
}

pub fn with_bots(inner: Box<dyn Session>, bots: Vec<String>, seed: u64) -> Box<dyn Session> {
    let mut rng = Rng::new(seed ^ 0x5eed_b075);
    let bots = bots.into_iter().map(|b| (b, rng.int(700, 1800))).collect();
    Box::new(WithBots { inner, bots, rng })
}

impl Session for WithBots {
    fn game_id(&self) -> &'static str {
        self.inner.game_id()
    }
    fn apply(&mut self, player: &str, mv: Value) -> Result<(), String> {
        if self.bots.iter().any(|(b, _)| b == player) {
            return Err("That seat is a bot".into());
        }
        self.inner.apply(player, mv)
    }
    fn tick(&mut self, ms: u64) {
        self.inner.tick(ms);
        if self.inner.is_over() {
            return;
        }
        let waiting = self.inner.to_move();
        for i in 0..self.bots.len() {
            let name = self.bots[i].0.clone();
            if !waiting.contains(&name) {
                continue;
            }
            self.bots[i].1 -= ms as i64;
            if self.bots[i].1 <= 0 {
                let seed = self.rng.int(0, i64::from(u32::MAX)) as u64;
                // A rejected bot move is retried on a later tick with a new seed.
                let _ = self.inner.bot_act(&name, seed);
                self.bots[i].1 = self.rng.int(700, 1800);
                if self.inner.is_over() {
                    return;
                }
            }
        }
    }
    fn view(&self, player: &str) -> Value {
        self.inner.view(player)
    }
    fn is_over(&self) -> bool {
        self.inner.is_over()
    }
    fn results(&self) -> Vec<PlayerResult> {
        self.inner.results()
    }
    fn to_move(&self) -> Vec<String> {
        self.inner.to_move()
    }
    fn bot_act(&mut self, player: &str, seed: u64) -> Result<(), String> {
        self.inner.bot_act(player, seed)
    }
}

/// Points for a correct timed answer: 100, plus up to 100 for speed.
pub fn speed_points(time_left_ms: i64, round_ms: i64) -> i64 {
    100 + (100 * time_left_ms.max(0) + round_ms / 2) / round_ms
}

/// Ranks from scores, highest first; equal scores share a rank.
pub fn rank_by_score(scores: &[(String, i64)]) -> Vec<PlayerResult> {
    scores
        .iter()
        .map(|(id, pts)| PlayerResult {
            player_id: id.clone(),
            rank: 1 + scores.iter().filter(|(_, other)| other > pts).count() as u32,
            points: *pts,
        })
        .collect()
}

#[cfg(test)]
pub mod testing {
    use super::Game;
    use crate::rng::Rng;

    /// "1,000 random bot games: nothing crashes, every game ends" for a solo game.
    pub fn bot_games<G: Game>(games: u64, max_steps: usize) {
        bot_games_with::<G>(games, 1, max_steps);
    }

    /// The same for `players` seats, all played by bots: whoever may move does,
    /// and time passes now and then.
    pub fn bot_games_with<G: Game>(games: u64, players: usize, max_steps: usize) {
        let ids: Vec<String> = (1..=players).map(|i| format!("p{i}")).collect();
        for seed in 1..=games {
            let mut rng = Rng::new(seed * 7919);
            let mut s = G::setup(&ids, seed).expect("setup");
            let mut steps = 0;
            while !G::is_over(&s) {
                let waiting = G::to_move(&s);
                let solo_turn = players == 1 && rng.next_f64() < 0.7;
                if !waiting.is_empty() && rng.next_f64() < 0.85 || solo_turn {
                    let p = if waiting.is_empty() { ids[0].clone() } else { rng.pick(&waiting).clone() };
                    let mv = G::bot_move(&s, &p, rng.int(0, 1_000_000_000) as u64);
                    // A bot may make a wrong move; it must be rejected, never crash.
                    let _ = G::apply_move(&mut s, &p, mv);
                } else {
                    G::tick(&mut s, rng.int(200, 30_000) as u64);
                }
                steps += 1;
                assert!(steps <= max_steps, "seed {seed} did not end");
            }
            let results = G::results(&s);
            assert_eq!(results.len(), players, "one result per player");
            assert!(results.iter().all(|r| r.rank >= 1 && r.rank as usize <= players));
            assert!(results.iter().any(|r| r.rank == 1));
            for id in &ids {
                // Views must serialize for every seat, also after the end.
                assert!(serde_json::to_value(G::view_for(&s, id)).is_ok());
            }
        }
    }
}

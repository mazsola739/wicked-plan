//! Wicked Plan game rules: one module per game, pure and seeded.
//! See docs/games.md for the catalogue and docs/games/<id>.md for each rules page.
//!
//! Every module in `games` exports `pub struct Rules` (or a `pub type Rules`)
//! implementing `Game`, and `pub const READY: bool`. Games that are not ready
//! yet are left out of the catalogue.

pub mod game;
pub mod quiz;
pub mod rng;

use game::{Game, GameInfo, Session, Typed, with_bots};

/// A placeholder for a game that is not built yet: listed nowhere, ends at once.
#[macro_export]
macro_rules! stub_game {
    ($id:literal, $kind:literal, $min:literal, $max:literal, $name:literal) => {
        pub const READY: bool = false;
        pub struct Rules;
        impl $crate::game::Game for Rules {
            type State = ();
            type Move = serde_json::Value;
            type View = ();
            fn info() -> $crate::game::GameInfo {
                $crate::game::GameInfo {
                    id: $id,
                    kind: $kind,
                    difficulty: "",
                    tags: &[],
                    min_players: $min,
                    max_players: $max,
                    name: $name,
                    short: "",
                }
            }
            fn setup(_: &[String], _: u64) -> Result<(), String> {
                Err("Not built yet".into())
            }
            fn apply_move(_: &mut (), _: &str, _: serde_json::Value) -> Result<(), String> {
                Err("Not built yet".into())
            }
            fn view_for(_: &(), _: &str) {}
            fn is_over(_: &()) -> bool {
                true
            }
            fn results(_: &()) -> Vec<$crate::game::PlayerResult> {
                Vec::new()
            }
            fn bot_move(_: &(), _: &str, _: u64) -> serde_json::Value {
                serde_json::Value::Null
            }
        }
    };
}

fn session<G: Game + Send + 'static>(players: &[String], seed: u64) -> Result<Box<dyn Session>, String> {
    Ok(Box::new(Typed::<G> { state: G::setup(players, seed)? }))
}

macro_rules! registry {
    ($($m:ident),* $(,)?) => {
        pub mod games {
            $(pub mod $m;)*
        }

        /// Every playable game, in catalogue order.
        pub fn catalogue() -> Vec<GameInfo> {
            let mut out = Vec::new();
            $(if games::$m::READY {
                out.push(<games::$m::Rules as Game>::info());
            })*
            out
        }

        /// A game with exactly these players (no bots attached).
        pub fn new_session(game_id: &str, players: &[String], seed: u64) -> Result<Box<dyn Session>, String> {
            $(if games::$m::READY && <games::$m::Rules as Game>::info().id == game_id {
                return session::<games::$m::Rules>(players, seed);
            })*
            Err(format!("Unknown game: {game_id}"))
        }
    };
}

registry!(
    // Built.
    count_the_cubes,
    count_the_rectangles,
    equation_target,
    hexagon,
    knights_tour,
    magic_square,
    // Main games.
    rules_race,
    animal_tile_auction,
    equation_hi_lo,
    colour_count_bluff,
    hidden_rules_race,
    virus_game,
    either_or,
    thieves_and_cops,
    secret_number,
    monster_alliance,
    mystery_stock_market,
    polyomino_territory,
    treasure_island,
    colour_mancala,
    jewel_black_market,
    auction_bingo,
    // Mini games.
    lay_the_grass,
    memory_poker,
    word_block_tower,
    duplicate_photo_buzzer,
    spinning_puzzle_wheel,
    memory_scene_quiz,
    weight_balance_puzzle,
    hidden_fourth_player,
    // Prison games.
    time_auction,
    minus_one,
    metal_cast_puzzle,
    sniper_holdem,
    dice_prediction,
    heavy_13,
    russian_roulette,
    wall_territory,
    piggy_bank,
    // Hidden and exile games.
    blind_gomoku,
    rising_water_well,
    unknown_language,
    piece_puzzle,
    find_the_differences,
    secret_code,
    fence_building,
    symmetry_puzzle,
    // Final duels.
    big_or_small,
    nine_mens_morris,
    tiger_and_goat,
    question_and_truth,
);

/// The id of the human seat in a game played on the page.
pub const YOU: &str = "you";

/// A game for the page: the human in seat 0, bots ("bot-1", "bot-2", …) in the
/// other seats the game needs (`GameInfo::seats`), playing by themselves on tick.
pub fn new_table(game_id: &str, seed: u64) -> Result<Box<dyn Session>, String> {
    let info = catalogue().into_iter().find(|g| g.id == game_id).ok_or_else(|| format!("Unknown game: {game_id}"))?;
    let bots: Vec<String> = (1..info.seats()).map(|i| format!("bot-{i}")).collect();
    let mut players = vec![YOU.to_string()];
    players.extend(bots.iter().cloned());
    let inner = new_session(game_id, &players, seed)?;
    Ok(if bots.is_empty() { inner } else { with_bots(inner, bots, seed) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalogue_game_starts_a_table() {
        for info in catalogue() {
            let mut t = new_table(info.id, 1).unwrap();
            assert_eq!(t.game_id(), info.id);
            assert!(!t.is_over());
            assert!(t.view(YOU).is_object());
            // Bots move on their own while time passes; nothing may crash.
            for _ in 0..40 {
                t.tick(500);
            }
            assert!(info.seats() >= info.min_players && info.seats() <= info.max_players);
        }
        assert!(new_table("nope", 1).is_err());
    }

    #[test]
    fn catalogue_ids_are_unique_and_kinds_known() {
        let all = catalogue();
        let mut ids: Vec<&str> = all.iter().map(|g| g.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
        let kinds = ["main", "mini-solo", "mini-team", "prison", "hidden", "exile", "final-duel"];
        assert!(all.iter().all(|g| kinds.contains(&g.kind)));
        let levels = ["easy", "medium", "hard"];
        assert!(all.iter().all(|g| levels.contains(&g.difficulty)), "every game needs a difficulty");
        for g in &all {
            assert!((1..=3).contains(&g.tags.len()), "{} needs 1-3 categories", g.id);
            assert!(g.tags.iter().all(|t| game::CATEGORIES.contains(t)), "{} has an unknown category", g.id);
        }
    }

    #[test]
    fn moves_arrive_as_json() {
        let mut s = new_session("count-the-cubes", &[YOU.into()], 1).unwrap();
        s.apply(YOU, serde_json::json!({ "type": "answer", "value": 3 })).unwrap();
        assert_eq!(s.view(YOU)["history"].as_array().unwrap().len(), 1);
        assert!(s.apply(YOU, serde_json::json!({ "type": "dance" })).is_err());
    }
}

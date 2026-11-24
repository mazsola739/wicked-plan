//! The rules crate for the browser. Views and moves cross the boundary as JSON
//! strings. The human plays seat "you"; bots fill the other seats and move by
//! themselves on `tick`.

use wasm_bindgen::prelude::*;
use wp_rules::YOU;
use wp_rules::game::Session;

/// The game picker's list, as JSON (`GameInfo[]`).
#[wasm_bindgen]
pub fn catalogue() -> String {
    serde_json::to_string(&wp_rules::catalogue()).unwrap_or_else(|_| "[]".into())
}

#[wasm_bindgen]
pub struct LocalGame {
    session: Box<dyn Session>,
}

#[wasm_bindgen]
impl LocalGame {
    #[wasm_bindgen(constructor)]
    pub fn new(game_id: &str, seed: u32) -> Result<LocalGame, JsError> {
        let session = wp_rules::new_table(game_id, u64::from(seed)).map_err(|e| JsError::new(&e))?;
        Ok(LocalGame { session })
    }

    /// `mv` is the move as JSON, e.g. `{"type":"answer","value":12}`.
    pub fn apply(&mut self, mv: &str) -> Result<(), JsError> {
        let value = serde_json::from_str(mv).map_err(|e| JsError::new(&e.to_string()))?;
        self.session.apply(YOU, value).map_err(|e| JsError::new(&e))
    }

    pub fn tick(&mut self, ms: u32) {
        self.session.tick(u64::from(ms));
    }

    /// Your view, as JSON.
    pub fn view(&self) -> String {
        self.session.view(YOU).to_string()
    }

    #[wasm_bindgen(js_name = isOver)]
    pub fn is_over(&self) -> bool {
        self.session.is_over()
    }

    /// Final results of every seat, as JSON (`PlayerResult[]`).
    pub fn results(&self) -> String {
        serde_json::to_string(&self.session.results()).unwrap_or_else(|_| "[]".into())
    }

    /// The big moments you may see, oldest first, as JSON (`Moment[]`); the page replays new ones.
    pub fn moments(&self) -> String {
        serde_json::to_string(&self.session.moments(YOU)).unwrap_or_else(|_| "[]".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plays_a_game_through_json() {
        assert!(catalogue().contains("count-the-cubes"));
        let mut g = LocalGame::new("hexagon", 7).unwrap_or_else(|_| panic!("setup"));
        g.apply(r#"{"type":"ready"}"#).unwrap_or_else(|_| panic!("ready"));
        assert!(g.view().contains("\"studying\":false"));
        // No round clock: any three different cells end a round.
        for _ in 0..5 {
            g.apply(r#"{"type":"answer","cells":[0,1,2]}"#).unwrap_or_else(|_| panic!("answer"));
        }
        assert!(g.is_over());
        assert!(g.results().contains("\"playerId\":\"you\""));
    }
}

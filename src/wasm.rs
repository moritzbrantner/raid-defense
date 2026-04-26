use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmRaidDefense {
    state: GameState,
    version: u64,
}

#[wasm_bindgen]
impl WasmRaidDefense {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmRaidDefense, JsValue> {
        Ok(Self {
            state: new_raid_defense_state()
                .map_err(|error| JsValue::from_str(&error.to_string()))?,
            version: 0,
        })
    }

    pub fn view_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&raid_defense_view(&self.state))
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn apply_json(&mut self, command_json: &str) -> Result<String, JsValue> {
        let command: GameCommand = serde_json::from_str(command_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let outcome = apply_raid_defense_command(&mut self.state, command)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        self.version += 1;
        let response = command_response(true, self.version, outcome.events, &self.state, None)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        serde_json::to_string(&response).map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn advance(&mut self, delta_seconds: u64) -> Result<String, JsValue> {
        let mut logic = RaidDefenseLogic;
        self.state
            .advance_time_with_logic(delta_seconds, &mut logic)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        self.version += 1;
        let response = command_response(true, self.version, Vec::new(), &self.state, None)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        serde_json::to_string(&response).map_err(|error| JsValue::from_str(&error.to_string()))
    }
}

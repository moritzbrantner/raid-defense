use super::*;
use farm_engine::GameStateSave;
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
        Self::new_with_seed(DEFAULT_RAID_DEFENSE_MAP_SEED)
    }

    #[wasm_bindgen(js_name = newWithSeed)]
    pub fn new_with_seed(seed: u64) -> Result<WasmRaidDefense, JsValue> {
        Ok(Self {
            state: new_raid_defense_state_with_seed(seed)
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
        response_json(self.version, outcome.events, &self.state)
    }

    pub fn advance(&mut self, delta_seconds: u64) -> Result<String, JsValue> {
        let mut logic = RaidDefenseLogic;
        self.state
            .advance_time_with_logic(delta_seconds, &mut logic)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        self.version += 1;
        response_json(self.version, Vec::new(), &self.state)
    }

    pub fn save_snapshot_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.state.save_snapshot())
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn load_snapshot_json(&mut self, snapshot_json: &str) -> Result<String, JsValue> {
        let snapshot: GameStateSave = serde_json::from_str(snapshot_json)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        self.state = GameState::from_save_snapshot(snapshot)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        self.version += 1;
        response_json(self.version, Vec::new(), &self.state)
    }

    #[wasm_bindgen(js_name = grantResource)]
    pub fn grant_resource(&mut self, resource: &str, amount: u64) -> Result<String, JsValue> {
        if let Some(capacity) = self.state.inventory().capacity(resource) {
            let current = self.state.inventory().amount(resource);
            let required = current.saturating_add(amount);
            if required > capacity {
                self.state.inventory_mut().set_capacity(resource, required);
            }
        }
        self.state
            .inventory_mut()
            .add(resource, amount)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        self.version += 1;
        response_json(self.version, Vec::new(), &self.state)
    }
}

fn response_json(
    version: u64,
    events: Vec<GameEvent>,
    state: &GameState,
) -> Result<String, JsValue> {
    let response = command_response(true, version, events, state, None)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    serde_json::to_string(&response).map_err(|error| JsValue::from_str(&error.to_string()))
}

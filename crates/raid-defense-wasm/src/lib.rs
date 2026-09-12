#![forbid(unsafe_code)]

use raid_defense_core::{
    Command, EntityKind, Event, GameError, GameState, TOWER_COST,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const CONTRACT_VERSION: u8 = 2;

#[wasm_bindgen]
pub struct RaidDefenseGame {
    state: GameState,
}

#[wasm_bindgen]
impl RaidDefenseGame {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        Self {
            state: GameState::new(u64::from(seed)),
        }
    }

    pub fn snapshot(&self) -> String {
        serialize(&SnapshotDto::from(&self.state))
    }

    pub fn dispatch(&mut self, command_json: &str) -> String {
        dispatch_json(&mut self.state, command_json)
    }

    pub fn checksum(&self) -> String {
        self.state.checksum().to_string()
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CommandDto {
    PlaceTower { x: i16, z: i16 },
    StartWave,
    AdvanceTick,
}

impl From<CommandDto> for Command {
    fn from(value: CommandDto) -> Self {
        match value {
            CommandDto::PlaceTower { x, z } => Self::PlaceTower { x, z },
            CommandDto::StartWave => Self::StartWave,
            CommandDto::AdvanceTick => Self::AdvanceTick,
        }
    }
}

#[derive(Serialize)]
struct DispatchResponseDto {
    contract_version: u8,
    ok: bool,
    event: Option<EventDto>,
    error: Option<ErrorDto>,
    snapshot: SnapshotDto,
}

#[derive(Serialize)]
struct ErrorDto {
    code: &'static str,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EventDto {
    TowerBuilt { entity: u32, cell: CellDto },
    WaveStarted { wave: u32, raiders: u16 },
    TickAdvanced {
        tick: u64,
        shots: u16,
        kills: u16,
        town_damage: u16,
    },
}

impl From<Event> for EventDto {
    fn from(value: Event) -> Self {
        match value {
            Event::TowerBuilt { entity, cell } => Self::TowerBuilt {
                entity,
                cell: CellDto::from(cell),
            },
            Event::WaveStarted { wave, raiders } => Self::WaveStarted { wave, raiders },
            Event::TickAdvanced {
                tick,
                shots,
                kills,
                town_damage,
            } => Self::TickAdvanced {
                tick,
                shots,
                kills,
                town_damage,
            },
        }
    }
}

#[derive(Clone, Copy, Serialize)]
struct CellDto {
    x: i16,
    z: i16,
}

impl From<raid_defense_core::Cell> for CellDto {
    fn from(value: raid_defense_core::Cell) -> Self {
        Self {
            x: value.x,
            z: value.z,
        }
    }
}

#[derive(Serialize)]
struct SnapshotDto {
    contract_version: u8,
    seed: String,
    tick: u64,
    gold: u32,
    wave: u32,
    town_health: u16,
    town_max_health: u16,
    grid_width: i16,
    grid_height: i16,
    tower_cost: u32,
    checksum: String,
    entities: Vec<EntityDto>,
}

impl From<&GameState> for SnapshotDto {
    fn from(state: &GameState) -> Self {
        let snapshot = state.snapshot();
        Self {
            contract_version: CONTRACT_VERSION,
            seed: snapshot.seed.to_string(),
            tick: snapshot.tick,
            gold: snapshot.gold,
            wave: snapshot.wave,
            town_health: snapshot.town_health,
            town_max_health: snapshot.town_max_health,
            grid_width: snapshot.grid_width,
            grid_height: snapshot.grid_height,
            tower_cost: TOWER_COST,
            checksum: state.checksum().to_string(),
            entities: snapshot
                .entities
                .into_iter()
                .map(|entity| EntityDto {
                    id: entity.id,
                    kind: match entity.kind {
                        EntityKind::Town => "town",
                        EntityKind::Tower => "tower",
                        EntityKind::Raider => "raider",
                    },
                    x_milli: entity.x_milli,
                    z_milli: entity.z_milli,
                    cell: CellDto::from(entity.cell),
                    health: entity.health,
                    max_health: entity.max_health,
                    attack_damage: entity.attack_damage,
                    attack_range_milli: entity.attack_range_milli,
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct EntityDto {
    id: u32,
    kind: &'static str,
    x_milli: i32,
    z_milli: i32,
    cell: CellDto,
    health: u16,
    max_health: u16,
    attack_damage: u16,
    attack_range_milli: i32,
}

fn dispatch_json(state: &mut GameState, command_json: &str) -> String {
    let command = match serde_json::from_str::<CommandDto>(command_json) {
        Ok(command) => Command::from(command),
        Err(_) => {
            return serialize(&DispatchResponseDto {
                contract_version: CONTRACT_VERSION,
                ok: false,
                event: None,
                error: Some(ErrorDto {
                    code: "invalid_command_json",
                }),
                snapshot: SnapshotDto::from(&*state),
            });
        }
    };

    match state.apply(command) {
        Ok(event) => serialize(&DispatchResponseDto {
            contract_version: CONTRACT_VERSION,
            ok: true,
            event: Some(EventDto::from(event)),
            error: None,
            snapshot: SnapshotDto::from(&*state),
        }),
        Err(error) => serialize(&DispatchResponseDto {
            contract_version: CONTRACT_VERSION,
            ok: false,
            event: None,
            error: Some(ErrorDto {
                code: error_code(error),
            }),
            snapshot: SnapshotDto::from(&*state),
        }),
    }
}

const fn error_code(error: GameError) -> &'static str {
    match error {
        GameError::OutOfBounds => "out_of_bounds",
        GameError::CellOccupied => "cell_occupied",
        GameError::ProtectedCell => "protected_cell",
        GameError::PathBlocked => "path_blocked",
        GameError::InsufficientGold => "insufficient_gold",
        GameError::RaidersStillActive => "raiders_still_active",
        GameError::GameOver => "game_over",
    }
}

fn serialize<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("raid-defense contract DTOs must serialize")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn checksum_from_response(response: &str) -> String {
        let value: Value = serde_json::from_str(response).expect("response should be JSON");
        value["snapshot"]["checksum"]
            .as_str()
            .expect("checksum should be a string")
            .to_owned()
    }

    #[test]
    fn invalid_json_is_fail_closed_and_does_not_mutate_state() {
        let mut state = GameState::new(42);
        let before = state.checksum();

        let response = dispatch_json(&mut state, "not json");
        let value: Value = serde_json::from_str(&response).expect("response should be JSON");

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "invalid_command_json");
        assert_eq!(state.checksum(), before);
    }

    #[test]
    fn adapter_checksum_matches_native_core_after_same_command() {
        let mut adapted = GameState::new(7);
        let response = dispatch_json(
            &mut adapted,
            r#"{"type":"place_tower","x":2,"z":2}"#,
        );

        let mut native = GameState::new(7);
        native
            .apply(Command::PlaceTower { x: 2, z: 2 })
            .expect("native command should succeed");

        assert_eq!(
            checksum_from_response(&response),
            native.checksum().to_string()
        );
        assert_eq!(adapted, native);
    }

    #[test]
    fn rejected_rule_error_uses_stable_code_and_preserves_checksum() {
        let mut state = GameState::new(9);
        let before = state.checksum();

        let response = dispatch_json(
            &mut state,
            r#"{"type":"place_tower","x":8,"z":6}"#,
        );
        let value: Value = serde_json::from_str(&response).expect("response should be JSON");

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "protected_cell");
        assert_eq!(checksum_from_response(&response), before.to_string());
        assert_eq!(state.checksum(), before);
    }
}

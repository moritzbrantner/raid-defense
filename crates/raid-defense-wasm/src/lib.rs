#![forbid(unsafe_code)]

use raid_defense_core::{
    ARROW_TOWER_COST, CANNON_TOWER_COST, Command, EntityKind, Event, GameError, GameState,
    MAX_TOWER_LEVEL, ResourceKind, SAWMILL_COST, SAWMILL_INTERVAL_TICKS, SAWMILL_OUTPUT,
    TowerArchetype,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const CONTRACT_VERSION: u8 = 4;

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

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TowerArchetypeDto {
    Arrow,
    Cannon,
}

impl From<TowerArchetypeDto> for TowerArchetype {
    fn from(value: TowerArchetypeDto) -> Self {
        match value {
            TowerArchetypeDto::Arrow => Self::Arrow,
            TowerArchetypeDto::Cannon => Self::Cannon,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CommandDto {
    PlaceTower {
        x: i16,
        z: i16,
        archetype: TowerArchetypeDto,
    },
    PlaceSawmill {
        x: i16,
        z: i16,
    },
    UpgradeTower {
        x: i16,
        z: i16,
    },
    StartWave,
    AdvanceTick,
}

impl From<CommandDto> for Command {
    fn from(value: CommandDto) -> Self {
        match value {
            CommandDto::PlaceTower { x, z, archetype } => Self::PlaceTower {
                x,
                z,
                archetype: archetype.into(),
            },
            CommandDto::PlaceSawmill { x, z } => Self::PlaceSawmill { x, z },
            CommandDto::UpgradeTower { x, z } => Self::UpgradeTower { x, z },
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
    TowerBuilt {
        entity: u32,
        cell: CellDto,
        archetype: &'static str,
        level: u8,
        wood_cost: u32,
    },
    SawmillBuilt {
        entity: u32,
        cell: CellDto,
        wood_cost: u32,
    },
    TowerUpgraded {
        entity: u32,
        archetype: &'static str,
        level: u8,
        wood_cost: u32,
    },
    WaveStarted {
        wave: u32,
        raiders: u16,
    },
    TickAdvanced {
        tick: u64,
        shots: u16,
        impacts: u16,
        kills: u16,
        wood_produced: u16,
        wood_stolen: u16,
        town_damage: u16,
    },
}

impl From<Event> for EventDto {
    fn from(value: Event) -> Self {
        match value {
            Event::TowerBuilt {
                entity,
                cell,
                archetype,
                level,
                wood_cost,
            } => Self::TowerBuilt {
                entity,
                cell: CellDto::from(cell),
                archetype: tower_archetype_label(archetype),
                level,
                wood_cost,
            },
            Event::SawmillBuilt {
                entity,
                cell,
                wood_cost,
            } => Self::SawmillBuilt {
                entity,
                cell: CellDto::from(cell),
                wood_cost,
            },
            Event::TowerUpgraded {
                entity,
                archetype,
                level,
                wood_cost,
            } => Self::TowerUpgraded {
                entity,
                archetype: tower_archetype_label(archetype),
                level,
                wood_cost,
            },
            Event::WaveStarted { wave, raiders } => Self::WaveStarted { wave, raiders },
            Event::TickAdvanced {
                tick,
                shots,
                impacts,
                kills,
                wood_produced,
                wood_stolen,
                town_damage,
            } => Self::TickAdvanced {
                tick,
                shots,
                impacts,
                kills,
                wood_produced,
                wood_stolen,
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
    wood: u32,
    wood_capacity: u32,
    wave: u32,
    town_health: u16,
    town_max_health: u16,
    grid_width: i16,
    grid_height: i16,
    sawmill_cost: u32,
    sawmill_output: u16,
    sawmill_interval_ticks: u16,
    arrow_tower_cost: u32,
    cannon_tower_cost: u32,
    max_tower_level: u8,
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
            wood: snapshot.wood,
            wood_capacity: snapshot.wood_capacity,
            wave: snapshot.wave,
            town_health: snapshot.town_health,
            town_max_health: snapshot.town_max_health,
            grid_width: snapshot.grid_width,
            grid_height: snapshot.grid_height,
            sawmill_cost: SAWMILL_COST,
            sawmill_output: SAWMILL_OUTPUT,
            sawmill_interval_ticks: SAWMILL_INTERVAL_TICKS,
            arrow_tower_cost: ARROW_TOWER_COST,
            cannon_tower_cost: CANNON_TOWER_COST,
            max_tower_level: MAX_TOWER_LEVEL,
            checksum: state.checksum().to_string(),
            entities: snapshot
                .entities
                .into_iter()
                .map(|entity| EntityDto {
                    id: entity.id,
                    kind: match entity.kind {
                        EntityKind::TownHall => "town_hall",
                        EntityKind::Tower => "tower",
                        EntityKind::Sawmill => "sawmill",
                        EntityKind::Raider => "raider",
                        EntityKind::Projectile => "projectile",
                    },
                    x_milli: entity.x_milli,
                    z_milli: entity.z_milli,
                    cell: CellDto::from(entity.cell),
                    health: entity.health,
                    max_health: entity.max_health,
                    attack_damage: entity.attack_damage,
                    attack_range_milli: entity.attack_range_milli,
                    tower_archetype: entity.tower_archetype.map(tower_archetype_label),
                    tower_level: entity.tower_level,
                    upgrade_cost: entity.upgrade_cost,
                    projectile_target: entity.projectile_target,
                    stored_wood: entity.stored_wood,
                    wood_capacity: entity.wood_capacity,
                    production_resource: entity.production_resource.map(resource_kind_label),
                    production_amount: entity.production_amount,
                    production_interval_ticks: entity.production_interval_ticks,
                    production_progress_ticks: entity.production_progress_ticks,
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
    tower_archetype: Option<&'static str>,
    tower_level: u8,
    upgrade_cost: Option<u32>,
    projectile_target: Option<u32>,
    stored_wood: u32,
    wood_capacity: u32,
    production_resource: Option<&'static str>,
    production_amount: u16,
    production_interval_ticks: u16,
    production_progress_ticks: u16,
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

const fn tower_archetype_label(archetype: TowerArchetype) -> &'static str {
    match archetype {
        TowerArchetype::Arrow => "arrow",
        TowerArchetype::Cannon => "cannon",
    }
}

const fn resource_kind_label(resource: ResourceKind) -> &'static str {
    match resource {
        ResourceKind::Wood => "wood",
    }
}

const fn error_code(error: GameError) -> &'static str {
    match error {
        GameError::OutOfBounds => "out_of_bounds",
        GameError::CellOccupied => "cell_occupied",
        GameError::ProtectedCell => "protected_cell",
        GameError::PathBlocked => "path_blocked",
        GameError::InsufficientWood => "insufficient_wood",
        GameError::NoTower => "no_tower",
        GameError::MaxTowerLevel => "max_tower_level",
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
    fn sawmill_contract_matches_native_core() {
        let mut adapted = GameState::new(7);
        let response = dispatch_json(&mut adapted, r#"{"type":"place_sawmill","x":2,"z":2}"#);

        let mut native = GameState::new(7);
        native
            .apply(Command::PlaceSawmill { x: 2, z: 2 })
            .expect("native command should succeed");

        assert_eq!(
            checksum_from_response(&response),
            native.checksum().to_string()
        );
        assert_eq!(adapted, native);
    }

    #[test]
    fn snapshot_exposes_town_hall_wood_and_sawmill_production() {
        let mut state = GameState::new(7);
        state
            .apply(Command::PlaceSawmill { x: 2, z: 2 })
            .expect("sawmill should build");
        let snapshot = SnapshotDto::from(&state);

        assert_eq!(snapshot.wood, 80);
        assert!(snapshot.entities.iter().any(|entity| {
            entity.kind == "town_hall" && entity.stored_wood == 80 && entity.wood_capacity > 80
        }));
        assert!(snapshot.entities.iter().any(|entity| {
            entity.kind == "sawmill"
                && entity.production_resource == Some("wood")
                && entity.production_amount > 0
        }));
    }

    #[test]
    fn rejected_rule_error_uses_stable_code_and_preserves_checksum() {
        let mut state = GameState::new(9);
        let before = state.checksum();

        let response = dispatch_json(
            &mut state,
            r#"{"type":"place_tower","x":8,"z":6,"archetype":"cannon"}"#,
        );
        let value: Value = serde_json::from_str(&response).expect("response should be JSON");

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "protected_cell");
        assert_eq!(checksum_from_response(&response), before.to_string());
        assert_eq!(state.checksum(), before);
    }

    #[test]
    fn tick_event_reports_resource_theft() {
        let mut state = GameState::new(11);
        state.apply(Command::StartWave).expect("wave should start");
        let mut stolen = 0_u16;
        for _ in 0..40 {
            let response = dispatch_json(&mut state, r#"{"type":"advance_tick"}"#);
            let value: Value =
                serde_json::from_str(&response).expect("tick response should be JSON");
            stolen = stolen.saturating_add(
                value["event"]["wood_stolen"]
                    .as_u64()
                    .and_then(|value| u16::try_from(value).ok())
                    .unwrap_or(0),
            );
            if state.raider_count() == 0 {
                break;
            }
        }
        assert!(stolen > 0);
    }
}

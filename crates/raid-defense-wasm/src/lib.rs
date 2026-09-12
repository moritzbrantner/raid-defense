#![forbid(unsafe_code)]

use raid_defense_core::{
    Command, EntityKind, Event, GameError, GameState, PersonState, ResourceKind, TowerArchetype,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const CONTRACT_VERSION: u8 = 7;

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
    PlaceStorageHouse {
        x: i16,
        z: i16,
    },
    PlaceHouse {
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
            CommandDto::PlaceStorageHouse { x, z } => Self::PlaceStorageHouse { x, z },
            CommandDto::PlaceHouse { x, z } => Self::PlaceHouse { x, z },
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
    TowerConstructionStarted {
        entity: u32,
        cell: CellDto,
        archetype: &'static str,
        wood_required: u32,
    },
    SawmillBuilt {
        entity: u32,
        cell: CellDto,
        wood_cost: u32,
    },
    StorageHouseBuilt {
        entity: u32,
        cell: CellDto,
        wood_cost: u32,
        wood_capacity: u32,
    },
    HouseBuilt {
        entity: u32,
        cell: CellDto,
        wood_cost: u32,
        people_added: u16,
        population_capacity: u16,
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
        wood_picked_up: u16,
        wood_delivered: u16,
        towers_completed: u16,
        wood_stolen: u16,
        town_damage: u16,
        completed_wave: Option<u32>,
    },
}

impl From<Event> for EventDto {
    fn from(value: Event) -> Self {
        match value {
            Event::TowerConstructionStarted {
                entity,
                cell,
                archetype,
                wood_required,
            } => Self::TowerConstructionStarted {
                entity,
                cell: CellDto::from(cell),
                archetype: tower_archetype_label(archetype),
                wood_required,
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
            Event::StorageHouseBuilt {
                entity,
                cell,
                wood_cost,
                wood_capacity,
            } => Self::StorageHouseBuilt {
                entity,
                cell: CellDto::from(cell),
                wood_cost,
                wood_capacity,
            },
            Event::HouseBuilt {
                entity,
                cell,
                wood_cost,
                people_added,
                population_capacity,
            } => Self::HouseBuilt {
                entity,
                cell: CellDto::from(cell),
                wood_cost,
                people_added,
                population_capacity,
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
                wood_picked_up,
                wood_delivered,
                towers_completed,
                wood_stolen,
                town_damage,
                completed_wave,
            } => Self::TickAdvanced {
                tick,
                shots,
                impacts,
                kills,
                wood_produced,
                wood_picked_up,
                wood_delivered,
                towers_completed,
                wood_stolen,
                town_damage,
                completed_wave,
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
    completed_waves: u32,
    day_ticks_remaining: u16,
    is_night: bool,
    people: u16,
    population_capacity: u16,
    houses_unlocked: bool,
    town_health: u16,
    town_max_health: u16,
    grid_width: i16,
    grid_height: i16,
    forest_tile_count: u16,
    forest_tile_wood: u32,
    sawmill_harvest_radius: u16,
    sawmill_cost: u32,
    sawmill_output: u16,
    sawmill_interval_ticks: u16,
    sawmill_local_wood_capacity: u32,
    storage_house_cost: u32,
    storage_house_wood_capacity: u32,
    house_cost: u32,
    house_unlock_completed_waves: u32,
    house_population_capacity: u16,
    person_carry_capacity: u16,
    arrow_tower_cost: u32,
    cannon_tower_cost: u32,
    max_tower_level: u8,
    checksum: String,
    entities: Vec<EntityDto>,
}

impl From<&GameState> for SnapshotDto {
    fn from(state: &GameState) -> Self {
        let snapshot = state.snapshot();
        let rules = state.rules();
        Self {
            contract_version: CONTRACT_VERSION,
            seed: snapshot.seed.to_string(),
            tick: snapshot.tick,
            wood: snapshot.wood,
            wood_capacity: snapshot.wood_capacity,
            wave: snapshot.wave,
            completed_waves: snapshot.completed_waves,
            day_ticks_remaining: state.day_ticks_remaining(),
            is_night: state.is_night(),
            people: snapshot.people,
            population_capacity: snapshot.population_capacity,
            houses_unlocked: snapshot.houses_unlocked,
            town_health: snapshot.town_health,
            town_max_health: snapshot.town_max_health,
            grid_width: snapshot.grid_width,
            grid_height: snapshot.grid_height,
            forest_tile_count: rules.economy.forest_tile_count,
            forest_tile_wood: rules.economy.forest_tile_wood,
            sawmill_harvest_radius: rules.economy.sawmill_harvest_radius,
            sawmill_cost: rules.buildings.sawmill.wood_cost,
            sawmill_output: rules.economy.sawmill_output,
            sawmill_interval_ticks: rules.economy.sawmill_interval_ticks,
            sawmill_local_wood_capacity: rules.economy.sawmill_local_wood_capacity,
            storage_house_cost: rules.buildings.storage_house.wood_cost,
            storage_house_wood_capacity: rules.economy.storage_house_wood_capacity,
            house_cost: rules.buildings.house.wood_cost,
            house_unlock_completed_waves: rules.buildings.house.unlock_completed_waves,
            house_population_capacity: rules.buildings.house.population_capacity,
            person_carry_capacity: rules.population.carry_capacity,
            arrow_tower_cost: rules.towers.arrow.build_cost,
            cannon_tower_cost: rules.towers.cannon.build_cost,
            max_tower_level: rules.towers.max_level,
            checksum: state.checksum().to_string(),
            entities: snapshot.entities.into_iter().map(EntityDto::from).collect(),
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
    housing_capacity: u16,
    person_state: Option<&'static str>,
    person_target_entity: Option<u32>,
    cargo_wood: u16,
    cargo_capacity: u16,
}

impl From<raid_defense_core::EntitySnapshot> for EntityDto {
    fn from(entity: raid_defense_core::EntitySnapshot) -> Self {
        Self {
            id: entity.id,
            kind: match entity.kind {
                EntityKind::TownHall => "town_hall",
                EntityKind::Tower => "tower",
                EntityKind::Sawmill => "sawmill",
                EntityKind::StorageHouse => "storage_house",
                EntityKind::House => "house",
                EntityKind::Forest => "forest",
                EntityKind::Person => "person",
                EntityKind::Raider => "raider",
                EntityKind::Projectile => "projectile",
            },
            x_milli: entity.x_milli,
            z_milli: entity.z_milli,
            cell: CellDto::from(entity.cell),
            health: entity.health,
            max_health: entity.maximum,
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
            housing_capacity: entity.housing_capacity,
            person_state: entity.person_state.map(person_state_label),
            person_target_entity: entity.person_target_entity,
            cargo_wood: entity.cargo_wood,
            cargo_capacity: entity.cargo_capacity,
        }
    }
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

const fn person_state_label(state: PersonState) -> &'static str {
    match state {
        PersonState::IdleAtTownHall => "idle_at_town_hall",
        PersonState::ToSawmill => "to_sawmill",
        PersonState::ToStorage => "to_storage",
        PersonState::ToConstructionStorage => "to_construction_storage",
        PersonState::ToConstructionSite => "to_construction_site",
        PersonState::ToTownHall => "to_town_hall",
    }
}

const fn error_code(error: GameError) -> &'static str {
    match error {
        GameError::OutOfBounds => "out_of_bounds",
        GameError::CellOccupied => "cell_occupied",
        GameError::ProtectedCell => "protected_cell",
        GameError::PathBlocked => "path_blocked",
        GameError::InsufficientWood => "insufficient_wood",
        GameError::NoForestInRange => "no_forest_in_range",
        GameError::HouseLocked => "house_locked",
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
    use raid_defense_core::Cell;
    use serde_json::Value;

    fn checksum_from_response(response: &str) -> String {
        let value: Value = serde_json::from_str(response).expect("response should be JSON");
        value["snapshot"]["checksum"]
            .as_str()
            .expect("checksum should be a string")
            .to_owned()
    }

    fn accepted_sawmill_cell(state: &GameState) -> Cell {
        let snapshot = state.snapshot();
        for z in 1..snapshot.grid_height - 1 {
            for x in 1..snapshot.grid_width - 1 {
                let mut probe = state.clone();
                if probe.apply(Command::PlaceSawmill { x, z }).is_ok() {
                    return Cell::new(x, z);
                }
            }
        }
        panic!("seeded world must allow a sawmill");
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
    fn snapshot_exposes_seeded_resource_world() {
        let state = GameState::new(7);
        let snapshot = SnapshotDto::from(&state);
        assert_eq!(snapshot.contract_version, CONTRACT_VERSION);
        assert_eq!(snapshot.grid_width, 21);
        assert_eq!(snapshot.grid_height, 21);
        assert_eq!(snapshot.people, 2);
        assert!(
            snapshot
                .entities
                .iter()
                .any(|entity| entity.kind == "forest")
        );
    }

    #[test]
    fn sawmill_contract_matches_native_core() {
        let mut adapted = GameState::new(7);
        let cell = accepted_sawmill_cell(&adapted);
        let response = dispatch_json(
            &mut adapted,
            &format!(
                r#"{{"type":"place_sawmill","x":{},"z":{}}}"#,
                cell.x, cell.z
            ),
        );
        let mut native = GameState::new(7);
        native
            .apply(Command::PlaceSawmill {
                x: cell.x,
                z: cell.z,
            })
            .expect("native command should succeed");
        assert_eq!(
            checksum_from_response(&response),
            native.checksum().to_string()
        );
        assert_eq!(adapted, native);
    }

    #[test]
    fn storage_house_command_is_exposed() {
        let mut state = GameState::new(17);
        let response = dispatch_json(&mut state, r#"{"type":"place_storage_house","x":2,"z":2}"#);
        let value: Value = serde_json::from_str(&response).expect("response should be JSON");
        if value["ok"] == true {
            assert!(
                value["snapshot"]["entities"]
                    .as_array()
                    .expect("entities")
                    .iter()
                    .any(|entity| entity["kind"] == "storage_house")
            );
        }
    }

    #[test]
    fn protected_town_build_is_transactional() {
        let mut state = GameState::new(9);
        let before = state.checksum();
        let response = dispatch_json(
            &mut state,
            r#"{"type":"place_tower","x":10,"z":10,"archetype":"cannon"}"#,
        );
        let value: Value = serde_json::from_str(&response).expect("response should be JSON");
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "protected_cell");
        assert_eq!(checksum_from_response(&response), before.to_string());
        assert_eq!(state.checksum(), before);
    }
}

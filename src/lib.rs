use farm_engine::{
    AreaDefinition, Building, BuildingDefinition, BuildingFootprint, BuildingId,
    BuildingLevelDefinition, BuildingStatus, Catalog, CommandId, CommandOutcome, Effect,
    EngineError, EntityBlueprintRef, EntityId, EntityRecord, GameCommand, GameEvent, GameLogic,
    GameState, GameWorld, GameWorldError, Job, JobCompletion, LevelDefinition, MapLocation,
    MapTopology, NpcDefinition, NpcKind, PathDefinition, PlacementRule, PlayerId,
    ProductionQueueConfig, ProductionRule, ProductionStatus, Requirement, ResourceAmount,
    ResourceDefinition, ResourceError, ResourceId, ResourceStorage, StatId, TechNodeDefinition,
    TileDefinition, UnitDefinition, UpgradeDefinition, WorldId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

mod api;
mod catalog;
mod constants;
mod construction_logistics;
mod economy;
mod food_logistics;
mod logic;
mod raiders;
mod seed;
mod terrain;
mod view;
mod wiki;
#[cfg(feature = "wasm")]
mod wasm;

pub use self::api::*;
pub use self::catalog::raid_defense_catalog;
pub use self::constants::*;
pub use self::logic::{
    RaidDefenseLogic, apply_raid_defense_command, claim_named_province, claim_province,
    province_claim_requirements, set_tax_rate,
};
pub use self::seed::{
    new_raid_defense_state, new_raid_defense_state_with_seed, new_raid_defense_world,
    new_raid_defense_world_with_seed,
};
pub use self::view::raid_defense_view;

pub(crate) use self::construction_logistics::{
    advance_construction_logistics, initialize_construction_logistics_for_building,
};
pub(crate) use self::economy::{
    advance_resource_economy, advance_worker_hunger, initialize_worker_hunger,
    recruit_worker_at_castle,
};
pub(crate) use self::food_logistics::{
    FRONTIER_FORT_LOW_SUPPLY_THRESHOLD, advance_food_logistics, building_logistics_view,
    food_logistics_view, stability_gain_divisor,
};
pub(crate) use self::logic::empire_tax_rate;
pub(crate) use self::raiders::{advance_raiders, spawn_raider};
pub(crate) use self::seed::imperial_road;
pub(crate) use self::terrain::generate_seeded_terrain;
pub(crate) use self::wiki::{
    encountered_attack_waves, encountered_units, record_attack_wave, record_unit_encounter,
};
#[cfg(test)]
pub(crate) use self::terrain::terrain_kind_count;

#[cfg(test)]
mod tests {
    use super::*;
    use farm_engine::{GameCommand, ProductionStatus};

    fn castle_location(state: &GameState) -> MapLocation {
        state
            .buildings()
            .find(|building| building.kind.as_str() == CAPITAL)
            .unwrap()
            .location
    }

    fn recruit_engineer(state: &mut GameState) -> EntityId {
        match apply_raid_defense_command(
            state,
            GameCommand::SpawnEntity {
                blueprint: EntityBlueprintRef::Unit(ENGINEER.into()),
                name: None,
                location: castle_location(state),
            },
        )
        .unwrap()
        .events
        .as_slice()
        {
            [GameEvent::EntityCreated(unit)] => *unit,
            events => panic!("unexpected recruit events: {events:?}"),
        }
    }

    fn workers_for_construction(state: &GameState, building: BuildingId) -> Vec<EntityId> {
        state
            .entity_ids_of_blueprint(EntityBlueprintRef::Unit(ENGINEER.into()))
            .into_iter()
            .filter(|worker| {
                state
                    .entity_stat(*worker, "construction_target_building_id")
                    .ok()
                    .is_some_and(|target| target == building.get() as i64)
            })
            .collect()
    }

    fn worker_carried_total(state: &GameState, worker: EntityId) -> u64 {
        [TIMBER, STONE, IRON, GRAIN]
            .into_iter()
            .map(|resource| {
                state
                    .entity_stat(worker, format!("construction_carried:{resource}"))
                    .unwrap_or(0)
                    .max(0) as u64
            })
            .sum()
    }

    fn worker_stomach(state: &GameState, worker: EntityId) -> i64 {
        state.entity_stat(worker, WORKER_STOMACH).unwrap()
    }

    #[test]
    fn catalog_validates() {
        let report = raid_defense_catalog().validate();
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    }

    #[test]
    fn starting_state_has_center_castle_workers_and_resources() {
        let state = new_raid_defense_state().unwrap();
        let view = raid_defense_view(&state);
        let castle = state
            .buildings()
            .find(|building| building.kind.as_str() == CAPITAL)
            .unwrap();

        assert_eq!(
            state.ground_locations().count(),
            (RAID_DEFENSE_SIZE * RAID_DEFENSE_SIZE) as usize
        );
        assert_eq!(castle.location, MapLocation::new(15, 15));
        assert_eq!(view.buildings.len(), 1);
        assert_eq!(view.paths.len(), 0);
        assert_eq!(
            view.entities
                .iter()
                .filter(|entity| entity.blueprint == EntityBlueprintRef::Unit(ENGINEER.into()))
                .count(),
            5
        );
        assert_eq!(state.map_topology(), MapTopology::Hexagonal);
        assert_eq!(state.inventory().amount(TIMBER), 100);
        assert_eq!(state.inventory().amount(STONE), 100);
        assert_eq!(state.inventory().amount(GRAIN), 100);
        assert!(view.resources.iter().any(|resource| resource.id == TIMBER));
        assert!(view.resources.iter().any(|resource| resource.id == STONE));
        assert!(view.resources.iter().any(|resource| resource.id == GRAIN));
        assert_eq!(
            view.encountered_units
                .iter()
                .map(|unit| unit.kind.as_str())
                .collect::<Vec<_>>(),
            vec![ENGINEER]
        );
        assert!(view.encountered_attack_waves.is_empty());
    }

    #[test]
    fn wiki_unlocks_recruited_units_and_logged_attack_waves() {
        let mut state = new_raid_defense_state().unwrap();
        let castle = castle_location(&state);
        state.inventory_mut().set_capacity(CROWNS, 50);
        state.inventory_mut().add(CROWNS, 50).unwrap();

        apply_raid_defense_command(
            &mut state,
            GameCommand::SpawnEntity {
                blueprint: EntityBlueprintRef::Unit(PREFECT.into()),
                name: None,
                location: castle,
            },
        )
        .unwrap();
        apply_raid_defense_command(
            &mut state,
            GameCommand::SpawnEntity {
                blueprint: EntityBlueprintRef::Unit(BASIC_RAIDER.into()),
                name: None,
                location: MapLocation::new(10, 12),
            },
        )
        .unwrap();

        let view = raid_defense_view(&state);
        assert!(
            view.encountered_units
                .iter()
                .any(|unit| unit.kind == PREFECT && unit.current_count == 1)
        );
        assert!(
            view.encountered_units
                .iter()
                .any(|unit| unit.kind == BASIC_RAIDER)
        );
        assert_eq!(view.encountered_attack_waves.len(), 1);
        assert_eq!(view.encountered_attack_waves[0].entry, MapLocation::new(10, 12));
        assert_eq!(view.encountered_attack_waves[0].units.len(), 1);
        assert_eq!(view.encountered_attack_waves[0].units[0].kind, BASIC_RAIDER);
        assert_eq!(view.encountered_attack_waves[0].units[0].count, 1);
    }

    #[test]
    fn seeded_terrain_is_reproducible_and_contains_water_trees_and_ground() {
        let first = new_raid_defense_state_with_seed(42).unwrap();
        let second = new_raid_defense_state_with_seed(42).unwrap();
        let third = new_raid_defense_state_with_seed(1337).unwrap();

        let first_tiles = first
            .tiles()
            .map(|tile| ((tile.location.x, tile.location.y), tile.kind.to_string()))
            .collect::<BTreeMap<_, _>>();
        let second_tiles = second
            .tiles()
            .map(|tile| ((tile.location.x, tile.location.y), tile.kind.to_string()))
            .collect::<BTreeMap<_, _>>();
        let third_tiles = third
            .tiles()
            .map(|tile| ((tile.location.x, tile.location.y), tile.kind.to_string()))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(first_tiles, second_tiles);
        assert!(first_tiles.iter().any(|(location, kind)| {
            third_tiles
                .get(location)
                .is_some_and(|other_kind| other_kind != kind)
        }));

        assert!(terrain_kind_count(&first, WATER_TILE) > 0);
        assert!(terrain_kind_count(&first, FOREST) > 0);
        assert!(terrain_kind_count(&first, PLAINS) > 0);
    }

    #[test]
    fn castle_is_unique() {
        let mut state = new_raid_defense_state().unwrap();
        assert!(matches!(
            state.start_construction_at(CAPITAL, MapLocation::new(14, 15)),
            Err(EngineError::RequirementsNotMet(_))
        ));
    }

    #[test]
    fn can_place_storage_house_farm_and_tower() {
        let mut state = new_raid_defense_state().unwrap();
        state.inventory_mut().set_capacity(TIMBER, 200);
        state.inventory_mut().add(TIMBER, 25).unwrap();
        let storage = state
            .start_construction_at(MARKET, MapLocation::new(10, 10))
            .unwrap();
        let farm = state
            .start_construction_at(FARMSTEAD, MapLocation::new(12, 10))
            .unwrap();
        let tower = state
            .start_construction_at(WATCHTOWER, MapLocation::new(16, 10))
            .unwrap();

        assert!(state.building(storage).is_some());
        assert!(state.building(farm).is_some());
        assert!(state.building(tower).is_some());
        assert_eq!(
            state.building(farm).unwrap().footprint,
            BuildingFootprint::new(4, 4)
        );
        assert_eq!(
            state.building(tower).unwrap().footprint,
            BuildingFootprint::new(2, 2)
        );
    }

    #[test]
    fn capacity_buildings_offer_a_level_two_upgrade_with_doubled_capacity() {
        let catalog = raid_defense_catalog();

        let market = catalog.building(MARKET).unwrap();
        assert_eq!(market.max_level(), Some(2));
        let total_storage_bonus = market
            .levels()
            .flat_map(|level| level.storage_bonus.iter())
            .fold(BTreeMap::new(), |mut totals, bonus| {
                *totals.entry(bonus.resource.clone()).or_insert(0) += bonus.amount;
                totals
            });
        assert_eq!(total_storage_bonus.get(&TIMBER.into()), Some(&300));
        assert_eq!(total_storage_bonus.get(&STONE.into()), Some(&300));
        assert_eq!(total_storage_bonus.get(&GRAIN.into()), Some(&300));

        let senate_hall = catalog.building(SENATE_HALL).unwrap();
        assert_eq!(
            senate_hall.level(2).unwrap().inventory_capacity,
            vec![ResourceAmount::new(GRAIN, 120)]
        );

        let barracks = catalog.building(BARRACKS).unwrap();
        assert_eq!(
            barracks.level(2).unwrap().inventory_capacity,
            vec![ResourceAmount::new(GRAIN, 120)]
        );

        let iron_mine = catalog.building(IRON_MINE).unwrap();
        assert_eq!(
            iron_mine.level(2).unwrap().inventory_capacity,
            vec![ResourceAmount::new(CITIZENS, 160)]
        );

        let embassy = catalog.building(EMBASSY).unwrap();
        assert_eq!(
            embassy.level(2).unwrap().inventory_capacity,
            vec![ResourceAmount::new(GRAIN, 120)]
        );

        let frontier_fort = catalog.building(FRONTIER_FORT).unwrap();
        assert_eq!(
            frontier_fort.level(2).unwrap().inventory_capacity,
            vec![ResourceAmount::new(GRAIN, 120)]
        );
    }

    #[test]
    fn command_wrapper_clamps_tax_rate() {
        let mut state = new_raid_defense_state().unwrap();
        let capital = state
            .buildings()
            .find(|building| building.kind.as_str() == CAPITAL)
            .unwrap()
            .id;

        let outcome = apply_raid_defense_command(
            &mut state,
            GameCommand::SetBuildingStat {
                building: capital,
                stat: TAX_RATE.into(),
                value: 99,
            },
        )
        .unwrap();

        assert_eq!(
            state.building_stat(capital, TAX_RATE).unwrap(),
            MAX_TAX_RATE
        );
        assert_eq!(outcome.events.len(), 1);
    }

    #[test]
    fn storage_house_upgrade_doubles_its_storage_bonus() {
        let mut state = new_raid_defense_state().unwrap();
        let storage_house = state
            .start_construction_at(MARKET, MapLocation::new(10, 10))
            .unwrap();

        state.advance_time(8).unwrap();

        assert_eq!(state.inventory().capacity(TIMBER), Some(250));
        assert_eq!(state.inventory().capacity(STONE), Some(250));
        assert_eq!(state.inventory().capacity(GRAIN), Some(250));

        let outcome = apply_raid_defense_command(
            &mut state,
            GameCommand::UpgradeBuilding {
                building: storage_house,
            },
        )
        .unwrap();

        assert!(matches!(
            outcome.events.as_slice(),
            [GameEvent::BuildingUpgradeStarted {
                building,
                job: _,
            }] if *building == storage_house
        ));

        state.advance_time(8).unwrap();

        assert_eq!(state.building(storage_house).unwrap().level, 2);
        assert_eq!(state.inventory().capacity(TIMBER), Some(400));
        assert_eq!(state.inventory().capacity(STONE), Some(400));
        assert_eq!(state.inventory().capacity(GRAIN), Some(400));
    }

    #[test]
    fn workers_are_recruited_at_the_castle_for_food() {
        let mut state = new_raid_defense_state().unwrap();
        let castle_location = state
            .buildings()
            .find(|building| building.kind.as_str() == CAPITAL)
            .unwrap()
            .location;

        let invalid = apply_raid_defense_command(
            &mut state,
            GameCommand::SpawnEntity {
                blueprint: EntityBlueprintRef::Unit(ENGINEER.into()),
                name: None,
                location: MapLocation::new(castle_location.x + 1, castle_location.y),
            },
        );
        assert!(matches!(invalid, Err(RaidDefenseError::InvalidCommand(_))));
        assert_eq!(state.inventory().amount(GRAIN), 100);
        assert_eq!(
            state
                .entity_ids_of_blueprint(EntityBlueprintRef::Unit(ENGINEER.into()))
                .len(),
            5
        );

        let outcome = apply_raid_defense_command(
            &mut state,
            GameCommand::SpawnEntity {
                blueprint: EntityBlueprintRef::Unit(ENGINEER.into()),
                name: None,
                location: castle_location,
            },
        )
        .unwrap();

        assert!(matches!(
            outcome.events.as_slice(),
            [GameEvent::EntityCreated(_)]
        ));
        assert_eq!(state.inventory().amount(GRAIN), 95);
        assert_eq!(
            state
                .entity_ids_of_blueprint(EntityBlueprintRef::Unit(ENGINEER.into()))
                .len(),
            6
        );
    }

    #[test]
    fn workers_start_full_and_eat_when_hunger_drops_below_thirty_percent() {
        let mut state = new_raid_defense_state().unwrap();
        let mut logic = RaidDefenseLogic;
        let workers = state.entity_ids_of_blueprint(EntityBlueprintRef::Unit(ENGINEER.into()));
        let worker = workers[0];

        assert_eq!(worker_stomach(&state, worker), 500);

        state.advance_time_with_logic(351, &mut logic).unwrap();

        assert_eq!(state.inventory().amount(GRAIN), 95);
        for worker in workers {
            assert_eq!(worker_stomach(&state, worker), 249);
        }
    }

    #[test]
    fn hungry_workers_do_not_eat_without_food_in_storage() {
        let mut state = new_raid_defense_state().unwrap();
        let mut logic = RaidDefenseLogic;
        let worker = state.entity_ids_of_blueprint(EntityBlueprintRef::Unit(ENGINEER.into()))[0];
        let food = state.inventory().amount(GRAIN);
        state.inventory_mut().remove(GRAIN, food).unwrap();

        state.advance_time_with_logic(351, &mut logic).unwrap();

        assert_eq!(state.inventory().amount(GRAIN), 0);
        assert_eq!(worker_stomach(&state, worker), 149);
    }

    #[test]
    fn construction_hauling_uses_the_nearest_storage_house_with_up_to_three_workers() {
        let mut state = new_raid_defense_state().unwrap();
        let mut logic = RaidDefenseLogic;

        let nearest_storage = state
            .start_construction_at(MARKET, MapLocation::new(10, 10))
            .unwrap();
        state
            .start_construction_at(MARKET, MapLocation::new(18, 10))
            .unwrap();
        state.advance_time_with_logic(8, &mut logic).unwrap();

        let farm = match apply_raid_defense_command(
            &mut state,
            GameCommand::ConstructBuilding {
                kind: FARMSTEAD.into(),
                location: MapLocation::new(12, 10),
            },
        )
        .unwrap()
        .events
        .as_slice()
        {
            [GameEvent::BuildingConstructionStarted { building, .. }] => *building,
            events => panic!("unexpected construction events: {events:?}"),
        };

        state.advance_time_with_logic(1, &mut logic).unwrap();

        let workers = workers_for_construction(&state, farm);
        assert_eq!(workers.len(), 3);
        for worker in workers {
            assert_eq!(
                state
                    .entity_stat(worker, "construction_source_building_id")
                    .unwrap(),
                nearest_storage.get() as i64
            );
        }
    }

    #[test]
    fn construction_workers_return_for_more_material_after_using_their_load() {
        let mut state = new_raid_defense_state().unwrap();
        let mut logic = RaidDefenseLogic;

        state
            .start_construction_at(MARKET, MapLocation::new(15, 10))
            .unwrap();
        state.advance_time_with_logic(8, &mut logic).unwrap();

        let workers = state.entity_ids_of_blueprint(EntityBlueprintRef::Unit(ENGINEER.into()));
        for worker in workers.into_iter().take(3) {
            state.move_entity(worker, MapLocation::new(15, 10)).unwrap();
        }

        let tower = match apply_raid_defense_command(
            &mut state,
            GameCommand::ConstructBuilding {
                kind: WATCHTOWER.into(),
                location: MapLocation::new(16, 10),
            },
        )
        .unwrap()
        .events
        .as_slice()
        {
            [GameEvent::BuildingConstructionStarted { building, .. }] => *building,
            events => panic!("unexpected construction events: {events:?}"),
        };

        for _ in 0..9 {
            state.advance_time_with_logic(1, &mut logic).unwrap();
        }

        let hauling_workers = workers_for_construction(&state, tower);
        assert_eq!(hauling_workers.len(), 3);
        for worker in hauling_workers {
            assert_eq!(state.entity_stat(worker, "construction_phase").unwrap(), 1);
            assert_eq!(worker_carried_total(&state, worker), 0);
        }
    }

    #[test]
    fn farm_needs_a_worker_and_cycles_food_back_into_storage() {
        let mut state = new_raid_defense_state().unwrap();
        let worker = recruit_engineer(&mut state);

        let farm = state
            .start_construction_at(FARMSTEAD, MapLocation::new(10, 10))
            .unwrap();
        let mut logic = RaidDefenseLogic;
        state.advance_time_with_logic(8, &mut logic).unwrap();
        state.advance_time_with_logic(1, &mut logic).unwrap();
        assert_eq!(
            state.building(farm).unwrap().production_status,
            ProductionStatus::Idle
        );

        state.assign_entity_to_building(worker, farm).unwrap();
        state.advance_time_with_logic(1, &mut logic).unwrap();
        assert!(matches!(
            state.building(farm).unwrap().production_status,
            ProductionStatus::InProgress { .. }
        ));

        state.advance_time_with_logic(300, &mut logic).unwrap();

        assert_eq!(state.inventory().amount(GRAIN), 95);
        assert_eq!(state.building(farm).unwrap().inventory.amount(GRAIN), 8);
        assert_eq!(
            state
                .entity(worker)
                .unwrap()
                .assignment
                .unwrap()
                .assigned_building,
            Some(farm)
        );
        assert_eq!(
            state
                .entity(worker)
                .unwrap()
                .assignment
                .unwrap()
                .assigned_job,
            None
        );
        assert!(matches!(
            state.building(farm).unwrap().production_status,
            ProductionStatus::InProgress { .. }
        ));
    }

    #[test]
    fn farms_only_deliver_food_after_storage_route_completes() {
        let mut state = new_raid_defense_state().unwrap();
        let worker = recruit_engineer(&mut state);
        let farm = state
            .start_construction_at(FARMSTEAD, MapLocation::new(10, 10))
            .unwrap();
        let storage = state
            .start_construction_at(MARKET, MapLocation::new(15, 10))
            .unwrap();

        let mut logic = RaidDefenseLogic;
        state.advance_time_with_logic(8, &mut logic).unwrap();
        state.assign_entity_to_building(worker, farm).unwrap();
        state.advance_time_with_logic(1, &mut logic).unwrap();
        state.advance_time_with_logic(60, &mut logic).unwrap();

        assert_eq!(state.inventory().amount(GRAIN), 95);
        assert_eq!(state.building(farm).unwrap().inventory.amount(GRAIN), 8);
        let view = raid_defense_view(&state);
        assert_eq!(view.food_logistics.delivered_last_minute, 0);
        assert_eq!(
            view.buildings
                .iter()
                .find(|building| building.id == storage.get())
                .and_then(|building| building.logistics.as_ref())
                .map(|logistics| logistics.active_routes),
            Some(1)
        );

        state.advance_time_with_logic(36, &mut logic).unwrap();

        assert_eq!(state.inventory().amount(GRAIN), 101);
        assert_eq!(state.building(farm).unwrap().inventory.amount(GRAIN), 2);
        let view = raid_defense_view(&state);
        assert_eq!(view.food_logistics.delivered_last_minute, 6);
    }

    #[test]
    fn farms_fill_and_then_spoil_without_storage_coverage() {
        let mut state = new_raid_defense_state().unwrap();
        let worker = recruit_engineer(&mut state);
        let farm = state
            .start_construction_at(FARMSTEAD, MapLocation::new(10, 10))
            .unwrap();

        let mut logic = RaidDefenseLogic;
        state.advance_time_with_logic(8, &mut logic).unwrap();
        state.assign_entity_to_building(worker, farm).unwrap();
        state.advance_time_with_logic(1, &mut logic).unwrap();
        state.advance_time_with_logic(60, &mut logic).unwrap();
        state.advance_time_with_logic(60, &mut logic).unwrap();
        state.advance_time_with_logic(60, &mut logic).unwrap();

        assert_eq!(state.building(farm).unwrap().inventory.amount(GRAIN), 24);
        let full_view = raid_defense_view(&state);
        assert_eq!(full_view.food_logistics.blocked_farms, 1);
        assert_eq!(
            full_view
                .buildings
                .iter()
                .find(|building| building.id == farm.get())
                .and_then(|building| building.logistics.as_ref())
                .map(|logistics| logistics.blocked),
            Some(true)
        );

        state.advance_time_with_logic(15, &mut logic).unwrap();

        assert_eq!(state.building(farm).unwrap().inventory.amount(GRAIN), 23);
        let spoiled_view = raid_defense_view(&state);
        assert_eq!(spoiled_view.food_logistics.spoiled_last_minute, 1);
        assert_eq!(
            spoiled_view
                .buildings
                .iter()
                .find(|building| building.id == farm.get())
                .and_then(|building| building.logistics.as_ref())
                .map(|logistics| logistics.spoiling),
            Some(true)
        );
    }

    #[test]
    fn roads_speed_up_food_routes() {
        let mut offroad = new_raid_defense_state().unwrap();
        let offroad_worker = recruit_engineer(&mut offroad);
        let offroad_farm = offroad
            .start_construction_at(FARMSTEAD, MapLocation::new(10, 10))
            .unwrap();
        offroad
            .start_construction_at(MARKET, MapLocation::new(10, 14))
            .unwrap();

        let mut road_backed = new_raid_defense_state().unwrap();
        let road_worker = recruit_engineer(&mut road_backed);
        let road_farm = road_backed
            .start_construction_at(FARMSTEAD, MapLocation::new(10, 10))
            .unwrap();
        road_backed
            .start_construction_at(MARKET, MapLocation::new(10, 14))
            .unwrap();
        apply_raid_defense_command(
            &mut road_backed,
            GameCommand::CreatePath {
                kind: IMPERIAL_ROAD.into(),
                waypoints: vec![
                    MapLocation::new(10, 10),
                    MapLocation::new(10, 11),
                    MapLocation::new(10, 12),
                    MapLocation::new(10, 13),
                    MapLocation::new(10, 14),
                ],
            },
        )
        .unwrap();

        let mut logic = RaidDefenseLogic;
        for state in [&mut offroad, &mut road_backed] {
            state.advance_time_with_logic(8, &mut logic).unwrap();
        }
        offroad
            .assign_entity_to_building(offroad_worker, offroad_farm)
            .unwrap();
        road_backed
            .assign_entity_to_building(road_worker, road_farm)
            .unwrap();
        for state in [&mut offroad, &mut road_backed] {
            state.advance_time_with_logic(1, &mut logic).unwrap();
            state.advance_time_with_logic(60, &mut logic).unwrap();
            state.advance_time_with_logic(24, &mut logic).unwrap();
        }

        assert_eq!(road_backed.inventory().amount(GRAIN), 101);
        assert_eq!(offroad.inventory().amount(GRAIN), 95);
        assert_eq!(
            offroad
                .building(offroad_farm)
                .unwrap()
                .inventory
                .amount(GRAIN),
            8
        );
    }

    #[test]
    fn strained_food_reserve_slows_food_consuming_production() {
        let mut state = new_raid_defense_state().unwrap();
        state.inventory_mut().add(CROWNS, 100).unwrap();
        state.inventory_mut().add(IRON, 10).unwrap();
        state.inventory_mut().add(LEGION_STRENGTH, 3).unwrap();
        let barracks = state
            .start_construction_at(BARRACKS, MapLocation::new(11, 10))
            .unwrap();
        let legate = state
            .spawn_entity(
                EntityBlueprintRef::Unit(LEGATE.into()),
                None,
                castle_location(&state),
            )
            .unwrap();

        let mut logic = RaidDefenseLogic;
        state.advance_time_with_logic(14, &mut logic).unwrap();
        state.assign_entity_to_building(legate, barracks).unwrap();
        let excess_grain = state.inventory().amount(GRAIN) - 12;
        state.inventory_mut().remove(GRAIN, excess_grain).unwrap();
        state.advance_time_with_logic(1, &mut logic).unwrap();
        state.start_production(barracks).unwrap();

        assert_eq!(
            state.building(barracks).unwrap().production_status,
            ProductionStatus::InProgress {
                completes_at_seconds: state.now_seconds() + 20,
            }
        );
    }

    #[test]
    fn empty_food_reserve_drains_frontier_fort_supply_and_security() {
        let mut state = new_raid_defense_state().unwrap();
        state.inventory_mut().add(CROWNS, 100).unwrap();
        let fort = state
            .start_construction_at(FRONTIER_FORT, MapLocation::new(23, 18))
            .unwrap();

        let mut logic = RaidDefenseLogic;
        state.advance_time_with_logic(24, &mut logic).unwrap();
        let grain = state.inventory().amount(GRAIN);
        state.inventory_mut().remove(GRAIN, grain).unwrap();
        state.advance_time_with_logic(240, &mut logic).unwrap();

        assert_eq!(state.building_stat(fort, SUPPLY).unwrap(), 38);
        assert_eq!(state.building_stat(fort, SECURITY).unwrap(), 50);
        assert_eq!(
            raid_defense_view(&state).food_logistics.reserve_state,
            "empty"
        );
    }

    #[test]
    fn lumber_camp_and_quarry_are_worker_run_resource_buildings() {
        let mut state = new_raid_defense_state().unwrap();
        let castle_location = state
            .buildings()
            .find(|building| building.kind.as_str() == CAPITAL)
            .unwrap()
            .location;
        let worker = match apply_raid_defense_command(
            &mut state,
            GameCommand::SpawnEntity {
                blueprint: EntityBlueprintRef::Unit(ENGINEER.into()),
                name: None,
                location: castle_location,
            },
        )
        .unwrap()
        .events
        .as_slice()
        {
            [GameEvent::EntityCreated(unit)] => *unit,
            events => panic!("unexpected recruit events: {events:?}"),
        };

        let lumber_camp = state
            .start_construction_at(LUMBER_CAMP, MapLocation::new(10, 10))
            .unwrap();
        let mut logic = RaidDefenseLogic;
        state.advance_time_with_logic(8, &mut logic).unwrap();
        state.advance_time_with_logic(1, &mut logic).unwrap();
        assert_eq!(
            state.building(lumber_camp).unwrap().footprint,
            BuildingFootprint::new(4, 4)
        );
        assert_eq!(
            state.building(lumber_camp).unwrap().production_status,
            ProductionStatus::Idle
        );

        state
            .assign_entity_to_building(worker, lumber_camp)
            .unwrap();
        state.advance_time_with_logic(1, &mut logic).unwrap();
        state.advance_time_with_logic(300, &mut logic).unwrap();

        assert_eq!(state.inventory().amount(TIMBER), 55);

        let quarry = state
            .start_construction_at(QUARRY, MapLocation::new(15, 10))
            .unwrap();
        state.advance_time_with_logic(8, &mut logic).unwrap();
        state.advance_time_with_logic(1, &mut logic).unwrap();
        assert_eq!(
            state.building(quarry).unwrap().footprint,
            BuildingFootprint::new(4, 4)
        );
    }
}

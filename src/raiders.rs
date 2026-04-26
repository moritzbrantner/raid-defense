use super::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

const STATE_SPAWNED: i64 = 0;
const STATE_MOVING: i64 = 1;
const STATE_ATTACKING_BLOCKER: i64 = 2;
const STATE_ATTACKING_STORAGE: i64 = 3;
const STATE_LOOTING: i64 = 4;
const STATE_RETREATING: i64 = 5;

#[derive(Copy, Clone)]
struct RaiderProfile {
    speed: u64,
    attack_damage: i64,
    carry_capacity: u64,
    looting_speed: u64,
}

#[derive(Clone)]
struct StorageTarget {
    storage_id: BuildingId,
    path: Vec<MapLocation>,
    blocker_id: Option<BuildingId>,
    blocker_path: Vec<MapLocation>,
}

pub(crate) fn spawn_raider(
    state: &mut GameState,
    location: MapLocation,
) -> Result<CommandOutcome, RaidDefenseError> {
    let raider = state.spawn_entity(
        EntityBlueprintRef::Unit(BASIC_RAIDER.into()),
        None,
        location,
    )?;
    initialize_raider(state, raider)?;
    Ok(CommandOutcome {
        events: vec![GameEvent::EntityCreated(raider)],
    })
}

pub(crate) fn advance_raiders(
    state: &mut GameState,
    delta_seconds: u64,
) -> Result<Vec<GameEvent>, EngineError> {
    if delta_seconds == 0 {
        return Ok(Vec::new());
    }

    let mut events = Vec::new();
    for _ in 0..delta_seconds {
        let raiders = state.entity_ids_of_blueprint(EntityBlueprintRef::Unit(BASIC_RAIDER.into()));
        for raider in raiders {
            if state.entity(raider).is_none() {
                continue;
            }
            initialize_raider(state, raider)?;
            step_raider(state, raider, &mut events)?;
        }
    }
    Ok(events)
}

fn step_raider(
    state: &mut GameState,
    raider: EntityId,
    events: &mut Vec<GameEvent>,
) -> Result<(), EngineError> {
    match state.entity_stat(raider, RAIDER_STATE)? {
        STATE_LOOTING => loot_from_pending_drop(state, raider)?,
        STATE_RETREATING => retreat_raider(state, raider, events)?,
        _ => advance_toward_storage(state, raider, events)?,
    }
    Ok(())
}

fn advance_toward_storage(
    state: &mut GameState,
    raider: EntityId,
    events: &mut Vec<GameEvent>,
) -> Result<(), EngineError> {
    let Some(location) = state.entity(raider).map(|entity| entity.location) else {
        return Ok(());
    };
    let Some(target) = find_nearest_storage_target(state, raider) else {
        begin_retreat(state, raider)?;
        return Ok(());
    };
    state.set_entity_stat(
        raider,
        RAIDER_TARGET_STORAGE_ID,
        target.storage_id.get() as i64,
    )?;

    if let Some(blocker) = target.blocker_id {
        state.set_entity_stat(raider, RAIDER_STATE, STATE_ATTACKING_BLOCKER)?;
        state.set_entity_stat(raider, RAIDER_TARGET_BLOCKER_ID, blocker.get() as i64)?;
        if adjacent_to_building(state, location, blocker) {
            attack_building(state, raider, blocker, false, events)?;
        } else {
            move_raider_along(state, raider, &target.blocker_path, events)?;
        }
        return Ok(());
    }

    state.set_entity_stat(raider, RAIDER_TARGET_BLOCKER_ID, 0)?;
    if adjacent_to_building(state, location, target.storage_id) {
        state.set_entity_stat(raider, RAIDER_STATE, STATE_ATTACKING_STORAGE)?;
        attack_building(state, raider, target.storage_id, true, events)?;
    } else {
        state.set_entity_stat(raider, RAIDER_STATE, STATE_MOVING)?;
        move_raider_along(state, raider, &target.path, events)?;
    }
    Ok(())
}

fn retreat_raider(
    state: &mut GameState,
    raider: EntityId,
    events: &mut Vec<GameEvent>,
) -> Result<(), EngineError> {
    let Some(location) = state.entity(raider).map(|entity| entity.location) else {
        return Ok(());
    };
    let entrance = MapLocation::new(
        state.entity_stat(raider, RAIDER_ENTRANCE_X)? as i32,
        state.entity_stat(raider, RAIDER_ENTRANCE_Y)? as i32,
    );
    if location == entrance {
        refund_pending_drop(state, raider)?;
        state.remove_entity(raider)?;
        events.push(GameEvent::EntityRemoved {
            kind: BASIC_RAIDER.to_owned(),
            id: raider.get(),
        });
        return Ok(());
    }
    if let Some(path) = shortest_path(state, location, &BTreeSet::from([key(entrance)]), false) {
        move_raider_along(state, raider, &path, events)?;
    } else {
        refund_pending_drop(state, raider)?;
        state.remove_entity(raider)?;
        events.push(GameEvent::EntityRemoved {
            kind: BASIC_RAIDER.to_owned(),
            id: raider.get(),
        });
    }
    Ok(())
}

fn attack_building(
    state: &mut GameState,
    raider: EntityId,
    building: BuildingId,
    storage_target: bool,
    events: &mut Vec<GameEvent>,
) -> Result<(), EngineError> {
    let damage = state.entity_stat(raider, RAIDER_ATTACK_DAMAGE)?;
    let current = building_hit_points(state, building)?;
    let next = (current - damage).max(0);
    state.set_building_stat(building, HIT_POINTS, next)?;
    events.push(GameEvent::BuildingStatChanged {
        building,
        stat: HIT_POINTS.into(),
        value: next,
    });
    if next > 0 {
        return Ok(());
    }

    if storage_target {
        destroy_storage_and_prepare_loot(state, raider, building)?;
    } else {
        state.remove_building(building)?;
        events.push(GameEvent::MapChanged);
    }
    Ok(())
}

fn destroy_storage_and_prepare_loot(
    state: &mut GameState,
    raider: EntityId,
    building: BuildingId,
) -> Result<(), EngineError> {
    let building_entry = state
        .building(building)
        .cloned()
        .ok_or(EngineError::UnknownBuilding(building))?;
    let drops = storage_drop_amounts(state, &building_entry)?;
    for (resource, amount) in &drops {
        if *amount > 0 {
            state
                .inventory_mut()
                .spend(&[ResourceAmount::new(resource.clone(), *amount)])?;
        }
    }
    state.remove_building(building)?;
    state.set_entity_stat(raider, RAIDER_TARGET_STORAGE_ID, 0)?;
    state.set_entity_stat(raider, RAIDER_TARGET_BLOCKER_ID, 0)?;
    state.set_entity_stat(
        raider,
        RAIDER_PENDING_GRAIN,
        drops.get(&GRAIN.into()).copied().unwrap_or(0) as i64,
    )?;
    state.set_entity_stat(
        raider,
        RAIDER_PENDING_TIMBER,
        drops.get(&TIMBER.into()).copied().unwrap_or(0) as i64,
    )?;
    state.set_entity_stat(
        raider,
        RAIDER_PENDING_STONE,
        drops.get(&STONE.into()).copied().unwrap_or(0) as i64,
    )?;
    if pending_drop_total(state, raider)? == 0 {
        begin_retreat(state, raider)?;
    } else {
        state.set_entity_stat(raider, RAIDER_STATE, STATE_LOOTING)?;
    }
    Ok(())
}

fn loot_from_pending_drop(state: &mut GameState, raider: EntityId) -> Result<(), EngineError> {
    let carry_capacity = state.entity_stat(raider, RAIDER_CARRY_CAPACITY)? as u64;
    let carried = state.entity_stat(raider, RAIDER_CARRIED_TOTAL)? as u64;
    let looting_speed = state.entity_stat(raider, RAIDER_LOOTING_SPEED)? as u64;
    let mut budget = looting_speed.min(carry_capacity.saturating_sub(carried));
    for (pending_stat, carried_stat) in [
        (RAIDER_PENDING_GRAIN, RAIDER_CARRIED_GRAIN),
        (RAIDER_PENDING_TIMBER, RAIDER_CARRIED_TIMBER),
        (RAIDER_PENDING_STONE, RAIDER_CARRIED_STONE),
    ] {
        if budget == 0 {
            break;
        }
        let available = state.entity_stat(raider, pending_stat)?.max(0) as u64;
        let taken = available.min(budget);
        if taken == 0 {
            continue;
        }
        state.adjust_entity_stat(raider, pending_stat, -(taken as i64))?;
        state.adjust_entity_stat(raider, carried_stat, taken as i64)?;
        state.adjust_entity_stat(raider, RAIDER_CARRIED_TOTAL, taken as i64)?;
        budget -= taken;
    }
    if pending_drop_total(state, raider)? == 0
        || state.entity_stat(raider, RAIDER_CARRIED_TOTAL)? as u64 >= carry_capacity
    {
        refund_pending_drop(state, raider)?;
        begin_retreat(state, raider)?;
    }
    Ok(())
}

fn find_nearest_storage_target(state: &GameState, raider: EntityId) -> Option<StorageTarget> {
    let start = state.entity(raider)?.location;
    let occupied = occupied_tiles(state);
    let mut reachable = Vec::new();
    let mut blocked = Vec::new();

    for building in state
        .buildings()
        .filter(|building| building.kind.as_str() == MARKET && building.level > 0)
    {
        let approach = approach_tiles(state, building, false);
        if let Some(path) = shortest_path(state, start, &approach, false) {
            reachable.push((path.len(), building.id, path));
            continue;
        }

        let occupied_approach = approach_tiles(state, building, true);
        let Some(pseudo_path) = shortest_path(state, start, &occupied_approach, true) else {
            continue;
        };
        let Some(blocker_id) = first_blocker_on_path(&occupied, &pseudo_path) else {
            continue;
        };
        let blocker = state.building(blocker_id)?;
        let blocker_path =
            shortest_path(state, start, &approach_tiles(state, blocker, false), false)
                .unwrap_or_else(|| vec![start]);
        blocked.push((
            pseudo_path.len(),
            building.id,
            blocker_id,
            blocker_path,
            pseudo_path,
        ));
    }

    if let Some((_, storage_id, path)) = reachable.into_iter().min_by_key(|candidate| candidate.0) {
        return Some(StorageTarget {
            storage_id,
            path,
            blocker_id: None,
            blocker_path: Vec::new(),
        });
    }

    blocked.into_iter().min_by_key(|candidate| candidate.0).map(
        |(_, storage_id, blocker_id, blocker_path, pseudo_path)| StorageTarget {
            storage_id,
            path: pseudo_path,
            blocker_id: Some(blocker_id),
            blocker_path,
        },
    )
}

fn shortest_path(
    state: &GameState,
    start: MapLocation,
    goals: &BTreeSet<(i32, i32)>,
    allow_occupied_goals: bool,
) -> Option<Vec<MapLocation>> {
    if goals.is_empty() {
        return None;
    }
    let occupied = occupied_tiles(state);
    let mut frontier = VecDeque::from([start]);
    let mut parents = HashMap::from([(key(start), start)]);
    while let Some(current) = frontier.pop_front() {
        if goals.contains(&key(current)) {
            return Some(reconstruct_path(current, &parents));
        }
        for next in state.cardinal_neighbors(current) {
            let next_key = key(next);
            if parents.contains_key(&next_key) || !tile_is_walkable(state, next) {
                continue;
            }
            let blocked = occupied.contains_key(&next_key)
                && !(allow_occupied_goals && goals.contains(&next_key));
            if blocked {
                continue;
            }
            parents.insert(next_key, current);
            frontier.push_back(next);
        }
    }
    None
}

fn move_raider_along(
    state: &mut GameState,
    raider: EntityId,
    path: &[MapLocation],
    events: &mut Vec<GameEvent>,
) -> Result<(), EngineError> {
    if path.len() <= 1 {
        return Ok(());
    }
    let speed = state.entity_stat(raider, RAIDER_SPEED)? as usize;
    let destination = path[speed.min(path.len() - 1)];
    if state
        .entity(raider)
        .is_some_and(|entity| entity.location != destination)
    {
        state.move_entity(raider, destination)?;
        events.push(GameEvent::EntityMoved(raider));
    }
    Ok(())
}

fn initialize_raider(state: &mut GameState, raider: EntityId) -> Result<(), EngineError> {
    if state.entity_stat(raider, RAIDER_SPEED)? > 0 {
        return Ok(());
    }
    let Some(profile) = raider_profile(
        state
            .entity(raider)
            .map(|entity| entity.blueprint == EntityBlueprintRef::Unit(BASIC_RAIDER.into()))
            .unwrap_or(false),
    ) else {
        return Ok(());
    };
    let location = state.entity(raider).expect("raider exists").location;
    for (stat, value) in [
        (RAIDER_STATE, STATE_SPAWNED),
        (RAIDER_SPEED, profile.speed as i64),
        (RAIDER_ATTACK_DAMAGE, profile.attack_damage),
        (RAIDER_CARRY_CAPACITY, profile.carry_capacity as i64),
        (RAIDER_LOOTING_SPEED, profile.looting_speed as i64),
        (RAIDER_ENTRANCE_X, location.x as i64),
        (RAIDER_ENTRANCE_Y, location.y as i64),
    ] {
        state.set_entity_stat(raider, stat, value)?;
    }
    Ok(())
}

fn begin_retreat(state: &mut GameState, raider: EntityId) -> Result<(), EngineError> {
    state.set_entity_stat(raider, RAIDER_STATE, STATE_RETREATING)?;
    state.set_entity_stat(raider, RAIDER_TARGET_STORAGE_ID, 0)?;
    state.set_entity_stat(raider, RAIDER_TARGET_BLOCKER_ID, 0)?;
    Ok(())
}

fn refund_pending_drop(state: &mut GameState, raider: EntityId) -> Result<(), EngineError> {
    for (resource, stat) in [
        (GRAIN, RAIDER_PENDING_GRAIN),
        (TIMBER, RAIDER_PENDING_TIMBER),
        (STONE, RAIDER_PENDING_STONE),
    ] {
        let amount = state.entity_stat(raider, stat)?.max(0) as u64;
        if amount > 0 {
            state.inventory_mut().add(resource, amount)?;
            state.set_entity_stat(raider, stat, 0)?;
        }
    }
    Ok(())
}

fn storage_drop_amounts(
    state: &GameState,
    building: &Building,
) -> Result<BTreeMap<ResourceId, u64>, EngineError> {
    let mut drops = BTreeMap::new();
    for resource in [TIMBER, STONE, GRAIN] {
        let mut capacity = 0_u64;
        for level in 1..=building.level {
            if let Some(definition) = state.catalog().building_level(building.kind.clone(), level) {
                capacity += definition
                    .storage_bonus
                    .iter()
                    .filter(|bonus| bonus.resource.as_str() == resource)
                    .map(|bonus| bonus.amount)
                    .sum::<u64>();
            }
        }
        drops.insert(
            resource.into(),
            state.inventory().amount(resource).min(capacity),
        );
    }
    Ok(drops)
}

fn pending_drop_total(state: &GameState, raider: EntityId) -> Result<u64, EngineError> {
    Ok(
        state.entity_stat(raider, RAIDER_PENDING_GRAIN)?.max(0) as u64
            + state.entity_stat(raider, RAIDER_PENDING_TIMBER)?.max(0) as u64
            + state.entity_stat(raider, RAIDER_PENDING_STONE)?.max(0) as u64,
    )
}

fn building_hit_points(state: &mut GameState, building: BuildingId) -> Result<i64, EngineError> {
    let current = state.building_stat(building, HIT_POINTS)?;
    if current > 0 {
        return Ok(current);
    }
    let max = state.building(building).map_or(60, |entry| {
        30 + i64::from(entry.level.max(1)) * 20
            + i64::from(entry.footprint.width * entry.footprint.depth) * 10
            + if entry.kind.as_str() == MARKET { 20 } else { 0 }
    });
    state.set_building_stat(building, HIT_POINTS, max)?;
    state.set_building_stat(building, MAX_HIT_POINTS, max)?;
    Ok(max)
}

fn raider_profile(is_basic_raider: bool) -> Option<RaiderProfile> {
    is_basic_raider.then_some(RaiderProfile {
        speed: 1,
        attack_damage: 30,
        carry_capacity: 36,
        looting_speed: 12,
    })
}

fn approach_tiles(
    state: &GameState,
    building: &Building,
    include_occupied: bool,
) -> BTreeSet<(i32, i32)> {
    let occupied = occupied_tiles(state);
    let footprint = footprint_tiles(building);
    let footprint_keys = footprint
        .iter()
        .map(|location| key(*location))
        .collect::<BTreeSet<_>>();
    footprint
        .into_iter()
        .flat_map(|tile| state.cardinal_neighbors(tile))
        .filter(|neighbor| !footprint_keys.contains(&key(*neighbor)))
        .filter(|neighbor| include_occupied || !occupied.contains_key(&key(*neighbor)))
        .filter(|neighbor| tile_is_walkable(state, *neighbor))
        .map(key)
        .collect()
}

fn occupied_tiles(state: &GameState) -> HashMap<(i32, i32), BuildingId> {
    state
        .buildings()
        .flat_map(|building| {
            footprint_tiles(building)
                .into_iter()
                .map(move |tile| (key(tile), building.id))
        })
        .collect()
}

fn footprint_tiles(building: &Building) -> Vec<MapLocation> {
    let width = i32::try_from(building.footprint.width).expect("footprint width fits");
    let depth = i32::try_from(building.footprint.depth).expect("footprint depth fits");
    (0..depth)
        .flat_map(|dy| {
            (0..width)
                .map(move |dx| MapLocation::new(building.location.x + dx, building.location.y + dy))
        })
        .collect()
}

fn first_blocker_on_path(
    occupied: &HashMap<(i32, i32), BuildingId>,
    path: &[MapLocation],
) -> Option<BuildingId> {
    path.iter()
        .skip(1)
        .find_map(|step| occupied.get(&key(*step)).copied())
}

fn adjacent_to_building(state: &GameState, location: MapLocation, building: BuildingId) -> bool {
    state.building(building).is_some_and(|entry| {
        footprint_tiles(entry)
            .into_iter()
            .any(|tile| state.cardinal_neighbors(tile).contains(&location))
    })
}

fn reconstruct_path(
    mut current: MapLocation,
    parents: &HashMap<(i32, i32), MapLocation>,
) -> Vec<MapLocation> {
    let mut path = vec![current];
    while let Some(parent) = parents.get(&key(current)).copied() {
        if parent == current {
            break;
        }
        current = parent;
        path.push(current);
    }
    path.reverse();
    path
}

fn tile_is_walkable(state: &GameState, location: MapLocation) -> bool {
    state
        .tile(location)
        .and_then(|tile| state.catalog().tile(tile.kind.clone()))
        .is_none_or(|tile| tile.walkable)
}

fn key(location: MapLocation) -> (i32, i32) {
    (location.x, location.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raider_targets_the_nearest_storage_house() {
        let mut state = new_raid_defense_state_with_seed(1).unwrap();
        let near = state
            .start_construction_at(MARKET, MapLocation::new(12, 12))
            .unwrap();
        let far = state
            .start_construction_at(MARKET, MapLocation::new(19, 12))
            .unwrap();
        state.advance_time(8).unwrap();

        let raider = spawn_raider_at(&mut state, MapLocation::new(10, 12));
        let target = find_nearest_storage_target(&state, raider).expect("nearest storage");

        assert_eq!(target.storage_id, near);
        assert_ne!(target.storage_id, far);
    }

    #[test]
    fn raider_breaks_blocker_when_storage_is_sealed_off() {
        let mut state = new_raid_defense_state_with_seed(1).unwrap();
        state.inventory_mut().add(CROWNS, 100).unwrap();
        let blocker = state
            .start_construction_at(SENATE_HALL, MapLocation::new(12, 12))
            .unwrap();
        let storage = state
            .start_construction_at(MARKET, MapLocation::new(13, 12))
            .unwrap();
        state.advance_time(18).unwrap();
        for water in [
            MapLocation::new(14, 12),
            MapLocation::new(13, 13),
            MapLocation::new(13, 11),
            MapLocation::new(14, 11),
            MapLocation::new(12, 13),
        ] {
            state.set_tile(WATER_TILE, water).unwrap();
        }

        let mut logic = RaidDefenseLogic;
        let raider = spawn_raider_at(&mut state, MapLocation::new(10, 12));
        state.advance_time_with_logic(5, &mut logic).unwrap();

        assert!(state.building(blocker).is_none());
        assert_eq!(
            state.entity_stat(raider, RAIDER_TARGET_STORAGE_ID).unwrap(),
            storage.get() as i64
        );
    }

    #[test]
    fn raider_loots_after_destroying_a_storage_house() {
        let mut state = new_raid_defense_state_with_seed(1).unwrap();
        let storage = state
            .start_construction_at(MARKET, MapLocation::new(12, 12))
            .unwrap();
        state.advance_time(8).unwrap();
        state.inventory_mut().add(GRAIN, 80).unwrap();

        let mut logic = RaidDefenseLogic;
        let raider = spawn_raider_at(&mut state, MapLocation::new(10, 12));
        let before = state.inventory().amount(GRAIN);
        state.advance_time_with_logic(7, &mut logic).unwrap();

        assert!(state.building(storage).is_none());
        assert!(state.entity_stat(raider, RAIDER_CARRIED_TOTAL).unwrap() > 0);
        assert!(state.inventory().amount(GRAIN) < before);
    }

    fn spawn_raider_at(state: &mut GameState, location: MapLocation) -> EntityId {
        match spawn_raider(state, location).unwrap().events.as_slice() {
            [GameEvent::EntityCreated(entity)] => *entity,
            other => panic!("unexpected events: {other:?}"),
        }
    }
}

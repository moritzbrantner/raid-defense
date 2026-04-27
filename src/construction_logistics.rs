use super::*;

const MAX_CONSTRUCTION_WORKERS_PER_SITE: usize = 3;
const CONSTRUCTION_CARRY_CAPACITY: u64 = 20;
const CONSTRUCTION_BUILD_RATE_PER_SECOND: u64 = 4;
const CONSTRUCTION_TRAVEL_SECONDS_PER_TILE: u64 = 2;
const PHASE_IDLE: i64 = 0;
const PHASE_TO_STORAGE: i64 = 1;
const PHASE_TO_SITE: i64 = 2;
const PHASE_BUILDING: i64 = 3;
const STARTED_AT_SECONDS_STAT: &str = "construction_started_at_seconds";
const COMPLETES_AT_SECONDS_STAT: &str = "construction_completes_at_seconds";
const TARGET_BUILDING_STAT: &str = "construction_target_building_id";
const SOURCE_BUILDING_STAT: &str = "construction_source_building_id";
const PHASE_STAT: &str = "construction_phase";
const ETA_SECONDS_STAT: &str = "construction_eta_seconds";

pub(crate) fn initialize_construction_logistics_for_building(
    state: &mut GameState,
    building_id: BuildingId,
) -> Result<(), EngineError> {
    if state.building(building_id).is_some_and(|building| {
        building
            .stats
            .contains_key(&StatId::from(STARTED_AT_SECONDS_STAT))
    }) {
        return Ok(());
    }

    let Some(building) = state.building(building_id).cloned() else {
        return Ok(());
    };
    let BuildingStatus::Constructing {
        job_id,
        completes_at_seconds,
        ..
    } = building.status
    else {
        return Ok(());
    };
    let Some(job) = state.job(job_id).cloned() else {
        return Ok(());
    };
    let started_at_seconds = completes_at_seconds.saturating_sub(job.duration_seconds);

    state.set_building_stat(
        building_id,
        STARTED_AT_SECONDS_STAT,
        i64::try_from(started_at_seconds).unwrap_or(i64::MAX),
    )?;
    state.set_building_stat(
        building_id,
        COMPLETES_AT_SECONDS_STAT,
        i64::try_from(completes_at_seconds).unwrap_or(i64::MAX),
    )?;

    for amount in job
        .reserved_resources
        .iter()
        .filter(|amount| is_haulable_construction_resource(amount.resource.as_str()))
    {
        state.set_building_stat(
            building_id,
            total_material_stat(amount.resource.as_str()),
            i64::try_from(amount.amount).unwrap_or(i64::MAX),
        )?;
        state.set_building_stat(
            building_id,
            delivered_material_stat(amount.resource.as_str()),
            0,
        )?;
    }

    Ok(())
}

pub(crate) fn advance_construction_logistics(
    state: &mut GameState,
    delta_seconds: u64,
) -> Result<(), EngineError> {
    if delta_seconds == 0 {
        return Ok(());
    }

    initialize_visible_construction_sites(state)?;
    let start_seconds = state.now_seconds().saturating_sub(delta_seconds);
    let end_seconds = state.now_seconds();

    for sim_now in (start_seconds + 1)..=end_seconds {
        initialize_visible_construction_sites(state)?;
        reconcile_construction_workers(state, sim_now)?;
        let active_sites = construction_sites_active_at(state, sim_now);
        for building_id in active_sites {
            assign_free_workers_to_site(state, building_id, sim_now)?;
        }
        let workers = busy_construction_workers(state);
        for worker in workers {
            step_construction_worker(state, worker, sim_now)?;
        }
    }

    release_workers_for_inactive_sites(state)?;
    Ok(())
}

fn initialize_visible_construction_sites(state: &mut GameState) -> Result<(), EngineError> {
    let building_ids = state
        .buildings()
        .filter(|building| matches!(building.status, BuildingStatus::Constructing { .. }))
        .map(|building| building.id)
        .collect::<Vec<_>>();
    for building_id in building_ids {
        initialize_construction_logistics_for_building(state, building_id)?;
    }
    Ok(())
}

fn reconcile_construction_workers(state: &mut GameState, sim_now: u64) -> Result<(), EngineError> {
    let workers = busy_construction_workers(state);
    for worker in workers {
        let Some(building_id) = construction_target_building(state, worker) else {
            clear_construction_worker(state, worker)?;
            continue;
        };
        let Some(entity) = state.entity(worker) else {
            continue;
        };
        let is_free = entity.assignment.as_ref().is_none_or(|assignment| {
            assignment.assigned_building.is_none() && assignment.assigned_job.is_none()
        });
        if !is_free || !construction_is_active_at(state, building_id, sim_now) {
            clear_construction_worker(state, worker)?;
        }
    }
    Ok(())
}

fn release_workers_for_inactive_sites(state: &mut GameState) -> Result<(), EngineError> {
    let workers = busy_construction_workers(state);
    for worker in workers {
        let Some(building_id) = construction_target_building(state, worker) else {
            clear_construction_worker(state, worker)?;
            continue;
        };
        if !state
            .building(building_id)
            .is_some_and(|building| matches!(building.status, BuildingStatus::Constructing { .. }))
        {
            clear_construction_worker(state, worker)?;
        }
    }
    Ok(())
}

fn assign_free_workers_to_site(
    state: &mut GameState,
    building_id: BuildingId,
    sim_now: u64,
) -> Result<(), EngineError> {
    if !construction_is_active_at(state, building_id, sim_now) {
        return Ok(());
    }
    if uncommitted_material_total(state, building_id) == 0 {
        return Ok(());
    }

    let Some(source_id) = nearest_storage_source_for_site(state, building_id, sim_now) else {
        return Ok(());
    };
    let Some(site_location) = state
        .building(building_id)
        .map(|building| building.location)
    else {
        return Ok(());
    };

    let assigned = workers_for_site(state, building_id).len();
    if assigned >= MAX_CONSTRUCTION_WORKERS_PER_SITE {
        return Ok(());
    }

    let mut free_workers = state
        .entity_ids_of_blueprint(EntityBlueprintRef::Unit(ENGINEER.into()))
        .into_iter()
        .filter(|entity_id| worker_is_free_for_construction(state, *entity_id))
        .collect::<Vec<_>>();
    free_workers.sort_by_key(|entity_id| {
        state
            .entity(*entity_id)
            .map(|entity| {
                (
                    hex_distance(entity.location, site_location),
                    hex_distance(
                        entity.location,
                        source_location(state, source_id).unwrap_or(site_location),
                    ),
                    entity.id.get(),
                )
            })
            .unwrap_or((i32::MAX, i32::MAX, entity_id.get()))
    });

    for worker in free_workers
        .into_iter()
        .take(MAX_CONSTRUCTION_WORKERS_PER_SITE.saturating_sub(assigned))
    {
        if uncommitted_material_total(state, building_id) == 0 {
            break;
        }
        let Some(worker_location) = state.entity(worker).map(|entity| entity.location) else {
            continue;
        };
        let Some(source_location) = source_location(state, source_id) else {
            break;
        };
        state.set_entity_stat(worker, TARGET_BUILDING_STAT, building_id.get() as i64)?;
        state.set_entity_stat(worker, SOURCE_BUILDING_STAT, source_id.get() as i64)?;
        state.set_entity_stat(worker, PHASE_STAT, PHASE_TO_STORAGE)?;
        state.set_entity_stat(
            worker,
            ETA_SECONDS_STAT,
            i64::try_from(route_duration_seconds(
                state,
                worker_location,
                source_location,
            ))
            .unwrap_or(i64::MAX),
        )?;
    }

    Ok(())
}

fn step_construction_worker(
    state: &mut GameState,
    worker: EntityId,
    sim_now: u64,
) -> Result<(), EngineError> {
    let Some(building_id) = construction_target_building(state, worker) else {
        return Ok(());
    };
    if !construction_is_active_at(state, building_id, sim_now) {
        clear_construction_worker(state, worker)?;
        return Ok(());
    }

    match state.entity_stat(worker, PHASE_STAT)? {
        PHASE_TO_STORAGE => step_worker_to_storage(state, worker, building_id, sim_now)?,
        PHASE_TO_SITE => step_worker_to_site(state, worker, building_id, sim_now)?,
        PHASE_BUILDING => step_worker_building(state, worker, building_id, sim_now)?,
        _ => clear_construction_worker(state, worker)?,
    }

    Ok(())
}

fn step_worker_to_storage(
    state: &mut GameState,
    worker: EntityId,
    building_id: BuildingId,
    sim_now: u64,
) -> Result<(), EngineError> {
    if tick_eta(state, worker)? {
        return Ok(());
    }

    let Some(source_id) = nearest_storage_source_for_site(state, building_id, sim_now) else {
        clear_construction_worker(state, worker)?;
        return Ok(());
    };
    let Some(source_location) = source_location(state, source_id) else {
        clear_construction_worker(state, worker)?;
        return Ok(());
    };
    state.move_entity(worker, source_location)?;
    state.set_entity_stat(worker, SOURCE_BUILDING_STAT, source_id.get() as i64)?;

    let loaded = load_worker_materials(state, worker, building_id)?;
    if loaded == 0 {
        clear_construction_worker(state, worker)?;
        return Ok(());
    }

    let Some(site_location) = state
        .building(building_id)
        .map(|building| building.location)
    else {
        clear_construction_worker(state, worker)?;
        return Ok(());
    };
    state.set_entity_stat(worker, PHASE_STAT, PHASE_TO_SITE)?;
    state.set_entity_stat(
        worker,
        ETA_SECONDS_STAT,
        i64::try_from(route_duration_seconds(
            state,
            source_location,
            site_location,
        ))
        .unwrap_or(i64::MAX),
    )?;
    Ok(())
}

fn step_worker_to_site(
    state: &mut GameState,
    worker: EntityId,
    building_id: BuildingId,
    _sim_now: u64,
) -> Result<(), EngineError> {
    if tick_eta(state, worker)? {
        return Ok(());
    }

    let Some(site_location) = state
        .building(building_id)
        .map(|building| building.location)
    else {
        clear_construction_worker(state, worker)?;
        return Ok(());
    };
    state.move_entity(worker, site_location)?;
    state.set_entity_stat(worker, PHASE_STAT, PHASE_BUILDING)?;
    state.set_entity_stat(worker, ETA_SECONDS_STAT, 0)?;
    Ok(())
}

fn step_worker_building(
    state: &mut GameState,
    worker: EntityId,
    building_id: BuildingId,
    sim_now: u64,
) -> Result<(), EngineError> {
    let mut budget = CONSTRUCTION_BUILD_RATE_PER_SECOND;
    for resource in haulable_resources() {
        if budget == 0 {
            break;
        }
        let stat = carried_material_stat(resource);
        let carried = worker_u64_stat(state, worker, &stat);
        if carried == 0 {
            continue;
        }
        let used = carried.min(budget);
        state.set_entity_stat(
            worker,
            stat,
            i64::try_from(carried.saturating_sub(used)).unwrap_or(0),
        )?;
        let delivered_key = delivered_material_stat(resource);
        let delivered = building_u64_stat(state, building_id, &delivered_key).saturating_add(used);
        state.set_building_stat(
            building_id,
            delivered_key,
            i64::try_from(delivered).unwrap_or(i64::MAX),
        )?;
        budget -= used;
    }

    if carried_material_total(state, worker) > 0 {
        return Ok(());
    }

    if uncommitted_material_total(state, building_id) == 0 {
        clear_construction_worker(state, worker)?;
        return Ok(());
    }

    let Some(site_location) = state
        .building(building_id)
        .map(|building| building.location)
    else {
        clear_construction_worker(state, worker)?;
        return Ok(());
    };
    let Some(source_id) = nearest_storage_source_for_site(state, building_id, sim_now) else {
        clear_construction_worker(state, worker)?;
        return Ok(());
    };
    let Some(source_location) = source_location(state, source_id) else {
        clear_construction_worker(state, worker)?;
        return Ok(());
    };
    state.set_entity_stat(worker, SOURCE_BUILDING_STAT, source_id.get() as i64)?;
    state.set_entity_stat(worker, PHASE_STAT, PHASE_TO_STORAGE)?;
    state.set_entity_stat(
        worker,
        ETA_SECONDS_STAT,
        i64::try_from(route_duration_seconds(
            state,
            site_location,
            source_location,
        ))
        .unwrap_or(i64::MAX),
    )?;
    Ok(())
}

fn tick_eta(state: &mut GameState, worker: EntityId) -> Result<bool, EngineError> {
    let eta = worker_u64_stat(state, worker, ETA_SECONDS_STAT);
    if eta > 1 {
        state.set_entity_stat(
            worker,
            ETA_SECONDS_STAT,
            i64::try_from(eta - 1).unwrap_or(0),
        )?;
        return Ok(true);
    }
    Ok(false)
}

fn load_worker_materials(
    state: &mut GameState,
    worker: EntityId,
    building_id: BuildingId,
) -> Result<u64, EngineError> {
    let mut remaining_capacity = CONSTRUCTION_CARRY_CAPACITY;
    let committed = committed_materials_for_site(state, building_id);
    let mut loaded_total = 0;

    for resource in haulable_resources() {
        if remaining_capacity == 0 {
            break;
        }
        let total = building_u64_stat(state, building_id, &total_material_stat(resource));
        if total == 0 {
            continue;
        }
        let available = total.saturating_sub(committed.get(resource).copied().unwrap_or(0));
        if available == 0 {
            continue;
        }
        let loaded = available.min(remaining_capacity);
        state.set_entity_stat(
            worker,
            carried_material_stat(resource),
            i64::try_from(loaded).unwrap_or(i64::MAX),
        )?;
        loaded_total += loaded;
        remaining_capacity -= loaded;
    }

    Ok(loaded_total)
}

fn committed_materials_for_site(
    state: &GameState,
    building_id: BuildingId,
) -> std::collections::BTreeMap<&'static str, u64> {
    let mut committed = std::collections::BTreeMap::new();
    for resource in haulable_resources() {
        committed.insert(
            resource,
            building_u64_stat(state, building_id, &delivered_material_stat(resource)),
        );
    }
    for worker in workers_for_site(state, building_id) {
        for resource in haulable_resources() {
            *committed.entry(resource).or_insert(0) +=
                worker_u64_stat(state, worker, &carried_material_stat(resource));
        }
    }
    committed
}

fn uncommitted_material_total(state: &GameState, building_id: BuildingId) -> u64 {
    let committed = committed_materials_for_site(state, building_id);
    haulable_resources()
        .into_iter()
        .map(|resource| {
            building_u64_stat(state, building_id, &total_material_stat(resource))
                .saturating_sub(committed.get(resource).copied().unwrap_or(0))
        })
        .sum()
}

fn construction_sites_active_at(state: &GameState, sim_now: u64) -> Vec<BuildingId> {
    state
        .buildings()
        .filter_map(|building| {
            construction_is_active_at(state, building.id, sim_now).then_some(building.id)
        })
        .collect()
}

fn construction_is_active_at(state: &GameState, building_id: BuildingId, sim_now: u64) -> bool {
    let started_at = building_u64_stat(state, building_id, STARTED_AT_SECONDS_STAT);
    let completes_at = building_u64_stat(state, building_id, COMPLETES_AT_SECONDS_STAT);
    if started_at == 0 && completes_at == 0 {
        return false;
    }
    started_at < sim_now && sim_now <= completes_at
}

fn workers_for_site(state: &GameState, building_id: BuildingId) -> Vec<EntityId> {
    busy_construction_workers(state)
        .into_iter()
        .filter(|worker| construction_target_building(state, *worker) == Some(building_id))
        .collect()
}

fn busy_construction_workers(state: &GameState) -> Vec<EntityId> {
    state
        .entity_ids_of_blueprint(EntityBlueprintRef::Unit(ENGINEER.into()))
        .into_iter()
        .filter(|entity_id| construction_target_building(state, *entity_id).is_some())
        .collect()
}

fn worker_is_free_for_construction(state: &GameState, worker: EntityId) -> bool {
    let Some(entity) = state.entity(worker) else {
        return false;
    };
    entity.assignment.as_ref().is_none_or(|assignment| {
        assignment.assigned_building.is_none() && assignment.assigned_job.is_none()
    }) && construction_target_building(state, worker).is_none()
}

fn nearest_storage_source_for_site(
    state: &GameState,
    building_id: BuildingId,
    sim_now: u64,
) -> Option<BuildingId> {
    let site_location = state.building(building_id)?.location;
    let mut candidates = storage_candidates_at(state, sim_now);
    if candidates.is_empty() {
        candidates = capital_fallback_candidates(state);
    }
    candidates.into_iter().min_by_key(|candidate| {
        let location = source_location(state, *candidate).unwrap_or(site_location);
        (hex_distance(site_location, location), candidate.get())
    })
}

fn storage_candidates_at(state: &GameState, sim_now: u64) -> Vec<BuildingId> {
    state
        .buildings()
        .filter(|building| {
            building.kind.as_str() == MARKET && source_available_at(state, building.id, sim_now)
        })
        .map(|building| building.id)
        .collect()
}

fn capital_fallback_candidates(state: &GameState) -> Vec<BuildingId> {
    state
        .buildings()
        .filter(|building| {
            building.kind.as_str() == CAPITAL && building.status == BuildingStatus::Active
        })
        .map(|building| building.id)
        .collect()
}

fn source_available_at(state: &GameState, building_id: BuildingId, sim_now: u64) -> bool {
    let Some(building) = state.building(building_id) else {
        return false;
    };
    if building.status == BuildingStatus::Active {
        let completes_at = building_u64_stat(state, building_id, COMPLETES_AT_SECONDS_STAT);
        return completes_at == 0 || sim_now > completes_at;
    }
    false
}

fn source_location(state: &GameState, building_id: BuildingId) -> Option<MapLocation> {
    state
        .building(building_id)
        .map(|building| building.location)
}

fn construction_target_building(state: &GameState, worker: EntityId) -> Option<BuildingId> {
    Some(BuildingId::new(std::num::NonZeroU64::new(
        worker_u64_stat(state, worker, TARGET_BUILDING_STAT),
    )?))
}

fn clear_construction_worker(state: &mut GameState, worker: EntityId) -> Result<(), EngineError> {
    state.set_entity_stat(worker, TARGET_BUILDING_STAT, 0)?;
    state.set_entity_stat(worker, SOURCE_BUILDING_STAT, 0)?;
    state.set_entity_stat(worker, PHASE_STAT, PHASE_IDLE)?;
    state.set_entity_stat(worker, ETA_SECONDS_STAT, 0)?;
    for resource in haulable_resources() {
        state.set_entity_stat(worker, carried_material_stat(resource), 0)?;
    }
    Ok(())
}

fn carried_material_total(state: &GameState, worker: EntityId) -> u64 {
    haulable_resources()
        .into_iter()
        .map(|resource| worker_u64_stat(state, worker, &carried_material_stat(resource)))
        .sum()
}

fn haulable_resources() -> Vec<&'static str> {
    vec![TIMBER, STONE, IRON, GRAIN]
}

fn is_haulable_construction_resource(resource: &str) -> bool {
    haulable_resources().contains(&resource)
}

fn carried_material_stat(resource: &str) -> String {
    format!("construction_carried:{resource}")
}

fn total_material_stat(resource: &str) -> String {
    format!("construction_total:{resource}")
}

fn delivered_material_stat(resource: &str) -> String {
    format!("construction_delivered:{resource}")
}

fn worker_u64_stat(state: &GameState, worker: EntityId, stat: &str) -> u64 {
    u64::try_from(state.entity_stat(worker, stat).unwrap_or(0).max(0)).unwrap_or(0)
}

fn building_u64_stat(state: &GameState, building_id: BuildingId, stat: &str) -> u64 {
    u64::try_from(state.building_stat(building_id, stat).unwrap_or(0).max(0)).unwrap_or(0)
}

fn route_duration_seconds(state: &GameState, source: MapLocation, target: MapLocation) -> u64 {
    let distance = hex_distance(source, target);
    if distance <= 0 {
        return 0;
    }
    let road_components = road_network_components(state);
    let effective_distance = effective_route_distance(
        distance,
        is_road_backed_route(state, source, target, &road_components),
        state.has_tech_node(ROAD_NETWORKS),
    );
    u64::try_from(effective_distance.max(1)).unwrap_or(1) * CONSTRUCTION_TRAVEL_SECONDS_PER_TILE
}

fn road_network_components(state: &GameState) -> Vec<Vec<MapLocation>> {
    let road_paths = state
        .paths()
        .filter(|path| path.kind.as_str() == IMPERIAL_ROAD)
        .map(|path| path.waypoints.clone())
        .collect::<Vec<_>>();
    let mut parent = (0..road_paths.len()).collect::<Vec<_>>();

    for left in 0..road_paths.len() {
        for right in (left + 1)..road_paths.len() {
            if road_paths_connected(&road_paths[left], &road_paths[right]) {
                union_indexes(&mut parent, left, right);
            }
        }
    }

    let mut groups = std::collections::BTreeMap::<usize, Vec<MapLocation>>::new();
    for (index, waypoints) in road_paths.into_iter().enumerate() {
        groups
            .entry(find_root(&mut parent, index))
            .or_default()
            .extend(waypoints);
    }
    groups.into_values().collect()
}

fn road_paths_connected(left: &[MapLocation], right: &[MapLocation]) -> bool {
    left.iter().any(|source| {
        right
            .iter()
            .any(|target| hex_distance(*source, *target) <= 1)
    })
}

fn union_indexes(parent: &mut [usize], left: usize, right: usize) {
    let left_root = find_root(parent, left);
    let right_root = find_root(parent, right);
    if left_root != right_root {
        parent[right_root] = left_root;
    }
}

fn find_root(parent: &mut [usize], index: usize) -> usize {
    if parent[index] == index {
        return index;
    }
    let root = find_root(parent, parent[index]);
    parent[index] = root;
    root
}

fn is_road_backed_route(
    _state: &GameState,
    source: MapLocation,
    target: MapLocation,
    road_components: &[Vec<MapLocation>],
) -> bool {
    let source_component = nearest_road_component(source, road_components);
    let target_component = nearest_road_component(target, road_components);
    source_component.is_some() && source_component == target_component
}

fn nearest_road_component(
    location: MapLocation,
    road_components: &[Vec<MapLocation>],
) -> Option<usize> {
    road_components.iter().position(|component| {
        component
            .iter()
            .any(|waypoint| hex_distance(location, *waypoint) <= 1)
    })
}

fn effective_route_distance(distance: i32, road_backed: bool, improved_roads: bool) -> i32 {
    const ROAD_DISTANCE_PERCENT: i32 = 70;
    const IMPROVED_ROAD_DISTANCE_PERCENT: i32 = 50;

    if !road_backed {
        return distance.max(0);
    }
    let percent = if improved_roads {
        IMPROVED_ROAD_DISTANCE_PERCENT
    } else {
        ROAD_DISTANCE_PERCENT
    };
    ((distance.max(0) * percent) + 99) / 100
}

fn hex_distance(left: MapLocation, right: MapLocation) -> i32 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    let dz = -dx - dy;
    dx.abs().max(dy.abs()).max(dz.abs())
}

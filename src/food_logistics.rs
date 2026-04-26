use super::*;

const FARM_BUFFER_CAPACITY: u64 = 24;
const FARM_FULL_THRESHOLD_SECONDS: u64 = 60;
const FARM_SPOILAGE_INTERVAL_SECONDS: u64 = 15;
const STORAGE_LEVEL_ONE_RADIUS: i32 = 8;
const STORAGE_LEVEL_TWO_RADIUS: i32 = 12;
const STORAGE_LEVEL_ONE_BATCH_SIZE: u64 = 6;
const STORAGE_LEVEL_TWO_BATCH_SIZE: u64 = 8;
const STORAGE_LEVEL_ONE_ROUTE_SLOTS: u32 = 1;
const STORAGE_LEVEL_TWO_ROUTE_SLOTS: u32 = 2;
const STORAGE_LEVEL_ONE_ENGINEER_BONUS_CAP: u32 = 1;
const STORAGE_LEVEL_TWO_ENGINEER_BONUS_CAP: u32 = 2;
const SURVEYORS_ROUTE_SLOT_BONUS: u32 = 1;
const ROAD_NETWORKS_RADIUS_BONUS: i32 = 2;
const ROAD_DISTANCE_PERCENT: i32 = 50;
const IMPROVED_ROAD_DISTANCE_PERCENT: i32 = 35;
const FOOD_ROUTE_LOAD_SECONDS: u64 = 8;
const FOOD_ROUTE_UNLOAD_SECONDS: u64 = 8;
const FOOD_ROUTE_SECONDS_PER_DISTANCE: u64 = 4;
const MAX_FOOD_ROUTE_SLOTS: usize = 5;
const FOOD_METRIC_BUCKET_SECONDS: u64 = 10;
const FOOD_METRIC_BUCKET_COUNT: usize = 6;
const FOOD_SUPPLY_INTERVAL_SECONDS: u64 = 30;
const FRONTIER_FORT_EMPTY_SUPPLY_LOSS: i64 = 2;
const FRONTIER_FORT_RECOVERY_SUPPLY_GAIN: i64 = 1;
pub(crate) const FRONTIER_FORT_LOW_SUPPLY_THRESHOLD: i64 = 40;
const FOOD_DURATION_PENALTY_PERCENT: i64 = 25;
const EMPTY_SUPPLY_PRESSURE_DELAY_SECONDS: u64 = 60;

const BLOCKED_FULL_SECONDS_STAT: &str = "blocked_full_seconds";
const LAST_SPOILAGE_AT_SECONDS_STAT: &str = "spoilage_last_at_seconds";
const FOOD_EMPTY_SINCE_SECONDS_STAT: &str = "food_empty_since_seconds";
const FOOD_SUPPLY_PENALTY_PROGRESS_SECONDS_STAT: &str = "food_supply_penalty_progress_seconds";
const FOOD_SUPPLY_RECOVERY_PROGRESS_SECONDS_STAT: &str = "food_supply_recovery_progress_seconds";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FoodReserveState {
    Healthy,
    Low,
    Strained,
    Empty,
}

impl FoodReserveState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Low => "low",
            Self::Strained => "strained",
            Self::Empty => "empty",
        }
    }

    pub(crate) fn strains_production(self) -> bool {
        matches!(self, Self::Strained | Self::Empty)
    }

    fn allows_supply_recovery(self) -> bool {
        matches!(self, Self::Healthy | Self::Low)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FoodRoute {
    source_building: BuildingId,
    target_building: BuildingId,
    amount: u64,
    started_at_seconds: u64,
    completes_at_seconds: u64,
    slot_index: usize,
}

pub(crate) fn advance_food_logistics(
    state: &mut GameState,
    delta_seconds: u64,
) -> Result<(), EngineError> {
    complete_food_routes(state)?;
    assign_food_routes(state)?;
    apply_farm_spoilage(state, delta_seconds)?;
    apply_food_reserve_pressure(state, delta_seconds)?;
    Ok(())
}

pub(crate) fn reserve_state(state: &GameState) -> FoodReserveState {
    let amount = state.inventory().amount(GRAIN);
    let Some(capacity) = state.inventory().capacity(GRAIN) else {
        return if amount == 0 {
            FoodReserveState::Empty
        } else {
            FoodReserveState::Healthy
        };
    };
    if amount == 0 {
        return FoodReserveState::Empty;
    }
    if capacity == 0 {
        return FoodReserveState::Healthy;
    }

    let fill_percent = amount.saturating_mul(100) / capacity;
    if fill_percent >= 50 {
        FoodReserveState::Healthy
    } else if fill_percent >= 25 {
        FoodReserveState::Low
    } else {
        FoodReserveState::Strained
    }
}

pub(crate) fn stability_gain_divisor(state: &GameState) -> u64 {
    if reserve_state(state).strains_production() {
        2
    } else {
        1
    }
}

pub(crate) fn food_logistics_view(state: &GameState) -> FoodLogisticsView {
    let blocked_farms = active_farms(state)
        .into_iter()
        .filter(|building| farm_is_blocked(building))
        .count() as u32;
    let strained_storage = active_storage_ids(state)
        .into_iter()
        .filter(|building_id| {
            let active_routes = routes_for_storage(state, *building_id).len() as u32;
            let route_slots = state
                .building(*building_id)
                .map(|building| storage_route_slots(state, building))
                .unwrap_or(0);
            active_routes >= route_slots && route_slots > 0
        })
        .count() as u32;

    FoodLogisticsView {
        delivered_last_minute: sum_metric_in_last_minute(state, MetricKind::Delivered),
        spoiled_last_minute: sum_metric_in_last_minute(state, MetricKind::Spoiled),
        blocked_farms,
        strained_storage,
        reserve_state: reserve_state(state).as_str().to_owned(),
    }
}

pub(crate) fn building_logistics_view(
    state: &GameState,
    building: &Building,
) -> Option<BuildingLogisticsView> {
    match building.kind.as_str() {
        FARMSTEAD => Some(BuildingLogisticsView {
            role: "producer".to_owned(),
            connected_storage_id: connected_storage_for_farm(state, building.id).map(|id| id.get()),
            active_routes: active_outbound_route_count(state, building.id),
            route_slots: 0,
            service_radius: None,
            blocked: farm_is_blocked(building),
            spoiling: farm_recently_spoiled(state, building.id),
        }),
        MARKET => Some(BuildingLogisticsView {
            role: "storage".to_owned(),
            connected_storage_id: None,
            active_routes: routes_for_storage(state, building.id).len() as u32,
            route_slots: storage_route_slots(state, building),
            service_radius: Some(storage_service_radius(state, building) as u32),
            blocked: false,
            spoiling: false,
        }),
        _ => None,
    }
}

fn complete_food_routes(state: &mut GameState) -> Result<(), EngineError> {
    let now = state.now_seconds();
    for storage_id in active_storage_ids(state) {
        for slot_index in 0..MAX_FOOD_ROUTE_SLOTS {
            let Some(route) = route_for_slot(state, storage_id, slot_index) else {
                continue;
            };
            if route.completes_at_seconds > now {
                continue;
            }

            let source_amount = state
                .building(route.source_building)
                .map(|building| building.inventory.amount(GRAIN))
                .unwrap_or(0);
            let deliverable = source_amount.min(route.amount).min(global_food_room(state));
            if deliverable > 0 {
                state.remove_from_building_inventory(route.source_building, GRAIN, deliverable)?;
                state.inventory_mut().add(GRAIN, deliverable)?;
                record_metric(state, MetricKind::Delivered, deliverable);
            }
            clear_route_slot(state, storage_id, slot_index)?;
        }
    }
    Ok(())
}

fn assign_food_routes(state: &mut GameState) -> Result<(), EngineError> {
    let road_components = road_network_components(state);
    let farm_ids = active_farms(state)
        .into_iter()
        .map(|building| building.id)
        .collect::<Vec<_>>();
    let mut reserved_by_farm = farm_ids
        .iter()
        .copied()
        .map(|building_id| {
            (
                building_id,
                active_reserved_food_for_farm(state, building_id),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut remaining_global_room =
        global_food_room(state).saturating_sub(total_active_route_amount(state));

    for storage_id in active_storage_ids(state) {
        let Some(storage) = state.building(storage_id).cloned() else {
            continue;
        };
        let route_slots = storage_route_slots(state, &storage) as usize;
        let batch_size = storage_batch_size(&storage);
        let service_radius = storage_service_radius(state, &storage);

        for slot_index in 0..route_slots.min(MAX_FOOD_ROUTE_SLOTS) {
            if remaining_global_room == 0 || route_for_slot(state, storage_id, slot_index).is_some()
            {
                continue;
            }

            let Some(source_building) = best_farm_for_storage(
                state,
                &storage,
                service_radius,
                &road_components,
                &reserved_by_farm,
            ) else {
                continue;
            };

            let current_amount = state
                .building(source_building)
                .map(|building| building.inventory.amount(GRAIN))
                .unwrap_or(0);
            let reserved = reserved_by_farm.get(&source_building).copied().unwrap_or(0);
            let available = current_amount.saturating_sub(reserved);
            let amount = available.min(batch_size).min(remaining_global_room);
            if amount == 0 {
                continue;
            }

            let distance = hex_distance(
                storage.location,
                state.building(source_building).unwrap().location,
            );
            let effective_distance = effective_route_distance(
                distance,
                is_road_backed_route(
                    state,
                    storage.location,
                    state.building(source_building).unwrap().location,
                    &road_components,
                ),
                state.has_tech_node(ROAD_NETWORKS),
            );
            let duration_seconds = FOOD_ROUTE_LOAD_SECONDS
                .checked_add(FOOD_ROUTE_UNLOAD_SECONDS)
                .and_then(|value| {
                    value.checked_add(
                        FOOD_ROUTE_SECONDS_PER_DISTANCE
                            .saturating_mul(u64::try_from(effective_distance).unwrap_or(0)),
                    )
                })
                .ok_or(EngineError::ArithmeticOverflow)?;
            let completes_at_seconds = state
                .now_seconds()
                .checked_add(duration_seconds)
                .ok_or(EngineError::ArithmeticOverflow)?;

            write_route_slot(
                state,
                storage_id,
                slot_index,
                FoodRoute {
                    source_building,
                    target_building: storage_id,
                    amount,
                    started_at_seconds: state.now_seconds(),
                    completes_at_seconds,
                    slot_index,
                },
            )?;
            reserved_by_farm
                .entry(source_building)
                .and_modify(|reserved_amount| {
                    *reserved_amount = reserved_amount.saturating_add(amount)
                })
                .or_insert(amount);
            remaining_global_room = remaining_global_room.saturating_sub(amount);
        }
    }

    Ok(())
}

fn apply_farm_spoilage(state: &mut GameState, delta_seconds: u64) -> Result<(), EngineError> {
    for farm in active_farms(state) {
        let was_full_for = building_u64_stat(state, farm.id, BLOCKED_FULL_SECONDS_STAT);
        if !farm_is_blocked(&farm) {
            state.set_building_stat(farm.id, BLOCKED_FULL_SECONDS_STAT, 0)?;
            continue;
        }

        let now_full_for = was_full_for.saturating_add(delta_seconds);
        state.set_building_stat(
            farm.id,
            BLOCKED_FULL_SECONDS_STAT,
            i64::try_from(now_full_for).unwrap_or(i64::MAX),
        )?;

        let previous_spoil_steps = spoilage_steps(was_full_for);
        let next_spoil_steps = spoilage_steps(now_full_for);
        let spoil_events = next_spoil_steps.saturating_sub(previous_spoil_steps);
        if spoil_events == 0 {
            continue;
        }

        let available = state
            .building(farm.id)
            .map(|building| building.inventory.amount(GRAIN))
            .unwrap_or(0);
        let spoiled = available.min(spoil_events);
        if spoiled == 0 {
            continue;
        }

        state.remove_from_building_inventory(farm.id, GRAIN, spoiled)?;
        state.set_building_stat(
            farm.id,
            LAST_SPOILAGE_AT_SECONDS_STAT,
            i64::try_from(state.now_seconds()).unwrap_or(i64::MAX),
        )?;
        record_metric(state, MetricKind::Spoiled, spoiled);
    }

    Ok(())
}

fn apply_food_reserve_pressure(
    state: &mut GameState,
    delta_seconds: u64,
) -> Result<(), EngineError> {
    let reserve_state = reserve_state(state);
    let duration_penalty = if reserve_state.strains_production() {
        FOOD_DURATION_PENALTY_PERCENT
    } else {
        0
    };
    state.set_stat(duration_penalty_stat(GRAIN), duration_penalty);

    if reserve_state == FoodReserveState::Empty {
        let previous_now = state.now_seconds().saturating_sub(delta_seconds);
        let empty_since = state.stat(FOOD_EMPTY_SINCE_SECONDS_STAT);
        let empty_since_seconds = if empty_since > 0 {
            u64::try_from(empty_since).unwrap_or(previous_now)
        } else {
            previous_now
        };
        if empty_since <= 0 {
            state.set_stat(
                FOOD_EMPTY_SINCE_SECONDS_STAT,
                i64::try_from(empty_since_seconds).unwrap_or(i64::MAX),
            );
        }
        let previous_empty_for = previous_now.saturating_sub(empty_since_seconds);
        let next_empty_for = state.now_seconds().saturating_sub(empty_since_seconds);
        let penalty_seconds = next_empty_for
            .saturating_sub(EMPTY_SUPPLY_PRESSURE_DELAY_SECONDS)
            .saturating_sub(previous_empty_for.saturating_sub(EMPTY_SUPPLY_PRESSURE_DELAY_SECONDS));
        let accumulated =
            state.stat(FOOD_SUPPLY_PENALTY_PROGRESS_SECONDS_STAT).max(0) as u64 + penalty_seconds;
        let steps = accumulated / FOOD_SUPPLY_INTERVAL_SECONDS;
        let remainder = accumulated % FOOD_SUPPLY_INTERVAL_SECONDS;
        if steps > 0 {
            adjust_frontier_fort_supply(
                state,
                -(FRONTIER_FORT_EMPTY_SUPPLY_LOSS * i64::try_from(steps).unwrap_or(0)),
            )?;
        }
        state.set_stat(
            FOOD_SUPPLY_PENALTY_PROGRESS_SECONDS_STAT,
            i64::try_from(remainder).unwrap_or(i64::MAX),
        );
        state.set_stat(FOOD_SUPPLY_RECOVERY_PROGRESS_SECONDS_STAT, 0);
        return Ok(());
    }

    state.set_stat(FOOD_EMPTY_SINCE_SECONDS_STAT, -1);
    state.set_stat(FOOD_SUPPLY_PENALTY_PROGRESS_SECONDS_STAT, 0);

    if reserve_state.allows_supply_recovery() {
        let accumulated = state
            .stat(FOOD_SUPPLY_RECOVERY_PROGRESS_SECONDS_STAT)
            .max(0) as u64
            + delta_seconds;
        let steps = accumulated / FOOD_SUPPLY_INTERVAL_SECONDS;
        let remainder = accumulated % FOOD_SUPPLY_INTERVAL_SECONDS;
        if steps > 0 {
            adjust_frontier_fort_supply(
                state,
                FRONTIER_FORT_RECOVERY_SUPPLY_GAIN * i64::try_from(steps).unwrap_or(0),
            )?;
        }
        state.set_stat(
            FOOD_SUPPLY_RECOVERY_PROGRESS_SECONDS_STAT,
            i64::try_from(remainder).unwrap_or(i64::MAX),
        );
    } else {
        state.set_stat(FOOD_SUPPLY_RECOVERY_PROGRESS_SECONDS_STAT, 0);
    }

    Ok(())
}

fn adjust_frontier_fort_supply(state: &mut GameState, delta: i64) -> Result<(), EngineError> {
    let building_ids = state
        .buildings()
        .filter(|building| {
            building.kind.as_str() == FRONTIER_FORT && building.status == BuildingStatus::Active
        })
        .map(|building| building.id)
        .collect::<Vec<_>>();
    for building_id in building_ids {
        let current = state.building_stat(building_id, SUPPLY).unwrap_or(50);
        let next = (current + delta).clamp(0, 100);
        state.set_building_stat(building_id, SUPPLY, next)?;
    }
    Ok(())
}

fn active_farms(state: &GameState) -> Vec<Building> {
    state
        .buildings()
        .filter(|building| {
            building.kind.as_str() == FARMSTEAD && building.status == BuildingStatus::Active
        })
        .cloned()
        .collect()
}

fn active_storage_ids(state: &GameState) -> Vec<BuildingId> {
    state
        .buildings()
        .filter(|building| {
            building.kind.as_str() == MARKET && building.status == BuildingStatus::Active
        })
        .map(|building| building.id)
        .collect()
}

fn active_reserved_food_for_farm(state: &GameState, building_id: BuildingId) -> u64 {
    active_food_routes(state)
        .into_iter()
        .filter(|route| route.source_building == building_id)
        .map(|route| route.amount)
        .sum()
}

fn active_outbound_route_count(state: &GameState, building_id: BuildingId) -> u32 {
    active_food_routes(state)
        .into_iter()
        .filter(|route| route.source_building == building_id)
        .count() as u32
}

fn connected_storage_for_farm(state: &GameState, building_id: BuildingId) -> Option<BuildingId> {
    active_food_routes(state)
        .into_iter()
        .filter(|route| route.source_building == building_id)
        .min_by_key(|route| (route.completes_at_seconds, route.target_building.get()))
        .map(|route| route.target_building)
}

fn routes_for_storage(state: &GameState, building_id: BuildingId) -> Vec<FoodRoute> {
    (0..MAX_FOOD_ROUTE_SLOTS)
        .filter_map(|slot_index| route_for_slot(state, building_id, slot_index))
        .collect()
}

fn active_food_routes(state: &GameState) -> Vec<FoodRoute> {
    active_storage_ids(state)
        .into_iter()
        .flat_map(|building_id| routes_for_storage(state, building_id))
        .collect()
}

fn total_active_route_amount(state: &GameState) -> u64 {
    active_food_routes(state)
        .into_iter()
        .map(|route| route.amount)
        .sum()
}

fn storage_service_radius(state: &GameState, building: &Building) -> i32 {
    let base = if building.level >= 2 {
        STORAGE_LEVEL_TWO_RADIUS
    } else {
        STORAGE_LEVEL_ONE_RADIUS
    };
    if state.has_tech_node(ROAD_NETWORKS) {
        base + ROAD_NETWORKS_RADIUS_BONUS
    } else {
        base
    }
}

fn storage_route_slots(state: &GameState, building: &Building) -> u32 {
    let base = if building.level >= 2 {
        STORAGE_LEVEL_TWO_ROUTE_SLOTS
    } else {
        STORAGE_LEVEL_ONE_ROUTE_SLOTS
    };
    let engineer_bonus_cap = if building.level >= 2 {
        STORAGE_LEVEL_TWO_ENGINEER_BONUS_CAP
    } else {
        STORAGE_LEVEL_ONE_ENGINEER_BONUS_CAP
    };
    let engineer_bonus = assigned_engineers(state, building.id).min(engineer_bonus_cap);
    let surveyor_bonus = u32::from(state.has_upgrade(SURVEYORS)) * SURVEYORS_ROUTE_SLOT_BONUS;
    (base + engineer_bonus + surveyor_bonus).min(MAX_FOOD_ROUTE_SLOTS as u32)
}

fn storage_batch_size(building: &Building) -> u64 {
    if building.level >= 2 {
        STORAGE_LEVEL_TWO_BATCH_SIZE
    } else {
        STORAGE_LEVEL_ONE_BATCH_SIZE
    }
}

fn assigned_engineers(state: &GameState, building_id: BuildingId) -> u32 {
    state
        .entity_ids_assigned_to_building(building_id)
        .into_iter()
        .filter(|entity_id| {
            state
                .entity(*entity_id)
                .is_some_and(|entity| entity.blueprint == EntityBlueprintRef::Unit(ENGINEER.into()))
        })
        .count() as u32
}

fn best_farm_for_storage(
    state: &GameState,
    storage: &Building,
    service_radius: i32,
    road_components: &[Vec<MapLocation>],
    reserved_by_farm: &BTreeMap<BuildingId, u64>,
) -> Option<BuildingId> {
    active_farms(state)
        .into_iter()
        .filter_map(|farm| {
            let distance = hex_distance(storage.location, farm.location);
            if distance > service_radius {
                return None;
            }

            let reserved = reserved_by_farm.get(&farm.id).copied().unwrap_or(0);
            let available = farm.inventory.amount(GRAIN).saturating_sub(reserved);
            if available == 0 {
                return None;
            }

            let effective_distance = effective_route_distance(
                distance,
                is_road_backed_route(state, storage.location, farm.location, road_components),
                state.has_tech_node(ROAD_NETWORKS),
            );
            let fill_ratio = if farm_food_capacity(&farm) == 0 {
                0
            } else {
                i64::try_from(
                    farm.inventory.amount(GRAIN).saturating_mul(100) / farm_food_capacity(&farm),
                )
                .unwrap_or(i64::MAX)
            };
            let score = fill_ratio - i64::from(effective_distance) * 8;
            Some((score, effective_distance, farm.id))
        })
        .max_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| right.1.cmp(&left.1))
                .then_with(|| right.2.get().cmp(&left.2.get()))
        })
        .map(|(_, _, building_id)| building_id)
}

fn farm_food_capacity(building: &Building) -> u64 {
    building
        .inventory
        .capacity(GRAIN)
        .unwrap_or(FARM_BUFFER_CAPACITY)
}

fn farm_is_blocked(building: &Building) -> bool {
    building.inventory.amount(GRAIN) >= farm_food_capacity(building)
}

fn farm_recently_spoiled(state: &GameState, building_id: BuildingId) -> bool {
    let last_spoilage = building_u64_stat(state, building_id, LAST_SPOILAGE_AT_SECONDS_STAT);
    last_spoilage > 0
        && state.now_seconds().saturating_sub(last_spoilage)
            < FOOD_METRIC_BUCKET_SECONDS * FOOD_METRIC_BUCKET_COUNT as u64
}

fn spoilage_steps(full_seconds: u64) -> u64 {
    full_seconds.saturating_sub(FARM_FULL_THRESHOLD_SECONDS) / FARM_SPOILAGE_INTERVAL_SECONDS
}

fn global_food_room(state: &GameState) -> u64 {
    state
        .inventory()
        .capacity(GRAIN)
        .map(|capacity| capacity.saturating_sub(state.inventory().amount(GRAIN)))
        .unwrap_or(u64::MAX)
}

fn route_for_slot(
    state: &GameState,
    building_id: BuildingId,
    slot_index: usize,
) -> Option<FoodRoute> {
    let source_building =
        building_u64_stat(state, building_id, &route_stat_name(slot_index, "source"));
    let amount = building_u64_stat(state, building_id, &route_stat_name(slot_index, "amount"));
    let started_at_seconds = building_u64_stat(
        state,
        building_id,
        &route_stat_name(slot_index, "started_at_seconds"),
    );
    let completes_at_seconds = building_u64_stat(
        state,
        building_id,
        &route_stat_name(slot_index, "completes_at_seconds"),
    );
    let source_building = BuildingId::new(std::num::NonZeroU64::new(source_building)?);
    if amount == 0 || completes_at_seconds == 0 {
        return None;
    }
    Some(FoodRoute {
        source_building,
        target_building: building_id,
        amount,
        started_at_seconds,
        completes_at_seconds,
        slot_index,
    })
}

fn write_route_slot(
    state: &mut GameState,
    building_id: BuildingId,
    slot_index: usize,
    route: FoodRoute,
) -> Result<(), EngineError> {
    state.set_building_stat(
        building_id,
        route_stat_name(slot_index, "source"),
        i64::try_from(route.source_building.get()).unwrap_or(i64::MAX),
    )?;
    state.set_building_stat(
        building_id,
        route_stat_name(slot_index, "amount"),
        i64::try_from(route.amount).unwrap_or(i64::MAX),
    )?;
    state.set_building_stat(
        building_id,
        route_stat_name(slot_index, "started_at_seconds"),
        i64::try_from(route.started_at_seconds).unwrap_or(i64::MAX),
    )?;
    state.set_building_stat(
        building_id,
        route_stat_name(slot_index, "completes_at_seconds"),
        i64::try_from(route.completes_at_seconds).unwrap_or(i64::MAX),
    )?;
    Ok(())
}

fn clear_route_slot(
    state: &mut GameState,
    building_id: BuildingId,
    slot_index: usize,
) -> Result<(), EngineError> {
    for field in [
        "source",
        "amount",
        "started_at_seconds",
        "completes_at_seconds",
    ] {
        state.set_building_stat(building_id, route_stat_name(slot_index, field), 0)?;
    }
    Ok(())
}

fn route_stat_name(slot_index: usize, field: &str) -> String {
    format!("food_route_slot_{}_{}", slot_index + 1, field)
}

fn record_metric(state: &mut GameState, kind: MetricKind, amount: u64) {
    if amount == 0 {
        return;
    }

    let bucket_start =
        state.now_seconds() / FOOD_METRIC_BUCKET_SECONDS * FOOD_METRIC_BUCKET_SECONDS;
    let bucket_index =
        ((bucket_start / FOOD_METRIC_BUCKET_SECONDS) as usize) % FOOD_METRIC_BUCKET_COUNT;
    let timestamp_key = metric_bucket_stat_name(bucket_index, "start");
    if state.stat(timestamp_key.clone()) != i64::try_from(bucket_start).unwrap_or(i64::MAX) {
        state.set_stat(
            timestamp_key,
            i64::try_from(bucket_start).unwrap_or(i64::MAX),
        );
        state.set_stat(metric_bucket_stat_name(bucket_index, "delivered"), 0);
        state.set_stat(metric_bucket_stat_name(bucket_index, "spoiled"), 0);
    }

    let metric_key = metric_bucket_stat_name(bucket_index, kind.as_str());
    let next = state.stat(metric_key.clone()).max(0) as u64 + amount;
    state.set_stat(metric_key, i64::try_from(next).unwrap_or(i64::MAX));
}

fn sum_metric_in_last_minute(state: &GameState, kind: MetricKind) -> u64 {
    let now = state.now_seconds();
    (0..FOOD_METRIC_BUCKET_COUNT)
        .filter_map(|bucket_index| {
            let bucket_start = state.stat(metric_bucket_stat_name(bucket_index, "start"));
            let bucket_start = u64::try_from(bucket_start).ok()?;
            if now.saturating_sub(bucket_start)
                >= FOOD_METRIC_BUCKET_SECONDS * FOOD_METRIC_BUCKET_COUNT as u64
            {
                return None;
            }
            u64::try_from(
                state
                    .stat(metric_bucket_stat_name(bucket_index, kind.as_str()))
                    .max(0),
            )
            .ok()
        })
        .sum()
}

fn metric_bucket_stat_name(bucket_index: usize, field: &str) -> String {
    format!("food_metric_bucket_{}_{}", bucket_index + 1, field)
}

fn duration_penalty_stat(resource: &str) -> String {
    format!("production_duration_penalty_percent:{resource}")
}

fn building_u64_stat(state: &GameState, building_id: BuildingId, stat: &str) -> u64 {
    u64::try_from(state.building_stat(building_id, stat).unwrap_or(0).max(0)).unwrap_or(0)
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

    let mut groups = BTreeMap::<usize, Vec<MapLocation>>::new();
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
    state: &GameState,
    source: MapLocation,
    target: MapLocation,
    road_components: &[Vec<MapLocation>],
) -> bool {
    let source_component = nearest_road_component(state, source, road_components);
    let target_component = nearest_road_component(state, target, road_components);
    source_component.is_some() && source_component == target_component
}

fn nearest_road_component(
    _state: &GameState,
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

#[derive(Clone, Copy)]
enum MetricKind {
    Delivered,
    Spoiled,
}

impl MetricKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Spoiled => "spoiled",
        }
    }
}

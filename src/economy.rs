use super::*;

pub(crate) fn recruit_worker_at_castle(
    state: &mut GameState,
    location: MapLocation,
) -> Result<CommandOutcome, RaidDefenseError> {
    let castle_location = active_castle_location(state).ok_or_else(|| {
        RaidDefenseError::InvalidCommand("cannot recruit a worker without an active castle".into())
    })?;
    let location = state.ground_location(location);
    if location != castle_location {
        return Err(RaidDefenseError::InvalidCommand(format!(
            "workers can only be recruited at the castle ({}, {})",
            castle_location.x, castle_location.y
        )));
    }

    let unit = state.spawn_entity(
        EntityBlueprintRef::Unit(ENGINEER.into()),
        None,
        castle_location,
    )?;
    initialize_worker_hunger(state, unit)?;
    record_unit_encounter(state, ENGINEER);
    Ok(CommandOutcome {
        events: vec![GameEvent::EntityCreated(unit)],
    })
}

pub(crate) fn initialize_worker_hunger(
    state: &mut GameState,
    worker: EntityId,
) -> Result<(), EngineError> {
    state.set_entity_stat(worker, WORKER_STOMACH, worker_stomach_capacity(state))
}

pub(crate) fn advance_worker_hunger(
    state: &mut GameState,
    delta_seconds: u64,
) -> Result<(), EngineError> {
    if delta_seconds == 0 {
        return Ok(());
    }

    let workers = state.entity_ids_of_blueprint(EntityBlueprintRef::Unit(ENGINEER.into()));
    for worker in workers {
        ensure_worker_hunger_initialized(state, worker)?;
        let current = worker_stomach(state, worker);
        let capacity = worker_stomach_capacity(state).max(1) as u64;
        let stomach_per_food = worker_stomach_per_food(state).max(1) as u64;
        let hunger_threshold =
            capacity.saturating_mul(worker_hunger_threshold_percent(state).clamp(0, 100) as u64) / 100;
        let drained = delta_seconds.saturating_mul(worker_hunger_drain_per_second(state).max(0) as u64);
        let mut next = current.saturating_sub(drained);

        while next < hunger_threshold && state.inventory().amount(GRAIN) > 0 {
            state.inventory_mut().remove(GRAIN, 1)?;
            next = (next + stomach_per_food).min(capacity);
        }

        state.set_entity_stat(
            worker,
            WORKER_STOMACH,
            i64::try_from(next).unwrap_or(i64::MAX),
        )?;
    }

    Ok(())
}

pub(crate) fn advance_resource_economy(
    state: &mut GameState,
) -> Result<Vec<GameEvent>, EngineError> {
    let building_ids = state
        .buildings()
        .filter(|building| {
            is_automated_resource_building(building.kind.as_str())
                && building.status == BuildingStatus::Active
        })
        .map(|building| building.id)
        .collect::<Vec<_>>();
    let mut events = Vec::new();

    for building_id in building_ids {
        collect_finished_resource_output(state, building_id, &mut events)?;
        restart_manned_resource_building(state, building_id, &mut events)?;
    }

    Ok(events)
}

fn active_castle_location(state: &GameState) -> Option<MapLocation> {
    state
        .buildings()
        .find(|building| {
            building.kind.as_str() == CAPITAL && building.status == BuildingStatus::Active
        })
        .map(|building| building.location)
}

fn ensure_worker_hunger_initialized(
    state: &mut GameState,
    worker: EntityId,
) -> Result<(), EngineError> {
    let initialized = state
        .entity(worker)
        .is_some_and(|entity| entity.stats.contains_key(&StatId::from(WORKER_STOMACH)));
    if initialized {
        return Ok(());
    }
    initialize_worker_hunger(state, worker)
}

fn is_automated_resource_building(kind: &str) -> bool {
    matches!(kind, FARMSTEAD | LUMBER_CAMP | QUARRY)
}

fn worker_stomach(state: &GameState, worker: EntityId) -> u64 {
    u64::try_from(
        state
            .entity_stat(worker, WORKER_STOMACH)
            .unwrap_or(0)
            .max(0),
    )
    .unwrap_or(0)
}

fn worker_stomach_capacity(state: &GameState) -> i64 {
    unit_config_stat(state, ENGINEER, "stomach_capacity").max(1)
}

fn worker_stomach_per_food(state: &GameState) -> i64 {
    unit_config_stat(state, ENGINEER, "stomach_per_food").max(1)
}

fn worker_hunger_threshold_percent(state: &GameState) -> i64 {
    unit_config_stat(state, ENGINEER, "hunger_threshold_percent")
}

fn worker_hunger_drain_per_second(state: &GameState) -> i64 {
    unit_config_stat(state, ENGINEER, "hunger_drain_per_second")
}

fn collect_finished_resource_output(
    state: &mut GameState,
    building_id: BuildingId,
    events: &mut Vec<GameEvent>,
) -> Result<(), EngineError> {
    let Some(building) = state.building(building_id) else {
        return Ok(());
    };
    let is_ready = matches!(
        building.production_status,
        ProductionStatus::InProgress {
            completes_at_seconds,
        } if completes_at_seconds <= state.now_seconds()
    );
    if !is_ready {
        return Ok(());
    }

    let is_farmstead = state
        .building(building_id)
        .is_some_and(|building| building.kind.as_str() == FARMSTEAD);
    let result = if is_farmstead {
        state.collect_production_to_building_inventory(building_id)
    } else {
        state.collect_production(building_id)
    };

    match result {
        Ok(()) => events.push(GameEvent::ProductionCollected {
            building: building_id,
        }),
        Err(EngineError::Resource(ResourceError::CapacityExceeded { .. })) => {}
        Err(error) => return Err(error),
    }

    Ok(())
}

fn restart_manned_resource_building(
    state: &mut GameState,
    building_id: BuildingId,
    events: &mut Vec<GameEvent>,
) -> Result<(), EngineError> {
    if state.building(building_id).is_some_and(|building| {
        building.kind.as_str() == FARMSTEAD
            && building
                .inventory
                .capacity(GRAIN)
                .is_some_and(|capacity| building.inventory.amount(GRAIN) >= capacity)
    }) {
        return Ok(());
    }

    if !state
        .building(building_id)
        .is_some_and(|building| building.production_status == ProductionStatus::Idle)
    {
        return Ok(());
    }

    match state.start_production(building_id) {
        Ok(()) => events.push(GameEvent::ProductionStarted {
            building: building_id,
            rule: None,
        }),
        Err(EngineError::InsufficientWorkers { .. }) => {}
        Err(error) => return Err(error),
    }

    Ok(())
}

use super::*;

#[derive(Default)]
pub struct RaidDefenseLogic;

impl GameLogic for RaidDefenseLogic {
    fn after_advance_time(
        &mut self,
        state: &mut GameState,
        delta_seconds: u64,
        _completions: &[JobCompletion],
    ) -> Result<Vec<GameEvent>, EngineError> {
        advance_construction_logistics(state, delta_seconds)?;
        let mut events = advance_raiders(state, delta_seconds)?;
        events.extend(advance_resource_economy(state)?);
        advance_food_logistics(state, delta_seconds)?;
        advance_worker_hunger(state, delta_seconds)?;
        update_building_stats(state)?;
        update_provinces_and_revenue(state, delta_seconds)?;
        update_progression(state)?;
        events.push(GameEvent::DomainEvent {
            kind: "raid_defense.logic_tick".to_owned(),
        });
        Ok(events)
    }
}

pub fn apply_raid_defense_command(
    state: &mut GameState,
    command: impl Into<RaidDefenseCommand>,
) -> Result<CommandOutcome, RaidDefenseError> {
    match command.into() {
        RaidDefenseCommand::Engine(command) => apply_raid_defense_engine_command(state, command),
        RaidDefenseCommand::SetTaxRate { building, value } => state.transact(|state| {
            let value = set_tax_rate(state, building, value)?;
            Ok(CommandOutcome {
                events: vec![GameEvent::BuildingStatChanged {
                    building,
                    stat: TAX_RATE.into(),
                    value,
                }],
            })
        }),
        RaidDefenseCommand::ClaimProvince {
            kind,
            name,
            location,
        } => state.transact(|state| {
            let province = match name {
                Some(name) => claim_named_province(state, kind, name, location)?,
                None => claim_province(state, kind, location)?,
            };
            Ok(CommandOutcome {
                events: vec![GameEvent::EntityCreated(province)],
            })
        }),
        RaidDefenseCommand::RecruitEngineer { location } => {
            state.transact(|state| recruit_worker_at_castle(state, location))
        }
    }
}

fn apply_raid_defense_engine_command(
    state: &mut GameState,
    command: GameCommand,
) -> Result<CommandOutcome, RaidDefenseError> {
    match command {
        GameCommand::SetBuildingStat {
            building,
            stat,
            value,
        } if stat.as_str() == TAX_RATE => {
            apply_raid_defense_command(state, RaidDefenseCommand::SetTaxRate { building, value })
        }
        GameCommand::SpawnEntity {
            blueprint: EntityBlueprintRef::Npc(kind),
            name,
            location,
        } if kind.as_str() == PROVINCE => apply_raid_defense_command(
            state,
            RaidDefenseCommand::ClaimProvince {
                kind,
                name,
                location,
            },
        ),
        GameCommand::SpawnEntity {
            blueprint: EntityBlueprintRef::Unit(kind),
            location,
            ..
        } if kind.as_str() == BASIC_RAIDER => spawn_raider(state, location),
        GameCommand::SpawnEntity {
            blueprint: EntityBlueprintRef::Unit(kind),
            location,
            ..
        } if kind.as_str() == ENGINEER => {
            apply_raid_defense_command(state, RaidDefenseCommand::RecruitEngineer { location })
        }
        GameCommand::SpawnEntity {
            blueprint: EntityBlueprintRef::Unit(kind),
            name,
            location,
        } => state.transact(|state| {
            let unit = state.spawn_entity(EntityBlueprintRef::Unit(kind.clone()), name, location)?;
            record_unit_encounter(state, kind.as_str());
            Ok(CommandOutcome {
                events: vec![GameEvent::EntityCreated(unit)],
            })
        }),
        GameCommand::ConstructBuilding { kind, location } => state.transact(|state| {
            let building = state.start_construction_at(kind, location)?;
            initialize_construction_logistics_for_building(state, building)?;
            let job = match state
                .building(building)
                .ok_or(EngineError::UnknownBuilding(building))?
                .status
            {
                BuildingStatus::Constructing { job_id, .. } => job_id,
                _ => unreachable!("newly started construction must still be under construction"),
            };
            Ok(CommandOutcome {
                events: vec![GameEvent::BuildingConstructionStarted { building, job }],
            })
        }),
        command => state.apply(command).map_err(RaidDefenseError::from),
    }
}

pub fn claim_province(
    state: &mut GameState,
    kind: impl Into<NpcKind>,
    location: MapLocation,
) -> Result<EntityId, RaidDefenseError> {
    let kind = kind.into();
    let name = state
        .catalog()
        .npc(kind.clone())
        .ok_or_else(|| EngineError::UnknownEntityBlueprint(EntityBlueprintRef::Npc(kind.clone())))?
        .name
        .clone();
    claim_named_province(state, kind, name, location)
}

pub fn claim_named_province(
    state: &mut GameState,
    kind: impl Into<NpcKind>,
    name: impl Into<String>,
    location: MapLocation,
) -> Result<EntityId, RaidDefenseError> {
    let kind = kind.into();
    let outpost = claim_outpost_at_location(state, location).ok_or_else(|| {
        ProvinceClaimError::NoOutpostAtLocation {
            province_kind: kind.to_string(),
            location,
        }
    })?;
    validate_province_claim(state, &kind, outpost)?;
    let requirements =
        province_claim_requirements(kind.as_str()).expect("validated province kind has rules");

    state
        .inventory_mut()
        .spend(&requirements.claim_cost)
        .map_err(EngineError::from)?;
    let province =
        state.spawn_entity(EntityBlueprintRef::Npc(kind), Some(name.into()), location)?;
    state.set_entity_stat(province, CONTROL, 48)?;
    state.set_entity_stat(province, LOYALTY, 55)?;
    state.set_entity_stat(province, THREAT, 32)?;
    state.set_entity_stat(province, PROSPERITY, 30)?;
    state.set_entity_stat(province, OUTPOST_ID, outpost.get() as i64)?;
    Ok(province)
}

pub fn province_claim_requirements(kind: &str) -> Option<ProvinceClaimRequirements> {
    if kind != PROVINCE {
        return None;
    }
    Some(ProvinceClaimRequirements {
        outpost_kinds: vec![FRONTIER_FORT, EMBASSY],
        min_level: 1,
        min_security: 35,
        claim_cost: vec![
            ResourceAmount::new(CROWNS, 40),
            ResourceAmount::new(INFLUENCE, 18),
            ResourceAmount::new(LEGION_STRENGTH, 8),
        ],
    })
}

pub fn set_tax_rate(
    state: &mut GameState,
    building: BuildingId,
    value: i64,
) -> Result<i64, EngineError> {
    let clamped = value.clamp(0, MAX_TAX_RATE);
    state.set_building_stat(building, TAX_RATE, clamped)?;
    Ok(clamped)
}

fn update_building_stats(state: &mut GameState) -> Result<(), EngineError> {
    let legates = assigned_unit_count(state, LEGATE, None);
    let prefects = assigned_unit_count(state, PREFECT, None);
    let building_ids = state
        .buildings()
        .map(|building| building.id)
        .collect::<Vec<_>>();

    for building in building_ids {
        let kind = state
            .building(building)
            .ok_or(EngineError::UnknownBuilding(building))?
            .kind
            .to_string();
        let level = state
            .building(building)
            .ok_or(EngineError::UnknownBuilding(building))?
            .level;
        let assigned_legates = assigned_unit_count(state, LEGATE, Some(building));
        let assigned_prefects = assigned_unit_count(state, PREFECT, Some(building));
        let supply = state.building_stat(building, SUPPLY).unwrap_or(50);
        let security = match kind.as_str() {
            CAPITAL => 55 + i64::from(prefects.min(3)) * 8,
            BARRACKS => 58 + i64::from(assigned_legates.min(2)) * 10,
            WATCHTOWER => 45 + i64::from(level) * 10 + i64::from(legates.min(3)) * 3,
            FRONTIER_FORT => {
                let low_supply_penalty = if supply < FRONTIER_FORT_LOW_SUPPLY_THRESHOLD {
                    10
                } else {
                    0
                };
                48 + i64::from(level) * 12 + i64::from(assigned_legates.min(2)) * 12
                    - low_supply_penalty
            }
            EMBASSY => 35 + i64::from(assigned_prefects.min(1)) * 8,
            _ => 25,
        }
        .clamp(0, 100);
        state.set_building_stat(building, SECURITY, security)?;
        if state
            .building(building)
            .is_some_and(|building| !building.stats.contains_key(&StatId::from(SUPPLY)))
        {
            state.set_building_stat(building, SUPPLY, 50)?;
        }
    }
    Ok(())
}

fn update_provinces_and_revenue(
    state: &mut GameState,
    delta_seconds: u64,
) -> Result<(), EngineError> {
    if delta_seconds == 0 {
        return Ok(());
    }

    let province_ids = state
        .entities()
        .filter(|entity| entity.kind() == PROVINCE)
        .map(|entity| entity.id)
        .collect::<Vec<_>>();
    let ticks = i64::try_from(delta_seconds / 10).unwrap_or(i64::MAX).max(1);
    let legions = i64::try_from(state.inventory().amount(LEGION_STRENGTH)).unwrap_or(i64::MAX);
    let tax_rate = empire_tax_rate(state);
    let envoys = i64::from(assigned_unit_count(state, ENVOY, None));
    let prefects = i64::from(assigned_unit_count(state, PREFECT, None));
    let mut stable_provinces = 0_u64;

    for province in province_ids {
        let outpost = outpost_for_province(state, province);
        let security = outpost
            .and_then(|building| state.building_stat(building, SECURITY).ok())
            .unwrap_or(25);
        let control = state.entity_stat(province, CONTROL)?;
        let loyalty = state.entity_stat(province, LOYALTY)?;
        let threat = state.entity_stat(province, THREAT)?;
        let prosperity = state.entity_stat(province, PROSPERITY)?;

        let military_pressure = (legions / 12).min(8);
        let tax_pressure = (tax_rate - DEFAULT_TAX_RATE).max(0) / 3;
        let next_threat = (threat + ticks - security / 18 - military_pressure).clamp(0, 100);
        let next_control = (control + security / 12 + prefects + military_pressure
            - next_threat / 10)
            .clamp(0, 100);
        let next_loyalty =
            (loyalty + envoys + security / 20 - tax_pressure - next_threat / 14).clamp(0, 100);
        let next_prosperity =
            (prosperity + next_control / 25 + next_loyalty / 30 - tax_pressure / 2).clamp(0, 100);

        state.set_entity_stat(province, CONTROL, next_control)?;
        state.set_entity_stat(province, LOYALTY, next_loyalty)?;
        state.set_entity_stat(province, THREAT, next_threat)?;
        state.set_entity_stat(province, PROSPERITY, next_prosperity)?;

        if next_control >= 55 && next_loyalty >= 45 {
            stable_provinces += 1;
            let revenue = u64::try_from((next_prosperity / 10).max(1)).unwrap_or(1)
                * u64::try_from(ticks).unwrap_or(1);
            add_capped(
                state.inventory_mut(),
                CROWNS,
                revenue * (1 + tax_rate as u64 / 10),
            )?;
            add_capped(state.inventory_mut(), TRADE_GOODS, revenue / 2 + 1)?;
            add_capped(state.inventory_mut(), INFLUENCE, 1)?;
        }
    }

    if stable_provinces > 0 {
        let stability_divisor = stability_gain_divisor(state);
        add_capped(
            state.inventory_mut(),
            STABILITY,
            stable_provinces / stability_divisor,
        )?;
    }

    Ok(())
}

fn update_progression(state: &mut GameState) -> Result<(), EngineError> {
    let province_count = state
        .entities()
        .filter(|entity| entity.kind() == PROVINCE)
        .count() as u64;
    let xp = state
        .inventory()
        .amount(INFLUENCE)
        .saturating_mul(6)
        .saturating_add(state.inventory().amount(STABILITY).saturating_mul(3))
        .saturating_add(province_count.saturating_mul(80))
        .saturating_add(state.inventory().amount(CROWNS) / 8);
    if xp > state.player_xp() {
        state.grant_xp(xp - state.player_xp())?;
    }
    Ok(())
}

fn add_capped(
    inventory: &mut ResourceStorage,
    resource: impl Into<ResourceId>,
    amount: u64,
) -> Result<(), EngineError> {
    let resource = resource.into();
    if amount == 0 {
        return Ok(());
    }
    let current = inventory.amount(resource.clone());
    let available = inventory
        .capacity(resource.clone())
        .map(|capacity| capacity.saturating_sub(current))
        .unwrap_or(amount);
    let to_add = amount.min(available);
    if to_add > 0 {
        inventory.add(resource, to_add)?;
    }
    Ok(())
}

fn validate_province_claim(
    state: &GameState,
    kind: &NpcKind,
    outpost: BuildingId,
) -> Result<(), ProvinceClaimError> {
    let requirements = province_claim_requirements(kind.as_str())
        .ok_or_else(|| ProvinceClaimError::UnknownProvinceKind(kind.to_string()))?;
    let building =
        state
            .building(outpost)
            .ok_or_else(|| ProvinceClaimError::OutpostUnavailable {
                province_kind: kind.to_string(),
                outpost,
            })?;

    let security = state.building_stat(outpost, SECURITY).unwrap_or(0);
    if !requirements
        .outpost_kinds
        .iter()
        .any(|candidate| *candidate == building.kind.as_str())
        || building.status != BuildingStatus::Active
        || building.level < requirements.min_level
        || security < requirements.min_security
    {
        return Err(ProvinceClaimError::RequirementsNotMet {
            province_kind: kind.to_string(),
            outpost,
            requirements,
        });
    }

    let outpost_stat = StatId::from(OUTPOST_ID);
    if state.entities().any(|entity| {
        entity.kind() == PROVINCE
            && entity.stats.get(&outpost_stat).copied() == Some(outpost.get() as i64)
    }) {
        return Err(ProvinceClaimError::AlreadyClaimed { outpost });
    }

    Ok(())
}

fn claim_outpost_at_location(state: &GameState, location: MapLocation) -> Option<BuildingId> {
    state
        .buildings()
        .find(|building| {
            matches!(building.kind.as_str(), FRONTIER_FORT | EMBASSY)
                && building.location == state.ground_location(location)
        })
        .map(|building| building.id)
}

fn outpost_for_province(state: &GameState, province: EntityId) -> Option<BuildingId> {
    let outpost = state.entity_stat(province, OUTPOST_ID).ok()?;
    let outpost = u64::try_from(outpost).ok()?;
    state
        .buildings()
        .find(|building| building.id.get() == outpost)
        .map(|building| building.id)
}

pub(crate) fn empire_tax_rate(state: &GameState) -> i64 {
    state
        .buildings()
        .find(|building| building.kind.as_str() == CAPITAL)
        .and_then(|building| building.stats.get(&StatId::from(TAX_RATE)).copied())
        .unwrap_or(DEFAULT_TAX_RATE)
        .clamp(0, MAX_TAX_RATE)
}

fn assigned_unit_count(state: &GameState, kind: &str, building: Option<BuildingId>) -> u32 {
    state
        .entity_ids_of_blueprint(EntityBlueprintRef::Unit(kind.into()))
        .into_iter()
        .filter(|entity| {
            building.is_none_or(|building| {
                state
                    .entity(*entity)
                    .and_then(|entity| entity.assignment)
                    .is_some_and(|assignment| assignment.assigned_building == Some(building))
            })
        })
        .count() as u32
}

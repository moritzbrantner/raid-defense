use super::*;

pub fn raid_defense_view(state: &GameState) -> RaidDefenseView {
    let resources = resource_ids()
        .into_iter()
        .map(|id| {
            resource_view(
                id,
                state.inventory().amount(id),
                state.inventory().capacity(id),
            )
        })
        .collect::<Vec<_>>();
    let buildings = state
        .buildings()
        .map(|building| building_view(state, building))
        .collect::<Vec<_>>();
    let jobs = state.jobs().map(job_view).collect::<Vec<_>>();
    let paths = state
        .paths()
        .map(|path| PathView {
            id: path.id.get(),
            kind: path.kind.to_string(),
            waypoints: path.waypoints.clone(),
        })
        .collect();
    let areas = state
        .areas()
        .map(|area| AreaView {
            id: area.id.get(),
            kind: area.kind.to_string(),
            tiles: area.tiles.clone(),
        })
        .collect();
    let entities = state.entities().map(entity_view).collect::<Vec<_>>();
    let provinces = entities
        .iter()
        .filter(|entity| entity.kind == PROVINCE)
        .map(province_view)
        .collect::<Vec<_>>();
    let encountered_units = encountered_unit_views(state);
    let encountered_attack_waves = attack_wave_views(state);
    let food_logistics = food_logistics_view(state);
    let summary = raid_defense_summary(state, &provinces);
    let alerts = raid_defense_alerts(state, &summary, &food_logistics);
    let objectives = raid_defense_objectives(state, &summary);

    RaidDefenseView {
        now_seconds: state.now_seconds(),
        resources,
        food_logistics,
        buildings,
        jobs,
        paths,
        areas,
        entities,
        tech_nodes: state
            .tech_nodes()
            .map(|node| node.kind.to_string())
            .collect::<Vec<_>>(),
        available_tech_nodes: state
            .available_tech_nodes()
            .into_iter()
            .map(|node| node.kind.to_string())
            .collect::<Vec<_>>(),
        upgrades: state
            .upgrades()
            .map(|upgrade| upgrade.kind.to_string())
            .collect::<Vec<_>>(),
        encountered_units,
        encountered_attack_waves,
        alerts,
        objectives,
        summary,
    }
}

fn building_view(state: &GameState, building: &Building) -> BuildingView {
    let required_workers = state
        .catalog()
        .building_level(building.kind.clone(), building.level)
        .map(|level| {
            let production_workers = level
                .production_rules
                .values()
                .map(|rule| rule.required_workers)
                .max()
                .unwrap_or(0);
            level.required_workers.max(production_workers)
        })
        .unwrap_or(0);
    let assigned_workers = state.entity_ids_assigned_to_building(building.id).len() as u32;

    BuildingView {
        id: building.id.get(),
        kind: building.kind.to_string(),
        label: label_for(building.kind.as_str()).to_owned(),
        location: building.location,
        height: building.height,
        footprint: building.footprint,
        level: building.level,
        required_workers,
        assigned_workers,
        manned: assigned_workers >= required_workers,
        status: building_status_label(&building.status),
        production: production_status_label(&building.production_status),
        inventory: resource_ids()
            .into_iter()
            .filter_map(|id| {
                let amount = building.inventory.amount(id);
                let capacity = building.inventory.capacity(id);
                (amount > 0 || capacity.is_some()).then(|| resource_view(id, amount, capacity))
            })
            .collect(),
        logistics: building_logistics_view(state, building),
        stats: stringify_stats(&building.stats),
    }
}

fn job_view(job: &Job) -> JobView {
    JobView {
        id: job.id.get(),
        kind: format!("{:?}", job.kind),
        completes_at_seconds: job.completes_at_seconds,
        assigned_entities: job.assigned_entities.iter().map(|id| id.get()).collect(),
    }
}

fn entity_view(entity: EntityRecord) -> EntityView {
    EntityView {
        id: entity.id.get(),
        blueprint: entity.blueprint.clone(),
        kind: entity.blueprint.kind().to_owned(),
        label: entity
            .name
            .unwrap_or_else(|| label_for(entity.blueprint.kind()).to_owned()),
        location: entity.location,
        assigned_building: entity
            .assignment
            .as_ref()
            .and_then(|assignment| assignment.assigned_building.map(|id| id.get())),
        assigned_job: entity
            .assignment
            .as_ref()
            .and_then(|assignment| assignment.assigned_job.map(|id| id.get())),
        stats: stringify_stats(&entity.stats),
    }
}

fn province_view(entity: &EntityView) -> ProvinceView {
    ProvinceView {
        id: entity.id,
        name: entity.label.clone(),
        location: entity.location,
        control: stat_from_entity(entity, CONTROL),
        loyalty: stat_from_entity(entity, LOYALTY),
        threat: stat_from_entity(entity, THREAT),
        prosperity: stat_from_entity(entity, PROSPERITY),
        stats: entity.stats.clone(),
    }
}

fn encountered_unit_views(state: &GameState) -> Vec<EncounteredUnitView> {
    let mut encountered = encountered_units(state)
        .into_iter()
        .collect::<BTreeMap<_, _>>();

    for entity in state.entities() {
        if let EntityBlueprintRef::Unit(kind) = &entity.blueprint {
            encountered.entry(kind.to_string()).or_insert(0);
        }
    }

    let current_counts = state
        .entities()
        .filter_map(|entity| match &entity.blueprint {
            EntityBlueprintRef::Unit(kind) => Some(kind.to_string()),
            EntityBlueprintRef::Npc(_) => None,
        })
        .fold(BTreeMap::new(), |mut counts, kind| {
            *counts.entry(kind).or_insert(0) += 1;
            counts
        });

    let mut units = encountered
        .into_iter()
        .map(|(kind, encountered_at_seconds)| EncounteredUnitView {
            label: label_for(kind.as_str()).to_owned(),
            current_count: current_counts.get(&kind).copied().unwrap_or(0),
            encountered_at_seconds,
            kind,
        })
        .collect::<Vec<_>>();
    units.sort_by(|left, right| {
        left.encountered_at_seconds
            .cmp(&right.encountered_at_seconds)
            .then_with(|| left.label.cmp(&right.label))
    });
    units
}

fn attack_wave_views(state: &GameState) -> Vec<AttackWaveView> {
    let mut waves = encountered_attack_waves(state)
        .into_iter()
        .map(|wave| {
            let total_units = wave.unit_counts.values().copied().sum::<u32>();
            let units = wave
                .unit_counts
                .into_iter()
                .map(|(kind, count)| AttackWaveUnitView {
                    label: label_for(kind.as_str()).to_owned(),
                    kind,
                    count,
                })
                .collect::<Vec<_>>();
            AttackWaveView {
                id: wave.id,
                label: attack_wave_label(wave.id, total_units),
                encountered_at_seconds: wave.encountered_at_seconds,
                entry: wave.entry,
                units,
            }
        })
        .collect::<Vec<_>>();
    waves.sort_by(|left, right| {
        left.encountered_at_seconds
            .cmp(&right.encountered_at_seconds)
            .then_with(|| left.id.cmp(&right.id))
    });
    waves
}

fn resource_view(id: &str, amount: u64, capacity: Option<u64>) -> ResourceView {
    ResourceView {
        id: id.to_owned(),
        label: label_for(id).to_owned(),
        amount,
        capacity,
    }
}

fn raid_defense_summary(state: &GameState, provinces: &[ProvinceView]) -> RaidDefenseSummary {
    let province_count = provinces.len() as u32;
    let average_control = average_stat(provinces.iter().map(|province| province.control));
    let average_loyalty = average_stat(provinces.iter().map(|province| province.loyalty));
    let total_threat = provinces
        .iter()
        .map(|province| province.threat)
        .sum::<i64>();
    let active_legions = state.inventory().amount(LEGION_STRENGTH);
    let influence_rank = state.player_level();
    let lost = !state.buildings().any(|building| building.kind.as_str() == CAPITAL);
    let won = province_count >= 4
        && average_control >= 70
        && average_loyalty >= 60
        && state.inventory().amount(INFLUENCE) >= 100
        && influence_rank >= 3
        && !lost;
    let critical = lost
        || state.inventory().amount(CROWNS) < 25
        || state.inventory().amount(STABILITY) < 20
        || provinces.iter().any(|province| province.control < 25);

    RaidDefenseSummary {
        province_count,
        average_control,
        average_loyalty,
        total_threat,
        tax_rate: empire_tax_rate(state),
        active_legions,
        influence_rank,
        won,
        lost,
        critical,
    }
}

fn raid_defense_alerts(
    state: &GameState,
    summary: &RaidDefenseSummary,
    food_logistics: &FoodLogisticsView,
) -> Vec<AlertView> {
    let mut alerts = Vec::new();
    if summary.lost {
        alerts.push(AlertView {
            severity: "critical".to_owned(),
            message: "The castle has fallen. The frontier is lost.".to_owned(),
        });
    }
    if state.inventory().amount(CROWNS) < 40 {
        alerts.push(AlertView {
            severity: "warning".to_owned(),
            message: "Treasury reserve is low.".to_owned(),
        });
    }
    if state.inventory().amount(LEGION_STRENGTH) < 12 {
        alerts.push(AlertView {
            severity: "warning".to_owned(),
            message: "Legion strength is thin for frontier claims.".to_owned(),
        });
    }
    if summary.province_count == 0 {
        alerts.push(AlertView {
            severity: "info".to_owned(),
            message: "Build a frontier fort and claim the first province.".to_owned(),
        });
    }
    if summary.average_control > 0 && summary.average_control < 40 {
        alerts.push(AlertView {
            severity: "critical".to_owned(),
            message: "Provincial control is slipping.".to_owned(),
        });
    }
    if food_logistics.reserve_state == "low" {
        alerts.push(AlertView {
            severity: "warning".to_owned(),
            message: "Food reserves are running low.".to_owned(),
        });
    }
    if food_logistics.reserve_state == "strained" {
        alerts.push(AlertView {
            severity: "warning".to_owned(),
            message: "Food logistics are strained and frontier work is slowing.".to_owned(),
        });
    }
    if food_logistics.spoiled_last_minute > 0 {
        alerts.push(AlertView {
            severity: "warning".to_owned(),
            message: "Food is spoiling in overloaded farm buffers.".to_owned(),
        });
    }
    alerts
}

fn raid_defense_objectives(state: &GameState, summary: &RaidDefenseSummary) -> Vec<ObjectiveView> {
    vec![
        objective(
            "province_count",
            "Claim 4 provinces",
            i64::from(summary.province_count),
            4,
        ),
        objective("control", "Average control 70", summary.average_control, 70),
        objective("loyalty", "Average loyalty 60", summary.average_loyalty, 60),
        objective(
            "influence",
            "Bank 100 influence",
            i64::try_from(state.inventory().amount(INFLUENCE)).unwrap_or(i64::MAX),
            100,
        ),
    ]
}

fn objective(id: &str, label: &str, current: i64, target: i64) -> ObjectiveView {
    ObjectiveView {
        id: id.to_owned(),
        label: label.to_owned(),
        current,
        target,
        complete: current >= target,
    }
}

fn average_stat(values: impl Iterator<Item = i64>) -> i64 {
    let mut count = 0_i64;
    let mut total = 0_i64;
    for value in values {
        count += 1;
        total += value;
    }
    if count == 0 { 0 } else { total / count }
}

fn stat_from_entity(entity: &EntityView, stat: &str) -> i64 {
    entity.stats.get(stat).copied().unwrap_or(0)
}

fn resource_ids() -> Vec<&'static str> {
    vec![
        CROWNS,
        GRAIN,
        TIMBER,
        STONE,
        IRON,
        INFLUENCE,
        LEGION_STRENGTH,
        STABILITY,
        INTELLIGENCE,
        TRADE_GOODS,
        CITIZENS,
    ]
}

fn stringify_stats(stats: &BTreeMap<StatId, i64>) -> BTreeMap<String, i64> {
    stats
        .iter()
        .map(|(key, value)| (key.to_string(), *value))
        .collect()
}

fn building_status_label(status: &BuildingStatus) -> String {
    match status {
        BuildingStatus::Constructing { .. } => "Constructing",
        BuildingStatus::Active => "Active",
        BuildingStatus::Upgrading { .. } => "Upgrading",
    }
    .to_owned()
}

fn production_status_label(status: &ProductionStatus) -> String {
    match status {
        ProductionStatus::Idle => "Idle".to_owned(),
        ProductionStatus::InProgress {
            completes_at_seconds,
        } => format!("In progress until {completes_at_seconds}s"),
    }
}

fn attack_wave_label(id: u32, total_units: u32) -> String {
    let class = match total_units {
        0 | 1 => "Scout Wave",
        2..=3 => "Raid Party",
        _ => "Assault Wave",
    };
    format!("{class} {id}")
}

fn label_for(id: &str) -> &str {
    match id {
        CROWNS => "Crowns",
        GRAIN => "Food",
        TIMBER => "Wood",
        STONE => "Stone",
        IRON => "Iron",
        INFLUENCE => "Influence",
        LEGION_STRENGTH => "Legions",
        STABILITY => "Stability",
        INTELLIGENCE => "Intel",
        TRADE_GOODS => "Trade Goods",
        CITIZENS => "Citizens",
        CAPITAL => "Castle",
        FARMSTEAD => "Farm",
        LUMBER_CAMP => "Lumber Camp",
        QUARRY => "Quarry",
        IRON_MINE => "Iron Mine",
        BARRACKS => "Barracks",
        MARKET => "Storage House",
        SENATE_HALL => "Senate Hall",
        WATCHTOWER => "Tower",
        EMBASSY => "Embassy",
        FRONTIER_FORT => "Frontier Fort",
        PREFECT => "Prefect",
        ENGINEER => "Worker",
        LEGATE => "Legate",
        ENVOY => "Envoy",
        BASIC_RAIDER => "Raider",
        PROVINCE => "Province",
        RIVAL_HOUSE => "Rival House",
        CARAVAN => "Caravan",
        _ => id,
    }
}

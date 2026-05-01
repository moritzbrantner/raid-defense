use super::*;

pub const SCENARIO_CONFIG_STAT_PREFIX: &str = "scenario_config";

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScenarioConfig {
    pub buildings: BTreeMap<String, BTreeMap<String, i64>>,
    pub units: BTreeMap<String, BTreeMap<String, i64>>,
}

impl Default for ScenarioConfig {
    fn default() -> Self {
        default_scenario_config()
    }
}

pub fn default_scenario_config() -> ScenarioConfig {
    ScenarioConfig {
        buildings: BTreeMap::from([
            (
                CAPITAL.to_owned(),
                building_stats(BTreeMap::from([
                    ("security_base", 55),
                    ("global_prefect_security", 8),
                ])),
            ),
            (
                FARMSTEAD.to_owned(),
                building_stats(BTreeMap::from([
                    ("security_base", 25),
                    ("hit_points_per_tile", 10),
                ])),
            ),
            (LUMBER_CAMP.to_owned(), building_stats(BTreeMap::from([("security_base", 25)]))),
            (QUARRY.to_owned(), building_stats(BTreeMap::from([("security_base", 25)]))),
            (IRON_MINE.to_owned(), building_stats(BTreeMap::from([("security_base", 25)]))),
            (
                MARKET.to_owned(),
                building_stats(BTreeMap::from([
                    ("security_base", 25),
                    ("hit_points_bonus", 20),
                ])),
            ),
            (
                SENATE_HALL.to_owned(),
                building_stats(BTreeMap::from([("security_base", 25)])),
            ),
            (
                BARRACKS.to_owned(),
                building_stats(BTreeMap::from([
                    ("security_base", 58),
                    ("assigned_legate_security", 10),
                ])),
            ),
            (
                WATCHTOWER.to_owned(),
                building_stats(BTreeMap::from([
                    ("security_base", 45),
                    ("security_per_level", 10),
                    ("global_legate_security", 3),
                ])),
            ),
            (
                FRONTIER_FORT.to_owned(),
                building_stats(BTreeMap::from([
                    ("security_base", 48),
                    ("security_per_level", 12),
                    ("assigned_legate_security", 12),
                    ("low_supply_security_penalty", 10),
                ])),
            ),
            (
                EMBASSY.to_owned(),
                building_stats(BTreeMap::from([
                    ("security_base", 35),
                    ("assigned_prefect_security", 8),
                ])),
            ),
        ]),
        units: BTreeMap::from([
            (
                ENGINEER.to_owned(),
                unit_stats(BTreeMap::from([
                    ("stomach_capacity", 500),
                    ("stomach_per_food", 100),
                    ("hunger_threshold_percent", 30),
                    ("hunger_drain_per_second", 1),
                ])),
            ),
            (PREFECT.to_owned(), unit_stats(BTreeMap::new())),
            (LEGATE.to_owned(), unit_stats(BTreeMap::new())),
            (ENVOY.to_owned(), unit_stats(BTreeMap::new())),
            (
                BASIC_RAIDER.to_owned(),
                unit_stats(BTreeMap::from([
                    ("speed", 1),
                    ("attack_damage", 30),
                    ("carry_capacity", 36),
                    ("looting_speed", 12),
                ])),
            ),
        ]),
    }
}

pub fn scenario_config(state: &GameState) -> ScenarioConfig {
    let mut config = default_scenario_config();
    merge_stored_values(state, "building", &mut config.buildings);
    merge_stored_values(state, "unit", &mut config.units);
    config
}

pub fn apply_scenario_config(
    state: &mut GameState,
    config: &ScenarioConfig,
) -> Result<(), EngineError> {
    let defaults = default_scenario_config();
    write_scope(state, "building", &defaults.buildings, &config.buildings);
    write_scope(state, "unit", &defaults.units, &config.units);
    refresh_configured_entity_stats(state)
}

pub(crate) fn building_config_stat(state: &GameState, kind: &str, stat: &str) -> i64 {
    scoped_config_stat(state, "building", kind, stat)
}

pub(crate) fn unit_config_stat(state: &GameState, kind: &str, stat: &str) -> i64 {
    scoped_config_stat(state, "unit", kind, stat)
}

pub(crate) fn configured_building_max_hit_points(state: &GameState, building: &Building) -> i64 {
    let kind = building.kind.as_str();
    let footprint_area = i64::from(building.footprint.width * building.footprint.depth);
    (building_config_stat(state, kind, "hit_points_base")
        + i64::from(building.level.max(1)) * building_config_stat(state, kind, "hit_points_per_level")
        + footprint_area * building_config_stat(state, kind, "hit_points_per_tile")
        + building_config_stat(state, kind, "hit_points_bonus"))
    .max(1)
}

fn refresh_configured_entity_stats(state: &mut GameState) -> Result<(), EngineError> {
    let buildings = state.buildings().map(|building| building.id).collect::<Vec<_>>();
    for building in buildings {
        let Some(entry) = state.building(building).cloned() else {
            continue;
        };
        let max_hit_points = configured_building_max_hit_points(state, &entry);
        state.set_building_stat(building, MAX_HIT_POINTS, max_hit_points)?;
        state.set_building_stat(building, HIT_POINTS, max_hit_points)?;
    }

    let units = state.entities().map(|entity| entity.id).collect::<Vec<_>>();
    for unit in units {
        let Some(entity) = state.entity(unit) else {
            continue;
        };
        if entity.blueprint == EntityBlueprintRef::Unit(BASIC_RAIDER.into()) {
            state.set_entity_stat(unit, RAIDER_SPEED, unit_config_stat(state, BASIC_RAIDER, "speed"))?;
            state.set_entity_stat(
                unit,
                RAIDER_ATTACK_DAMAGE,
                unit_config_stat(state, BASIC_RAIDER, "attack_damage"),
            )?;
            state.set_entity_stat(
                unit,
                RAIDER_CARRY_CAPACITY,
                unit_config_stat(state, BASIC_RAIDER, "carry_capacity"),
            )?;
            state.set_entity_stat(
                unit,
                RAIDER_LOOTING_SPEED,
                unit_config_stat(state, BASIC_RAIDER, "looting_speed"),
            )?;
        }
        if entity.blueprint == EntityBlueprintRef::Unit(ENGINEER.into()) {
            let capacity = unit_config_stat(state, ENGINEER, "stomach_capacity");
            let current = state.entity_stat(unit, WORKER_STOMACH).unwrap_or(capacity);
            state.set_entity_stat(unit, WORKER_STOMACH, current.clamp(0, capacity))?;
        }
    }

    Ok(())
}

fn building_stats(overrides: BTreeMap<&str, i64>) -> BTreeMap<String, i64> {
    scoped_stats(
        BTreeMap::from([
            ("security_base", 25),
            ("security_per_level", 0),
            ("assigned_prefect_security", 0),
            ("assigned_legate_security", 0),
            ("global_prefect_security", 0),
            ("global_legate_security", 0),
            ("low_supply_security_penalty", 0),
            ("hit_points_base", 30),
            ("hit_points_per_level", 20),
            ("hit_points_per_tile", 10),
            ("hit_points_bonus", 0),
        ]),
        overrides,
    )
}

fn unit_stats(overrides: BTreeMap<&str, i64>) -> BTreeMap<String, i64> {
    scoped_stats(
        BTreeMap::from([
            ("speed", 0),
            ("attack_damage", 0),
            ("carry_capacity", 0),
            ("looting_speed", 0),
            ("stomach_capacity", 0),
            ("stomach_per_food", 0),
            ("hunger_threshold_percent", 0),
            ("hunger_drain_per_second", 0),
        ]),
        overrides,
    )
}

fn scoped_stats(
    defaults: BTreeMap<&str, i64>,
    overrides: BTreeMap<&str, i64>,
) -> BTreeMap<String, i64> {
    let mut merged = defaults
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect::<BTreeMap<_, _>>();
    for (key, value) in overrides {
        merged.insert(key.to_owned(), value);
    }
    merged
}

fn merge_stored_values(
    state: &GameState,
    scope: &str,
    config: &mut BTreeMap<String, BTreeMap<String, i64>>,
) {
    let stored_values = state
        .stats()
        .map(|(key, value)| (key.to_string(), *value))
        .collect::<BTreeMap<_, _>>();
    for (kind, stats) in config {
        for (stat, value) in stats {
            if let Some(stored) = stored_values.get(&config_stat_key(scope, kind, stat)) {
                *value = *stored;
            }
        }
    }
}

fn write_scope(
    state: &mut GameState,
    scope: &str,
    defaults: &BTreeMap<String, BTreeMap<String, i64>>,
    requested: &BTreeMap<String, BTreeMap<String, i64>>,
) {
    for (kind, default_stats) in defaults {
        for (stat, default_value) in default_stats {
            let value = requested
                .get(kind)
                .and_then(|stats| stats.get(stat))
                .copied()
                .unwrap_or(*default_value);
            state.set_stat(config_stat_key(scope, kind, stat), value);
        }
    }
}

fn scoped_config_stat(state: &GameState, scope: &str, kind: &str, stat: &str) -> i64 {
    let defaults = default_scenario_config();
    let default_value = match scope {
        "building" => defaults
            .buildings
            .get(kind)
            .and_then(|stats| stats.get(stat))
            .copied(),
        "unit" => defaults.units.get(kind).and_then(|stats| stats.get(stat)).copied(),
        _ => None,
    }
    .unwrap_or(0);
    let key = config_stat_key(scope, kind, stat);
    state
        .stats()
        .find_map(|(stat, value)| (stat.to_string() == key).then_some(*value))
        .unwrap_or(default_value)
}

fn config_stat_key(scope: &str, kind: &str, stat: &str) -> String {
    format!("{SCENARIO_CONFIG_STAT_PREFIX}:{scope}:{kind}:{stat}")
}

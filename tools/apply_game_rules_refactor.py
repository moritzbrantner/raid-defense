from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


core_path = Path("crates/raid-defense-core/src/lib.rs")
core = core_path.read_text()

old_constants = '''use collection_kernels::{SparseMap, SparseSet};

pub type EntityId = u32;

pub const GRID_WIDTH: i16 = 17;
pub const GRID_HEIGHT: i16 = 13;
pub const CELL_SCALE: i32 = 1_000;
pub const STARTING_WOOD: u32 = 120;
pub const TOWN_WOOD_CAPACITY: u32 = 500;
pub const SAWMILL_COST: u32 = 40;
pub const SAWMILL_OUTPUT: u16 = 4;
pub const SAWMILL_INTERVAL_TICKS: u16 = 10;
pub const SAWMILL_LOCAL_WOOD_CAPACITY: u32 = 24;
pub const ARROW_TOWER_COST: u32 = 25;
pub const CANNON_TOWER_COST: u32 = 45;
pub const MAX_TOWER_LEVEL: u8 = 3;
pub const HOUSE_UNLOCK_COMPLETED_WAVES: u32 = 10;
pub const HOUSE_COST: u32 = 60;
pub const BASE_POPULATION_CAPACITY: u16 = 2;
pub const HOUSE_POPULATION_CAPACITY: u16 = 2;
pub const PERSON_CARRY_CAPACITY: u16 = 8;
pub const PERSON_SPEED_MILLI: u16 = 500;
pub const TOWN_MAX_HEALTH: u16 = 250;
pub const DAY_LENGTH_TICKS: u16 = 600;

const TOWER_MAX_HEALTH: u16 = 100;
const SAWMILL_MAX_HEALTH: u16 = 80;
const HOUSE_MAX_HEALTH: u16 = 90;
const PERSON_MAX_HEALTH: u16 = 20;
const RAIDER_BASE_HEALTH: u16 = 30;
const RAIDER_BASE_DAMAGE: u16 = 10;
const RAIDER_SPEED_MILLI: u16 = 250;
const RAIDER_BASE_WOOD_STEAL: u32 = 15;
const TOWN_ENTITY: EntityId = 0;
'''
new_constants = '''use collection_kernels::{SparseMap, SparseSet};

mod default_rules;
pub mod rules;

pub use default_rules::STANDARD_RULES;
pub use rules::*;

pub type EntityId = u32;

// Topology and deterministic representation are engine invariants, not balance knobs.
pub const GRID_WIDTH: i16 = 17;
pub const GRID_HEIGHT: i16 = 13;
pub const CELL_SCALE: i32 = 1_000;

// Compatibility aliases for callers that use the standard profile. New simulation
// code must read the immutable `GameState::rules()` value instead.
pub const STARTING_WOOD: u32 = STANDARD_RULES.economy.starting_wood;
pub const TOWN_WOOD_CAPACITY: u32 = STANDARD_RULES.economy.town_wood_capacity;
pub const SAWMILL_COST: u32 = STANDARD_RULES.buildings.sawmill.wood_cost;
pub const SAWMILL_OUTPUT: u16 = STANDARD_RULES.economy.sawmill_output;
pub const SAWMILL_INTERVAL_TICKS: u16 = STANDARD_RULES.economy.sawmill_interval_ticks;
pub const SAWMILL_LOCAL_WOOD_CAPACITY: u32 = STANDARD_RULES.economy.sawmill_local_wood_capacity;
pub const ARROW_TOWER_COST: u32 = STANDARD_RULES.towers.arrow.build_cost;
pub const CANNON_TOWER_COST: u32 = STANDARD_RULES.towers.cannon.build_cost;
pub const MAX_TOWER_LEVEL: u8 = STANDARD_RULES.towers.max_level;
pub const HOUSE_UNLOCK_COMPLETED_WAVES: u32 = STANDARD_RULES.buildings.house.unlock_completed_waves;
pub const HOUSE_COST: u32 = STANDARD_RULES.buildings.house.wood_cost;
pub const BASE_POPULATION_CAPACITY: u16 = STANDARD_RULES.population.base_capacity;
pub const HOUSE_POPULATION_CAPACITY: u16 = STANDARD_RULES.buildings.house.population_capacity;
pub const PERSON_CARRY_CAPACITY: u16 = STANDARD_RULES.population.carry_capacity;
pub const PERSON_SPEED_MILLI: u16 = STANDARD_RULES.population.speed_milli;
pub const TOWN_MAX_HEALTH: u16 = STANDARD_RULES.buildings.town_hall.max_health;
pub const DAY_LENGTH_TICKS: u16 = STANDARD_RULES.cycle.day_length_ticks;

const TOWER_MAX_HEALTH: u16 = STANDARD_RULES.towers.max_health;
const SAWMILL_MAX_HEALTH: u16 = STANDARD_RULES.buildings.sawmill.max_health;
const HOUSE_MAX_HEALTH: u16 = STANDARD_RULES.buildings.house.max_health;
const PERSON_MAX_HEALTH: u16 = STANDARD_RULES.population.person_health;
const RAIDER_BASE_HEALTH: u16 = STANDARD_RULES.raids.base_health;
const RAIDER_BASE_DAMAGE: u16 = STANDARD_RULES.raids.base_damage;
const RAIDER_SPEED_MILLI: u16 = STANDARD_RULES.raids.speed_milli;
const RAIDER_BASE_WOOD_STEAL: u32 = STANDARD_RULES.raids.base_wood_steal;
const TOWN_ENTITY: EntityId = 0;
'''
core = replace_once(core, old_constants, new_constants, "constants/module boundary")

core = replace_once(
    core,
    '''pub struct GameState {
    seed: u64,
''',
    '''pub struct GameState {
    seed: u64,
    rules: GameRules,
''',
    "GameState rules field",
)

core = replace_once(
    core,
    '''    pub fn new(seed: u64) -> Self {
        let mut state = Self {
            seed,
            tick: 0,
            wave: 0,
            completed_waves: 0,
            day_ticks_remaining: DAY_LENGTH_TICKS,
            next_entity: 1,
''',
    '''    pub fn new(seed: u64) -> Self {
        Self::with_rules(seed, STANDARD_RULES)
    }

    #[must_use]
    pub fn with_rules(seed: u64, rules: GameRules) -> Self {
        Self::try_with_rules(seed, rules).expect("game rules must be valid")
    }

    pub fn try_with_rules(seed: u64, rules: GameRules) -> Result<Self, RulesError> {
        rules.validate()?;
        let mut state = Self {
            seed,
            rules,
            tick: 0,
            wave: 0,
            completed_waves: 0,
            day_ticks_remaining: rules.cycle.day_length_ticks,
            next_entity: 1,
''',
    "GameState constructors",
)

core = replace_once(
    core,
    '''            Health {
                current: TOWN_MAX_HEALTH,
                maximum: TOWN_MAX_HEALTH,
            },
''',
    '''            Health {
                current: rules.buildings.town_hall.max_health,
                maximum: rules.buildings.town_hall.max_health,
            },
''',
    "town health rules",
)
core = replace_once(
    core,
    '''            ResourceStorage {
                wood: STARTING_WOOD,
                wood_capacity: TOWN_WOOD_CAPACITY,
            },
''',
    '''            ResourceStorage {
                wood: rules.economy.starting_wood,
                wood_capacity: rules.economy.town_wood_capacity,
            },
''',
    "town storage rules",
)
core = replace_once(
    core,
    '''            Housing {
                capacity: BASE_POPULATION_CAPACITY,
            },
        );
        for _ in 0..BASE_POPULATION_CAPACITY {
            state.spawn_person();
        }
        state
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
''',
    '''            Housing {
                capacity: rules.population.base_capacity,
            },
        );
        for _ in 0..rules.population.starting_people {
            state.spawn_person();
        }
        Ok(state)
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
''',
    "population initialization",
)
core = replace_once(
    core,
    '''    pub const fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn tick(&self) -> u64 {
''',
    '''    pub const fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn rules(&self) -> GameRules {
        self.rules
    }

    #[must_use]
    pub const fn tick(&self) -> u64 {
''',
    "rules accessor",
)
core = replace_once(
    core,
    '''    pub fn houses_unlocked(&self) -> bool {
        self.completed_waves >= HOUSE_UNLOCK_COMPLETED_WAVES
    }
''',
    '''    pub fn houses_unlocked(&self) -> bool {
        self.rules.houses_unlocked(self.completed_waves)
    }
''',
    "house unlock rules",
)
core = replace_once(core, "            town_max_health: TOWN_MAX_HEALTH,\n", "            town_max_health: self.rules.buildings.town_hall.max_health,\n", "snapshot town health")
core = replace_once(core, "        feed_u64(&mut hash, self.seed);\n", "        feed_u64(&mut hash, self.seed);\n        feed_u64(&mut hash, self.rules.fingerprint());\n", "rules checksum")

core = replace_once(core, "        let cost = tower_cost(archetype);\n", "        let cost = self.rules.tower(archetype).build_cost;\n", "tower cost")
core = replace_once(
    core,
    '''            Health {
                current: TOWER_MAX_HEALTH,
                maximum: TOWER_MAX_HEALTH,
            },
''',
    '''            Health {
                current: self.rules.towers.max_health,
                maximum: self.rules.towers.max_health,
            },
''',
    "tower health",
)
core = replace_once(core, "            .insert(entity_key(entity), attack_for(archetype, 1, 0));\n", "            .insert(entity_key(entity), attack_for(self.rules, archetype, 1, 0));\n", "tower attack rules")

core = replace_once(
    core,
    '''    fn place_sawmill(&mut self, cell: Cell) -> Result<Event, GameError> {
        self.validate_build_cell(cell)?;
        self.require_wood(SAWMILL_COST)?;
        self.validate_blocking_build(cell, Some(cell))?;
''',
    '''    fn place_sawmill(&mut self, cell: Cell) -> Result<Event, GameError> {
        self.validate_build_cell(cell)?;
        let building_rules = self.rules.buildings.sawmill;
        let cost = building_rules.wood_cost;
        self.require_wood(cost)?;
        self.validate_blocking_build(cell, Some(cell))?;
''',
    "sawmill setup rules",
)
core = replace_once(
    core,
    '''            Health {
                current: SAWMILL_MAX_HEALTH,
                maximum: SAWMILL_MAX_HEALTH,
            },
''',
    '''            Health {
                current: building_rules.max_health,
                maximum: building_rules.max_health,
            },
''',
    "sawmill health",
)
core = replace_once(core, "                amount: SAWMILL_OUTPUT,\n                interval_ticks: SAWMILL_INTERVAL_TICKS,\n", "                amount: self.rules.economy.sawmill_output,\n                interval_ticks: self.rules.economy.sawmill_interval_ticks,\n", "sawmill production")
core = replace_once(core, "                wood_capacity: SAWMILL_LOCAL_WOOD_CAPACITY,\n", "                wood_capacity: self.rules.economy.sawmill_local_wood_capacity,\n", "sawmill local capacity")
core = replace_once(
    core,
    '''        self.spend_wood(SAWMILL_COST);

        Ok(Event::SawmillBuilt {
            entity,
            cell,
            wood_cost: SAWMILL_COST,
        })
''',
    '''        self.spend_wood(cost);

        Ok(Event::SawmillBuilt {
            entity,
            cell,
            wood_cost: cost,
        })
''',
    "sawmill spend/event",
)

core = replace_once(
    core,
    '''        self.validate_build_cell(cell)?;
        self.require_wood(HOUSE_COST)?;
        self.validate_blocking_build(cell, None)?;

        let entity = self.allocate_entity();
''',
    '''        self.validate_build_cell(cell)?;
        let house_rules = self.rules.buildings.house;
        let cost = house_rules.wood_cost;
        self.require_wood(cost)?;
        self.validate_blocking_build(cell, None)?;

        let entity = self.allocate_entity();
''',
    "house setup rules",
)
core = replace_once(
    core,
    '''            Health {
                current: HOUSE_MAX_HEALTH,
                maximum: HOUSE_MAX_HEALTH,
            },
''',
    '''            Health {
                current: house_rules.max_health,
                maximum: house_rules.max_health,
            },
''',
    "house health",
)
core = replace_once(core, "                capacity: HOUSE_POPULATION_CAPACITY,\n", "                capacity: house_rules.population_capacity,\n", "house capacity")
core = replace_once(
    core,
    '''        self.spend_wood(HOUSE_COST);
        for _ in 0..HOUSE_POPULATION_CAPACITY {
            self.spawn_person();
        }

        Ok(Event::HouseBuilt {
            entity,
            cell,
            wood_cost: HOUSE_COST,
            people_added: HOUSE_POPULATION_CAPACITY,
''',
    '''        self.spend_wood(cost);
        for _ in 0..house_rules.people_added {
            self.spawn_person();
        }

        Ok(Event::HouseBuilt {
            entity,
            cell,
            wood_cost: cost,
            people_added: house_rules.people_added,
''',
    "house people/spend/event",
)

core = replace_once(core, "            tower_upgrade_cost(tower.archetype, tower.level).ok_or(GameError::MaxTowerLevel)?;\n", "            self.rules.tower_upgrade_cost(tower.archetype, tower.level).ok_or(GameError::MaxTowerLevel)?;\n", "upgrade cost rules")
core = replace_once(core, "        let next_attack = attack_for(tower.archetype, next_level, cooldown_remaining);\n", "        let next_attack = attack_for(self.rules, tower.archetype, next_level, cooldown_remaining);\n", "upgrade stats rules")

core = replace_once(
    core,
    '''        self.day_ticks_remaining = 0;
        self.wave = self.wave.saturating_add(1);
        let offset = (mix64(self.seed ^ u64::from(self.wave)) % 4) as usize;
        for index in 0..4 {
            let edge = Edge::ALL[(index + offset) % Edge::ALL.len()];
            self.spawn_raider(edge);
        }

        Ok(Event::WaveStarted {
            wave: self.wave,
            raiders: 4,
        })
''',
    '''        self.day_ticks_remaining = 0;
        self.wave = self.wave.saturating_add(1);
        let offset = (mix64(self.seed ^ u64::from(self.wave)) % 4) as usize;
        for index in 0..usize::from(self.rules.raids.raiders_per_wave) {
            let edge = Edge::ALL[(index + offset) % Edge::ALL.len()];
            self.spawn_raider(edge);
        }

        Ok(Event::WaveStarted {
            wave: self.wave,
            raiders: self.rules.raids.raiders_per_wave,
        })
''',
    "wave size rules",
)
core = replace_once(
    core,
    '''        let (wood_produced, wood_picked_up, wood_delivered) = if had_raiders {
            (0, 0, 0)
        } else {
''',
    '''        let economy_paused = had_raiders && self.rules.cycle.pause_economy_during_raids;
        let (wood_produced, wood_picked_up, wood_delivered) = if economy_paused {
            (0, 0, 0)
        } else {
''',
    "night economy rule",
)
core = replace_once(
    core,
    '''        if !had_raiders {
            self.day_ticks_remaining = self.day_ticks_remaining.saturating_sub(1);
            if self.day_ticks_remaining == 0 && self.town_health() > 0 {
''',
    '''        if !had_raiders && self.rules.cycle.automatic_raids {
            self.day_ticks_remaining = self.day_ticks_remaining.saturating_sub(1);
            if self.day_ticks_remaining == 0 && self.town_health() > 0 {
''',
    "automatic raid rule",
)
core = replace_once(core, "            self.day_ticks_remaining = DAY_LENGTH_TICKS;\n", "            self.day_ticks_remaining = self.rules.cycle.day_length_ticks;\n", "day reset rules")

# Person movement has two authoritative construction sites.
if core.count("                    speed_milli: PERSON_SPEED_MILLI,\n") != 2:
    raise SystemExit(f"person speed rules: expected 2 matches, found {core.count('                    speed_milli: PERSON_SPEED_MILLI,\\n')}")
core = core.replace("                    speed_milli: PERSON_SPEED_MILLI,\n", "                    speed_milli: self.rules.population.speed_milli,\n", 2)

core = replace_once(
    core,
    '''    fn raider_wood_steal_amount(&self) -> u32 {
        RAIDER_BASE_WOOD_STEAL.saturating_add(self.wave.saturating_sub(1).saturating_mul(2))
    }
''',
    '''    fn raider_wood_steal_amount(&self) -> u32 {
        self.rules.raids.base_wood_steal.saturating_add(
            self.wave
                .saturating_sub(1)
                .saturating_mul(self.rules.raids.wood_steal_per_wave),
        )
    }
''',
    "raid theft scaling",
)
core = replace_once(
    core,
    '''        let wave_bonus = u16::try_from(self.wave.saturating_sub(1))
            .unwrap_or(u16::MAX)
            .saturating_mul(3);
        let max_health = RAIDER_BASE_HEALTH.saturating_add(wave_bonus);
        let damage_bonus = u16::try_from(self.wave / 3).unwrap_or(u16::MAX);
''',
    '''        let wave_bonus = u16::try_from(self.wave.saturating_sub(1))
            .unwrap_or(u16::MAX)
            .saturating_mul(self.rules.raids.health_per_wave);
        let max_health = self.rules.raids.base_health.saturating_add(wave_bonus);
        let damage_bonus = u16::try_from(
            self.wave / self.rules.raids.damage_increase_every_waves,
        )
        .unwrap_or(u16::MAX);
''',
    "raid health/damage scaling",
)
core = replace_once(core, "                damage: RAIDER_BASE_DAMAGE.saturating_add(damage_bonus),\n", "                damage: self.rules.raids.base_damage.saturating_add(damage_bonus),\n", "raid base damage")
core = replace_once(core, "                speed_milli: RAIDER_SPEED_MILLI,\n", "                speed_milli: self.rules.raids.speed_milli,\n", "raid speed")
core = replace_once(
    core,
    '''            Health {
                current: PERSON_MAX_HEALTH,
                maximum: PERSON_MAX_HEALTH,
            },
''',
    '''            Health {
                current: self.rules.population.person_health,
                maximum: self.rules.population.person_health,
            },
''',
    "person health",
)
core = replace_once(core, "                cargo_capacity: PERSON_CARRY_CAPACITY,\n", "                cargo_capacity: self.rules.population.carry_capacity,\n", "person carry rules")
core = replace_once(core, "                    .and_then(|tower| tower_upgrade_cost(tower.archetype, tower.level)),\n", "                    .and_then(|tower| self.rules.tower_upgrade_cost(tower.archetype, tower.level)),\n", "snapshot upgrade rules")

core = replace_once(
    core,
    '''#[must_use]
pub const fn tower_cost(archetype: TowerArchetype) -> u32 {
    match archetype {
        TowerArchetype::Arrow => ARROW_TOWER_COST,
        TowerArchetype::Cannon => CANNON_TOWER_COST,
    }
}

#[must_use]
pub const fn tower_upgrade_cost(archetype: TowerArchetype, level: u8) -> Option<u32> {
    match (archetype, level) {
        (TowerArchetype::Arrow, 1) => Some(20),
        (TowerArchetype::Arrow, 2) => Some(30),
        (TowerArchetype::Cannon, 1) => Some(30),
        (TowerArchetype::Cannon, 2) => Some(45),
        _ => None,
    }
}

#[must_use]
pub const fn tower_stats(archetype: TowerArchetype, level: u8) -> TowerStats {
    match (archetype, level) {
        (TowerArchetype::Arrow, 1) => TowerStats {
            damage: 8,
            range_milli: 3_200,
            cooldown_ticks: 3,
            projectile_speed_milli: 900,
        },
        (TowerArchetype::Arrow, 2) => TowerStats {
            damage: 12,
            range_milli: 3_500,
            cooldown_ticks: 3,
            projectile_speed_milli: 1_000,
        },
        (TowerArchetype::Arrow, _) => TowerStats {
            damage: 17,
            range_milli: 3_800,
            cooldown_ticks: 2,
            projectile_speed_milli: 1_100,
        },
        (TowerArchetype::Cannon, 1) => TowerStats {
            damage: 18,
            range_milli: 4_200,
            cooldown_ticks: 7,
            projectile_speed_milli: 500,
        },
        (TowerArchetype::Cannon, 2) => TowerStats {
            damage: 27,
            range_milli: 4_500,
            cooldown_ticks: 6,
            projectile_speed_milli: 550,
        },
        (TowerArchetype::Cannon, _) => TowerStats {
            damage: 40,
            range_milli: 4_800,
            cooldown_ticks: 5,
            projectile_speed_milli: 600,
        },
    }
}

fn attack_for(archetype: TowerArchetype, level: u8, cooldown_remaining: u8) -> Attack {
    let stats = tower_stats(archetype, level);
''',
    '''#[must_use]
pub const fn tower_cost(archetype: TowerArchetype) -> u32 {
    STANDARD_RULES.tower(archetype).build_cost
}

#[must_use]
pub const fn tower_upgrade_cost(archetype: TowerArchetype, level: u8) -> Option<u32> {
    STANDARD_RULES.tower_upgrade_cost(archetype, level)
}

#[must_use]
pub const fn tower_stats(archetype: TowerArchetype, level: u8) -> TowerStats {
    STANDARD_RULES.tower_level(archetype, level).stats()
}

fn attack_for(
    rules: GameRules,
    archetype: TowerArchetype,
    level: u8,
    cooldown_remaining: u8,
) -> Attack {
    let stats = rules.tower_level(archetype, level).stats();
''',
    "tower helper rules",
)
core = replace_once(
    core,
    '''pub fn replay(seed: u64, commands: &[Command]) -> Result<GameState, GameError> {
    let mut state = GameState::new(seed);
    for command in commands {
        state.apply(*command)?;
    }
    Ok(state)
}
''',
    '''pub fn replay(seed: u64, commands: &[Command]) -> Result<GameState, GameError> {
    replay_with_rules(seed, STANDARD_RULES, commands)
}

pub fn replay_with_rules(
    seed: u64,
    rules: GameRules,
    commands: &[Command],
) -> Result<GameState, GameError> {
    let mut state = GameState::with_rules(seed, rules);
    for command in commands {
        state.apply(*command)?;
    }
    Ok(state)
}
''',
    "replay rules",
)

# Add focused configurability/deterministic identity tests before the final math test.
test_anchor = '''    #[test]
    fn integer_square_root_is_deterministic_at_boundaries() {
'''
custom_tests = '''    #[test]
    fn custom_rules_change_mechanics_without_engine_changes() {
        let mut rules = STANDARD_RULES;
        rules.buildings.house.unlock_completed_waves = 0;
        rules.buildings.house.wood_cost = 5;
        rules.buildings.house.population_capacity = 3;
        rules.buildings.house.people_added = 1;
        rules.cycle.day_length_ticks = 2;

        let mut state = GameState::with_rules(23, rules);
        let starting_wood = state.wood();
        state
            .apply(Command::PlaceHouse { x: 2, z: 2 })
            .expect("custom rules should unlock the house immediately");
        assert_eq!(state.wood(), starting_wood - 5);
        assert_eq!(state.population_capacity(), rules.population.base_capacity + 3);
        assert_eq!(state.people_count(), usize::from(rules.population.starting_people + 1));

        state
            .apply(Command::AdvanceTick)
            .expect("first short day tick should advance");
        assert_eq!(state.wave(), 0);
        state
            .apply(Command::AdvanceTick)
            .expect("second short day tick should auto-start a wave");
        assert_eq!(state.wave(), 1);
        assert_eq!(state.raider_count(), usize::from(rules.raids.raiders_per_wave));
    }

    #[test]
    fn rules_are_part_of_deterministic_identity() {
        let standard = GameState::new(29);
        let mut changed = STANDARD_RULES;
        changed.buildings.house.unlock_completed_waves += 1;
        let custom = GameState::with_rules(29, changed);

        assert_ne!(standard.rules().fingerprint(), custom.rules().fingerprint());
        assert_ne!(standard.checksum(), custom.checksum());
    }

'''
core = replace_once(core, test_anchor, custom_tests + test_anchor, "custom rules tests")
core_path.write_text(core)

# WASM must expose values from the active rules profile, not duplicate standard constants.
wasm_path = Path("crates/raid-defense-wasm/src/lib.rs")
wasm = wasm_path.read_text()
wasm = replace_once(
    wasm,
    '''use raid_defense_core::{
    ARROW_TOWER_COST, CANNON_TOWER_COST, Command, EntityKind, Event, GameError, GameState,
    HOUSE_COST, HOUSE_POPULATION_CAPACITY, HOUSE_UNLOCK_COMPLETED_WAVES, MAX_TOWER_LEVEL,
    PERSON_CARRY_CAPACITY, PersonState, ResourceKind, SAWMILL_COST, SAWMILL_INTERVAL_TICKS,
    SAWMILL_LOCAL_WOOD_CAPACITY, SAWMILL_OUTPUT, TowerArchetype,
};
''',
    '''use raid_defense_core::{
    Command, EntityKind, Event, GameError, GameState, PersonState, ResourceKind, TowerArchetype,
};
''',
    "wasm rule imports",
)
wasm = replace_once(
    wasm,
    '''    fn from(state: &GameState) -> Self {
        let snapshot = state.snapshot();
        Self {
''',
    '''    fn from(state: &GameState) -> Self {
        let snapshot = state.snapshot();
        let rules = state.rules();
        Self {
''',
    "wasm active rules",
)
for old, new, label in [
    ("            sawmill_cost: SAWMILL_COST,\n", "            sawmill_cost: rules.buildings.sawmill.wood_cost,\n", "wasm sawmill cost"),
    ("            sawmill_output: SAWMILL_OUTPUT,\n", "            sawmill_output: rules.economy.sawmill_output,\n", "wasm sawmill output"),
    ("            sawmill_interval_ticks: SAWMILL_INTERVAL_TICKS,\n", "            sawmill_interval_ticks: rules.economy.sawmill_interval_ticks,\n", "wasm sawmill interval"),
    ("            sawmill_local_wood_capacity: SAWMILL_LOCAL_WOOD_CAPACITY,\n", "            sawmill_local_wood_capacity: rules.economy.sawmill_local_wood_capacity,\n", "wasm sawmill buffer"),
    ("            house_cost: HOUSE_COST,\n", "            house_cost: rules.buildings.house.wood_cost,\n", "wasm house cost"),
    ("            house_unlock_completed_waves: HOUSE_UNLOCK_COMPLETED_WAVES,\n", "            house_unlock_completed_waves: rules.buildings.house.unlock_completed_waves,\n", "wasm house unlock"),
    ("            house_population_capacity: HOUSE_POPULATION_CAPACITY,\n", "            house_population_capacity: rules.buildings.house.population_capacity,\n", "wasm house capacity"),
    ("            person_carry_capacity: PERSON_CARRY_CAPACITY,\n", "            person_carry_capacity: rules.population.carry_capacity,\n", "wasm carry capacity"),
    ("            arrow_tower_cost: ARROW_TOWER_COST,\n", "            arrow_tower_cost: rules.towers.arrow.build_cost,\n", "wasm arrow cost"),
    ("            cannon_tower_cost: CANNON_TOWER_COST,\n", "            cannon_tower_cost: rules.towers.cannon.build_cost,\n", "wasm cannon cost"),
    ("            max_tower_level: MAX_TOWER_LEVEL,\n", "            max_tower_level: rules.towers.max_level,\n", "wasm tower level"),
]:
    wasm = replace_once(wasm, old, new, label)
wasm_path.write_text(wasm)

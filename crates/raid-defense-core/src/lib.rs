#![forbid(unsafe_code)]

use std::collections::{BTreeMap, VecDeque};

use collection_kernels::{SparseMap, SparseSet};

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

const TOWN_ENTITY: EntityId = 0;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Cell {
    pub x: i16,
    pub z: i16,
}

impl Cell {
    #[must_use]
    pub const fn new(x: i16, z: i16) -> Self {
        Self { x, z }
    }

    #[must_use]
    pub fn center_x_milli(self) -> i32 {
        i32::from(self.x) * CELL_SCALE + CELL_SCALE / 2
    }

    #[must_use]
    pub fn center_z_milli(self) -> i32 {
        i32::from(self.z) * CELL_SCALE + CELL_SCALE / 2
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Edge {
    North,
    East,
    South,
    West,
}

impl Edge {
    pub const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    #[must_use]
    pub const fn spawn_cell(self) -> Cell {
        match self {
            Self::North => Cell::new(GRID_WIDTH / 2, 0),
            Self::East => Cell::new(GRID_WIDTH - 1, GRID_HEIGHT / 2),
            Self::South => Cell::new(GRID_WIDTH / 2, GRID_HEIGHT - 1),
            Self::West => Cell::new(0, GRID_HEIGHT / 2),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Transform {
    pub x_milli: i32,
    pub z_milli: i32,
}

impl Transform {
    fn at_cell(cell: Cell) -> Self {
        Self {
            x_milli: cell.center_x_milli(),
            z_milli: cell.center_z_milli(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Health {
    pub current: u16,
    pub maximum: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Attack {
    pub damage: u16,
    pub range_milli: i32,
    pub cooldown_ticks: u8,
    pub cooldown_remaining: u8,
    pub projectile_speed_milli: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    Wood,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceStorage {
    pub wood: u32,
    pub wood_capacity: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceProducer {
    pub resource: ResourceKind,
    pub amount: u16,
    pub interval_ticks: u16,
    pub progress_ticks: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Housing {
    pub capacity: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildingKind {
    TownHall,
    Tower,
    Sawmill,
    House,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Building {
    pub kind: BuildingKind,
    pub cell: Cell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TowerArchetype {
    Arrow,
    Cannon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tower {
    pub archetype: TowerArchetype,
    pub level: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TowerStats {
    pub damage: u16,
    pub range_milli: i32,
    pub cooldown_ticks: u8,
    pub projectile_speed_milli: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Raider {
    pub edge: Edge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Movement {
    pub from: Cell,
    pub to: Cell,
    pub progress_milli: u16,
    pub speed_milli: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Projectile {
    pub target: EntityId,
    pub damage: u16,
    pub speed_milli: u16,
    pub archetype: TowerArchetype,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonState {
    IdleAtTownHall,
    ToSawmill,
    ToTownHall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Person {
    pub state: PersonState,
    pub target_sawmill: Option<EntityId>,
    pub cargo_wood: u16,
    pub cargo_capacity: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    PlaceTower {
        x: i16,
        z: i16,
        archetype: TowerArchetype,
    },
    PlaceSawmill {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    TowerBuilt {
        entity: EntityId,
        cell: Cell,
        archetype: TowerArchetype,
        level: u8,
        wood_cost: u32,
    },
    SawmillBuilt {
        entity: EntityId,
        cell: Cell,
        wood_cost: u32,
    },
    HouseBuilt {
        entity: EntityId,
        cell: Cell,
        wood_cost: u32,
        people_added: u16,
        population_capacity: u16,
    },
    TowerUpgraded {
        entity: EntityId,
        archetype: TowerArchetype,
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
        wood_stolen: u16,
        town_damage: u16,
        completed_wave: Option<u32>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameError {
    OutOfBounds,
    CellOccupied,
    ProtectedCell,
    PathBlocked,
    InsufficientWood,
    HouseLocked,
    NoTower,
    MaxTowerLevel,
    RaidersStillActive,
    GameOver,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityKind {
    TownHall,
    Tower,
    Sawmill,
    House,
    Person,
    Raider,
    Projectile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntitySnapshot {
    pub id: EntityId,
    pub kind: EntityKind,
    pub x_milli: i32,
    pub z_milli: i32,
    pub cell: Cell,
    pub health: u16,
    pub max_health: u16,
    pub attack_damage: u16,
    pub attack_range_milli: i32,
    pub tower_archetype: Option<TowerArchetype>,
    pub tower_level: u8,
    pub upgrade_cost: Option<u32>,
    pub projectile_target: Option<EntityId>,
    pub stored_wood: u32,
    pub wood_capacity: u32,
    pub production_resource: Option<ResourceKind>,
    pub production_amount: u16,
    pub production_interval_ticks: u16,
    pub production_progress_ticks: u16,
    pub housing_capacity: u16,
    pub person_state: Option<PersonState>,
    pub person_target_sawmill: Option<EntityId>,
    pub cargo_wood: u16,
    pub cargo_capacity: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameSnapshot {
    pub seed: u64,
    pub tick: u64,
    pub wood: u32,
    pub wood_capacity: u32,
    pub wave: u32,
    pub completed_waves: u32,
    pub people: u16,
    pub population_capacity: u16,
    pub houses_unlocked: bool,
    pub town_health: u16,
    pub town_max_health: u16,
    pub grid_width: i16,
    pub grid_height: i16,
    pub entities: Vec<EntitySnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    seed: u64,
    rules: GameRules,
    tick: u64,
    wave: u32,
    completed_waves: u32,
    day_ticks_remaining: u16,
    next_entity: EntityId,
    transforms: SparseMap<Transform>,
    health: SparseMap<Health>,
    attacks: SparseMap<Attack>,
    buildings: SparseMap<Building>,
    towers: SparseMap<Tower>,
    raiders: SparseMap<Raider>,
    movements: SparseMap<Movement>,
    projectiles: SparseMap<Projectile>,
    storage: SparseMap<ResourceStorage>,
    producers: SparseMap<ResourceProducer>,
    housing: SparseMap<Housing>,
    people: SparseMap<Person>,
    alive: SparseSet,
}

impl GameState {
    #[must_use]
    pub fn new(seed: u64) -> Self {
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
            transforms: SparseMap::new(),
            health: SparseMap::new(),
            attacks: SparseMap::new(),
            buildings: SparseMap::new(),
            towers: SparseMap::new(),
            raiders: SparseMap::new(),
            movements: SparseMap::new(),
            projectiles: SparseMap::new(),
            storage: SparseMap::new(),
            producers: SparseMap::new(),
            housing: SparseMap::new(),
            people: SparseMap::new(),
            alive: SparseSet::new(),
        };
        let town_cell = town_center();
        state.spawn_entity(TOWN_ENTITY);
        state
            .transforms
            .insert(entity_key(TOWN_ENTITY), Transform::at_cell(town_cell));
        state.health.insert(
            entity_key(TOWN_ENTITY),
            Health {
                current: rules.buildings.town_hall.max_health,
                maximum: rules.buildings.town_hall.max_health,
            },
        );
        state.buildings.insert(
            entity_key(TOWN_ENTITY),
            Building {
                kind: BuildingKind::TownHall,
                cell: town_cell,
            },
        );
        state.storage.insert(
            entity_key(TOWN_ENTITY),
            ResourceStorage {
                wood: rules.economy.starting_wood,
                wood_capacity: rules.economy.town_wood_capacity,
            },
        );
        state.housing.insert(
            entity_key(TOWN_ENTITY),
            Housing {
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
        self.seed
    }

    #[must_use]
    pub const fn rules(&self) -> GameRules {
        self.rules
    }

    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    #[must_use]
    pub const fn wave(&self) -> u32 {
        self.wave
    }

    #[must_use]
    pub const fn completed_waves(&self) -> u32 {
        self.completed_waves
    }

    #[must_use]
    pub const fn day_ticks_remaining(&self) -> u16 {
        self.day_ticks_remaining
    }

    #[must_use]
    pub fn is_night(&self) -> bool {
        self.raider_count() != 0
    }

    #[must_use]
    pub fn houses_unlocked(&self) -> bool {
        self.rules.houses_unlocked(self.completed_waves)
    }

    #[must_use]
    pub fn wood(&self) -> u32 {
        self.storage
            .get(entity_key(TOWN_ENTITY))
            .map_or(0, |storage| storage.wood)
    }

    #[must_use]
    pub fn wood_capacity(&self) -> u32 {
        self.storage
            .get(entity_key(TOWN_ENTITY))
            .map_or(0, |storage| storage.wood_capacity)
    }

    #[must_use]
    pub fn population_capacity(&self) -> u16 {
        self.housing.iter().fold(0_u16, |total, (_, housing)| {
            total.saturating_add(housing.capacity)
        })
    }

    #[must_use]
    pub fn people_count(&self) -> usize {
        self.people.iter().count()
    }

    #[must_use]
    pub fn town_health(&self) -> u16 {
        self.health
            .get(entity_key(TOWN_ENTITY))
            .map_or(0, |health| health.current)
    }

    #[must_use]
    pub fn raider_count(&self) -> usize {
        self.raiders.iter().count()
    }

    #[must_use]
    pub fn tower_count(&self) -> usize {
        self.towers.iter().count()
    }

    #[must_use]
    pub fn sawmill_count(&self) -> usize {
        self.producers.iter().count()
    }

    #[must_use]
    pub fn house_count(&self) -> usize {
        self.buildings
            .iter()
            .filter(|(_, building)| building.kind == BuildingKind::House)
            .count()
    }

    #[must_use]
    pub fn projectile_count(&self) -> usize {
        self.projectiles.iter().count()
    }

    pub fn apply(&mut self, command: Command) -> Result<Event, GameError> {
        match command {
            Command::PlaceTower { x, z, archetype } => self.place_tower(Cell::new(x, z), archetype),
            Command::PlaceSawmill { x, z } => self.place_sawmill(Cell::new(x, z)),
            Command::PlaceHouse { x, z } => self.place_house(Cell::new(x, z)),
            Command::UpgradeTower { x, z } => self.upgrade_tower(Cell::new(x, z)),
            Command::StartWave => self.start_wave(),
            Command::AdvanceTick => Ok(self.advance_tick()),
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> GameSnapshot {
        let mut ids = self.alive.iter().map(key_entity).collect::<Vec<_>>();
        ids.sort_unstable();
        let entities = ids
            .into_iter()
            .filter_map(|id| self.entity_snapshot(id))
            .collect();

        GameSnapshot {
            seed: self.seed,
            tick: self.tick,
            wood: self.wood(),
            wood_capacity: self.wood_capacity(),
            wave: self.wave,
            completed_waves: self.completed_waves,
            people: u16::try_from(self.people_count()).unwrap_or(u16::MAX),
            population_capacity: self.population_capacity(),
            houses_unlocked: self.houses_unlocked(),
            town_health: self.town_health(),
            town_max_health: self.rules.buildings.town_hall.max_health,
            grid_width: GRID_WIDTH,
            grid_height: GRID_HEIGHT,
            entities,
        }
    }

    #[must_use]
    pub fn checksum(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        feed_u64(&mut hash, self.seed);
        feed_u64(&mut hash, self.rules.fingerprint());
        feed_u64(&mut hash, self.tick);
        feed_u64(&mut hash, u64::from(self.wave));
        feed_u64(&mut hash, u64::from(self.completed_waves));
        feed_u16(&mut hash, self.day_ticks_remaining);
        feed_u64(&mut hash, u64::from(self.next_entity));

        for entity in &self.snapshot().entities {
            feed_u64(&mut hash, u64::from(entity.id));
            feed_byte(
                &mut hash,
                match entity.kind {
                    EntityKind::TownHall => 0,
                    EntityKind::Tower => 1,
                    EntityKind::Sawmill => 2,
                    EntityKind::House => 3,
                    EntityKind::Person => 4,
                    EntityKind::Raider => 5,
                    EntityKind::Projectile => 6,
                },
            );
            feed_i32(&mut hash, entity.x_milli);
            feed_i32(&mut hash, entity.z_milli);
            feed_i16(&mut hash, entity.cell.x);
            feed_i16(&mut hash, entity.cell.z);
            feed_u16(&mut hash, entity.health);
            feed_u16(&mut hash, entity.max_health);

            if let Some(attack) = self.attacks.get(entity_key(entity.id)) {
                feed_byte(&mut hash, 1);
                feed_u16(&mut hash, attack.damage);
                feed_i32(&mut hash, attack.range_milli);
                feed_byte(&mut hash, attack.cooldown_ticks);
                feed_byte(&mut hash, attack.cooldown_remaining);
                feed_u16(&mut hash, attack.projectile_speed_milli);
            } else {
                feed_byte(&mut hash, 0);
            }

            if let Some(tower) = self.towers.get(entity_key(entity.id)) {
                feed_byte(&mut hash, 1);
                feed_byte(&mut hash, tower_archetype_code(tower.archetype));
                feed_byte(&mut hash, tower.level);
            } else {
                feed_byte(&mut hash, 0);
            }

            if let Some(movement) = self.movements.get(entity_key(entity.id)) {
                feed_byte(&mut hash, 1);
                feed_i16(&mut hash, movement.from.x);
                feed_i16(&mut hash, movement.from.z);
                feed_i16(&mut hash, movement.to.x);
                feed_i16(&mut hash, movement.to.z);
                feed_u16(&mut hash, movement.progress_milli);
                feed_u16(&mut hash, movement.speed_milli);
            } else {
                feed_byte(&mut hash, 0);
            }

            if let Some(raider) = self.raiders.get(entity_key(entity.id)) {
                feed_byte(&mut hash, 1);
                feed_byte(
                    &mut hash,
                    match raider.edge {
                        Edge::North => 0,
                        Edge::East => 1,
                        Edge::South => 2,
                        Edge::West => 3,
                    },
                );
            } else {
                feed_byte(&mut hash, 0);
            }

            if let Some(projectile) = self.projectiles.get(entity_key(entity.id)) {
                feed_byte(&mut hash, 1);
                feed_u64(&mut hash, u64::from(projectile.target));
                feed_u16(&mut hash, projectile.damage);
                feed_u16(&mut hash, projectile.speed_milli);
                feed_byte(&mut hash, tower_archetype_code(projectile.archetype));
            } else {
                feed_byte(&mut hash, 0);
            }

            if let Some(storage) = self.storage.get(entity_key(entity.id)) {
                feed_byte(&mut hash, 1);
                feed_u64(&mut hash, u64::from(storage.wood));
                feed_u64(&mut hash, u64::from(storage.wood_capacity));
            } else {
                feed_byte(&mut hash, 0);
            }

            if let Some(producer) = self.producers.get(entity_key(entity.id)) {
                feed_byte(&mut hash, 1);
                feed_byte(&mut hash, resource_kind_code(producer.resource));
                feed_u16(&mut hash, producer.amount);
                feed_u16(&mut hash, producer.interval_ticks);
                feed_u16(&mut hash, producer.progress_ticks);
            } else {
                feed_byte(&mut hash, 0);
            }

            if let Some(housing) = self.housing.get(entity_key(entity.id)) {
                feed_byte(&mut hash, 1);
                feed_u16(&mut hash, housing.capacity);
            } else {
                feed_byte(&mut hash, 0);
            }

            if let Some(person) = self.people.get(entity_key(entity.id)) {
                feed_byte(&mut hash, 1);
                feed_byte(&mut hash, person_state_code(person.state));
                if let Some(target) = person.target_sawmill {
                    feed_byte(&mut hash, 1);
                    feed_u64(&mut hash, u64::from(target));
                } else {
                    feed_byte(&mut hash, 0);
                }
                feed_u16(&mut hash, person.cargo_wood);
                feed_u16(&mut hash, person.cargo_capacity);
            } else {
                feed_byte(&mut hash, 0);
            }
        }

        hash
    }

    fn place_tower(&mut self, cell: Cell, archetype: TowerArchetype) -> Result<Event, GameError> {
        self.validate_build_cell(cell)?;
        let cost = self.rules.tower(archetype).build_cost;
        self.require_wood(cost)?;
        self.validate_blocking_build(cell, None)?;

        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(cell));
        self.health.insert(
            entity_key(entity),
            Health {
                current: self.rules.towers.max_health,
                maximum: self.rules.towers.max_health,
            },
        );
        self.attacks
            .insert(entity_key(entity), attack_for(self.rules, archetype, 1, 0));
        self.buildings.insert(
            entity_key(entity),
            Building {
                kind: BuildingKind::Tower,
                cell,
            },
        );
        self.towers.insert(
            entity_key(entity),
            Tower {
                archetype,
                level: 1,
            },
        );
        self.spend_wood(cost);

        Ok(Event::TowerBuilt {
            entity,
            cell,
            archetype,
            level: 1,
            wood_cost: cost,
        })
    }

    fn place_sawmill(&mut self, cell: Cell) -> Result<Event, GameError> {
        self.validate_build_cell(cell)?;
        let building_rules = self.rules.buildings.sawmill;
        let cost = building_rules.wood_cost;
        self.require_wood(cost)?;
        self.validate_blocking_build(cell, Some(cell))?;

        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(cell));
        self.health.insert(
            entity_key(entity),
            Health {
                current: building_rules.max_health,
                maximum: building_rules.max_health,
            },
        );
        self.buildings.insert(
            entity_key(entity),
            Building {
                kind: BuildingKind::Sawmill,
                cell,
            },
        );
        self.producers.insert(
            entity_key(entity),
            ResourceProducer {
                resource: ResourceKind::Wood,
                amount: self.rules.economy.sawmill_output,
                interval_ticks: self.rules.economy.sawmill_interval_ticks,
                progress_ticks: 0,
            },
        );
        self.storage.insert(
            entity_key(entity),
            ResourceStorage {
                wood: 0,
                wood_capacity: self.rules.economy.sawmill_local_wood_capacity,
            },
        );
        self.spend_wood(cost);

        Ok(Event::SawmillBuilt {
            entity,
            cell,
            wood_cost: cost,
        })
    }

    fn place_house(&mut self, cell: Cell) -> Result<Event, GameError> {
        if !self.houses_unlocked() {
            return Err(GameError::HouseLocked);
        }
        self.validate_build_cell(cell)?;
        let house_rules = self.rules.buildings.house;
        let cost = house_rules.wood_cost;
        self.require_wood(cost)?;
        self.validate_blocking_build(cell, None)?;

        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(cell));
        self.health.insert(
            entity_key(entity),
            Health {
                current: house_rules.max_health,
                maximum: house_rules.max_health,
            },
        );
        self.buildings.insert(
            entity_key(entity),
            Building {
                kind: BuildingKind::House,
                cell,
            },
        );
        self.housing.insert(
            entity_key(entity),
            Housing {
                capacity: house_rules.population_capacity,
            },
        );
        self.spend_wood(cost);
        for _ in 0..house_rules.people_added {
            self.spawn_person();
        }

        Ok(Event::HouseBuilt {
            entity,
            cell,
            wood_cost: cost,
            people_added: house_rules.people_added,
            population_capacity: self.population_capacity(),
        })
    }

    fn upgrade_tower(&mut self, cell: Cell) -> Result<Event, GameError> {
        if self.town_health() == 0 {
            return Err(GameError::GameOver);
        }
        if !in_bounds(cell) {
            return Err(GameError::OutOfBounds);
        }
        let entity = self.tower_entity_at(cell).ok_or(GameError::NoTower)?;
        let tower = self
            .towers
            .get(entity_key(entity))
            .copied()
            .expect("tower buildings always have a tower component");
        let cost = self
            .rules
            .tower_upgrade_cost(tower.archetype, tower.level)
            .ok_or(GameError::MaxTowerLevel)?;
        self.require_wood(cost)?;

        let next_level = tower.level + 1;
        let cooldown_remaining = self
            .attacks
            .get(entity_key(entity))
            .map_or(0, |attack| attack.cooldown_remaining);
        let next_attack = attack_for(self.rules, tower.archetype, next_level, cooldown_remaining);
        self.towers.insert(
            entity_key(entity),
            Tower {
                archetype: tower.archetype,
                level: next_level,
            },
        );
        self.attacks.insert(entity_key(entity), next_attack);
        self.spend_wood(cost);

        Ok(Event::TowerUpgraded {
            entity,
            archetype: tower.archetype,
            level: next_level,
            wood_cost: cost,
        })
    }

    fn validate_build_cell(&self, cell: Cell) -> Result<(), GameError> {
        if self.town_health() == 0 {
            return Err(GameError::GameOver);
        }
        if !in_bounds(cell) {
            return Err(GameError::OutOfBounds);
        }
        if is_town_cell(cell) || Edge::ALL.into_iter().any(|edge| edge.spawn_cell() == cell) {
            return Err(GameError::ProtectedCell);
        }
        if self.cell_has_building(cell) || self.unit_uses_cell(cell) {
            return Err(GameError::CellOccupied);
        }
        Ok(())
    }

    fn validate_blocking_build(
        &self,
        cell: Cell,
        hypothetical_sawmill: Option<Cell>,
    ) -> Result<(), GameError> {
        if !self.routes_remain_open(Some(cell))
            || !self.all_sawmills_accessible(Some(cell), hypothetical_sawmill)
            || !self.worker_routes_remain_open(Some(cell))
        {
            return Err(GameError::PathBlocked);
        }
        Ok(())
    }

    fn require_wood(&self, amount: u32) -> Result<(), GameError> {
        if self.wood() < amount {
            Err(GameError::InsufficientWood)
        } else {
            Ok(())
        }
    }

    fn spend_wood(&mut self, amount: u32) {
        let storage = self
            .storage
            .get_mut(entity_key(TOWN_ENTITY))
            .expect("town hall always owns resource storage");
        storage.wood -= amount;
    }

    fn store_town_wood(&mut self, amount: u32) -> u32 {
        let storage = self
            .storage
            .get_mut(entity_key(TOWN_ENTITY))
            .expect("town hall always owns resource storage");
        let available = storage.wood_capacity.saturating_sub(storage.wood);
        let stored = available.min(amount);
        storage.wood = storage.wood.saturating_add(stored);
        stored
    }

    fn steal_wood(&mut self, amount: u32) -> u32 {
        let storage = self
            .storage
            .get_mut(entity_key(TOWN_ENTITY))
            .expect("town hall always owns resource storage");
        let stolen = storage.wood.min(amount);
        storage.wood -= stolen;
        stolen
    }

    fn start_wave(&mut self) -> Result<Event, GameError> {
        if self.town_health() == 0 {
            return Err(GameError::GameOver);
        }
        if self.raider_count() != 0 {
            return Err(GameError::RaidersStillActive);
        }

        self.day_ticks_remaining = 0;
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
    }

    fn advance_tick(&mut self) -> Event {
        self.tick = self.tick.saturating_add(1);
        let had_raiders = self.raider_count() > 0;
        let economy_paused = had_raiders && self.rules.cycle.pause_economy_during_raids;
        let (wood_produced, wood_picked_up, wood_delivered) = if economy_paused {
            (0, 0, 0)
        } else {
            let wood_produced = self.run_resource_production_system();
            let (wood_picked_up, wood_delivered) = self.run_person_logistics_system();
            (wood_produced, wood_picked_up, wood_delivered)
        };

        let mut wave_active_this_tick = had_raiders;
        if !had_raiders && self.rules.cycle.automatic_raids {
            self.day_ticks_remaining = self.day_ticks_remaining.saturating_sub(1);
            if self.day_ticks_remaining == 0 && self.town_health() > 0 {
                self.start_wave()
                    .expect("an expired peaceful day can always begin its next wave");
                wave_active_this_tick = true;
            }
        }

        let shots = self.run_tower_attack_system();
        let (impacts, killed) = self.run_projectile_system();
        let kills = u16::try_from(killed.len()).unwrap_or(u16::MAX);
        for entity in killed {
            self.despawn_raider(entity);
        }
        let (wood_stolen, town_damage) = self.run_raider_movement_system();
        let completed_wave = if wave_active_this_tick
            && self.raider_count() == 0
            && self.completed_waves < self.wave
        {
            self.completed_waves = self.wave;
            self.day_ticks_remaining = self.rules.cycle.day_length_ticks;
            Some(self.wave)
        } else {
            None
        };

        Event::TickAdvanced {
            tick: self.tick,
            shots,
            impacts,
            kills,
            wood_produced,
            wood_picked_up,
            wood_delivered,
            wood_stolen,
            town_damage,
            completed_wave,
        }
    }

    fn run_resource_production_system(&mut self) -> u16 {
        let mut ids = self.producers.keys().map(key_entity).collect::<Vec<_>>();
        ids.sort_unstable();
        let mut produced_total = 0_u16;

        for entity in ids {
            let Some(mut producer) = self.producers.get(entity_key(entity)).copied() else {
                continue;
            };
            producer.progress_ticks = producer.progress_ticks.saturating_add(1);
            let ready = producer.progress_ticks >= producer.interval_ticks;
            if ready {
                producer.progress_ticks -= producer.interval_ticks;
            }
            self.producers.insert(entity_key(entity), producer);
            if !ready {
                continue;
            }

            let Some(storage) = self.storage.get_mut(entity_key(entity)) else {
                continue;
            };
            let available = storage.wood_capacity.saturating_sub(storage.wood);
            let produced = available.min(u32::from(producer.amount));
            storage.wood = storage.wood.saturating_add(produced);
            produced_total =
                produced_total.saturating_add(u16::try_from(produced).unwrap_or(u16::MAX));
        }

        produced_total
    }

    fn run_person_logistics_system(&mut self) -> (u16, u16) {
        let mut picked_up_total = 0_u16;
        let mut delivered_total = 0_u16;

        let mut person_ids = self.people.keys().map(key_entity).collect::<Vec<_>>();
        person_ids.sort_unstable();

        for entity in &person_ids {
            let Some(mut person) = self.people.get(entity_key(*entity)).copied() else {
                continue;
            };
            if person.state != PersonState::IdleAtTownHall || person.cargo_wood == 0 {
                continue;
            }
            let delivered = self.store_town_wood(u32::from(person.cargo_wood));
            let delivered_u16 = u16::try_from(delivered).unwrap_or(u16::MAX);
            person.cargo_wood = person.cargo_wood.saturating_sub(delivered_u16);
            delivered_total = delivered_total.saturating_add(delivered_u16);
            self.people.insert(entity_key(*entity), person);
        }

        self.assign_idle_people();

        for entity in person_ids {
            let Some(mut person) = self.people.get(entity_key(entity)).copied() else {
                continue;
            };
            if person.state == PersonState::IdleAtTownHall {
                continue;
            }
            let Some(mut movement) = self.movements.get(entity_key(entity)).copied() else {
                self.rebuild_person_movement(entity, person);
                continue;
            };

            movement.progress_milli = movement.progress_milli.saturating_add(movement.speed_milli);
            if movement.progress_milli < CELL_SCALE as u16 {
                self.movements.insert(entity_key(entity), movement);
                self.transforms
                    .insert(entity_key(entity), interpolate_transform(movement));
                continue;
            }

            movement.progress_milli -= CELL_SCALE as u16;
            movement.from = movement.to;
            self.transforms
                .insert(entity_key(entity), Transform::at_cell(movement.from));

            match person.state {
                PersonState::IdleAtTownHall => {}
                PersonState::ToSawmill => {
                    let Some(target) = person.target_sawmill else {
                        person.state = PersonState::IdleAtTownHall;
                        self.movements.remove(entity_key(entity));
                        self.people.insert(entity_key(entity), person);
                        continue;
                    };
                    let goals = self.sawmill_pickup_cells(target, None);
                    if goals.contains(&movement.from) {
                        let pickup = self.take_sawmill_wood(target, person.cargo_capacity);
                        person.cargo_wood = pickup;
                        picked_up_total = picked_up_total.saturating_add(pickup);
                        person.state = PersonState::ToTownHall;
                        person.target_sawmill = None;
                        if let Some(next) =
                            self.next_path_step_to_any(movement.from, &town_goal_cells(), None)
                        {
                            movement.to = next;
                            movement.progress_milli = 0;
                            self.movements.insert(entity_key(entity), movement);
                        } else {
                            self.movements.remove(entity_key(entity));
                        }
                    } else if let Some(next) =
                        self.next_path_step_to_any(movement.from, &goals, None)
                    {
                        movement.to = next;
                        self.movements.insert(entity_key(entity), movement);
                    } else {
                        self.movements.remove(entity_key(entity));
                    }
                }
                PersonState::ToTownHall => {
                    if is_town_cell(movement.from) {
                        let delivered = self.store_town_wood(u32::from(person.cargo_wood));
                        let delivered_u16 = u16::try_from(delivered).unwrap_or(u16::MAX);
                        person.cargo_wood = person.cargo_wood.saturating_sub(delivered_u16);
                        delivered_total = delivered_total.saturating_add(delivered_u16);
                        person.state = PersonState::IdleAtTownHall;
                        self.movements.remove(entity_key(entity));
                    } else if let Some(next) =
                        self.next_path_step_to_any(movement.from, &town_goal_cells(), None)
                    {
                        movement.to = next;
                        self.movements.insert(entity_key(entity), movement);
                    } else {
                        self.movements.remove(entity_key(entity));
                    }
                }
            }
            self.people.insert(entity_key(entity), person);
        }

        (picked_up_total, delivered_total)
    }

    fn assign_idle_people(&mut self) {
        let mut reserved = BTreeMap::<EntityId, u32>::new();
        for (_, person) in self.people.iter() {
            if person.state == PersonState::ToSawmill
                && let Some(target) = person.target_sawmill
            {
                let entry = reserved.entry(target).or_default();
                *entry = entry.saturating_add(u32::from(person.cargo_capacity));
            }
        }

        let mut ids = self.people.keys().map(key_entity).collect::<Vec<_>>();
        ids.sort_unstable();
        for entity in ids {
            let Some(mut person) = self.people.get(entity_key(entity)).copied() else {
                continue;
            };
            if person.state != PersonState::IdleAtTownHall || person.cargo_wood != 0 {
                continue;
            }
            let Some(target) = self.best_sawmill_for_person(&reserved) else {
                continue;
            };
            let start = self
                .transforms
                .get(entity_key(entity))
                .copied()
                .map_or(town_center(), cell_for_transform);
            let goals = self.sawmill_pickup_cells(target, None);
            let Some(next) = self.next_path_step_to_any(start, &goals, None) else {
                continue;
            };
            person.state = PersonState::ToSawmill;
            person.target_sawmill = Some(target);
            self.people.insert(entity_key(entity), person);
            self.movements.insert(
                entity_key(entity),
                Movement {
                    from: start,
                    to: next,
                    progress_milli: 0,
                    speed_milli: self.rules.population.speed_milli,
                },
            );
            let entry = reserved.entry(target).or_default();
            *entry = entry.saturating_add(u32::from(person.cargo_capacity));
        }
    }

    fn best_sawmill_for_person(&self, reserved: &BTreeMap<EntityId, u32>) -> Option<EntityId> {
        let mut candidates = self
            .producers
            .keys()
            .map(key_entity)
            .filter_map(|entity| {
                let stored = self.storage.get(entity_key(entity))?.wood;
                let reserved = reserved.get(&entity).copied().unwrap_or(0);
                let available = stored.saturating_sub(reserved);
                (available > 0).then_some((available, entity))
            })
            .collect::<Vec<_>>();
        candidates.sort_by_key(|(available, entity)| (std::cmp::Reverse(*available), *entity));
        candidates.first().map(|(_, entity)| *entity)
    }

    fn take_sawmill_wood(&mut self, sawmill: EntityId, capacity: u16) -> u16 {
        let Some(storage) = self.storage.get_mut(entity_key(sawmill)) else {
            return 0;
        };
        let amount = storage.wood.min(u32::from(capacity));
        storage.wood -= amount;
        u16::try_from(amount).unwrap_or(u16::MAX)
    }

    fn rebuild_person_movement(&mut self, entity: EntityId, person: Person) {
        let start = self
            .transforms
            .get(entity_key(entity))
            .copied()
            .map_or(town_center(), cell_for_transform);
        let goals = match person.state {
            PersonState::IdleAtTownHall => return,
            PersonState::ToTownHall => town_goal_cells(),
            PersonState::ToSawmill => person
                .target_sawmill
                .map_or_else(Vec::new, |target| self.sawmill_pickup_cells(target, None)),
        };
        if let Some(next) = self.next_path_step_to_any(start, &goals, None) {
            self.movements.insert(
                entity_key(entity),
                Movement {
                    from: start,
                    to: next,
                    progress_milli: 0,
                    speed_milli: self.rules.population.speed_milli,
                },
            );
        }
    }

    fn run_tower_attack_system(&mut self) -> u16 {
        let mut tower_ids = self.towers.keys().map(key_entity).collect::<Vec<_>>();
        tower_ids.sort_unstable();

        let mut shots = 0_u16;
        for tower in tower_ids {
            let Some(mut attack) = self.attacks.get(entity_key(tower)).copied() else {
                continue;
            };
            if attack.cooldown_remaining > 0 {
                attack.cooldown_remaining -= 1;
                self.attacks.insert(entity_key(tower), attack);
                continue;
            }

            let Some(target) = self.target_for_tower(tower, attack.range_milli) else {
                continue;
            };
            let tower_component = self
                .towers
                .get(entity_key(tower))
                .copied()
                .expect("tower ids come from the tower component store");
            let tower_position = self
                .transforms
                .get(entity_key(tower))
                .copied()
                .expect("towers always have transforms");
            let projectile_entity = self.allocate_entity();
            self.transforms
                .insert(entity_key(projectile_entity), tower_position);
            self.projectiles.insert(
                entity_key(projectile_entity),
                Projectile {
                    target,
                    damage: attack.damage,
                    speed_milli: attack.projectile_speed_milli,
                    archetype: tower_component.archetype,
                },
            );

            shots = shots.saturating_add(1);
            attack.cooldown_remaining = attack.cooldown_ticks;
            self.attacks.insert(entity_key(tower), attack);
        }

        shots
    }

    fn run_projectile_system(&mut self) -> (u16, Vec<EntityId>) {
        let mut ids = self.projectiles.keys().map(key_entity).collect::<Vec<_>>();
        ids.sort_unstable();
        let mut impacts = 0_u16;
        let mut killed = Vec::new();
        let mut despawn = Vec::new();

        for entity in ids {
            let Some(projectile) = self.projectiles.get(entity_key(entity)).copied() else {
                continue;
            };
            let Some(target_health) = self.health.get(entity_key(projectile.target)).copied()
            else {
                despawn.push(entity);
                continue;
            };
            if target_health.current == 0 {
                despawn.push(entity);
                continue;
            }
            let Some(position) = self.transforms.get(entity_key(entity)).copied() else {
                despawn.push(entity);
                continue;
            };
            let Some(target_position) = self.transforms.get(entity_key(projectile.target)).copied()
            else {
                despawn.push(entity);
                continue;
            };

            let dx = i64::from(target_position.x_milli - position.x_milli);
            let dz = i64::from(target_position.z_milli - position.z_milli);
            let distance_sq = dx * dx + dz * dz;
            let speed = i64::from(projectile.speed_milli);

            if distance_sq <= speed * speed {
                let target_died = {
                    let health = self
                        .health
                        .get_mut(entity_key(projectile.target))
                        .expect("target health was checked above");
                    health.current = health.current.saturating_sub(projectile.damage);
                    health.current == 0
                };
                impacts = impacts.saturating_add(1);
                despawn.push(entity);
                if target_died {
                    killed.push(projectile.target);
                }
                continue;
            }

            let distance = i64::try_from(integer_sqrt(distance_sq as u64))
                .unwrap_or(i64::MAX)
                .max(1);
            let step_x = dx * speed / distance;
            let step_z = dz * speed / distance;
            let next = Transform {
                x_milli: position
                    .x_milli
                    .saturating_add(saturating_i64_to_i32(step_x)),
                z_milli: position
                    .z_milli
                    .saturating_add(saturating_i64_to_i32(step_z)),
            };
            self.transforms.insert(entity_key(entity), next);
        }

        for entity in despawn {
            self.despawn_projectile(entity);
        }
        killed.sort_unstable();
        killed.dedup();
        (impacts, killed)
    }

    fn target_for_tower(&self, tower: EntityId, range_milli: i32) -> Option<EntityId> {
        let tower_position = *self.transforms.get(entity_key(tower))?;
        let range_sq = i64::from(range_milli) * i64::from(range_milli);
        let mut candidates = self
            .raiders
            .keys()
            .map(key_entity)
            .filter(|entity| {
                self.health
                    .get(entity_key(*entity))
                    .is_some_and(|health| health.current > 0)
            })
            .filter_map(|entity| {
                let position = self.transforms.get(entity_key(entity))?;
                let dx = i64::from(position.x_milli - tower_position.x_milli);
                let dz = i64::from(position.z_milli - tower_position.z_milli);
                let distance_sq = dx * dx + dz * dz;
                (distance_sq <= range_sq).then_some((distance_sq, entity))
            })
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        candidates.first().map(|(_, entity)| *entity)
    }

    fn run_raider_movement_system(&mut self) -> (u16, u16) {
        let mut raiders = self.raiders.keys().map(key_entity).collect::<Vec<_>>();
        raiders.sort_unstable();
        let mut wood_stolen = 0_u16;
        let mut town_damage = 0_u16;
        let mut reached_town = Vec::new();

        for entity in raiders {
            let Some(mut movement) = self.movements.get(entity_key(entity)).copied() else {
                continue;
            };
            movement.progress_milli = movement.progress_milli.saturating_add(movement.speed_milli);

            if movement.progress_milli >= CELL_SCALE as u16 {
                movement.progress_milli -= CELL_SCALE as u16;
                movement.from = movement.to;
                if is_town_cell(movement.from) {
                    let stolen = self.steal_wood(self.raider_wood_steal_amount());
                    wood_stolen =
                        wood_stolen.saturating_add(u16::try_from(stolen).unwrap_or(u16::MAX));
                    if stolen == 0 {
                        let damage = self
                            .attacks
                            .get(entity_key(entity))
                            .map_or(0, |attack| attack.damage);
                        town_damage = town_damage.saturating_add(damage);
                    }
                    reached_town.push(entity);
                    continue;
                }

                let Some(next) =
                    self.next_path_step_to_any(movement.from, &town_goal_cells(), None)
                else {
                    movement.to = movement.from;
                    self.movements.insert(entity_key(entity), movement);
                    self.transforms
                        .insert(entity_key(entity), Transform::at_cell(movement.from));
                    continue;
                };
                movement.to = next;
            }

            let transform = interpolate_transform(movement);
            self.movements.insert(entity_key(entity), movement);
            self.transforms.insert(entity_key(entity), transform);
        }

        if town_damage > 0
            && let Some(health) = self.health.get_mut(entity_key(TOWN_ENTITY))
        {
            health.current = health.current.saturating_sub(town_damage);
        }
        for entity in reached_town {
            self.despawn_raider(entity);
        }

        (wood_stolen, town_damage)
    }

    fn raider_wood_steal_amount(&self) -> u32 {
        self.rules.raids.base_wood_steal.saturating_add(
            self.wave
                .saturating_sub(1)
                .saturating_mul(self.rules.raids.wood_steal_per_wave),
        )
    }

    fn spawn_raider(&mut self, edge: Edge) {
        let from = edge.spawn_cell();
        let to = self
            .next_path_step_to_any(from, &town_goal_cells(), None)
            .expect("validated building placements keep every edge connected to town hall");
        let entity = self.allocate_entity();
        let wave_bonus = u16::try_from(self.wave.saturating_sub(1))
            .unwrap_or(u16::MAX)
            .saturating_mul(self.rules.raids.health_per_wave);
        let max_health = self.rules.raids.base_health.saturating_add(wave_bonus);
        let damage_bonus = u16::try_from(self.wave / self.rules.raids.damage_increase_every_waves)
            .unwrap_or(u16::MAX);
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(from));
        self.health.insert(
            entity_key(entity),
            Health {
                current: max_health,
                maximum: max_health,
            },
        );
        self.attacks.insert(
            entity_key(entity),
            Attack {
                damage: self.rules.raids.base_damage.saturating_add(damage_bonus),
                range_milli: 0,
                cooldown_ticks: 0,
                cooldown_remaining: 0,
                projectile_speed_milli: 0,
            },
        );
        self.raiders.insert(entity_key(entity), Raider { edge });
        self.movements.insert(
            entity_key(entity),
            Movement {
                from,
                to,
                progress_milli: 0,
                speed_milli: self.rules.raids.speed_milli,
            },
        );
    }

    fn spawn_person(&mut self) -> EntityId {
        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(town_center()));
        self.health.insert(
            entity_key(entity),
            Health {
                current: self.rules.population.person_health,
                maximum: self.rules.population.person_health,
            },
        );
        self.people.insert(
            entity_key(entity),
            Person {
                state: PersonState::IdleAtTownHall,
                target_sawmill: None,
                cargo_wood: 0,
                cargo_capacity: self.rules.population.carry_capacity,
            },
        );
        entity
    }

    fn despawn_raider(&mut self, entity: EntityId) {
        let mut dependent_projectiles = self
            .projectiles
            .iter()
            .filter_map(|(key, projectile)| {
                (projectile.target == entity).then_some(key_entity(key))
            })
            .collect::<Vec<_>>();
        dependent_projectiles.sort_unstable();
        for projectile in dependent_projectiles {
            self.despawn_projectile(projectile);
        }

        let key = entity_key(entity);
        self.transforms.remove(key);
        self.health.remove(key);
        self.attacks.remove(key);
        self.raiders.remove(key);
        self.movements.remove(key);
        self.alive.remove(key);
    }

    fn despawn_projectile(&mut self, entity: EntityId) {
        let key = entity_key(entity);
        self.transforms.remove(key);
        self.projectiles.remove(key);
        self.alive.remove(key);
    }

    fn allocate_entity(&mut self) -> EntityId {
        let entity = self.next_entity;
        self.next_entity = self
            .next_entity
            .checked_add(1)
            .expect("entity id space exhausted");
        self.spawn_entity(entity);
        entity
    }

    fn spawn_entity(&mut self, entity: EntityId) {
        let inserted = self.alive.insert(entity_key(entity));
        assert!(inserted, "entity ids must be unique");
    }

    fn entity_snapshot(&self, entity: EntityId) -> Option<EntitySnapshot> {
        let key = entity_key(entity);
        let transform = *self.transforms.get(key)?;
        let health = self.health.get(key).copied().unwrap_or(Health {
            current: 0,
            maximum: 0,
        });
        let attack = self.attacks.get(key).copied().unwrap_or(Attack {
            damage: 0,
            range_milli: 0,
            cooldown_ticks: 0,
            cooldown_remaining: 0,
            projectile_speed_milli: 0,
        });
        let storage = self.storage.get(key).copied().unwrap_or(ResourceStorage {
            wood: 0,
            wood_capacity: 0,
        });
        let producer = self.producers.get(key).copied();
        let housing = self.housing.get(key).copied();

        if let Some(building) = self.buildings.get(key) {
            let tower = self.towers.get(key).copied();
            return Some(EntitySnapshot {
                id: entity,
                kind: match building.kind {
                    BuildingKind::TownHall => EntityKind::TownHall,
                    BuildingKind::Tower => EntityKind::Tower,
                    BuildingKind::Sawmill => EntityKind::Sawmill,
                    BuildingKind::House => EntityKind::House,
                },
                x_milli: transform.x_milli,
                z_milli: transform.z_milli,
                cell: building.cell,
                health: health.current,
                max_health: health.maximum,
                attack_damage: attack.damage,
                attack_range_milli: attack.range_milli,
                tower_archetype: tower.map(|tower| tower.archetype),
                tower_level: tower.map_or(0, |tower| tower.level),
                upgrade_cost: tower
                    .and_then(|tower| self.rules.tower_upgrade_cost(tower.archetype, tower.level)),
                projectile_target: None,
                stored_wood: storage.wood,
                wood_capacity: storage.wood_capacity,
                production_resource: producer.map(|producer| producer.resource),
                production_amount: producer.map_or(0, |producer| producer.amount),
                production_interval_ticks: producer.map_or(0, |producer| producer.interval_ticks),
                production_progress_ticks: producer.map_or(0, |producer| producer.progress_ticks),
                housing_capacity: housing.map_or(0, |housing| housing.capacity),
                person_state: None,
                person_target_sawmill: None,
                cargo_wood: 0,
                cargo_capacity: 0,
            });
        }

        if let Some(person) = self.people.get(key).copied() {
            let cell = self
                .movements
                .get(key)
                .map_or_else(|| cell_for_transform(transform), |movement| movement.from);
            return Some(EntitySnapshot {
                id: entity,
                kind: EntityKind::Person,
                x_milli: transform.x_milli,
                z_milli: transform.z_milli,
                cell,
                health: health.current,
                max_health: health.maximum,
                attack_damage: 0,
                attack_range_milli: 0,
                tower_archetype: None,
                tower_level: 0,
                upgrade_cost: None,
                projectile_target: None,
                stored_wood: 0,
                wood_capacity: 0,
                production_resource: None,
                production_amount: 0,
                production_interval_ticks: 0,
                production_progress_ticks: 0,
                housing_capacity: 0,
                person_state: Some(person.state),
                person_target_sawmill: person.target_sawmill,
                cargo_wood: person.cargo_wood,
                cargo_capacity: person.cargo_capacity,
            });
        }

        if self.raiders.get(key).is_some() {
            let movement = self.movements.get(key)?;
            return Some(EntitySnapshot {
                id: entity,
                kind: EntityKind::Raider,
                x_milli: transform.x_milli,
                z_milli: transform.z_milli,
                cell: movement.from,
                health: health.current,
                max_health: health.maximum,
                attack_damage: attack.damage,
                attack_range_milli: attack.range_milli,
                tower_archetype: None,
                tower_level: 0,
                upgrade_cost: None,
                projectile_target: None,
                stored_wood: 0,
                wood_capacity: 0,
                production_resource: None,
                production_amount: 0,
                production_interval_ticks: 0,
                production_progress_ticks: 0,
                housing_capacity: 0,
                person_state: None,
                person_target_sawmill: None,
                cargo_wood: 0,
                cargo_capacity: 0,
            });
        }

        let projectile = self.projectiles.get(key)?;
        Some(EntitySnapshot {
            id: entity,
            kind: EntityKind::Projectile,
            x_milli: transform.x_milli,
            z_milli: transform.z_milli,
            cell: cell_for_transform(transform),
            health: 0,
            max_health: 0,
            attack_damage: projectile.damage,
            attack_range_milli: 0,
            tower_archetype: Some(projectile.archetype),
            tower_level: 0,
            upgrade_cost: None,
            projectile_target: Some(projectile.target),
            stored_wood: 0,
            wood_capacity: 0,
            production_resource: None,
            production_amount: 0,
            production_interval_ticks: 0,
            production_progress_ticks: 0,
            housing_capacity: 0,
            person_state: None,
            person_target_sawmill: None,
            cargo_wood: 0,
            cargo_capacity: 0,
        })
    }

    fn tower_entity_at(&self, cell: Cell) -> Option<EntityId> {
        self.buildings.iter().find_map(|(key, building)| {
            (building.kind == BuildingKind::Tower && building.cell == cell)
                .then_some(key_entity(key))
        })
    }

    fn cell_has_building(&self, cell: Cell) -> bool {
        self.buildings
            .iter()
            .any(|(_, building)| building.kind != BuildingKind::TownHall && building.cell == cell)
    }

    fn unit_uses_cell(&self, cell: Cell) -> bool {
        self.movements
            .iter()
            .any(|(_, movement)| movement.from == cell || movement.to == cell)
    }

    fn routes_remain_open(&self, extra_block: Option<Cell>) -> bool {
        let goals = town_goal_cells();
        Edge::ALL.into_iter().all(|edge| {
            self.next_path_step_to_any(edge.spawn_cell(), &goals, extra_block)
                .is_some()
        }) && self.raiders.iter().all(|(key, _)| {
            self.movements.get(key).is_some_and(|movement| {
                self.next_path_step_to_any(movement.from, &goals, extra_block)
                    .is_some()
            })
        })
    }

    fn all_sawmills_accessible(
        &self,
        extra_block: Option<Cell>,
        hypothetical_sawmill: Option<Cell>,
    ) -> bool {
        let mut cells = self
            .buildings
            .iter()
            .filter_map(|(_, building)| {
                (building.kind == BuildingKind::Sawmill).then_some(building.cell)
            })
            .collect::<Vec<_>>();
        if let Some(cell) = hypothetical_sawmill {
            cells.push(cell);
        }
        cells.into_iter().all(|cell| {
            let goals = self.pickup_cells_for_sawmill_cell(cell, extra_block);
            !goals.is_empty()
                && self
                    .next_path_step_to_any(town_center(), &goals, extra_block)
                    .is_some()
        })
    }

    fn worker_routes_remain_open(&self, extra_block: Option<Cell>) -> bool {
        self.people.iter().all(|(key, person)| {
            let start = self.movements.get(key).map_or_else(
                || {
                    self.transforms
                        .get(key)
                        .copied()
                        .map_or(town_center(), cell_for_transform)
                },
                |movement| movement.from,
            );
            match person.state {
                PersonState::IdleAtTownHall => true,
                PersonState::ToTownHall => self
                    .next_path_step_to_any(start, &town_goal_cells(), extra_block)
                    .is_some(),
                PersonState::ToSawmill => person.target_sawmill.is_some_and(|target| {
                    let goals = self.sawmill_pickup_cells(target, extra_block);
                    !goals.is_empty()
                        && self
                            .next_path_step_to_any(start, &goals, extra_block)
                            .is_some()
                }),
            }
        })
    }

    fn sawmill_pickup_cells(&self, entity: EntityId, extra_block: Option<Cell>) -> Vec<Cell> {
        let Some(building) = self.buildings.get(entity_key(entity)) else {
            return Vec::new();
        };
        self.pickup_cells_for_sawmill_cell(building.cell, extra_block)
    }

    fn pickup_cells_for_sawmill_cell(
        &self,
        sawmill_cell: Cell,
        extra_block: Option<Cell>,
    ) -> Vec<Cell> {
        neighbors(sawmill_cell)
            .into_iter()
            .filter(|cell| !self.path_cell_blocked(*cell, extra_block))
            .collect()
    }

    fn next_path_step_to_any(
        &self,
        start: Cell,
        goals: &[Cell],
        extra_block: Option<Cell>,
    ) -> Option<Cell> {
        if goals.contains(&start) {
            return Some(start);
        }
        if !in_bounds(start) || goals.is_empty() {
            return None;
        }

        let cell_count = usize::try_from(GRID_WIDTH).ok()? * usize::try_from(GRID_HEIGHT).ok()?;
        let mut visited = vec![false; cell_count];
        let mut parent = vec![None; cell_count];
        let mut queue = VecDeque::new();
        let start_index = cell_index(start)?;
        visited[start_index] = true;
        queue.push_back(start);

        let mut goal = None;
        while let Some(cell) = queue.pop_front() {
            if goals.contains(&cell) {
                goal = Some(cell);
                break;
            }
            for neighbor in neighbors(cell) {
                let index = cell_index(neighbor)?;
                if visited[index] || self.path_cell_blocked(neighbor, extra_block) {
                    continue;
                }
                visited[index] = true;
                parent[index] = Some(cell);
                queue.push_back(neighbor);
            }
        }

        let mut cursor = goal?;
        loop {
            let cursor_index = cell_index(cursor)?;
            let previous = parent[cursor_index]?;
            if previous == start {
                return Some(cursor);
            }
            cursor = previous;
        }
    }

    fn path_cell_blocked(&self, cell: Cell, extra_block: Option<Cell>) -> bool {
        !is_town_cell(cell) && (extra_block == Some(cell) || self.cell_has_building(cell))
    }
}

#[must_use]
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
    Attack {
        damage: stats.damage,
        range_milli: stats.range_milli,
        cooldown_ticks: stats.cooldown_ticks,
        cooldown_remaining: cooldown_remaining.min(stats.cooldown_ticks),
        projectile_speed_milli: stats.projectile_speed_milli,
    }
}

pub fn replay(seed: u64, commands: &[Command]) -> Result<GameState, GameError> {
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

#[must_use]
pub const fn town_center() -> Cell {
    Cell::new(GRID_WIDTH / 2, GRID_HEIGHT / 2)
}

#[must_use]
pub const fn is_town_cell(cell: Cell) -> bool {
    let center = town_center();
    cell.x >= center.x - 1
        && cell.x <= center.x + 1
        && cell.z >= center.z - 1
        && cell.z <= center.z + 1
}

fn town_goal_cells() -> Vec<Cell> {
    let center = town_center();
    let mut cells = Vec::with_capacity(9);
    for z in center.z - 1..=center.z + 1 {
        for x in center.x - 1..=center.x + 1 {
            cells.push(Cell::new(x, z));
        }
    }
    cells
}

const fn in_bounds(cell: Cell) -> bool {
    cell.x >= 0 && cell.x < GRID_WIDTH && cell.z >= 0 && cell.z < GRID_HEIGHT
}

fn cell_index(cell: Cell) -> Option<usize> {
    if !in_bounds(cell) {
        return None;
    }
    let x = usize::try_from(cell.x).ok()?;
    let z = usize::try_from(cell.z).ok()?;
    let width = usize::try_from(GRID_WIDTH).ok()?;
    Some(z * width + x)
}

fn neighbors(cell: Cell) -> Vec<Cell> {
    [
        Cell::new(cell.x, cell.z - 1),
        Cell::new(cell.x + 1, cell.z),
        Cell::new(cell.x, cell.z + 1),
        Cell::new(cell.x - 1, cell.z),
    ]
    .into_iter()
    .filter(|candidate| in_bounds(*candidate))
    .collect()
}

fn interpolate_transform(movement: Movement) -> Transform {
    let from_x = movement.from.center_x_milli();
    let from_z = movement.from.center_z_milli();
    let to_x = movement.to.center_x_milli();
    let to_z = movement.to.center_z_milli();
    let progress = i32::from(movement.progress_milli);
    Transform {
        x_milli: from_x + (to_x - from_x) * progress / CELL_SCALE,
        z_milli: from_z + (to_z - from_z) * progress / CELL_SCALE,
    }
}

fn cell_for_transform(transform: Transform) -> Cell {
    let x = transform
        .x_milli
        .div_euclid(CELL_SCALE)
        .clamp(0, i32::from(GRID_WIDTH - 1));
    let z = transform
        .z_milli
        .div_euclid(CELL_SCALE)
        .clamp(0, i32::from(GRID_HEIGHT - 1));
    Cell::new(x as i16, z as i16)
}

fn integer_sqrt(value: u64) -> u64 {
    if value < 2 {
        return value;
    }
    let mut x = value;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x
}

fn saturating_i64_to_i32(value: i64) -> i32 {
    i32::try_from(value).unwrap_or_else(|_| {
        if value.is_negative() {
            i32::MIN
        } else {
            i32::MAX
        }
    })
}

const fn tower_archetype_code(archetype: TowerArchetype) -> u8 {
    match archetype {
        TowerArchetype::Arrow => 0,
        TowerArchetype::Cannon => 1,
    }
}

const fn resource_kind_code(resource: ResourceKind) -> u8 {
    match resource {
        ResourceKind::Wood => 0,
    }
}

const fn person_state_code(state: PersonState) -> u8 {
    match state {
        PersonState::IdleAtTownHall => 0,
        PersonState::ToSawmill => 1,
        PersonState::ToTownHall => 2,
    }
}

const fn entity_key(entity: EntityId) -> usize {
    entity as usize
}

const fn key_entity(key: usize) -> EntityId {
    key as EntityId
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn feed_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        feed_byte(hash, byte);
    }
}

fn feed_i32(hash: &mut u64, value: i32) {
    for byte in value.to_le_bytes() {
        feed_byte(hash, byte);
    }
}

fn feed_i16(hash: &mut u64, value: i16) {
    for byte in value.to_le_bytes() {
        feed_byte(hash, byte);
    }
}

fn feed_u16(hash: &mut u64, value: u16) {
    for byte in value.to_le_bytes() {
        feed_byte(hash, byte);
    }
}

fn feed_byte(hash: &mut u64, byte: u8) {
    *hash ^= u64::from(byte);
    *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sawmill_entity(state: &GameState) -> EntityId {
        state
            .buildings
            .iter()
            .find_map(|(key, building)| {
                (building.kind == BuildingKind::Sawmill).then_some(key_entity(key))
            })
            .expect("sawmill should exist")
    }

    #[test]
    fn population_starts_with_two_real_people() {
        let state = GameState::new(7);
        assert_eq!(state.people_count(), 2);
        assert_eq!(state.population_capacity(), 2);
        assert!(!state.houses_unlocked());
        assert_eq!(
            state
                .snapshot()
                .entities
                .iter()
                .filter(|entity| entity.kind == EntityKind::Person)
                .count(),
            2
        );
    }

    #[test]
    fn sawmill_output_stays_local_until_a_person_carries_it_home() {
        let mut state = GameState::new(7);
        state
            .apply(Command::PlaceSawmill { x: 2, z: 2 })
            .expect("sawmill should build");
        assert_eq!(state.wood(), STARTING_WOOD - SAWMILL_COST);

        for _ in 0..SAWMILL_INTERVAL_TICKS {
            state
                .apply(Command::AdvanceTick)
                .expect("tick should advance");
        }

        let sawmill = sawmill_entity(&state);
        assert_eq!(state.wood(), STARTING_WOOD - SAWMILL_COST);
        assert!(
            state
                .storage
                .get(entity_key(sawmill))
                .is_some_and(|storage| storage.wood > 0),
            "produced wood must wait at the sawmill for a carrier"
        );

        for _ in 0..80 {
            state
                .apply(Command::AdvanceTick)
                .expect("tick should advance");
            if state.wood() > STARTING_WOOD - SAWMILL_COST {
                break;
            }
        }
        assert!(state.wood() > STARTING_WOOD - SAWMILL_COST);
        assert!(
            state
                .people
                .iter()
                .any(|(_, person)| person.cargo_capacity > 0)
        );
    }

    #[test]
    fn first_night_starts_after_sixty_seconds_of_day_ticks() {
        let mut state = GameState::new(7);

        for _ in 0..DAY_LENGTH_TICKS - 1 {
            state
                .apply(Command::AdvanceTick)
                .expect("day tick should advance");
        }

        assert_eq!(state.wave(), 0);
        assert_eq!(state.raider_count(), 0);
        assert_eq!(state.day_ticks_remaining(), 1);
        assert!(!state.is_night());

        state
            .apply(Command::AdvanceTick)
            .expect("last day tick should begin the night");

        assert_eq!(state.wave(), 1);
        assert_eq!(state.raider_count(), 4);
        assert_eq!(state.day_ticks_remaining(), 0);
        assert!(state.is_night());
    }

    #[test]
    fn night_pauses_production_and_person_logistics() {
        let mut state = GameState::new(13);
        state
            .apply(Command::PlaceSawmill { x: 2, z: 2 })
            .expect("sawmill should build");
        let sawmill = sawmill_entity(&state);
        state
            .storage
            .get_mut(entity_key(sawmill))
            .expect("sawmill should have storage")
            .wood = u32::from(PERSON_CARRY_CAPACITY);
        state.run_person_logistics_system();
        state.day_ticks_remaining = 1;

        state
            .apply(Command::AdvanceTick)
            .expect("last day tick should begin the night");
        assert!(state.is_night());

        let producer_before = *state
            .producers
            .get(entity_key(sawmill))
            .expect("sawmill should have a producer");
        let person = state
            .people
            .iter()
            .find_map(|(key, person)| {
                (person.state != PersonState::IdleAtTownHall).then_some(key_entity(key))
            })
            .expect("a carrier should be active before night");
        let person_before = *state
            .people
            .get(entity_key(person))
            .expect("person should exist");
        let movement_before = *state
            .movements
            .get(entity_key(person))
            .expect("active carrier should have movement");
        let transform_before = *state
            .transforms
            .get(entity_key(person))
            .expect("person should have a transform");

        let Event::TickAdvanced {
            wood_produced,
            wood_picked_up,
            wood_delivered,
            ..
        } = state
            .apply(Command::AdvanceTick)
            .expect("night tick should advance")
        else {
            unreachable!();
        };

        assert_eq!(wood_produced, 0);
        assert_eq!(wood_picked_up, 0);
        assert_eq!(wood_delivered, 0);
        assert_eq!(
            *state
                .producers
                .get(entity_key(sawmill))
                .expect("sawmill should still have a producer"),
            producer_before
        );
        assert_eq!(
            *state
                .people
                .get(entity_key(person))
                .expect("person should still exist"),
            person_before
        );
        assert_eq!(
            *state
                .movements
                .get(entity_key(person))
                .expect("carrier movement should remain paused"),
            movement_before
        );
        assert_eq!(
            *state
                .transforms
                .get(entity_key(person))
                .expect("carrier position should remain paused"),
            transform_before
        );
    }

    #[test]
    fn replay_includes_deterministic_people_logistics() {
        let mut commands = vec![Command::PlaceSawmill { x: 2, z: 2 }];
        commands.extend(std::iter::repeat_n(Command::AdvanceTick, 80));
        let first = replay(9, &commands).expect("valid replay");
        let second = replay(9, &commands).expect("same replay remains valid");
        assert_eq!(first, second);
        assert_eq!(first.checksum(), second.checksum());
    }

    #[test]
    fn houses_are_locked_until_ten_completed_waves() {
        let mut state = GameState::new(5);
        let before = state.clone();
        assert_eq!(
            state.apply(Command::PlaceHouse { x: 2, z: 2 }),
            Err(GameError::HouseLocked)
        );
        assert_eq!(state, before);

        state.completed_waves = HOUSE_UNLOCK_COMPLETED_WAVES;
        state
            .apply(Command::PlaceHouse { x: 2, z: 2 })
            .expect("house should unlock after ten completed waves");
        assert_eq!(state.house_count(), 1);
        assert_eq!(state.people_count(), 4);
        assert_eq!(state.population_capacity(), 4);
    }

    #[test]
    fn completing_wave_ten_unlocks_houses() {
        let mut state = GameState::new(3);
        state.wave = HOUSE_UNLOCK_COMPLETED_WAVES;
        state.completed_waves = HOUSE_UNLOCK_COMPLETED_WAVES - 1;
        state.day_ticks_remaining = 0;
        state.spawn_raider(Edge::North);
        let raider = state
            .raiders
            .keys()
            .map(key_entity)
            .min()
            .expect("raider should exist");
        let town = town_center();
        state.movements.insert(
            entity_key(raider),
            Movement {
                from: Cell::new(town.x, town.z - 2),
                to: Cell::new(town.x, town.z - 1),
                progress_milli: 900,
                speed_milli: STANDARD_RULES.raids.speed_milli,
            },
        );
        state.transforms.insert(
            entity_key(raider),
            interpolate_transform(
                *state
                    .movements
                    .get(entity_key(raider))
                    .expect("movement should exist"),
            ),
        );

        let Event::TickAdvanced { completed_wave, .. } = state
            .apply(Command::AdvanceTick)
            .expect("tick should advance")
        else {
            unreachable!();
        };
        assert_eq!(completed_wave, Some(HOUSE_UNLOCK_COMPLETED_WAVES));
        assert_eq!(state.day_ticks_remaining(), DAY_LENGTH_TICKS);
        assert!(!state.is_night());
        assert!(state.houses_unlocked());
    }

    #[test]
    fn house_build_is_transactional_when_wood_is_insufficient() {
        let mut state = GameState::new(11);
        state.completed_waves = HOUSE_UNLOCK_COMPLETED_WAVES;
        state
            .apply(Command::PlaceSawmill { x: 2, z: 2 })
            .expect("sawmill should build");
        state
            .apply(Command::PlaceTower {
                x: 3,
                z: 2,
                archetype: TowerArchetype::Cannon,
            })
            .expect("tower should build");
        let before = state.clone();
        assert_eq!(
            state.apply(Command::PlaceHouse { x: 4, z: 2 }),
            Err(GameError::InsufficientWood)
        );
        assert_eq!(state, before);
    }

    #[test]
    fn projectiles_eventually_impact_moving_raiders() {
        let mut state = GameState::new(11);
        state
            .apply(Command::PlaceTower {
                x: 7,
                z: 2,
                archetype: TowerArchetype::Arrow,
            })
            .expect("tower should build");
        state.apply(Command::StartWave).expect("wave should start");

        let mut impacts = 0_u16;
        for _ in 0..8 {
            let Event::TickAdvanced {
                impacts: tick_impacts,
                ..
            } = state
                .apply(Command::AdvanceTick)
                .expect("tick should advance")
            else {
                unreachable!();
            };
            impacts = impacts.saturating_add(tick_impacts);
        }
        assert!(impacts > 0);
    }

    #[test]
    fn rejected_builds_preserve_state() {
        let mut state = GameState::new(9);
        let before = state.clone();
        assert_eq!(
            state.apply(Command::PlaceTower {
                x: town_center().x,
                z: town_center().z,
                archetype: TowerArchetype::Arrow,
            }),
            Err(GameError::ProtectedCell)
        );
        assert_eq!(state, before);
    }

    #[test]
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
        assert_eq!(
            state.population_capacity(),
            rules.population.base_capacity + 3
        );
        assert_eq!(
            state.people_count(),
            usize::from(rules.population.starting_people + 1)
        );

        state
            .apply(Command::AdvanceTick)
            .expect("first short day tick should advance");
        assert_eq!(state.wave(), 0);
        state
            .apply(Command::AdvanceTick)
            .expect("second short day tick should auto-start a wave");
        assert_eq!(state.wave(), 1);
        assert_eq!(
            state.raider_count(),
            usize::from(rules.raids.raiders_per_wave)
        );
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

    #[test]
    fn integer_square_root_is_deterministic_at_boundaries() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(2), 1);
        assert_eq!(integer_sqrt(4), 2);
        assert_eq!(integer_sqrt(15), 3);
        assert_eq!(integer_sqrt(16), 4);
    }
}

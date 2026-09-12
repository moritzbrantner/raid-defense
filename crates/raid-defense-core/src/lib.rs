#![forbid(unsafe_code)]

use std::collections::VecDeque;

use collection_kernels::{SparseMap, SparseSet};

pub type EntityId = u32;

pub const GRID_WIDTH: i16 = 17;
pub const GRID_HEIGHT: i16 = 13;
pub const CELL_SCALE: i32 = 1_000;
pub const STARTING_WOOD: u32 = 120;
pub const TOWN_WOOD_CAPACITY: u32 = 500;
pub const SAWMILL_COST: u32 = 40;
pub const SAWMILL_OUTPUT: u16 = 2;
pub const SAWMILL_INTERVAL_TICKS: u16 = 10;
pub const ARROW_TOWER_COST: u32 = 25;
pub const CANNON_TOWER_COST: u32 = 45;
pub const MAX_TOWER_LEVEL: u8 = 3;
pub const TOWN_MAX_HEALTH: u16 = 250;

const TOWER_MAX_HEALTH: u16 = 100;
const SAWMILL_MAX_HEALTH: u16 = 80;
const RAIDER_BASE_HEALTH: u16 = 30;
const RAIDER_BASE_DAMAGE: u16 = 10;
const RAIDER_SPEED_MILLI: u16 = 250;
const RAIDER_BASE_WOOD_STEAL: u32 = 15;
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
pub enum BuildingKind {
    TownHall,
    Tower,
    Sawmill,
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
        wood_stolen: u16,
        town_damage: u16,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameError {
    OutOfBounds,
    CellOccupied,
    ProtectedCell,
    PathBlocked,
    InsufficientWood,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameSnapshot {
    pub seed: u64,
    pub tick: u64,
    pub wood: u32,
    pub wood_capacity: u32,
    pub wave: u32,
    pub town_health: u16,
    pub town_max_health: u16,
    pub grid_width: i16,
    pub grid_height: i16,
    pub entities: Vec<EntitySnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    seed: u64,
    tick: u64,
    wave: u32,
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
    alive: SparseSet,
}

impl GameState {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let mut state = Self {
            seed,
            tick: 0,
            wave: 0,
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
                current: TOWN_MAX_HEALTH,
                maximum: TOWN_MAX_HEALTH,
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
                wood: STARTING_WOOD,
                wood_capacity: TOWN_WOOD_CAPACITY,
            },
        );
        state
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
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
    pub fn projectile_count(&self) -> usize {
        self.projectiles.iter().count()
    }

    pub fn apply(&mut self, command: Command) -> Result<Event, GameError> {
        match command {
            Command::PlaceTower { x, z, archetype } => self.place_tower(Cell::new(x, z), archetype),
            Command::PlaceSawmill { x, z } => self.place_sawmill(Cell::new(x, z)),
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
            town_health: self.town_health(),
            town_max_health: TOWN_MAX_HEALTH,
            grid_width: GRID_WIDTH,
            grid_height: GRID_HEIGHT,
            entities,
        }
    }

    #[must_use]
    pub fn checksum(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        feed_u64(&mut hash, self.seed);
        feed_u64(&mut hash, self.tick);
        feed_u64(&mut hash, u64::from(self.wave));
        feed_u64(&mut hash, u64::from(self.next_entity));

        for entity in &self.snapshot().entities {
            feed_u64(&mut hash, u64::from(entity.id));
            feed_byte(
                &mut hash,
                match entity.kind {
                    EntityKind::TownHall => 0,
                    EntityKind::Tower => 1,
                    EntityKind::Sawmill => 2,
                    EntityKind::Raider => 3,
                    EntityKind::Projectile => 4,
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
        }

        hash
    }

    fn place_tower(&mut self, cell: Cell, archetype: TowerArchetype) -> Result<Event, GameError> {
        self.validate_build_cell(cell)?;
        let cost = tower_cost(archetype);
        self.require_wood(cost)?;
        if !self.routes_remain_open(Some(cell)) {
            return Err(GameError::PathBlocked);
        }

        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(cell));
        self.health.insert(
            entity_key(entity),
            Health {
                current: TOWER_MAX_HEALTH,
                maximum: TOWER_MAX_HEALTH,
            },
        );
        self.attacks
            .insert(entity_key(entity), attack_for(archetype, 1, 0));
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
        self.require_wood(SAWMILL_COST)?;
        if !self.routes_remain_open(Some(cell)) {
            return Err(GameError::PathBlocked);
        }

        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(cell));
        self.health.insert(
            entity_key(entity),
            Health {
                current: SAWMILL_MAX_HEALTH,
                maximum: SAWMILL_MAX_HEALTH,
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
                amount: SAWMILL_OUTPUT,
                interval_ticks: SAWMILL_INTERVAL_TICKS,
                progress_ticks: 0,
            },
        );
        self.spend_wood(SAWMILL_COST);

        Ok(Event::SawmillBuilt {
            entity,
            cell,
            wood_cost: SAWMILL_COST,
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
        let cost =
            tower_upgrade_cost(tower.archetype, tower.level).ok_or(GameError::MaxTowerLevel)?;
        self.require_wood(cost)?;

        let next_level = tower.level + 1;
        let cooldown_remaining = self
            .attacks
            .get(entity_key(entity))
            .map_or(0, |attack| attack.cooldown_remaining);
        let next_attack = attack_for(tower.archetype, next_level, cooldown_remaining);
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
        if self.cell_has_building(cell) || self.raider_uses_cell(cell) {
            return Err(GameError::CellOccupied);
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

    fn store_wood(&mut self, amount: u32) -> u32 {
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
    }

    fn advance_tick(&mut self) -> Event {
        self.tick = self.tick.saturating_add(1);
        let wood_produced = self.run_resource_production_system();
        let shots = self.run_tower_attack_system();
        let (impacts, killed) = self.run_projectile_system();
        let kills = u16::try_from(killed.len()).unwrap_or(u16::MAX);
        for entity in killed {
            self.despawn_raider(entity);
        }
        let (wood_stolen, town_damage) = self.run_movement_system();

        Event::TickAdvanced {
            tick: self.tick,
            shots,
            impacts,
            kills,
            wood_produced,
            wood_stolen,
            town_damage,
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
            match producer.resource {
                ResourceKind::Wood => {
                    let stored = self.store_wood(u32::from(producer.amount));
                    produced_total =
                        produced_total.saturating_add(u16::try_from(stored).unwrap_or(u16::MAX));
                }
            }
        }

        produced_total
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

    fn run_movement_system(&mut self) -> (u16, u16) {
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

                let Some(next) = self.next_path_step(movement.from, None) else {
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
        RAIDER_BASE_WOOD_STEAL.saturating_add(self.wave.saturating_sub(1).saturating_mul(2))
    }

    fn spawn_raider(&mut self, edge: Edge) {
        let from = edge.spawn_cell();
        let to = self
            .next_path_step(from, None)
            .expect("validated building placements keep every edge connected to town hall");
        let entity = self.allocate_entity();
        let wave_bonus = u16::try_from(self.wave.saturating_sub(1))
            .unwrap_or(u16::MAX)
            .saturating_mul(3);
        let max_health = RAIDER_BASE_HEALTH.saturating_add(wave_bonus);
        let damage_bonus = u16::try_from(self.wave / 3).unwrap_or(u16::MAX);
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
                damage: RAIDER_BASE_DAMAGE.saturating_add(damage_bonus),
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
                speed_milli: RAIDER_SPEED_MILLI,
            },
        );
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

        if let Some(building) = self.buildings.get(key) {
            let tower = self.towers.get(key).copied();
            return Some(EntitySnapshot {
                id: entity,
                kind: match building.kind {
                    BuildingKind::TownHall => EntityKind::TownHall,
                    BuildingKind::Tower => EntityKind::Tower,
                    BuildingKind::Sawmill => EntityKind::Sawmill,
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
                    .and_then(|tower| tower_upgrade_cost(tower.archetype, tower.level)),
                projectile_target: None,
                stored_wood: storage.wood,
                wood_capacity: storage.wood_capacity,
                production_resource: producer.map(|producer| producer.resource),
                production_amount: producer.map_or(0, |producer| producer.amount),
                production_interval_ticks: producer.map_or(0, |producer| producer.interval_ticks),
                production_progress_ticks: producer.map_or(0, |producer| producer.progress_ticks),
            });
        }

        if let Some(movement) = self.movements.get(key) {
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
        })
    }

    fn tower_entity_at(&self, cell: Cell) -> Option<EntityId> {
        self.buildings.iter().find_map(|(key, building)| {
            (building.kind == BuildingKind::Tower && building.cell == cell)
                .then_some(key_entity(key))
        })
    }

    fn cell_has_building(&self, cell: Cell) -> bool {
        self.buildings.iter().any(|(_, building)| {
            matches!(building.kind, BuildingKind::Tower | BuildingKind::Sawmill)
                && building.cell == cell
        })
    }

    fn raider_uses_cell(&self, cell: Cell) -> bool {
        self.movements
            .iter()
            .any(|(_, movement)| movement.from == cell || movement.to == cell)
    }

    fn routes_remain_open(&self, extra_block: Option<Cell>) -> bool {
        Edge::ALL.into_iter().all(|edge| {
            self.next_path_step(edge.spawn_cell(), extra_block)
                .is_some()
        }) && self
            .movements
            .iter()
            .all(|(_, movement)| self.next_path_step(movement.from, extra_block).is_some())
    }

    fn next_path_step(&self, start: Cell, extra_block: Option<Cell>) -> Option<Cell> {
        if is_town_cell(start) {
            return Some(start);
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
            if is_town_cell(cell) {
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
    Attack {
        damage: stats.damage,
        range_milli: stats.range_milli,
        cooldown_ticks: stats.cooldown_ticks,
        cooldown_remaining: cooldown_remaining.min(stats.cooldown_ticks),
        projectile_speed_milli: stats.projectile_speed_milli,
    }
}

pub fn replay(seed: u64, commands: &[Command]) -> Result<GameState, GameError> {
    let mut state = GameState::new(seed);
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

    fn tower_at(state: &GameState, cell: Cell) -> EntitySnapshot {
        state
            .snapshot()
            .entities
            .into_iter()
            .find(|entity| entity.kind == EntityKind::Tower && entity.cell == cell)
            .expect("tower should exist at requested cell")
    }

    #[test]
    fn same_seed_and_commands_replay_identically_with_economy() {
        let mut commands = vec![
            Command::PlaceSawmill { x: 2, z: 2 },
            Command::PlaceTower {
                x: 7,
                z: 2,
                archetype: TowerArchetype::Arrow,
            },
            Command::StartWave,
        ];
        commands.extend(std::iter::repeat_n(Command::AdvanceTick, 30));

        let first = replay(7, &commands).expect("valid replay");
        let second = replay(7, &commands).expect("same replay remains valid");

        assert_eq!(first, second);
        assert_eq!(first.checksum(), second.checksum());
    }

    #[test]
    fn sawmill_spends_then_produces_wood_into_town_hall_storage() {
        let mut state = GameState::new(7);
        state
            .apply(Command::PlaceSawmill { x: 2, z: 2 })
            .expect("sawmill should build");
        assert_eq!(state.wood(), STARTING_WOOD - SAWMILL_COST);
        assert_eq!(state.sawmill_count(), 1);

        for _ in 0..SAWMILL_INTERVAL_TICKS {
            state
                .apply(Command::AdvanceTick)
                .expect("tick should advance");
        }

        assert_eq!(
            state.wood(),
            STARTING_WOOD - SAWMILL_COST + u32::from(SAWMILL_OUTPUT)
        );
        let town = state
            .snapshot()
            .entities
            .into_iter()
            .find(|entity| entity.kind == EntityKind::TownHall)
            .expect("town hall should exist");
        assert_eq!(town.stored_wood, state.wood());
        assert_eq!(town.wood_capacity, TOWN_WOOD_CAPACITY);
    }

    #[test]
    fn towers_and_upgrades_consume_wood() {
        let mut state = GameState::new(7);
        state
            .apply(Command::PlaceTower {
                x: 2,
                z: 2,
                archetype: TowerArchetype::Arrow,
            })
            .expect("arrow tower should build");
        assert_eq!(state.wood(), STARTING_WOOD - ARROW_TOWER_COST);

        state
            .apply(Command::UpgradeTower { x: 2, z: 2 })
            .expect("upgrade should spend wood");
        assert_eq!(state.wood(), STARTING_WOOD - ARROW_TOWER_COST - 20);
    }

    #[test]
    fn raiders_steal_town_hall_wood_before_damaging_it() {
        let mut state = GameState::new(11);
        let health_before = state.town_health();
        state.apply(Command::StartWave).expect("wave should start");

        let mut stolen = 0_u16;
        for _ in 0..40 {
            let Event::TickAdvanced {
                wood_stolen,
                town_damage,
                ..
            } = state
                .apply(Command::AdvanceTick)
                .expect("tick should advance")
            else {
                unreachable!();
            };
            stolen = stolen.saturating_add(wood_stolen);
            assert_eq!(
                town_damage, 0,
                "stored wood should be stolen before health is damaged"
            );
            if state.raider_count() == 0 {
                break;
            }
        }

        assert!(stolen > 0);
        assert!(state.wood() < STARTING_WOOD);
        assert_eq!(state.town_health(), health_before);
    }

    #[test]
    fn raiders_damage_town_hall_when_there_is_no_wood_left() {
        let mut state = GameState::new(11);
        state
            .storage
            .get_mut(entity_key(TOWN_ENTITY))
            .expect("town storage should exist")
            .wood = 0;
        let health_before = state.town_health();
        state.apply(Command::StartWave).expect("wave should start");

        let mut damage = 0_u16;
        for _ in 0..40 {
            let Event::TickAdvanced { town_damage, .. } = state
                .apply(Command::AdvanceTick)
                .expect("tick should advance")
            else {
                unreachable!();
            };
            damage = damage.saturating_add(town_damage);
            if state.raider_count() == 0 {
                break;
            }
        }

        assert!(damage > 0);
        assert!(state.town_health() < health_before);
    }

    #[test]
    fn tower_archetypes_keep_distinct_stats() {
        let mut state = GameState::new(7);
        state
            .apply(Command::PlaceTower {
                x: 2,
                z: 2,
                archetype: TowerArchetype::Arrow,
            })
            .expect("arrow tower should build");
        state
            .apply(Command::PlaceTower {
                x: 3,
                z: 2,
                archetype: TowerArchetype::Cannon,
            })
            .expect("cannon tower should build");

        let arrow = tower_at(&state, Cell::new(2, 2));
        let cannon = tower_at(&state, Cell::new(3, 2));
        assert_eq!(arrow.tower_archetype, Some(TowerArchetype::Arrow));
        assert_eq!(cannon.tower_archetype, Some(TowerArchetype::Cannon));
        assert!(arrow.attack_damage < cannon.attack_damage);
        assert!(arrow.attack_range_milli < cannon.attack_range_milli);
    }

    #[test]
    fn projectiles_are_authoritative_and_cleanup_with_targets() {
        let mut state = GameState::new(11);
        state
            .apply(Command::PlaceTower {
                x: 7,
                z: 2,
                archetype: TowerArchetype::Arrow,
            })
            .expect("tower should build");
        state.apply(Command::StartWave).expect("wave should start");

        let Event::TickAdvanced { shots, impacts, .. } = state
            .apply(Command::AdvanceTick)
            .expect("tick should advance")
        else {
            unreachable!();
        };
        assert!(shots > 0);
        assert_eq!(impacts, 0);
        assert!(state.projectile_count() > 0);

        for _ in 0..60 {
            state
                .apply(Command::AdvanceTick)
                .expect("tick should advance");
            if state.raider_count() == 0 {
                break;
            }
        }
        assert_eq!(state.raider_count(), 0);
        assert_eq!(state.projectile_count(), 0);
    }

    #[test]
    fn rejected_builds_and_upgrades_are_transactional() {
        let mut state = GameState::new(9);
        let before = state.clone();
        assert_eq!(
            state.apply(Command::PlaceSawmill {
                x: town_center().x,
                z: town_center().z,
            }),
            Err(GameError::ProtectedCell)
        );
        assert_eq!(state, before);

        let before_upgrade = state.clone();
        assert_eq!(
            state.apply(Command::UpgradeTower { x: 2, z: 2 }),
            Err(GameError::NoTower)
        );
        assert_eq!(state, before_upgrade);
    }

    #[test]
    fn buildings_cannot_spend_more_wood_than_the_town_hall_stores() {
        let mut state = GameState::new(7);
        state
            .storage
            .get_mut(entity_key(TOWN_ENTITY))
            .expect("town storage should exist")
            .wood = 10;
        let before = state.clone();
        assert_eq!(
            state.apply(Command::PlaceTower {
                x: 2,
                z: 2,
                archetype: TowerArchetype::Arrow,
            }),
            Err(GameError::InsufficientWood)
        );
        assert_eq!(state, before);
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

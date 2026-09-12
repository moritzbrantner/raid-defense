#![forbid(unsafe_code)]

use std::collections::VecDeque;

use collection_kernels::{SparseMap, SparseSet};

pub type EntityId = u32;

pub const GRID_WIDTH: i16 = 17;
pub const GRID_HEIGHT: i16 = 13;
pub const CELL_SCALE: i32 = 1_000;
pub const STARTING_GOLD: u32 = 120;
pub const TOWER_COST: u32 = 25;
pub const TOWN_MAX_HEALTH: u16 = 250;

const TOWER_MAX_HEALTH: u16 = 100;
const RAIDER_BASE_HEALTH: u16 = 30;
const RAIDER_BASE_DAMAGE: u16 = 10;
const RAIDER_SPEED_MILLI: u16 = 250;
const TOWER_DAMAGE: u16 = 10;
const TOWER_RANGE_MILLI: i32 = 3 * CELL_SCALE;
const TOWER_COOLDOWN_TICKS: u8 = 5;
const KILL_REWARD: u32 = 5;
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildingKind {
    Town,
    GuardTower,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Building {
    pub kind: BuildingKind,
    pub cell: Cell,
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
pub enum Command {
    PlaceTower { x: i16, z: i16 },
    StartWave,
    AdvanceTick,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    TowerBuilt { entity: EntityId, cell: Cell },
    WaveStarted { wave: u32, raiders: u16 },
    TickAdvanced {
        tick: u64,
        shots: u16,
        kills: u16,
        town_damage: u16,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameError {
    OutOfBounds,
    CellOccupied,
    ProtectedCell,
    PathBlocked,
    InsufficientGold,
    RaidersStillActive,
    GameOver,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityKind {
    Town,
    Tower,
    Raider,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameSnapshot {
    pub seed: u64,
    pub tick: u64,
    pub gold: u32,
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
    gold: u32,
    wave: u32,
    next_entity: EntityId,
    transforms: SparseMap<Transform>,
    health: SparseMap<Health>,
    attacks: SparseMap<Attack>,
    buildings: SparseMap<Building>,
    raiders: SparseMap<Raider>,
    movements: SparseMap<Movement>,
    alive: SparseSet,
}

impl GameState {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let mut state = Self {
            seed,
            tick: 0,
            gold: STARTING_GOLD,
            wave: 0,
            next_entity: 1,
            transforms: SparseMap::new(),
            health: SparseMap::new(),
            attacks: SparseMap::new(),
            buildings: SparseMap::new(),
            raiders: SparseMap::new(),
            movements: SparseMap::new(),
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
                kind: BuildingKind::Town,
                cell: town_cell,
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
    pub const fn gold(&self) -> u32 {
        self.gold
    }

    #[must_use]
    pub const fn wave(&self) -> u32 {
        self.wave
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
        self.buildings
            .iter()
            .filter(|(_, building)| building.kind == BuildingKind::GuardTower)
            .count()
    }

    pub fn apply(&mut self, command: Command) -> Result<Event, GameError> {
        match command {
            Command::PlaceTower { x, z } => self.place_tower(Cell::new(x, z)),
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
            gold: self.gold,
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
        feed_u64(&mut hash, u64::from(self.gold));
        feed_u64(&mut hash, u64::from(self.wave));
        feed_u64(&mut hash, u64::from(self.next_entity));

        for entity in &self.snapshot().entities {
            feed_u64(&mut hash, u64::from(entity.id));
            feed_byte(
                &mut hash,
                match entity.kind {
                    EntityKind::Town => 0,
                    EntityKind::Tower => 1,
                    EntityKind::Raider => 2,
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
        }

        hash
    }

    fn place_tower(&mut self, cell: Cell) -> Result<Event, GameError> {
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
        if self.gold < TOWER_COST {
            return Err(GameError::InsufficientGold);
        }
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
        self.attacks.insert(
            entity_key(entity),
            Attack {
                damage: TOWER_DAMAGE,
                range_milli: TOWER_RANGE_MILLI,
                cooldown_ticks: TOWER_COOLDOWN_TICKS,
                cooldown_remaining: 0,
            },
        );
        self.buildings.insert(
            entity_key(entity),
            Building {
                kind: BuildingKind::GuardTower,
                cell,
            },
        );
        self.gold -= TOWER_COST;

        Ok(Event::TowerBuilt { entity, cell })
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
        let (shots, killed) = self.run_tower_attack_system();
        let kills = u16::try_from(killed.len()).unwrap_or(u16::MAX);
        for entity in killed {
            self.despawn_raider(entity);
            self.gold = self.gold.saturating_add(KILL_REWARD);
        }
        let town_damage = self.run_movement_system();

        Event::TickAdvanced {
            tick: self.tick,
            shots,
            kills,
            town_damage,
        }
    }

    fn run_tower_attack_system(&mut self) -> (u16, Vec<EntityId>) {
        let mut tower_ids = self
            .buildings
            .iter()
            .filter_map(|(key, building)| {
                (building.kind == BuildingKind::GuardTower).then_some(key_entity(key))
            })
            .collect::<Vec<_>>();
        tower_ids.sort_unstable();

        let mut shots = 0_u16;
        let mut killed = Vec::new();
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
            shots = shots.saturating_add(1);
            attack.cooldown_remaining = attack.cooldown_ticks;
            self.attacks.insert(entity_key(tower), attack);

            let health = self
                .health
                .get_mut(entity_key(target))
                .expect("raiders always have health");
            health.current = health.current.saturating_sub(attack.damage);
            if health.current == 0 {
                killed.push(target);
            }
        }
        killed.sort_unstable();
        killed.dedup();
        (shots, killed)
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

    fn run_movement_system(&mut self) -> u16 {
        let mut raiders = self.raiders.keys().map(key_entity).collect::<Vec<_>>();
        raiders.sort_unstable();
        let mut town_damage = 0_u16;
        let mut reached_town = Vec::new();

        for entity in raiders {
            let Some(mut movement) = self.movements.get(entity_key(entity)).copied() else {
                continue;
            };
            movement.progress_milli = movement
                .progress_milli
                .saturating_add(movement.speed_milli);

            if movement.progress_milli >= CELL_SCALE as u16 {
                movement.progress_milli -= CELL_SCALE as u16;
                movement.from = movement.to;
                if is_town_cell(movement.from) {
                    let damage = self
                        .attacks
                        .get(entity_key(entity))
                        .map_or(0, |attack| attack.damage);
                    town_damage = town_damage.saturating_add(damage);
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

        if town_damage > 0 {
            if let Some(health) = self.health.get_mut(entity_key(TOWN_ENTITY)) {
                health.current = health.current.saturating_sub(town_damage);
            }
        }
        for entity in reached_town {
            self.despawn_raider(entity);
        }

        town_damage
    }

    fn spawn_raider(&mut self, edge: Edge) {
        let from = edge.spawn_cell();
        let to = self
            .next_path_step(from, None)
            .expect("validated tower placements keep every edge connected to town");
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
        let key = entity_key(entity);
        self.transforms.remove(key);
        self.health.remove(key);
        self.attacks.remove(key);
        self.raiders.remove(key);
        self.movements.remove(key);
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
        });
        if let Some(building) = self.buildings.get(key) {
            return Some(EntitySnapshot {
                id: entity,
                kind: match building.kind {
                    BuildingKind::Town => EntityKind::Town,
                    BuildingKind::GuardTower => EntityKind::Tower,
                },
                x_milli: transform.x_milli,
                z_milli: transform.z_milli,
                cell: building.cell,
                health: health.current,
                max_health: health.maximum,
                attack_damage: attack.damage,
                attack_range_milli: attack.range_milli,
            });
        }
        let movement = self.movements.get(key)?;
        Some(EntitySnapshot {
            id: entity,
            kind: EntityKind::Raider,
            x_milli: transform.x_milli,
            z_milli: transform.z_milli,
            cell: movement.from,
            health: health.current,
            max_health: health.maximum,
            attack_damage: attack.damage,
            attack_range_milli: attack.range_milli,
        })
    }

    fn cell_has_building(&self, cell: Cell) -> bool {
        self.buildings.iter().any(|(_, building)| {
            building.kind == BuildingKind::GuardTower && building.cell == cell
        })
    }

    fn raider_uses_cell(&self, cell: Cell) -> bool {
        self.movements
            .iter()
            .any(|(_, movement)| movement.from == cell || movement.to == cell)
    }

    fn routes_remain_open(&self, extra_block: Option<Cell>) -> bool {
        Edge::ALL
            .into_iter()
            .all(|edge| self.next_path_step(edge.spawn_cell(), extra_block).is_some())
            && self.movements.iter().all(|(_, movement)| {
                self.next_path_step(movement.from, extra_block).is_some()
            })
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

    #[test]
    fn same_seed_and_commands_replay_identically() {
        let mut commands = vec![
            Command::PlaceTower { x: 7, z: 2 },
            Command::StartWave,
        ];
        commands.extend(std::iter::repeat_n(Command::AdvanceTick, 24));

        let first = replay(7, &commands).expect("valid replay");
        let second = replay(7, &commands).expect("same replay should remain valid");

        assert_eq!(first, second);
        assert_eq!(first.checksum(), second.checksum());
    }

    #[test]
    fn rejected_tower_placement_is_transactional() {
        let mut state = GameState::new(11);
        let before = state.clone();
        let center = town_center();

        assert_eq!(
            state.apply(Command::PlaceTower {
                x: center.x,
                z: center.z,
            }),
            Err(GameError::ProtectedCell)
        );
        assert_eq!(state, before);
    }

    #[test]
    fn a_complete_wall_that_seals_an_edge_is_rejected() {
        let mut state = GameState::new(3);
        state.gold = 10_000;
        for z in 0..GRID_HEIGHT - 1 {
            state
                .apply(Command::PlaceTower { x: 1, z })
                .expect("wall remains open until final cell");
        }

        assert_eq!(
            state.apply(Command::PlaceTower {
                x: 1,
                z: GRID_HEIGHT - 1,
            }),
            Err(GameError::PathBlocked)
        );
    }

    #[test]
    fn waves_spawn_from_all_four_edges() {
        let mut state = GameState::new(5);
        state.apply(Command::StartWave).expect("wave should start");

        let mut edges = state
            .raiders
            .iter()
            .map(|(_, raider)| raider.edge)
            .collect::<Vec<_>>();
        edges.sort_by_key(|edge| match edge {
            Edge::North => 0,
            Edge::East => 1,
            Edge::South => 2,
            Edge::West => 3,
        });
        assert_eq!(edges, Edge::ALL);
    }

    #[test]
    fn tower_attack_system_can_kill_a_raider() {
        let mut state = GameState::new(9);
        state
            .apply(Command::PlaceTower { x: 7, z: 2 })
            .expect("tower should build");
        state.apply(Command::StartWave).expect("wave should start");

        let mut kills = 0_u16;
        for _ in 0..40 {
            let Event::TickAdvanced {
                kills: tick_kills,
                ..
            } = state.apply(Command::AdvanceTick).expect("tick should advance")
            else {
                unreachable!();
            };
            kills = kills.saturating_add(tick_kills);
        }
        assert!(kills > 0);
    }

    #[test]
    fn unopposed_raiders_damage_the_town() {
        let mut state = GameState::new(13);
        state.apply(Command::StartWave).expect("wave should start");
        for _ in 0..40 {
            state
                .apply(Command::AdvanceTick)
                .expect("tick should advance");
        }
        assert!(state.town_health() < TOWN_MAX_HEALTH);
    }
}

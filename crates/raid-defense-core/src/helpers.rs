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
        PersonState::ToStorage => 2,
        PersonState::ToConstructionStorage => 3,
        PersonState::ToConstructionSite => 4,
        PersonState::ToTownHall => 5,
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

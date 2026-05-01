use super::*;

pub(crate) fn generate_seeded_terrain(state: &mut GameState, seed: u64) -> Result<(), EngineError> {
    let mut rng = SeededRng::new(seed);
    let protected = protected_terrain_tiles();

    for y in 0..RAID_DEFENSE_SIZE {
        for x in 0..RAID_DEFENSE_SIZE {
            let location = MapLocation::new(x, y);
            state.set_ground_elevation(location, 0);
            state.set_tile(PLAINS, location)?;
        }
    }

    // Poisson-disk-like center placement keeps clusters spread apart.
    let water_centers =
        poisson_disk_points(&mut rng, 6, 5, 2_000, |x, y| !protected.contains(&(x, y)));
    for center in &water_centers {
        let radius = 2 + rng.next_bounded_i32(3);
        paint_terrain_blob(state, WATER_TILE, *center, radius, &protected, &mut rng)?;
    }

    let forest_centers = poisson_disk_points(&mut rng, 4, 12, 2_500, |x, y| {
        !protected.contains(&(x, y))
            && water_centers
                .iter()
                .all(|water| grid_distance((x, y), *water) >= 3)
    });
    for center in forest_centers {
        let radius = 2 + rng.next_bounded_i32(4);
        paint_terrain_blob(state, FOREST, center, radius, &protected, &mut rng)?;
    }

    if terrain_kind_count(state, WATER_TILE) == 0 {
        for y in (0..RAID_DEFENSE_SIZE).rev() {
            for x in (0..RAID_DEFENSE_SIZE).rev() {
                if protected.contains(&(x, y)) {
                    continue;
                }
                state.set_tile(WATER_TILE, MapLocation::new(x, y))?;
                break;
            }
            if terrain_kind_count(state, WATER_TILE) > 0 {
                break;
            }
        }
    }

    if terrain_kind_count(state, FOREST) == 0 {
        for y in 0..RAID_DEFENSE_SIZE {
            for x in 0..RAID_DEFENSE_SIZE {
                if protected.contains(&(x, y)) {
                    continue;
                }
                let location = MapLocation::new(x, y);
                if state
                    .tile(location)
                    .is_some_and(|tile| tile.kind.as_str() != WATER_TILE)
                {
                    state.set_tile(FOREST, location)?;
                    break;
                }
            }
            if terrain_kind_count(state, FOREST) > 0 {
                break;
            }
        }
    }

    // Keep key gameplay anchors consistently buildable.
    for &(x, y) in &protected {
        state.set_tile(PLAINS, MapLocation::new(x, y))?;
    }

    Ok(())
}

fn protected_terrain_tiles() -> BTreeSet<(i32, i32)> {
    let mut protected = BTreeSet::new();

    for waypoint in imperial_road() {
        mark_grid_radius(&mut protected, waypoint, 1);
    }

    for location in [
        MapLocation::new(6, 10),
        MapLocation::new(5, 9),
        MapLocation::new(7, 11),
        MapLocation::new(8, 9),
        MapLocation::new(23, 18),
        MapLocation::new(21, 7),
        MapLocation::new(10, 10),
    ] {
        mark_grid_radius(&mut protected, location, 2);
    }

    protected
}

fn mark_grid_radius(set: &mut BTreeSet<(i32, i32)>, center: MapLocation, radius: i32) {
    for y in 0..RAID_DEFENSE_SIZE {
        for x in 0..RAID_DEFENSE_SIZE {
            if grid_distance((x, y), (center.x, center.y)) <= radius {
                set.insert((x, y));
            }
        }
    }
}

fn poisson_disk_points(
    rng: &mut SeededRng,
    min_distance: i32,
    target_count: usize,
    max_attempts: usize,
    mut allowed: impl FnMut(i32, i32) -> bool,
) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let mut attempts = 0_usize;

    while points.len() < target_count && attempts < max_attempts {
        attempts += 1;
        let x = rng.next_bounded_i32(RAID_DEFENSE_SIZE);
        let y = rng.next_bounded_i32(RAID_DEFENSE_SIZE);
        if !allowed(x, y) {
            continue;
        }
        if points
            .iter()
            .all(|existing| grid_distance((x, y), *existing) >= min_distance)
        {
            points.push((x, y));
        }
    }

    points
}

fn paint_terrain_blob(
    state: &mut GameState,
    kind: &str,
    center: (i32, i32),
    radius: i32,
    protected: &BTreeSet<(i32, i32)>,
    rng: &mut SeededRng,
) -> Result<(), EngineError> {
    for y in (center.1 - radius)..=(center.1 + radius) {
        for x in (center.0 - radius)..=(center.0 + radius) {
            if !in_map_bounds(x, y) || protected.contains(&(x, y)) {
                continue;
            }
            let distance = grid_distance((x, y), center);
            if distance > radius {
                continue;
            }
            if distance == radius && !rng.chance(9, 20) {
                continue;
            }

            let location = MapLocation::new(x, y);
            if kind == FOREST
                && state
                    .tile(location)
                    .is_some_and(|tile| tile.kind.as_str() == WATER_TILE)
            {
                continue;
            }

            state.set_tile(kind, location)?;
        }
    }
    Ok(())
}

pub(crate) fn terrain_kind_count(state: &GameState, kind: &str) -> usize {
    state
        .tiles()
        .filter(|tile| tile.kind.as_str() == kind)
        .count()
}

fn grid_distance(left: (i32, i32), right: (i32, i32)) -> i32 {
    (left.0 - right.0).abs() + (left.1 - right.1).abs()
}

fn in_map_bounds(x: i32, y: i32) -> bool {
    (0..RAID_DEFENSE_SIZE).contains(&x) && (0..RAID_DEFENSE_SIZE).contains(&y)
}

#[derive(Clone, Debug)]
struct SeededRng {
    state: u64,
}

impl SeededRng {
    fn new(seed: u64) -> Self {
        let mixed = seed ^ 0x9e37_79b9_7f4a_7c15;
        let state = if mixed == 0 {
            0xa076_1d64_78bd_642f
        } else {
            mixed
        };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;
        value.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn next_bounded_i32(&mut self, upper_exclusive: i32) -> i32 {
        if upper_exclusive <= 1 {
            return 0;
        }
        (self.next_u64() % upper_exclusive as u64) as i32
    }

    fn chance(&mut self, numerator: u64, denominator: u64) -> bool {
        if denominator == 0 || numerator >= denominator {
            return true;
        }
        self.next_u64() % denominator < numerator
    }
}

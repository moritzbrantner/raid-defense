use super::*;

pub fn new_raid_defense_state() -> Result<GameState, EngineError> {
    new_raid_defense_state_with_seed(DEFAULT_RAID_DEFENSE_MAP_SEED)
}

pub fn new_raid_defense_state_with_seed(seed: u64) -> Result<GameState, EngineError> {
    let mut state = GameState::new(raid_defense_catalog());
    seed_raid_defense_state(&mut state, seed)?;
    Ok(state)
}

pub fn new_raid_defense_world(
    players: impl IntoIterator<Item = PlayerId>,
) -> Result<GameWorld, GameWorldError> {
    new_raid_defense_world_with_seed(players, DEFAULT_RAID_DEFENSE_MAP_SEED)
}

pub fn new_raid_defense_world_with_seed(
    players: impl IntoIterator<Item = PlayerId>,
    seed: u64,
) -> Result<GameWorld, GameWorldError> {
    let mut world = GameWorld::new(raid_defense_catalog());
    for player in players {
        world.add_player(player.clone())?;
        let state = new_raid_defense_state_with_seed(seed).map_err(|source| {
            GameWorldError::PlayerEngine {
                player: player.clone(),
                source,
            }
        })?;
        *world.require_player_mut(player)? = state;
    }
    Ok(world)
}

fn seed_raid_defense_state(state: &mut GameState, seed: u64) -> Result<(), EngineError> {
    state.set_map_topology(MapTopology::Hexagonal);
    state.set_map_bounds(0, 0, RAID_DEFENSE_SIZE - 1, RAID_DEFENSE_SIZE - 1);
    for (resource, capacity) in [(GRAIN, 100), (TIMBER, 100), (STONE, 100)] {
        state.inventory_mut().set_capacity(resource, capacity);
    }
    state.inventory_mut().add_many(&[
        ResourceAmount::new(GRAIN, 100),
        ResourceAmount::new(TIMBER, 100),
        ResourceAmount::new(STONE, 100),
    ])?;
    state.grant_tech_node_kind(IMPERIAL_CHARTER)?;
    generate_seeded_terrain(state, seed)?;

    // Reserve a central clearing so the base always starts and expands on open ground.
    for location in rectangle(10, 10, 20, 20) {
        state.set_tile(PLAINS, location)?;
    }

    let castle_location = MapLocation::new(RAID_DEFENSE_SIZE / 2, RAID_DEFENSE_SIZE / 2);
    let capital = state.start_construction_at(CAPITAL, castle_location)?;

    set_tax_rate(state, capital, DEFAULT_TAX_RATE)?;
    state.set_building_stat(capital, SECURITY, 60)?;

    for _ in 0..5 {
        state.spawn_entity(
            EntityBlueprintRef::Unit(ENGINEER.into()),
            None,
            castle_location,
        )?;
        state.inventory_mut().add(GRAIN, 50)?;
    }

    Ok(())
}

fn rectangle(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Vec<MapLocation> {
    (min_y..=max_y)
        .flat_map(|y| (min_x..=max_x).map(move |x| MapLocation::new(x, y)))
        .collect()
}

fn row(y: i32, min_x: i32, max_x: i32) -> Vec<MapLocation> {
    (min_x..=max_x).map(|x| MapLocation::new(x, y)).collect()
}

fn column(x: i32, min_y: i32, max_y: i32) -> Vec<MapLocation> {
    (min_y..=max_y).map(|y| MapLocation::new(x, y)).collect()
}

pub(crate) fn imperial_road() -> Vec<MapLocation> {
    let mut waypoints = row(10, 0, 24);
    waypoints.extend(column(24, 10, 25));
    waypoints.extend(row(25, 20, 28));
    waypoints
}

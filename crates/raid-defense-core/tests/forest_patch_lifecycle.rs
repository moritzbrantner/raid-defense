use raid_defense_core::{
    Cell, Command, EntityKind, Event, GRID_HEIGHT, GRID_WIDTH, GameState, PersonState,
    STANDARD_RULES,
};

fn place_first_reachable_sawmill(state: &mut GameState) -> Cell {
    for z in 1..GRID_HEIGHT - 1 {
        for x in 1..GRID_WIDTH - 1 {
            let command = Command::PlaceSawmill { x, z };
            let mut probe = state.clone();
            if probe.apply(command).is_ok() {
                state
                    .apply(command)
                    .expect("probed sawmill placement must remain valid");
                return Cell::new(x, z);
            }
        }
    }
    panic!("seeded world must expose a reachable sawmill cell");
}

fn advance_tick(state: &mut GameState) -> Event {
    state
        .apply(Command::AdvanceTick)
        .expect("advancing deterministic simulation must succeed")
}

fn forest_wood(state: &GameState) -> Vec<(u32, u32)> {
    state
        .snapshot()
        .entities
        .into_iter()
        .filter(|entity| entity.kind == EntityKind::Forest)
        .map(|entity| (entity.id, entity.stored_wood))
        .collect()
}

#[test]
fn idle_forestry_workers_fan_out_across_stocked_patches() {
    let mut rules = STANDARD_RULES;
    rules.economy.forest_tile_count = 3;
    rules.population.starting_people = 3;
    rules.population.base_capacity = 3;
    rules.economy.forest_regrowth_interval_ticks = u16::MAX;
    let mut state = GameState::with_rules(23, rules);
    place_first_reachable_sawmill(&mut state);

    advance_tick(&mut state);

    let mut targets = state
        .snapshot()
        .entities
        .into_iter()
        .filter(|entity| entity.kind == EntityKind::Person)
        .filter(|entity| {
            matches!(
                entity.person_state,
                Some(PersonState::ToForest | PersonState::HarvestingForest)
            )
        })
        .filter_map(|entity| entity.person_target_entity)
        .collect::<Vec<_>>();
    targets.sort_unstable();
    targets.dedup();

    assert_eq!(
        targets.len(),
        3,
        "workers should reserve separate stocked patches before stacking onto one forest"
    );
}

#[test]
fn depleted_patch_waits_for_recovery_then_returns_full() {
    let mut rules = STANDARD_RULES;
    rules.economy.forest_tile_count = 1;
    rules.economy.forest_tile_wood = 5;
    rules.economy.forest_regrowth_amount = 2;
    rules.economy.forest_regrowth_interval_ticks = 3;
    rules.economy.sawmill_output = 5;
    rules.economy.sawmill_interval_ticks = 1;
    rules.population.starting_people = 1;
    rules.population.base_capacity = 1;
    let mut state = GameState::with_rules(41, rules);
    place_first_reachable_sawmill(&mut state);

    let depleted_tick = loop {
        advance_tick(&mut state);
        let forests = forest_wood(&state);
        if forests[0].1 == 0 {
            break state.tick();
        }
        assert!(
            state.tick() < 500,
            "worker must eventually deplete the patch"
        );
    };

    // ceil(5 / 2) * 3 preserves the previous empty-to-full recovery duration.
    let recovery_ticks = 9_u64;
    for _ in 0..recovery_ticks - 1 {
        advance_tick(&mut state);
    }
    assert_eq!(
        forest_wood(&state)[0].1,
        0,
        "a depleted patch must stay dormant for its complete recovery window"
    );
    assert_eq!(state.tick(), depleted_tick + recovery_ticks - 1);

    advance_tick(&mut state);
    assert_eq!(state.tick(), depleted_tick + recovery_ticks);
    assert_eq!(
        forest_wood(&state)[0].1,
        5,
        "the patch must return at full configured capacity in one deterministic step"
    );
}

#[test]
fn partially_harvested_patch_does_not_regrow_in_background() {
    let mut rules = STANDARD_RULES;
    rules.economy.forest_tile_count = 1;
    rules.economy.forest_tile_wood = 5;
    rules.economy.forest_regrowth_amount = 2;
    rules.economy.forest_regrowth_interval_ticks = 3;
    rules.economy.sawmill_output = 2;
    rules.economy.sawmill_interval_ticks = 1;
    rules.population.starting_people = 1;
    rules.population.base_capacity = 1;
    let mut state = GameState::with_rules(43, rules);
    place_first_reachable_sawmill(&mut state);

    loop {
        let Event::TickAdvanced { wood_produced, .. } = advance_tick(&mut state) else {
            unreachable!("advance tick command always emits TickAdvanced");
        };
        if wood_produced > 0 {
            break;
        }
        assert!(
            state.tick() < 500,
            "worker must eventually harvest the patch"
        );
    }
    assert_eq!(forest_wood(&state)[0].1, 3);

    state
        .apply(Command::StartWave)
        .expect("rally should pause new forestry assignments");
    for _ in 0..12 {
        advance_tick(&mut state);
    }

    assert_eq!(
        forest_wood(&state)[0].1,
        3,
        "only full depletion should start a patch regeneration cooldown"
    );
}

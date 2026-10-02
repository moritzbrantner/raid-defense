use raid_defense_core::{
    Command, EntityKind, GameError, GameState, RaiderArchetype, STANDARD_RULES, WaveGroupRules,
    WaveRules,
};

fn explicit_rules() -> raid_defense_core::GameRules {
    let mut rules = STANDARD_RULES;
    rules.cycle.automatic_raids = false;
    rules.cycle.raid_rally_ticks = 1;
    rules.raids.spawn_interval_ticks = 1;
    rules.raids.wave_plan.wave_count = 2;

    let mut first = WaveRules::EMPTY;
    first.group_count = 2;
    first.groups[0] = WaveGroupRules {
        archetype: RaiderArchetype::Basic,
        count: 2,
    };
    first.groups[1] = WaveGroupRules {
        archetype: RaiderArchetype::Advanced,
        count: 1,
    };
    rules.raids.wave_plan.waves[0] = first;

    let mut second = WaveRules::EMPTY;
    second.group_count = 2;
    second.groups[0] = WaveGroupRules {
        archetype: RaiderArchetype::Basic,
        count: 2,
    };
    second.groups[1] = WaveGroupRules {
        archetype: RaiderArchetype::Advanced,
        count: 2,
    };
    rules.raids.wave_plan.waves[1] = second;
    rules
}

#[test]
fn explicit_wave_spawns_configured_raider_groups_in_order() {
    let mut state = GameState::with_rules(17, explicit_rules());
    let started = state
        .apply(Command::StartWave)
        .expect("configured first wave should start");
    assert!(matches!(
        started,
        raid_defense_core::Event::WaveStarted { raiders: 3, .. }
    ));

    state.apply(Command::AdvanceTick).unwrap();
    state.apply(Command::AdvanceTick).unwrap();
    state.apply(Command::AdvanceTick).unwrap();

    let raiders = state
        .snapshot()
        .entities
        .into_iter()
        .filter(|entity| entity.kind == EntityKind::Raider)
        .collect::<Vec<_>>();
    assert_eq!(raiders.len(), 3);
    assert_eq!(raiders[0].raider_archetype, Some(RaiderArchetype::Basic));
    assert_eq!(raiders[1].raider_archetype, Some(RaiderArchetype::Basic));
    assert_eq!(raiders[2].raider_archetype, Some(RaiderArchetype::Advanced));
}

#[test]
fn scenario_complete_is_fail_closed_without_starting_an_invented_wave() {
    let mut rules = explicit_rules();
    rules.raids.wave_plan.wave_count = 1;
    let mut state = GameState::with_rules(23, rules);
    state.apply(Command::StartWave).unwrap();
    state.apply(Command::AdvanceTick).unwrap();

    // Remove active raiders through ordinary simulation by giving the Town Hall no wood,
    // then advance until the configured wave is complete.
    for _ in 0..300 {
        state.apply(Command::AdvanceTick).unwrap();
        if state.completed_waves() == 1 {
            break;
        }
    }
    assert_eq!(state.completed_waves(), 1);
    let before = state.checksum();
    assert_eq!(
        state.apply(Command::StartWave),
        Err(GameError::ScenarioComplete)
    );
    assert_eq!(state.checksum(), before);
}

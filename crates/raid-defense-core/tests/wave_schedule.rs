use raid_defense_core::{Command, GameError, GameState, STANDARD_RULES, replay_with_rules};

const SEED: u64 = 0x5eed;

#[test]
fn wave_spawns_raiders_at_authoritative_interval() {
    let mut rules = STANDARD_RULES;
    rules.cycle.automatic_raids = false;
    rules.raids.raiders_per_wave = 4;
    rules.raids.spawn_interval_ticks = 3;
    rules.raids.speed_milli = 1;
    let mut state = GameState::with_rules(SEED, rules);

    state.apply(Command::StartWave).expect("wave should start");
    assert_eq!(
        state.raider_count(),
        1,
        "the first raider spawns immediately"
    );
    assert!(state.is_night());

    for expected_count in 2..=4 {
        for _ in 0..2 {
            state
                .apply(Command::AdvanceTick)
                .expect("scheduled wave tick should advance");
            assert_eq!(
                state.raider_count(),
                expected_count - 1,
                "a raider must not spawn before the configured interval"
            );
        }
        state
            .apply(Command::AdvanceTick)
            .expect("scheduled spawn tick should advance");
        assert_eq!(
            state.raider_count(),
            expected_count,
            "the next raider must spawn exactly at the configured interval"
        );
    }
}

#[test]
fn raid_stays_active_between_scheduled_spawns() {
    let mut rules = STANDARD_RULES;
    rules.cycle.automatic_raids = false;
    rules.raids.raiders_per_wave = 2;
    rules.raids.spawn_interval_ticks = 200;
    rules.raids.speed_milli = 1_000;
    let mut state = GameState::with_rules(SEED, rules);

    state.apply(Command::StartWave).expect("wave should start");
    assert_eq!(state.raider_count(), 1);

    let mut observed_gap = false;
    for _ in 0..rules.raids.spawn_interval_ticks - 1 {
        state
            .apply(Command::AdvanceTick)
            .expect("raid tick should advance");
        if state.raider_count() == 0 {
            observed_gap = true;
            break;
        }
    }

    assert!(
        observed_gap,
        "the first raider should finish before the second spawn"
    );
    assert!(
        state.is_night(),
        "pending scheduled raiders keep the authoritative raid active"
    );
    assert_eq!(state.completed_waves(), 0);
    assert_eq!(
        state.apply(Command::StartWave),
        Err(GameError::RaidersStillActive),
        "a zero-live-raider scheduling gap must not allow a second wave"
    );

    for _ in 0..rules.raids.spawn_interval_ticks {
        state
            .apply(Command::AdvanceTick)
            .expect("raid tick should advance");
        if state.raider_count() != 0 {
            break;
        }
    }
    assert_eq!(state.wave(), 1);
    assert_eq!(
        state.raider_count(),
        1,
        "the pending raider must eventually spawn"
    );
}

#[test]
fn timed_wave_schedule_replays_to_the_same_checksum() {
    let mut rules = STANDARD_RULES;
    rules.cycle.automatic_raids = false;
    rules.raids.raiders_per_wave = 4;
    rules.raids.spawn_interval_ticks = 3;
    rules.raids.speed_milli = 1;

    let mut commands = vec![Command::StartWave];
    commands.extend(std::iter::repeat_n(Command::AdvanceTick, 10));

    let first = replay_with_rules(SEED, rules, &commands).expect("timed wave trace should replay");
    let second = replay_with_rules(SEED, rules, &commands).expect("same trace should replay");

    assert_eq!(first, second);
    assert_eq!(first.checksum(), second.checksum());
    assert_eq!(first.raider_count(), 4);
    assert!(first.is_night());
}

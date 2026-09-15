use raid_defense_core::{Command, GameError, GameState, STANDARD_RULES, replay_with_rules};

const SEED: u64 = 0x5eed;

fn finish_rally(state: &mut GameState, rally_ticks: u16) {
    for tick in 0..rally_ticks {
        state
            .apply(Command::AdvanceTick)
            .expect("rally tick should advance");
        if tick + 1 < rally_ticks {
            assert_eq!(
                state.raider_count(),
                0,
                "raiders must wait for the full rally"
            );
            assert!(state.is_rallying());
        }
    }
    assert!(!state.is_rallying());
}

#[test]
fn wave_spawns_raiders_after_authoritative_rally_and_then_at_interval() {
    let mut rules = STANDARD_RULES;
    rules.cycle.automatic_raids = false;
    rules.cycle.raid_rally_ticks = 3;
    rules.raids.raiders_per_wave = 4;
    rules.raids.spawn_interval_ticks = 3;
    rules.raids.speed_milli = 1;
    let mut state = GameState::with_rules(SEED, rules);

    let event = state
        .apply(Command::StartWave)
        .expect("wave rally should start");
    assert_eq!(
        state.raider_count(),
        0,
        "no raider spawns when the rally is called"
    );
    assert!(state.is_rallying());
    assert!(!state.is_night());
    assert!(matches!(
        event,
        raid_defense_core::Event::WaveStarted {
            wave: 1,
            raiders: 4,
            rally_ticks: 3
        }
    ));

    finish_rally(&mut state, rules.cycle.raid_rally_ticks);
    assert_eq!(
        state.raider_count(),
        1,
        "the first raider spawns when rally time expires"
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
    rules.cycle.raid_rally_ticks = 2;
    rules.raids.raiders_per_wave = 2;
    rules.raids.spawn_interval_ticks = 200;
    rules.raids.speed_milli = 1_000;
    let mut state = GameState::with_rules(SEED, rules);

    state
        .apply(Command::StartWave)
        .expect("wave rally should start");
    finish_rally(&mut state, rules.cycle.raid_rally_ticks);
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
fn rally_rejects_duplicate_start_commands() {
    let mut rules = STANDARD_RULES;
    rules.cycle.automatic_raids = false;
    let mut state = GameState::with_rules(SEED, rules);

    state
        .apply(Command::StartWave)
        .expect("rally should start");
    assert!(state.is_rallying());
    assert_eq!(
        state.apply(Command::StartWave),
        Err(GameError::RaidersStillActive),
        "calling the raid again during rally must be fail-closed"
    );
}

#[test]
fn timed_wave_schedule_replays_to_the_same_checksum() {
    let mut rules = STANDARD_RULES;
    rules.cycle.automatic_raids = false;
    rules.cycle.raid_rally_ticks = 3;
    rules.raids.raiders_per_wave = 4;
    rules.raids.spawn_interval_ticks = 3;
    rules.raids.speed_milli = 1;

    let mut commands = vec![Command::StartWave];
    commands.extend(std::iter::repeat_n(Command::AdvanceTick, 13));

    let first = replay_with_rules(SEED, rules, &commands).expect("timed wave trace should replay");
    let second = replay_with_rules(SEED, rules, &commands).expect("same trace should replay");

    assert_eq!(first, second);
    assert_eq!(first.checksum(), second.checksum());
    assert_eq!(first.raider_count(), 4);
    assert!(first.is_night());
}

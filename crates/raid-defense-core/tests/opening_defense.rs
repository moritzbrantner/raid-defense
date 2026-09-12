use raid_defense_core::{
    Command, GameError, GameState, HOUSE_UNLOCK_COMPLETED_WAVES, TowerArchetype, replay,
};

fn opening_trace() -> Vec<Command> {
    let mut commands = vec![
        Command::PlaceSawmill { x: 2, z: 2 },
        Command::PlaceTower {
            x: 7,
            z: 2,
            archetype: TowerArchetype::Arrow,
        },
        Command::UpgradeTower { x: 7, z: 2 },
        Command::StartWave,
    ];
    commands.extend(std::iter::repeat_n(Command::AdvanceTick, 60));
    commands
}

#[test]
fn opening_economy_defense_trace_is_replayable_with_people_logistics() {
    let commands = opening_trace();
    let first = replay(0x5eed, &commands).expect("opening trace should remain valid");
    let second = replay(0x5eed, &commands).expect("same trace should replay");

    assert_eq!(first, second);
    assert_eq!(first.checksum(), second.checksum());
    assert_eq!(first.wave(), 1);
    assert_eq!(first.tower_count(), 1);
    assert_eq!(first.sawmill_count(), 1);
    assert_eq!(first.people_count(), 2);
    assert_eq!(first.population_capacity(), 2);
}

#[test]
fn sawmill_requires_people_to_deliver_wood_to_town_hall() {
    let mut state = GameState::new(0x5eed);
    state
        .apply(Command::PlaceSawmill { x: 2, z: 2 })
        .expect("sawmill should build");
    let after_build = state.wood();

    for _ in 0..100 {
        state
            .apply(Command::AdvanceTick)
            .expect("economy tick should advance");
        if state.wood() > after_build {
            break;
        }
    }

    assert!(
        state.wood() > after_build,
        "a carrier should eventually deliver produced wood"
    );
    assert_eq!(state.people_count(), 2);
}

#[test]
fn houses_are_unavailable_before_ten_completed_waves() {
    let mut state = GameState::new(0x5eed);
    let before = state.clone();

    let rejected = state.apply(Command::PlaceHouse { x: 2, z: 2 });

    assert_eq!(rejected, Err(GameError::HouseLocked));
    assert_eq!(state, before);
    assert_eq!(state.completed_waves(), 0);
    assert_eq!(HOUSE_UNLOCK_COMPLETED_WAVES, 10);
}

#[test]
fn invalid_grid_build_is_transactional_inside_a_trace() {
    let mut state = GameState::new(0x5eed);
    let before = state.clone();

    let rejected = state.apply(Command::PlaceTower {
        x: -1,
        z: 2,
        archetype: TowerArchetype::Arrow,
    });

    assert_eq!(rejected, Err(GameError::OutOfBounds));
    assert_eq!(state, before);
}

#[test]
fn invalid_upgrade_is_transactional_inside_a_trace() {
    let mut state = GameState::new(0x5eed);
    let before = state.clone();

    let rejected = state.apply(Command::UpgradeTower { x: 2, z: 2 });

    assert_eq!(rejected, Err(GameError::NoTower));
    assert_eq!(state, before);
}

#[test]
fn ending_a_wave_does_not_leave_orphaned_projectiles() {
    let mut state = GameState::new(0x5eed);
    state
        .apply(Command::PlaceTower {
            x: 6,
            z: 4,
            archetype: TowerArchetype::Arrow,
        })
        .expect("tower should build");
    state.apply(Command::StartWave).expect("wave should start");

    for _ in 0..100 {
        state
            .apply(Command::AdvanceTick)
            .expect("wave tick should advance");
        if state.raider_count() == 0 {
            break;
        }
    }

    assert_eq!(state.raider_count(), 0, "wave should eventually end");
    assert_eq!(
        state.projectile_count(),
        0,
        "despawned raiders must not retain targeting projectiles"
    );
}

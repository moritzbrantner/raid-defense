use raid_defense_core::{
    Cell, Command, GameError, GameState, HOUSE_UNLOCK_COMPLETED_WAVES, TowerArchetype, replay,
};

const SEED: u64 = 0x5eed;

fn first_accepted_cell(state: &GameState, command: impl Fn(Cell) -> Command) -> Cell {
    let snapshot = state.snapshot();
    for z in 1..snapshot.grid_height - 1 {
        for x in 1..snapshot.grid_width - 1 {
            let cell = Cell::new(x, z);
            let mut probe = state.clone();
            if probe.apply(command(cell)).is_ok() {
                return cell;
            }
        }
    }
    panic!("seeded map must expose an accepted build cell");
}

fn opening_trace() -> Vec<Command> {
    let mut state = GameState::new(SEED);
    let mut commands = Vec::new();

    let sawmill_cell = first_accepted_cell(&state, |cell| Command::PlaceSawmill {
        x: cell.x,
        z: cell.z,
    });
    let sawmill = Command::PlaceSawmill {
        x: sawmill_cell.x,
        z: sawmill_cell.z,
    };
    state.apply(sawmill).expect("sawmill should build");
    commands.push(sawmill);

    let tower_cell = first_accepted_cell(&state, |cell| Command::PlaceTower {
        x: cell.x,
        z: cell.z,
        archetype: TowerArchetype::Arrow,
    });
    let tower = Command::PlaceTower {
        x: tower_cell.x,
        z: tower_cell.z,
        archetype: TowerArchetype::Arrow,
    };
    state.apply(tower).expect("tower construction should start");
    commands.push(tower);

    for _ in 0..200 {
        if state.tower_count() == 1 {
            break;
        }
        state
            .apply(Command::AdvanceTick)
            .expect("construction tick should advance");
        commands.push(Command::AdvanceTick);
    }
    assert_eq!(state.tower_count(), 1, "workers should finish the tower");

    let upgrade = Command::UpgradeTower {
        x: tower_cell.x,
        z: tower_cell.z,
    };
    state
        .apply(upgrade)
        .expect("completed tower should be upgradeable");
    commands.push(upgrade);

    state
        .apply(Command::StartWave)
        .expect("wave rally should start after construction");
    commands.push(Command::StartWave);
    for _ in 0..60 {
        state
            .apply(Command::AdvanceTick)
            .expect("wave tick should advance");
        commands.push(Command::AdvanceTick);
    }
    commands
}

#[test]
fn opening_economy_defense_trace_is_replayable_with_seeded_logistics() {
    let commands = opening_trace();
    let first = replay(SEED, &commands).expect("opening trace should remain valid");
    let second = replay(SEED, &commands).expect("same trace should replay");

    assert_eq!(first, second);
    assert_eq!(first.checksum(), second.checksum());
    assert_eq!(first.wave(), 1);
    assert_eq!(first.tower_count(), 1);
    assert_eq!(first.sawmill_count(), 1);
    assert_eq!(first.people_count(), 2);
    assert_eq!(first.population_capacity(), 2);
    assert_eq!(first.snapshot().grid_width, 21);
    assert_eq!(first.snapshot().grid_height, 21);
}

#[test]
fn sawmill_requires_people_to_gather_and_deliver_wood() {
    let mut state = GameState::new(SEED);
    let cell = first_accepted_cell(&state, |cell| Command::PlaceSawmill {
        x: cell.x,
        z: cell.z,
    });
    state
        .apply(Command::PlaceSawmill {
            x: cell.x,
            z: cell.z,
        })
        .expect("sawmill should build on a valid worker route");
    let after_build = state.wood();

    for _ in 0..300 {
        state
            .apply(Command::AdvanceTick)
            .expect("economy tick should advance");
        if state.wood() > after_build {
            break;
        }
    }

    assert!(
        state.wood() > after_build,
        "a person should eventually gather forest wood, deliver it to the sawmill, and haul it into settlement storage"
    );
    assert_eq!(state.people_count(), 2);
}

#[test]
fn tower_material_is_hauled_before_the_tower_becomes_active() {
    let mut state = GameState::new(SEED);
    let cell = first_accepted_cell(&state, |cell| Command::PlaceTower {
        x: cell.x,
        z: cell.z,
        archetype: TowerArchetype::Arrow,
    });
    let wood_before = state.wood();
    state
        .apply(Command::PlaceTower {
            x: cell.x,
            z: cell.z,
            archetype: TowerArchetype::Arrow,
        })
        .expect("construction should start");

    assert_eq!(state.tower_count(), 0);
    assert_eq!(state.construction_site_count(), 1);
    assert_eq!(
        state.wood(),
        wood_before,
        "placement does not teleport material"
    );

    for _ in 0..160 {
        state
            .apply(Command::AdvanceTick)
            .expect("construction tick should advance");
        if state.tower_count() == 1 {
            break;
        }
    }

    assert_eq!(state.tower_count(), 1);
    assert_eq!(state.construction_site_count(), 0);
    assert!(
        state.wood() < wood_before,
        "workers consumed stored material"
    );
}

#[test]
fn houses_are_unavailable_before_ten_completed_waves() {
    let mut state = GameState::new(SEED);
    let before = state.clone();
    let cell = first_accepted_cell(&state, |cell| Command::PlaceTower {
        x: cell.x,
        z: cell.z,
        archetype: TowerArchetype::Arrow,
    });

    let rejected = state.apply(Command::PlaceHouse {
        x: cell.x,
        z: cell.z,
    });

    assert_eq!(rejected, Err(GameError::HouseLocked));
    assert_eq!(state, before);
    assert_eq!(state.completed_waves(), 0);
    assert_eq!(HOUSE_UNLOCK_COMPLETED_WAVES, 10);
}

#[test]
fn invalid_grid_build_is_transactional_inside_a_trace() {
    let mut state = GameState::new(SEED);
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
    let mut state = GameState::new(SEED);
    let before = state.clone();
    let cell = first_accepted_cell(&state, |cell| Command::PlaceTower {
        x: cell.x,
        z: cell.z,
        archetype: TowerArchetype::Arrow,
    });

    let rejected = state.apply(Command::UpgradeTower {
        x: cell.x,
        z: cell.z,
    });

    assert_eq!(rejected, Err(GameError::NoTower));
    assert_eq!(state, before);
}

#[test]
fn ending_a_wave_does_not_leave_orphaned_projectiles() {
    let mut state = GameState::new(SEED);
    let cell = first_accepted_cell(&state, |cell| Command::PlaceTower {
        x: cell.x,
        z: cell.z,
        archetype: TowerArchetype::Arrow,
    });
    state
        .apply(Command::PlaceTower {
            x: cell.x,
            z: cell.z,
            archetype: TowerArchetype::Arrow,
        })
        .expect("tower construction should start");
    for _ in 0..160 {
        state
            .apply(Command::AdvanceTick)
            .expect("construction tick should advance");
        if state.tower_count() == 1 {
            break;
        }
    }
    assert_eq!(state.tower_count(), 1);
    state.apply(Command::StartWave).expect("wave rally should start");

    for _ in 0..300 {
        state
            .apply(Command::AdvanceTick)
            .expect("wave tick should advance");
        if state.wave() == 1 && !state.is_rallying() && !state.is_night() {
            break;
        }
    }

    assert_eq!(state.wave(), 1, "the rally must actually enter wave one");
    assert!(!state.is_rallying());
    assert!(!state.is_night(), "wave should eventually end");
    assert_eq!(
        state.raider_count(),
        0,
        "completed wave has no live raiders"
    );
    assert_eq!(
        state.projectile_count(),
        0,
        "despawned raiders must not retain targeting projectiles"
    );
}

use raid_defense_core::{Command, Event, GameState, replay};

fn opening_trace() -> Vec<Command> {
    vec![
        Command::BuildFort { province: 0 },
        Command::Recruit {
            province: 0,
            soldiers: 10,
        },
        Command::AdvanceDay,
        Command::AdvanceDay,
        Command::AdvanceDay,
        Command::ClaimProvince { province: 1 },
        Command::BuildFort { province: 1 },
        Command::Recruit {
            province: 1,
            soldiers: 8,
        },
        Command::AdvanceDay,
        Command::AdvanceDay,
    ]
}

#[test]
fn opening_defense_trace_is_replayable() {
    let commands = opening_trace();
    let first = replay(0x5eed, &commands).expect("opening trace should remain valid");
    let second = replay(0x5eed, &commands).expect("same trace should replay");

    assert_eq!(first, second);
    assert_eq!(first.checksum(), second.checksum());
    assert_eq!(first.day(), 5);
    assert!(first.provinces()[1].controlled);
    assert_eq!(first.provinces()[1].fort_level, 1);
    assert_eq!(first.provinces()[1].garrison, 8);
}

#[test]
fn invalid_frontier_command_is_transactional_inside_a_trace() {
    let mut state = GameState::new(0x5eed);
    state
        .apply(Command::BuildFort { province: 0 })
        .expect("capital fort should build");

    let before = state.clone();
    let rejected = state.apply(Command::ClaimProvince { province: 2 });

    assert!(rejected.is_err());
    assert_eq!(state, before);

    let accepted = state
        .apply(Command::ClaimProvince { province: 1 })
        .expect("adjacent frontier claim should still succeed");
    assert_eq!(accepted, Event::ProvinceClaimed { province: 1 });
}

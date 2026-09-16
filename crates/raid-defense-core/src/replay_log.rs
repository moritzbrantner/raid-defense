use super::*;

pub type PlayerId = u32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerAction {
    PlaceTower {
        x: i16,
        z: i16,
        archetype: TowerArchetype,
    },
    PlaceSawmill {
        x: i16,
        z: i16,
    },
    PlaceStorageHouse {
        x: i16,
        z: i16,
    },
    PlaceHouse {
        x: i16,
        z: i16,
    },
    UpgradeTower {
        x: i16,
        z: i16,
    },
    StartWave,
}

impl From<PlayerAction> for Command {
    fn from(action: PlayerAction) -> Self {
        match action {
            PlayerAction::PlaceTower { x, z, archetype } => Self::PlaceTower { x, z, archetype },
            PlayerAction::PlaceSawmill { x, z } => Self::PlaceSawmill { x, z },
            PlayerAction::PlaceStorageHouse { x, z } => Self::PlaceStorageHouse { x, z },
            PlayerAction::PlaceHouse { x, z } => Self::PlaceHouse { x, z },
            PlayerAction::UpgradeTower { x, z } => Self::UpgradeTower { x, z },
            PlayerAction::StartWave => Self::StartWave,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordedPlayerAction {
    pub tick: u64,
    pub sequence: u64,
    pub player_id: PlayerId,
    pub action: PlayerAction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayError {
    InvalidRules(RulesError),
    NonContiguousSequence {
        expected: u64,
        actual: u64,
    },
    NonMonotonicTick {
        previous: u64,
        actual: u64,
    },
    ActionAfterReplayCursor {
        action_tick: u64,
        recorded_through_tick: u64,
    },
    ActionRejected {
        sequence: u64,
        error: GameError,
    },
}

pub fn replay_player_actions(
    seed: u64,
    rules: GameRules,
    recorded_through_tick: u64,
    actions: &[RecordedPlayerAction],
) -> Result<GameState, ReplayError> {
    let mut state = GameState::try_with_rules(seed, rules).map_err(ReplayError::InvalidRules)?;
    let mut expected_sequence = 0_u64;
    let mut previous_tick = 0_u64;

    for recorded in actions {
        if recorded.sequence != expected_sequence {
            return Err(ReplayError::NonContiguousSequence {
                expected: expected_sequence,
                actual: recorded.sequence,
            });
        }
        if expected_sequence != 0 && recorded.tick < previous_tick {
            return Err(ReplayError::NonMonotonicTick {
                previous: previous_tick,
                actual: recorded.tick,
            });
        }
        if recorded.tick > recorded_through_tick {
            return Err(ReplayError::ActionAfterReplayCursor {
                action_tick: recorded.tick,
                recorded_through_tick,
            });
        }

        while state.tick() < recorded.tick {
            state
                .apply(Command::AdvanceTick)
                .map_err(|error| ReplayError::ActionRejected {
                    sequence: recorded.sequence,
                    error,
                })?;
        }
        state
            .apply(recorded.action.into())
            .map_err(|error| ReplayError::ActionRejected {
                sequence: recorded.sequence,
                error,
            })?;

        previous_tick = recorded.tick;
        expected_sequence += 1;
    }

    while state.tick() < recorded_through_tick {
        state
            .apply(Command::AdvanceTick)
            .map_err(|error| ReplayError::ActionRejected {
                sequence: expected_sequence,
                error,
            })?;
    }

    Ok(state)
}

pub fn snapshot_from_player_actions(
    seed: u64,
    rules: GameRules,
    recorded_through_tick: u64,
    actions: &[RecordedPlayerAction],
) -> Result<GameSnapshot, ReplayError> {
    replay_player_actions(seed, rules, recorded_through_tick, actions).map(|state| state.snapshot())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_sawmill_cell(state: &GameState) -> Cell {
        let snapshot = state.snapshot();
        for z in 1..snapshot.grid_height - 1 {
            for x in 1..snapshot.grid_width - 1 {
                let cell = Cell::new(x, z);
                let mut probe = state.clone();
                if probe.apply(Command::PlaceSawmill { x, z }).is_ok() {
                    return cell;
                }
            }
        }
        panic!("seeded world must expose a valid sawmill cell");
    }

    #[test]
    fn action_log_reconstructs_authoritative_state_and_snapshot() {
        let seed = 0x5eed;
        let mut direct = GameState::new(seed);
        let cell = first_sawmill_cell(&direct);
        direct
            .apply(Command::PlaceSawmill {
                x: cell.x,
                z: cell.z,
            })
            .expect("sawmill action should be accepted");
        for _ in 0..40 {
            direct
                .apply(Command::AdvanceTick)
                .expect("simulation tick should advance");
        }

        let actions = [RecordedPlayerAction {
            tick: 0,
            sequence: 0,
            player_id: 7,
            action: PlayerAction::PlaceSawmill {
                x: cell.x,
                z: cell.z,
            },
        }];
        let replayed = replay_player_actions(seed, STANDARD_RULES, 40, &actions)
            .expect("action log should replay deterministically");
        let snapshot = snapshot_from_player_actions(seed, STANDARD_RULES, 40, &actions)
            .expect("snapshot is derived from the replay log");

        assert_eq!(replayed.checksum(), direct.checksum());
        assert_eq!(snapshot, direct.snapshot());
    }

    #[test]
    fn replay_rejects_sequence_gaps_and_actions_after_cursor() {
        let action = RecordedPlayerAction {
            tick: 1,
            sequence: 2,
            player_id: 0,
            action: PlayerAction::StartWave,
        };
        assert_eq!(
            replay_player_actions(1, STANDARD_RULES, 1, &[action]),
            Err(ReplayError::NonContiguousSequence {
                expected: 0,
                actual: 2,
            })
        );

        let action = RecordedPlayerAction {
            tick: 2,
            sequence: 0,
            player_id: 0,
            action: PlayerAction::StartWave,
        };
        assert_eq!(
            replay_player_actions(1, STANDARD_RULES, 1, &[action]),
            Err(ReplayError::ActionAfterReplayCursor {
                action_tick: 2,
                recorded_through_tick: 1,
            })
        );
    }
}

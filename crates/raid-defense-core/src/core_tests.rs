#[cfg(test)]
mod tests {
    use super::*;

    fn first_buildable_sawmill_cell(state: &GameState) -> Cell {
        for z in 1..GRID_HEIGHT - 1 {
            for x in 1..GRID_WIDTH - 1 {
                let cell = Cell::new(x, z);
                if state.validate_build_cell(cell).is_ok()
                    && state.has_harvestable_forest_near(cell)
                    && state.validate_blocking_build(cell, Some(cell)).is_ok()
                {
                    return cell;
                }
            }
        }
        panic!("seeded world must expose a valid sawmill cell");
    }

    fn first_buildable_cell(state: &GameState) -> Cell {
        for z in 1..GRID_HEIGHT - 1 {
            for x in 1..GRID_WIDTH - 1 {
                let cell = Cell::new(x, z);
                if state.validate_build_cell(cell).is_ok()
                    && state.validate_blocking_build(cell, None).is_ok()
                {
                    return cell;
                }
            }
        }
        panic!("seeded world must expose a valid build cell");
    }

    #[test]
    fn standard_map_is_twenty_one_square_with_seeded_forests() {
        let first = GameState::new(7);
        let second = GameState::new(7);
        let other = GameState::new(8);
        assert_eq!(GRID_WIDTH, 21);
        assert_eq!(GRID_HEIGHT, 21);
        assert_eq!(first.forest_count(), usize::from(first.rules.economy.forest_tile_count));
        assert_eq!(first.snapshot(), second.snapshot());
        let first_forests = first
            .snapshot()
            .entities
            .into_iter()
            .filter(|entity| entity.kind == EntityKind::Forest)
            .map(|entity| entity.cell)
            .collect::<Vec<_>>();
        let other_forests = other
            .snapshot()
            .entities
            .into_iter()
            .filter(|entity| entity.kind == EntityKind::Forest)
            .map(|entity| entity.cell)
            .collect::<Vec<_>>();
        assert_ne!(first_forests, other_forests);
    }

    #[test]
    fn sawmill_consumes_finite_forest_wood_before_creating_local_stock() {
        let mut state = GameState::new(7);
        let cell = first_buildable_sawmill_cell(&state);
        state
            .apply(Command::PlaceSawmill { x: cell.x, z: cell.z })
            .expect("sawmill should build near forest");
        let forest_before: u32 = state
            .snapshot()
            .entities
            .iter()
            .filter(|entity| entity.kind == EntityKind::Forest)
            .map(|entity| entity.stored_wood)
            .sum();
        for _ in 0..state.rules.economy.sawmill_interval_ticks {
            state.advance_tick();
        }
        let forest_after: u32 = state
            .snapshot()
            .entities
            .iter()
            .filter(|entity| entity.kind == EntityKind::Forest)
            .map(|entity| entity.stored_wood)
            .sum();
        assert!(forest_after < forest_before);
    }

    #[test]
    fn tower_is_inactive_until_workers_deliver_its_material() {
        let mut state = GameState::new(13);
        let cell = first_buildable_cell(&state);
        state
            .apply(Command::PlaceTower {
                x: cell.x,
                z: cell.z,
                archetype: TowerArchetype::Arrow,
            })
            .expect("tower construction should start");
        assert_eq!(state.tower_count(), 0);
        assert_eq!(state.construction_site_count(), 1);

        for _ in 0..200 {
            state.advance_tick();
            if state.tower_count() == 1 {
                break;
            }
        }
        assert_eq!(state.tower_count(), 1);
        assert_eq!(state.construction_site_count(), 0);
    }

    #[test]
    fn replay_includes_seeded_world_and_worker_construction() {
        let mut probe = GameState::new(29);
        let cell = first_buildable_cell(&probe);
        let mut commands = vec![Command::PlaceTower {
            x: cell.x,
            z: cell.z,
            archetype: TowerArchetype::Arrow,
        }];
        commands.extend(std::iter::repeat_n(Command::AdvanceTick, 100));
        let first = replay(29, &commands).expect("valid replay");
        let second = replay(29, &commands).expect("same replay remains valid");
        assert_eq!(first, second);
        assert_eq!(first.checksum(), second.checksum());
        probe.apply(Command::AdvanceTick).expect("probe remains usable");
    }

    #[test]
    fn integer_square_root_is_deterministic_at_boundaries() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(2), 1);
        assert_eq!(integer_sqrt(4), 2);
        assert_eq!(integer_sqrt(15), 3);
        assert_eq!(integer_sqrt(16), 4);
    }
}

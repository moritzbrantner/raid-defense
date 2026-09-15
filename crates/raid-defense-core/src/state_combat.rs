use super::*;

const UNREACHABLE_FLOW_DISTANCE: u16 = u16::MAX;

#[derive(Clone, Debug, Eq, PartialEq)]
struct RaiderFlowField {
    distances: Vec<u16>,
}

impl RaiderFlowField {
    fn build(state: &GameState, goals: &[Cell]) -> Option<Self> {
        if goals.is_empty() {
            return None;
        }

        let cell_count = usize::try_from(GRID_WIDTH).ok()? * usize::try_from(GRID_HEIGHT).ok()?;
        let mut distances = vec![UNREACHABLE_FLOW_DISTANCE; cell_count];
        let mut queue = VecDeque::new();
        let mut ordered_goals = goals.to_vec();
        ordered_goals.sort_unstable();
        ordered_goals.dedup();

        for goal in ordered_goals {
            if state.path_cell_blocked(goal, None) {
                continue;
            }
            let index = cell_index(goal)?;
            if distances[index] == 0 {
                continue;
            }
            distances[index] = 0;
            queue.push_back(goal);
        }

        if queue.is_empty() {
            return None;
        }

        while let Some(cell) = queue.pop_front() {
            let cell_distance = distances[cell_index(cell)?];
            let next_distance = cell_distance.saturating_add(1);
            for neighbor in neighbors(cell) {
                if state.path_cell_blocked(neighbor, None) {
                    continue;
                }
                let index = cell_index(neighbor)?;
                if distances[index] <= next_distance {
                    continue;
                }
                distances[index] = next_distance;
                queue.push_back(neighbor);
            }
        }

        Some(Self { distances })
    }

    fn distance(&self, cell: Cell) -> Option<u16> {
        let distance = *self.distances.get(cell_index(cell)?)?;
        (distance != UNREACHABLE_FLOW_DISTANCE).then_some(distance)
    }

    fn next_step(&self, start: Cell) -> Option<Cell> {
        let current_distance = self.distance(start)?;
        if current_distance == 0 {
            return Some(start);
        }

        let mut best = None;
        for neighbor in neighbors(start) {
            let Some(distance) = self.distance(neighbor) else {
                continue;
            };
            if distance >= current_distance {
                continue;
            }
            match best {
                None => best = Some((distance, neighbor)),
                Some((best_distance, _)) if distance < best_distance => {
                    best = Some((distance, neighbor));
                }
                _ => {}
            }
        }
        best.map(|(_, cell)| cell)
    }
}

impl GameState {
    pub(super) fn run_tower_attack_system(&mut self) -> u16 {
        let mut tower_ids = self
            .towers
            .iter()
            .filter_map(|(key, tower)| (tower.level > 0).then_some(key_entity(key)))
            .collect::<Vec<_>>();
        tower_ids.sort_unstable();

        let mut shots = 0_u16;
        for tower in tower_ids {
            let Some(mut attack) = self.attacks.get(entity_key(tower)).copied() else {
                continue;
            };
            if attack.cooldown_remaining > 0 {
                attack.cooldown_remaining -= 1;
                self.attacks.insert(entity_key(tower), attack);
                continue;
            }

            let Some(target) = self.target_for_tower(tower, attack.range_milli) else {
                continue;
            };
            let tower_component = self
                .towers
                .get(entity_key(tower))
                .copied()
                .expect("tower ids come from the tower component store");
            let tower_position = self
                .transforms
                .get(entity_key(tower))
                .copied()
                .expect("towers always have transforms");
            let projectile_entity = self.allocate_entity();
            self.transforms
                .insert(entity_key(projectile_entity), tower_position);
            self.projectiles.insert(
                entity_key(projectile_entity),
                Projectile {
                    target,
                    damage: attack.damage,
                    speed_milli: attack.projectile_speed_milli,
                    archetype: tower_component.archetype,
                },
            );

            shots = shots.saturating_add(1);
            attack.cooldown_remaining = attack.cooldown_ticks;
            self.attacks.insert(entity_key(tower), attack);
        }

        shots
    }

    pub(super) fn run_projectile_system(&mut self) -> (u16, Vec<EntityId>) {
        let mut ids = self.projectiles.keys().map(key_entity).collect::<Vec<_>>();
        ids.sort_unstable();
        let mut impacts = 0_u16;
        let mut killed = Vec::new();
        let mut despawn = Vec::new();

        for entity in ids {
            let Some(projectile) = self.projectiles.get(entity_key(entity)).copied() else {
                continue;
            };
            let Some(target_health) = self.health.get(entity_key(projectile.target)).copied()
            else {
                despawn.push(entity);
                continue;
            };
            if target_health.current == 0 {
                despawn.push(entity);
                continue;
            }
            let Some(position) = self.transforms.get(entity_key(entity)).copied() else {
                despawn.push(entity);
                continue;
            };
            let Some(target_position) = self.transforms.get(entity_key(projectile.target)).copied()
            else {
                despawn.push(entity);
                continue;
            };

            let dx = i64::from(target_position.x_milli - position.x_milli);
            let dz = i64::from(target_position.z_milli - position.z_milli);
            let distance_sq = dx * dx + dz * dz;
            let speed = i64::from(projectile.speed_milli);

            if distance_sq <= speed * speed {
                let target_died = {
                    let health = self
                        .health
                        .get_mut(entity_key(projectile.target))
                        .expect("target health was checked above");
                    health.current = health.current.saturating_sub(projectile.damage);
                    health.current == 0
                };
                impacts = impacts.saturating_add(1);
                despawn.push(entity);
                if target_died {
                    killed.push(projectile.target);
                }
                continue;
            }

            let distance = i64::try_from(integer_sqrt(distance_sq as u64))
                .unwrap_or(i64::MAX)
                .max(1);
            let step_x = dx * speed / distance;
            let step_z = dz * speed / distance;
            let next = Transform {
                x_milli: position
                    .x_milli
                    .saturating_add(saturating_i64_to_i32(step_x)),
                z_milli: position
                    .z_milli
                    .saturating_add(saturating_i64_to_i32(step_z)),
            };
            self.transforms.insert(entity_key(entity), next);
        }

        for entity in despawn {
            self.despawn_projectile(entity);
        }
        killed.sort_unstable();
        killed.dedup();
        (impacts, killed)
    }

    pub(super) fn target_for_tower(&self, tower: EntityId, range_milli: i32) -> Option<EntityId> {
        let tower_position = *self.transforms.get(entity_key(tower))?;
        let range_sq = i64::from(range_milli) * i64::from(range_milli);
        let mut candidates = self
            .raiders
            .keys()
            .map(key_entity)
            .filter(|entity| {
                self.health
                    .get(entity_key(*entity))
                    .is_some_and(|health| health.current > 0)
            })
            .filter_map(|entity| {
                let position = self.transforms.get(entity_key(entity))?;
                let dx = i64::from(position.x_milli - tower_position.x_milli);
                let dz = i64::from(position.z_milli - tower_position.z_milli);
                let distance_sq = dx * dx + dz * dz;
                (distance_sq <= range_sq).then_some((distance_sq, entity))
            })
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        candidates.first().map(|(_, entity)| *entity)
    }

    pub(super) fn run_raider_movement_system(&mut self) -> (u16, u16) {
        let mut raiders = self.raiders.keys().map(key_entity).collect::<Vec<_>>();
        raiders.sort_unstable();
        let flow_fields = self.raider_flow_fields(&raiders);
        let mut wood_stolen = 0_u16;
        let mut town_damage = 0_u16;
        let mut finished = Vec::new();

        for entity in raiders {
            let Some(mut movement) = self.movements.get(entity_key(entity)).copied() else {
                continue;
            };
            movement.progress_milli = movement.progress_milli.saturating_add(movement.speed_milli);

            if movement.progress_milli >= CELL_SCALE as u16 {
                movement.progress_milli -= CELL_SCALE as u16;
                movement.from = movement.to;
                let mut raider = self
                    .raiders
                    .get(entity_key(entity))
                    .copied()
                    .expect("raider ids come from the raider store");

                if self.storage_reached(raider.target_storage, movement.from) {
                    let stolen = self.take_wood_at(
                        raider.target_storage,
                        self.raider_wood_steal_amount(entity),
                    );
                    if stolen > 0 {
                        wood_stolen =
                            wood_stolen.saturating_add(u16::try_from(stolen).unwrap_or(u16::MAX));
                        finished.push(entity);
                        continue;
                    }
                    if raider.target_storage == TOWN_ENTITY {
                        let damage = self
                            .attacks
                            .get(entity_key(entity))
                            .map_or(0, |attack| attack.damage);
                        town_damage = town_damage.saturating_add(damage);
                        finished.push(entity);
                        continue;
                    }
                    raider.target_storage =
                        self.nearest_raider_storage_from_flow_fields(movement.from, &flow_fields);
                    self.raiders.insert(entity_key(entity), raider);
                }

                let mut next = flow_fields
                    .get(&raider.target_storage)
                    .and_then(|field| field.next_step(movement.from));
                if next.is_none() && raider.target_storage != TOWN_ENTITY {
                    raider.target_storage = TOWN_ENTITY;
                    self.raiders.insert(entity_key(entity), raider);
                    next = flow_fields
                        .get(&TOWN_ENTITY)
                        .and_then(|field| field.next_step(movement.from));
                }
                let Some(next) = next else {
                    movement.to = movement.from;
                    self.movements.insert(entity_key(entity), movement);
                    self.transforms
                        .insert(entity_key(entity), Transform::at_cell(movement.from));
                    continue;
                };
                movement.to = next;
            }

            let transform = interpolate_transform(movement);
            self.movements.insert(entity_key(entity), movement);
            self.transforms.insert(entity_key(entity), transform);
        }

        if town_damage > 0
            && let Some(health) = self.health.get_mut(entity_key(TOWN_ENTITY))
        {
            health.current = health.current.saturating_sub(town_damage);
        }
        for entity in finished {
            self.despawn_raider(entity);
        }

        (wood_stolen, town_damage)
    }

    fn raider_flow_fields(
        &self,
        active_raiders: &[EntityId],
    ) -> BTreeMap<EntityId, RaiderFlowField> {
        let needs_route = active_raiders.iter().any(|entity| {
            self.movements
                .get(entity_key(*entity))
                .is_some_and(|movement| {
                    movement.progress_milli.saturating_add(movement.speed_milli)
                        >= CELL_SCALE as u16
                })
        });
        if !needs_route {
            return BTreeMap::new();
        }

        let mut targets = active_raiders
            .iter()
            .filter_map(|entity| {
                self.raiders
                    .get(entity_key(*entity))
                    .map(|raider| raider.target_storage)
            })
            .collect::<Vec<_>>();
        targets.extend(self.settlement_storage_ids().into_iter().filter(|entity| {
            self.storage
                .get(entity_key(*entity))
                .is_some_and(|storage| storage.wood > 0)
        }));
        targets.push(TOWN_ENTITY);
        targets.sort_unstable();
        targets.dedup();

        targets
            .into_iter()
            .filter_map(|entity| {
                let goals = self.storage_goal_cells(entity, None);
                RaiderFlowField::build(self, &goals).map(|field| (entity, field))
            })
            .collect()
    }

    fn nearest_raider_storage_from_flow_fields(
        &self,
        start: Cell,
        flow_fields: &BTreeMap<EntityId, RaiderFlowField>,
    ) -> EntityId {
        let mut candidates = self
            .settlement_storage_ids()
            .into_iter()
            .filter_map(|entity| {
                let storage = self.storage.get(entity_key(entity))?;
                if storage.wood == 0 {
                    return None;
                }
                let distance = flow_fields.get(&entity)?.distance(start)?;
                Some((distance, entity))
            })
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        candidates
            .first()
            .map_or(TOWN_ENTITY, |(_, entity)| *entity)
    }

    pub(super) fn raider_wood_steal_amount(&self, entity: EntityId) -> u32 {
        if self.rules.raids.wave_plan.is_explicit() {
            return self
                .raiders
                .get(entity_key(entity))
                .map_or(0, |raider| self.rules.raider(raider.archetype).wood_steal);
        }
        self.rules.raids.base_wood_steal.saturating_add(
            self.wave
                .saturating_sub(1)
                .saturating_mul(self.rules.raids.wood_steal_per_wave),
        )
    }

    pub(super) fn spawn_raider(&mut self, edge: Edge) {
        self.spawn_raider_as(edge, RaiderArchetype::Basic);
    }

    pub(super) fn spawn_raider_as(&mut self, edge: Edge, archetype: RaiderArchetype) {
        let from = edge.spawn_cell();
        let target_storage = self.nearest_raider_storage(from);
        let goals = self.storage_goal_cells(target_storage, None);
        let to = self
            .next_path_step_to_any(from, &goals, None)
            .or_else(|| self.next_path_step_to_any(from, &town_goal_cells(), None))
            .expect("validated building placements keep every edge connected to town hall");
        let entity = self.allocate_entity();

        let (max_health, damage, speed_milli) = if self.rules.raids.wave_plan.is_explicit() {
            let profile = self.rules.raider(archetype);
            (profile.health, profile.damage, profile.speed_milli)
        } else {
            let wave_bonus = u16::try_from(self.wave.saturating_sub(1))
                .unwrap_or(u16::MAX)
                .saturating_mul(self.rules.raids.health_per_wave);
            let max_health = self.rules.raids.base_health.saturating_add(wave_bonus);
            let damage_steps =
                u16::try_from(self.wave / self.rules.raids.damage_increase_every_waves)
                    .unwrap_or(u16::MAX);
            let damage_bonus = damage_steps.saturating_mul(self.rules.raids.damage_increase_amount);
            (
                max_health,
                self.rules.raids.base_damage.saturating_add(damage_bonus),
                self.rules.raids.speed_milli,
            )
        };

        self.transforms
            .insert(entity_key(entity), Transform::at_cell(from));
        self.health.insert(
            entity_key(entity),
            Health {
                current: max_health,
                maximum: max_health,
            },
        );
        self.attacks.insert(
            entity_key(entity),
            Attack {
                damage,
                range_milli: 0,
                cooldown_ticks: 0,
                cooldown_remaining: 0,
                projectile_speed_milli: 0,
            },
        );
        self.raiders.insert(
            entity_key(entity),
            Raider {
                archetype,
                edge,
                target_storage,
            },
        );
        self.movements.insert(
            entity_key(entity),
            Movement {
                from,
                to,
                progress_milli: 0,
                speed_milli,
            },
        );
    }

    pub(super) fn spawn_person(&mut self) -> EntityId {
        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(town_center()));
        self.health.insert(
            entity_key(entity),
            Health {
                current: self.rules.population.person_health,
                maximum: self.rules.population.person_health,
            },
        );
        self.people.insert(
            entity_key(entity),
            Person {
                state: PersonState::IdleAtTownHall,
                target_entity: None,
                cargo_wood: 0,
                cargo_capacity: self.rules.population.carry_capacity,
            },
        );
        entity
    }

    pub(super) fn despawn_raider(&mut self, entity: EntityId) {
        let mut dependent_projectiles = self
            .projectiles
            .iter()
            .filter_map(|(key, projectile)| {
                (projectile.target == entity).then_some(key_entity(key))
            })
            .collect::<Vec<_>>();
        dependent_projectiles.sort_unstable();
        for projectile in dependent_projectiles {
            self.despawn_projectile(projectile);
        }

        let key = entity_key(entity);
        self.transforms.remove(key);
        self.health.remove(key);
        self.attacks.remove(key);
        self.raiders.remove(key);
        self.movements.remove(key);
        self.alive.remove(key);
    }

    pub(super) fn despawn_projectile(&mut self, entity: EntityId) {
        let key = entity_key(entity);
        self.transforms.remove(key);
        self.projectiles.remove(key);
        self.alive.remove(key);
    }

    pub(super) fn allocate_entity(&mut self) -> EntityId {
        let entity = self.next_entity;
        self.next_entity = self
            .next_entity
            .checked_add(1)
            .expect("entity id space exhausted");
        self.spawn_entity(entity);
        entity
    }

    pub(super) fn spawn_entity(&mut self, entity: EntityId) {
        let inserted = self.alive.insert(entity_key(entity));
        assert!(inserted, "entity ids must be unique");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raider_flow_field_matches_existing_bfs_routes() {
        let state = GameState::new(0x5eed);
        let goals = town_goal_cells();
        let flow = RaiderFlowField::build(&state, &goals).expect("town must have a flow field");

        for z in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = Cell::new(x, z);
                if state.path_cell_blocked(cell, None) {
                    continue;
                }
                assert_eq!(
                    flow.next_step(cell),
                    state.next_path_step_to_any(cell, &goals, None),
                    "flow field must preserve deterministic BFS routing from {cell:?}"
                );
            }
        }
    }

    #[test]
    fn raiders_between_cells_require_no_flow_fields() {
        let mut state = GameState::new(17);
        state.wave = 1;
        state.spawn_raider(Edge::North);
        let raiders = state.raiders.keys().map(key_entity).collect::<Vec<_>>();

        assert!(state.raider_flow_fields(&raiders).is_empty());
    }

    #[test]
    fn raiders_crossing_cells_get_all_retarget_candidates() {
        let mut state = GameState::new(17);
        state.wave = 1;
        state.spawn_raider(Edge::North);
        let raiders = state.raiders.keys().map(key_entity).collect::<Vec<_>>();
        let raider = raiders[0];
        let movement = state
            .movements
            .get_mut(entity_key(raider))
            .expect("spawned raider must move");
        movement.progress_milli = (CELL_SCALE as u16).saturating_sub(movement.speed_milli);
        let fields = state.raider_flow_fields(&raiders);

        assert!(fields.contains_key(&TOWN_ENTITY));
        for target in state.settlement_storage_ids().into_iter().filter(|entity| {
            state
                .storage
                .get(entity_key(*entity))
                .is_some_and(|storage| storage.wood > 0)
        }) {
            assert!(fields.contains_key(&target));
        }
    }

    #[test]
    fn flow_field_retargeting_matches_existing_bfs_storage_choice() {
        let mut state = GameState::new(17);
        let storage_cell = (1..GRID_HEIGHT - 1)
            .flat_map(|z| (1..GRID_WIDTH - 1).map(move |x| Cell::new(x, z)))
            .find(|cell| {
                let mut probe = state.clone();
                probe
                    .apply(Command::PlaceStorageHouse {
                        x: cell.x,
                        z: cell.z,
                    })
                    .is_ok()
            })
            .expect("seeded map must expose an accepted storage-house cell");
        state
            .apply(Command::PlaceStorageHouse {
                x: storage_cell.x,
                z: storage_cell.z,
            })
            .expect("storage house should build");
        let storage_house = state
            .settlement_storage_ids()
            .into_iter()
            .find(|entity| *entity != TOWN_ENTITY)
            .expect("storage house must join settlement storage");
        let transferred = state.take_wood_at(TOWN_ENTITY, 20);
        assert_eq!(transferred, 20);
        assert_eq!(state.store_wood_at(storage_house, transferred), transferred);

        state.wave = 1;
        state.spawn_raider(Edge::North);
        let raiders = state.raiders.keys().map(key_entity).collect::<Vec<_>>();
        let raider = raiders[0];
        let movement = state
            .movements
            .get_mut(entity_key(raider))
            .expect("spawned raider must move");
        movement.progress_milli = (CELL_SCALE as u16).saturating_sub(movement.speed_milli);
        let fields = state.raider_flow_fields(&raiders);

        for z in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = Cell::new(x, z);
                if state.path_cell_blocked(cell, None) {
                    continue;
                }
                assert_eq!(
                    state.nearest_raider_storage_from_flow_fields(cell, &fields),
                    state.nearest_raider_storage(cell),
                    "flow-field retargeting must preserve BFS storage choice from {cell:?}"
                );
            }
        }
    }
}

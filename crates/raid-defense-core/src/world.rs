use super::*;

const FOREST_MIN_DISTANCE_SQ: i32 = 9;
const FOREST_TOWN_CLEARANCE: i16 = 3;
const FOREST_LATTICE_PERIOD: i16 = 3;
const FOREST_LATTICE_VARIANTS: usize = 9;

impl GameState {
    pub(super) fn spawn_seeded_forests(&mut self) {
        let desired = usize::from(self.rules.economy.forest_tile_count);
        let mut candidates = Vec::new();
        for z in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = Cell::new(x, z);
                if self.forest_candidate_cell(cell) {
                    candidates.push(cell);
                }
            }
        }
        candidates.sort_by_key(|cell| {
            let index = u64::try_from(cell_index(*cell).unwrap_or_default()).unwrap_or_default();
            (
                mix64(self.seed ^ index.wrapping_mul(0x9e37_79b9)),
                cell.z,
                cell.x,
            )
        });

        let primary = self.select_forest_cells(&candidates, desired);
        let selected = if primary.len() == desired {
            primary
        } else {
            self.select_lattice_forest_cells(&candidates, desired)
                .unwrap_or(primary)
        };
        assert_eq!(
            selected.len(),
            desired,
            "forest_tile_count exceeds deterministic seeded layout capacity"
        );

        for cell in selected {
            self.spawn_forest_cell(cell);
        }
    }

    fn select_forest_cells(&self, candidates: &[Cell], desired: usize) -> Vec<Cell> {
        let mut probe = self.clone();
        let mut selected = Vec::with_capacity(desired);
        for &cell in candidates {
            if selected.len() >= desired {
                break;
            }
            if selected
                .iter()
                .any(|other| cell_distance_sq(cell, *other) < FOREST_MIN_DISTANCE_SQ)
            {
                continue;
            }
            if !probe.routes_remain_open(Some(cell)) {
                continue;
            }

            probe.spawn_forest_cell(cell);
            selected.push(cell);
        }
        selected
    }

    fn select_lattice_forest_cells(
        &self,
        candidates: &[Cell],
        desired: usize,
    ) -> Option<Vec<Cell>> {
        let first_variant = usize::try_from(mix64(self.seed) % 9).unwrap_or_default();
        for offset in 0..FOREST_LATTICE_VARIANTS {
            let variant = (first_variant + offset) % FOREST_LATTICE_VARIANTS;
            let x_residue = i16::try_from(variant % 3).unwrap_or_default();
            let z_residue = i16::try_from(variant / 3).unwrap_or_default();
            let lattice = candidates
                .iter()
                .copied()
                .filter(|cell| {
                    cell.x.rem_euclid(FOREST_LATTICE_PERIOD) == x_residue
                        && cell.z.rem_euclid(FOREST_LATTICE_PERIOD) == z_residue
                })
                .collect::<Vec<_>>();
            let selected = self.select_forest_cells(&lattice, desired);
            if selected.len() == desired {
                return Some(selected);
            }
        }
        None
    }

    fn spawn_forest_cell(&mut self, cell: Cell) {
        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(cell));
        self.buildings.insert(
            entity_key(entity),
            Building {
                kind: BuildingKind::Forest,
                cell,
            },
        );
        self.storage.insert(
            entity_key(entity),
            ResourceStorage {
                wood: self.rules.economy.forest_tile_wood,
                wood_capacity: self.rules.economy.forest_tile_wood,
            },
        );
    }

    fn forest_candidate_cell(&self, cell: Cell) -> bool {
        if cell.x <= 0 || cell.x >= GRID_WIDTH - 1 || cell.z <= 0 || cell.z >= GRID_HEIGHT - 1 {
            return false;
        }
        if is_town_cell(cell) || Edge::ALL.into_iter().any(|edge| edge.spawn_cell() == cell) {
            return false;
        }
        let town = town_center();
        (cell.x - town.x).abs() > FOREST_TOWN_CLEARANCE
            || (cell.z - town.z).abs() > FOREST_TOWN_CLEARANCE
    }

    pub(super) fn forest_count_internal(&self) -> usize {
        self.buildings
            .iter()
            .filter(|(_, building)| building.kind == BuildingKind::Forest)
            .count()
    }

    pub(super) fn run_forest_regrowth_system(&mut self) {
        let interval = u64::from(self.rules.economy.forest_regrowth_interval_ticks);
        if !self.tick.is_multiple_of(interval) {
            return;
        }

        let amount = u32::from(self.rules.economy.forest_regrowth_amount);
        let mut forests = self
            .buildings
            .iter()
            .filter_map(|(key, building)| {
                (building.kind == BuildingKind::Forest).then_some(key_entity(key))
            })
            .collect::<Vec<_>>();
        forests.sort_unstable();

        for forest in forests {
            let Some(storage) = self.storage.get_mut(entity_key(forest)) else {
                continue;
            };
            storage.wood = storage
                .wood
                .saturating_add(amount)
                .min(storage.wood_capacity);
        }
    }

    fn best_forest_for_cell(&self, cell: Cell) -> Option<EntityId> {
        let sawmill_goals = self.adjacent_walkable_cells(cell, None);
        if sawmill_goals.is_empty() {
            return None;
        }

        let mut candidates = self
            .buildings
            .iter()
            .filter_map(|(key, building)| {
                if building.kind != BuildingKind::Forest {
                    return None;
                }
                let entity = key_entity(key);
                let storage = self.storage.get(key)?;
                if storage.wood == 0 {
                    return None;
                }
                let forest_goals = self.adjacent_walkable_cells(building.cell, None);
                let distance = self.distance_between_goal_sets(&sawmill_goals, &forest_goals)?;
                Some((distance, entity))
            })
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        candidates.first().map(|(_, entity)| *entity)
    }

    pub(super) fn harvest_forest_for_sawmill(&mut self, sawmill: EntityId, amount: u32) -> u32 {
        let Some(sawmill_cell) = self
            .buildings
            .get(entity_key(sawmill))
            .map(|building| building.cell)
        else {
            return 0;
        };
        let Some(forest) = self.best_forest_for_cell(sawmill_cell) else {
            return 0;
        };
        let Some(storage) = self.storage.get_mut(entity_key(forest)) else {
            return 0;
        };
        let harvested = storage.wood.min(amount);
        storage.wood -= harvested;
        harvested
    }

    pub(super) fn settlement_storage_ids(&self) -> Vec<EntityId> {
        let mut ids = self
            .buildings
            .iter()
            .filter_map(|(key, building)| {
                matches!(
                    building.kind,
                    BuildingKind::TownHall | BuildingKind::StorageHouse
                )
                .then_some(key_entity(key))
            })
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    pub(super) fn settlement_wood_total(&self) -> u32 {
        self.settlement_storage_ids()
            .into_iter()
            .filter_map(|entity| self.storage.get(entity_key(entity)))
            .fold(0_u32, |total, storage| total.saturating_add(storage.wood))
    }

    pub(super) fn settlement_wood_capacity_total(&self) -> u32 {
        self.settlement_storage_ids()
            .into_iter()
            .filter_map(|entity| self.storage.get(entity_key(entity)))
            .fold(0_u32, |total, storage| {
                total.saturating_add(storage.wood_capacity)
            })
    }

    pub(super) fn spend_settlement_wood(&mut self, amount: u32) {
        let mut remaining = amount;
        for entity in self.settlement_storage_ids() {
            if remaining == 0 {
                break;
            }
            let Some(storage) = self.storage.get_mut(entity_key(entity)) else {
                continue;
            };
            let spent = storage.wood.min(remaining);
            storage.wood -= spent;
            remaining -= spent;
        }
        debug_assert_eq!(remaining, 0, "callers must check available settlement wood");
    }

    pub(super) fn store_wood_at(&mut self, storage_entity: EntityId, amount: u32) -> u32 {
        let Some(storage) = self.storage.get_mut(entity_key(storage_entity)) else {
            return 0;
        };
        let available = storage.wood_capacity.saturating_sub(storage.wood);
        let stored = available.min(amount);
        storage.wood = storage.wood.saturating_add(stored);
        stored
    }

    pub(super) fn take_wood_at(&mut self, storage_entity: EntityId, amount: u32) -> u32 {
        let Some(storage) = self.storage.get_mut(entity_key(storage_entity)) else {
            return 0;
        };
        let taken = storage.wood.min(amount);
        storage.wood -= taken;
        taken
    }

    pub(super) fn nearest_storage_with_capacity(
        &self,
        start: Cell,
        extra_block: Option<Cell>,
    ) -> Option<EntityId> {
        let mut candidates = self
            .settlement_storage_ids()
            .into_iter()
            .filter_map(|entity| {
                let storage = self.storage.get(entity_key(entity))?;
                if storage.wood >= storage.wood_capacity {
                    return None;
                }
                let goals = self.storage_goal_cells(entity, extra_block);
                let distance = self.path_distance_to_any(start, &goals, extra_block)?;
                Some((distance, entity))
            })
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        candidates.first().map(|(_, entity)| *entity)
    }

    pub(super) fn nearest_storage_with_wood_for_site(
        &self,
        site: EntityId,
        worker_start: Cell,
    ) -> Option<EntityId> {
        let site_cell = self.buildings.get(entity_key(site))?.cell;
        let site_goals = self.adjacent_walkable_cells(site_cell, None);
        let mut candidates = self
            .settlement_storage_ids()
            .into_iter()
            .filter_map(|entity| {
                let storage = self.storage.get(entity_key(entity))?;
                if storage.wood == 0 {
                    return None;
                }
                let storage_goals = self.storage_goal_cells(entity, None);
                self.path_distance_to_any(worker_start, &storage_goals, None)?;
                let site_distance = self.distance_between_goal_sets(&storage_goals, &site_goals)?;
                Some((site_distance, entity))
            })
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        candidates.first().map(|(_, entity)| *entity)
    }

    pub(super) fn nearest_raider_storage(&self, start: Cell) -> EntityId {
        let mut candidates = self
            .settlement_storage_ids()
            .into_iter()
            .filter_map(|entity| {
                let storage = self.storage.get(entity_key(entity))?;
                if storage.wood == 0 {
                    return None;
                }
                let goals = self.storage_goal_cells(entity, None);
                let distance = self.path_distance_to_any(start, &goals, None)?;
                Some((distance, entity))
            })
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        candidates
            .first()
            .map_or(TOWN_ENTITY, |(_, entity)| *entity)
    }

    pub(super) fn storage_goal_cells(
        &self,
        entity: EntityId,
        extra_block: Option<Cell>,
    ) -> Vec<Cell> {
        if entity == TOWN_ENTITY {
            return town_goal_cells();
        }
        let Some(building) = self.buildings.get(entity_key(entity)) else {
            return Vec::new();
        };
        self.adjacent_walkable_cells(building.cell, extra_block)
    }

    pub(super) fn storage_reached(&self, entity: EntityId, cell: Cell) -> bool {
        self.storage_goal_cells(entity, None).contains(&cell)
    }

    pub(super) fn construction_sites(&self) -> Vec<EntityId> {
        let mut sites = self
            .towers
            .iter()
            .filter_map(|(key, tower)| (tower.level == 0).then_some(key_entity(key)))
            .collect::<Vec<_>>();
        sites.sort_unstable();
        sites
    }

    pub(super) fn construction_remaining(&self, site: EntityId) -> u32 {
        self.storage.get(entity_key(site)).map_or(0, |storage| {
            storage.wood_capacity.saturating_sub(storage.wood)
        })
    }

    pub(super) fn deliver_construction_wood(&mut self, site: EntityId, amount: u16) -> u16 {
        let stored = self.store_wood_at(site, u32::from(amount));
        u16::try_from(stored).unwrap_or(u16::MAX)
    }

    pub(super) fn complete_construction_if_ready(&mut self, site: EntityId) -> bool {
        let ready = self
            .storage
            .get(entity_key(site))
            .is_some_and(|storage| storage.wood >= storage.wood_capacity);
        if !ready {
            return false;
        }
        let Some(mut tower) = self.towers.get(entity_key(site)).copied() else {
            return false;
        };
        if tower.level != 0 {
            return false;
        }
        tower.level = 1;
        self.towers.insert(entity_key(site), tower);
        self.storage.remove(entity_key(site));
        self.attacks.insert(
            entity_key(site),
            attack_for(self.rules, tower.archetype, 1, 0),
        );
        true
    }

    pub(super) fn adjacent_walkable_cells(
        &self,
        cell: Cell,
        extra_block: Option<Cell>,
    ) -> Vec<Cell> {
        neighbors(cell)
            .into_iter()
            .filter(|candidate| !self.path_cell_blocked(*candidate, extra_block))
            .collect()
    }

    fn distance_between_goal_sets(&self, starts: &[Cell], goals: &[Cell]) -> Option<u16> {
        starts
            .iter()
            .filter_map(|start| self.path_distance_to_any(*start, goals, None))
            .min()
    }

    pub(super) fn path_distance_to_any(
        &self,
        start: Cell,
        goals: &[Cell],
        extra_block: Option<Cell>,
    ) -> Option<u16> {
        if goals.contains(&start) {
            return Some(0);
        }
        if !in_bounds(start) || goals.is_empty() {
            return None;
        }

        let cell_count = usize::try_from(GRID_WIDTH).ok()? * usize::try_from(GRID_HEIGHT).ok()?;
        let mut visited = vec![false; cell_count];
        let mut queue = VecDeque::new();
        let start_index = cell_index(start)?;
        visited[start_index] = true;
        queue.push_back((start, 0_u16));

        while let Some((cell, distance)) = queue.pop_front() {
            for neighbor in neighbors(cell) {
                let index = cell_index(neighbor)?;
                if visited[index] || self.path_cell_blocked(neighbor, extra_block) {
                    continue;
                }
                let next_distance = distance.saturating_add(1);
                if goals.contains(&neighbor) {
                    return Some(next_distance);
                }
                visited[index] = true;
                queue.push_back((neighbor, next_distance));
            }
        }
        None
    }
}

fn cell_distance_sq(a: Cell, b: Cell) -> i32 {
    let dx = i32::from(a.x - b.x);
    let dz = i32::from(a.z - b.z);
    dx.saturating_mul(dx).saturating_add(dz.saturating_mul(dz))
}

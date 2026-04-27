use super::*;
use std::collections::BTreeMap;

const ENCOUNTER_TIME_OFFSET: u64 = 1;
const UNIT_ENCOUNTER_PREFIX: &str = "wiki_seen_unit:";
const ATTACK_WAVE_COUNT: &str = "wiki_attack_wave_count";
const ATTACK_WAVE_STARTED_PREFIX: &str = "wiki_attack_wave_started:";
const ATTACK_WAVE_ENTRY_X_PREFIX: &str = "wiki_attack_wave_entry_x:";
const ATTACK_WAVE_ENTRY_Y_PREFIX: &str = "wiki_attack_wave_entry_y:";
const ATTACK_WAVE_UNIT_COUNT_PREFIX: &str = "wiki_attack_wave_unit_count:";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoredAttackWave {
    pub id: u32,
    pub encountered_at_seconds: u64,
    pub entry: MapLocation,
    pub unit_counts: BTreeMap<String, u32>,
}

pub(crate) fn record_unit_encounter(state: &mut GameState, kind: &str) {
    let stat = unit_encounter_stat(kind);
    if state.stat(stat.clone()) == 0 {
        state.set_stat(stat, encode_encounter_seconds(state.now_seconds()));
    }
}

pub(crate) fn record_attack_wave(
    state: &mut GameState,
    unit_kind: &str,
    entry: MapLocation,
) -> Result<u32, EngineError> {
    record_unit_encounter(state, unit_kind);

    let wave_id = next_attack_wave_id(state, entry)?;
    let unit_stat = attack_wave_unit_count_stat(wave_id, unit_kind);
    let current_count = u32::try_from(state.stat(unit_stat.clone()).max(0)).unwrap_or(0);
    state.set_stat(unit_stat, i64::from(current_count.saturating_add(1)));
    Ok(wave_id)
}

pub(crate) fn encountered_units(state: &GameState) -> Vec<(String, u64)> {
    state
        .stats()
        .filter_map(|(stat, value)| {
            let kind = stat.as_str().strip_prefix(UNIT_ENCOUNTER_PREFIX)?;
            decode_encounter_seconds(*value).map(|encountered_at_seconds| {
                (kind.to_owned(), encountered_at_seconds)
            })
        })
        .collect()
}

pub(crate) fn encountered_attack_waves(state: &GameState) -> Vec<StoredAttackWave> {
    let total = u32::try_from(state.stat(ATTACK_WAVE_COUNT).max(0)).unwrap_or(0);
    (1..=total)
        .filter_map(|wave_id| {
            let encountered_at_seconds =
                decode_encounter_seconds(state.stat(attack_wave_started_stat(wave_id)))?;
            let entry = MapLocation::new(
                i32::try_from(state.stat(attack_wave_entry_x_stat(wave_id))).unwrap_or_default(),
                i32::try_from(state.stat(attack_wave_entry_y_stat(wave_id))).unwrap_or_default(),
            );
            let unit_counts = state
                .stats()
                .filter_map(|(stat, value)| {
                    let prefix = format!("{ATTACK_WAVE_UNIT_COUNT_PREFIX}{wave_id}:");
                    let kind = stat.as_str().strip_prefix(&prefix)?;
                    let count = u32::try_from((*value).max(0)).unwrap_or(0);
                    (count > 0).then(|| (kind.to_owned(), count))
                })
                .collect::<BTreeMap<_, _>>();
            Some(StoredAttackWave {
                id: wave_id,
                encountered_at_seconds,
                entry,
                unit_counts,
            })
        })
        .collect()
}

fn next_attack_wave_id(state: &mut GameState, entry: MapLocation) -> Result<u32, EngineError> {
    let current = u32::try_from(state.stat(ATTACK_WAVE_COUNT).max(0)).unwrap_or(0);
    if current > 0 {
        let last_started = decode_encounter_seconds(state.stat(attack_wave_started_stat(current)));
        let last_entry = MapLocation::new(
            i32::try_from(state.stat(attack_wave_entry_x_stat(current))).unwrap_or_default(),
            i32::try_from(state.stat(attack_wave_entry_y_stat(current))).unwrap_or_default(),
        );
        if last_started == Some(state.now_seconds()) && last_entry == entry {
            return Ok(current);
        }
    }

    let next = current.saturating_add(1);
    state.set_stat(ATTACK_WAVE_COUNT, i64::from(next));
    state.set_stat(
        attack_wave_started_stat(next),
        encode_encounter_seconds(state.now_seconds()),
    );
    state.set_stat(attack_wave_entry_x_stat(next), i64::from(entry.x));
    state.set_stat(attack_wave_entry_y_stat(next), i64::from(entry.y));
    Ok(next)
}

fn encode_encounter_seconds(seconds: u64) -> i64 {
    i64::try_from(seconds.saturating_add(ENCOUNTER_TIME_OFFSET)).unwrap_or(i64::MAX)
}

fn decode_encounter_seconds(value: i64) -> Option<u64> {
    (value > 0).then(|| u64::try_from(value).unwrap_or(u64::MAX) - ENCOUNTER_TIME_OFFSET)
}

fn unit_encounter_stat(kind: &str) -> String {
    format!("{UNIT_ENCOUNTER_PREFIX}{kind}")
}

fn attack_wave_started_stat(wave_id: u32) -> String {
    format!("{ATTACK_WAVE_STARTED_PREFIX}{wave_id}")
}

fn attack_wave_entry_x_stat(wave_id: u32) -> String {
    format!("{ATTACK_WAVE_ENTRY_X_PREFIX}{wave_id}")
}

fn attack_wave_entry_y_stat(wave_id: u32) -> String {
    format!("{ATTACK_WAVE_ENTRY_Y_PREFIX}{wave_id}")
}

fn attack_wave_unit_count_stat(wave_id: u32, kind: &str) -> String {
    format!("{ATTACK_WAVE_UNIT_COUNT_PREFIX}{wave_id}:{kind}")
}

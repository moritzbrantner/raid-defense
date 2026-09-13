use raid_defense_core::{ForestDensity, GameState, ScenarioOptions};

#[test]
fn dense_forest_profile_realizes_requested_count_deterministically() {
    let mut scenario = ScenarioOptions::standard();
    scenario.forest_density = ForestDensity::Dense;
    let rules = scenario.into_rules().expect("dense scenario rules must validate");

    for seed in [0_u64, 1, 7, 8, 189, 24_301, u64::from(u32::MAX)] {
        let first = GameState::try_with_rules(seed, rules)
            .expect("dense scenario must produce a realizable seeded world");
        let second = GameState::try_with_rules(seed, rules)
            .expect("replaying the same dense scenario must remain realizable");

        assert_eq!(
            first.forest_count(),
            usize::from(rules.economy.forest_tile_count),
            "seed {seed} must realize the authoritative forest count"
        );
        assert_eq!(
            first.snapshot(),
            second.snapshot(),
            "seed {seed} must reproduce the exact same authoritative starting state"
        );
    }
}

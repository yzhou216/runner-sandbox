use crate::common::run_recorded_test;

#[test]
fn score_deterministic() {
    run_recorded_test(
        "score-deterministic.json",
        "config-deterministic.json",
        true,
    );
}

#[test]
fn score_deterministic2() {
    run_recorded_test(
        "score-deterministic2.json",
        "config-deterministic.json",
        true,
    );
}

#[test]
fn score_deterministic3() {
    run_recorded_test(
        "score-deterministic3.json",
        "config-deterministic.json",
        true,
    );
}

#[test]
fn score_deterministic4() {
    run_recorded_test(
        "score-deterministic4.json",
        "config-deterministic.json",
        true,
    );
}

#[test]
#[cfg(all(feature = "hard_drop", feature = "hold"))]
fn level_up() {
    run_recorded_test("level-up.json", "config-deterministic.json", true);
}

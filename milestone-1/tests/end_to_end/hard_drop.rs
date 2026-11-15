use crate::common::run_recorded_test;

#[test]
fn hard_drop_deterministic() {
    run_recorded_test(
        "hard-drop-deterministic.json",
        "config-deterministic.json",
        false,
    );
}

#[test]
fn hard_drop_deterministic2() {
    run_recorded_test(
        "hard-drop-deterministic2.json",
        "config-deterministic.json",
        false,
    );
}

#[test]
#[cfg(feature = "rng")]
fn hard_drop_67() {
    run_recorded_test("hard-drop-67.json", "config-67.json", false);
}

#[test]
#[cfg(feature = "rng")]
fn hard_drop_727() {
    run_recorded_test("hard-drop-727.json", "config-727.json", false);
}

#[test]
#[cfg(all(feature = "rng", feature = "hold"))]
fn hard_drop_hold_0() {
    run_recorded_test("hard-drop-hold-0.json", "config-0.json", false);
}

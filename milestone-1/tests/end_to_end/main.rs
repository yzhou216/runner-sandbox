#[macro_use]
extern crate libtest_mimic_collect;

extern crate image;

mod baseline;
mod common;
mod test_bag;

#[cfg(feature = "collision")]
mod collision;

#[cfg(feature = "rng")]
mod rng;

#[cfg(feature = "score")]
mod score;

#[cfg(feature = "hard_drop")]
mod hard_drop;

#[cfg(feature = "hold")]
mod hold;

fn main() {
    if cfg!(not(target_os = "linux")) {
        eprintln!(
            r"WARNING: YOU ARE RUNNING AN OS OTHER THAN LINUX!
THE TEST HARNESS DOES NOT OFFICIALLY SUPPORT OTHER OPERATING SYSTEMS!

G O O D  L U C K
"
        );
    }
    libtest_mimic_collect::TestCollection::run();
}

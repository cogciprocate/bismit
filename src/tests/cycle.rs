use tests::testbed_vibi;
// use cortex::{Cortex};

#[test]
fn cycle() {
	let mut cortex = testbed_vibi::new_cortex();
	let area_name = "v1".to_owned();
	let mut cycle_iters = 2000;

	for _ in 0..cycle_iters {
		cortex.cycle();
	}
}
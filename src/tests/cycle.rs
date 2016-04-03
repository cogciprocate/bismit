#![allow(unused_variables, unused_mut)]

use tests::testbed_vibi;
// use cortex::Cortex;
const ITERS: i32 = 10000;

#[test]
fn cycle() {
	let mut cortex = testbed_vibi::new_cortex();
	let area_name = "v1".to_owned();
	let mut cycle_iters = ITERS;

	let aff_out_slc_range = cortex.area(&area_name).area_map().aff_out_slc_range();
	let tract_map = cortex.area(&area_name).axn_tract_map();

	for _ in 0..cycle_iters {
		cortex.cycle();
	}
}
use tests::testbed_vibi;

const ITERS: i32 = 10000;


/// Just cycle enough times to allow learning and any other infrequent things
/// to happen.
#[test]
fn cycle_a_bunch() {
	let mut cortex = testbed_vibi::new_cortex();
	// let area_name = "v1".to_owned();
	let cycle_iters = ITERS;

	// let aff_out_slc_range = cortex.areas().by_key(&area_name).area_map().aff_out_slc_range();
	// let tract_map = cortex.areas().by_key(&area_name).axn_tract_map();

	for _ in 0..cycle_iters {
		cortex.cycle().unwrap();
	}
}
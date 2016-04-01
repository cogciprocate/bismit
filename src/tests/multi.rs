use tests::testbed_vibi;

#[test]
fn multi_layer_input() {
	let mut cortex = testbed_vibi::new_cortex();

	let cycle_iters = 5;

	for _ in 0..cycle_iters {
		cortex.cycle();
	}
}
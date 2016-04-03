use map;
use tests::testbed_vibi;

#[test]
fn multi_layer_input() {
	let mut cortex = testbed_vibi::new_cortex();

	let cycle_iters = 3;

	for _ in 0..cycle_iters {
		cortex.cycle();

		for (_, area) in cortex.areas().iter() {
			let slc_ids = area.area_map().axn_base_slc_ids_by_tags(map::NS_IN);
			let mut buf = vec![0; 256];
			area.sample_axn_slc(slc_ids[0], &mut buf[..]);
			println!("TESTS::MULTI_LAYER_INPUT(): 'NS_IN' output: slice_id: {}, buf: {:?}", 
				slc_ids[0], buf);
		}
	}
}
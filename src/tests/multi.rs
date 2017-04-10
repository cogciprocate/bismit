// use map;
use tests::testbed_vibi;

#[test]
fn multi_layer_input() {
	let mut cortex = testbed_vibi::new_cortex();

	let cycle_iters = 3;

	for _ in 0..cycle_iters {
		cortex.cycle();

		for area in cortex.areas().values().iter() {
			// let slc_ranges = area.area_map().layers().layers_containing_tags_slc_range(map::NS_IN);

			let input_layers = area.area_map().layers().iter()
				.filter(|li| li.is_input()).map(|li| li)
				.collect::<Vec<_>>();

			let slc_ranges = input_layers.iter()
	            .filter(|l| l.slc_range().is_some())
	            .map(|l| l.slc_range().unwrap().clone())
	            .collect::<Vec<_>>();

			let mut buf = vec![0; 256];

			for slc_range in slc_ranges {
				if slc_range.len() > 0 {
					area.sample_axn_slc_range(&slc_range, &mut buf[..]).wait_for().unwrap();
					println!("TESTS::MULTI_LAYER_INPUT(): 'NS_IN' output: slice_id: {}, buf: {:?}",
						slc_range.start, buf);
				}
			}
		}
	}
}
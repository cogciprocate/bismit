/*pub struct Somata {
	depth: u8,
	dims: CorticalDimensions, height: u32, 
	pub dst_dens: Dendrites,
	pub states: Envoy<ocl::cl_uchar>,
	pub hcol_max_vals: Envoy<ocl::cl_uchar>,
	pub hcol_max_ids: Envoy<ocl::cl_uchar>,
	pub rand_ofs: Envoy<ocl::cl_char>,
}

impl Somata {
	pub fn new(dims: CorticalDimensions, height: u32,  depth: u8, protoregion: &Protoregion, ocl: &Ocl) -> Somata {
		Somata { 
			depth: depth,
			width: width, height: height, 
			states: Envoy::<ocl::cl_uchar>::new(width, depth, cmn::STATE_ZERO, ocl),
			hcol_max_vals: Envoy::<ocl::cl_uchar>::new(dims.width / cmn::COLUMNS_PER_HYPERCOLUMN, depth, cmn::STATE_ZERO, ocl),
			hcol_max_ids: Envoy::<ocl::cl_uchar>::new(dims.width / cmn::COLUMNS_PER_HYPERCOLUMN, depth, 0u8, ocl),
			rand_ofs: Envoy::<ocl::cl_char>::shuffled(256, 1, -128, 127, ocl),
			dst_dens: Dendrites::new(width, depth, DendriteKind::Distal, cmn::DENDRITES_PER_CELL_DISTAL, protoregion, ocl),

		}
	}

	fn cycle_pre(&self, dst_dens: &Dendrites, prx_dens: &Dendrites, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_cycle_pre");
		ocl::set_kernel_arg(1, prx_dens.states.buf, kern);
		ocl::set_kernel_arg(2, self.states.buf, kern);

		let gws = (self.depth as usize, self.dims.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	}

	fn cycle(&self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_cycle_post");
		ocl::set_kernel_arg(0, self.dst_dens.states.buf, kern);
		//ocl::set_kernel_arg(1, self.bsl_prx_dens.states.buf, kern);
		ocl::set_kernel_arg(1, self.states.buf, kern);
		ocl::set_kernel_arg(2, self.depth as u32, kern);

		let gws = (self.depth as usize, self.dims.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	}

	pub fn inhib(&self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_inhib");
		ocl::set_kernel_arg(0, self.states.buf, kern);
		ocl::set_kernel_arg(1, self.hcol_max_ids.buf, kern);
		ocl::set_kernel_arg(2, self.hcol_max_vals.buf, kern);
		let mut kern_dims.width = self.dims.width as usize / cmn::COLUMNS_PER_HYPERCOLUMN as usize;
		let gws = (self.depth as usize, kern_width);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

		ocl::set_kernel_arg(0, self.aux.chars_0.buf, kern);
		ocl::set_kernel_arg(1, self.aux.chars_1.buf, kern);
		kern_dims.width = kern_dims.width / (1 << grp_size_log2);
		let gws = (self.depth_cellular as usize, self.dims.width as usize / 64);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);
	}

	pub fn ltp(&mut self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "syns_ltp");
		ocl::set_kernel_arg(0, self.hcol_max_ids.buf, kern);
		ocl::set_kernel_arg(1, self.dst_dens.syns.states.buf, kern);
		ocl::set_kernel_arg(2, self.dst_dens.thresholds.buf, kern);
		ocl::set_kernel_arg(3, self.dst_dens.states.buf, kern);
		ocl::set_kernel_arg(4, self.dst_dens.syns.strengths.buf, kern);
		ocl::set_kernel_arg(5, self.rand_ofs.buf, kern);

		let mut kern_dims.width = self.dims.width as usize / cmn::COLUMNS_PER_HYPERCOLUMN as usize;
		let gws = (self.depth as usize, kern_width);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);
	}
}*/


/*let peek_vec = world.peek_from(worm.uid).unfold();

	cmn::print_vec_simple(&peek_vec);

	world.entities().get_mut(worm.uid).turn(0.0004f32);*/


	//let mut worm_brain = EntityBrain::new(worm.uid, &world);
	//let mut snake_brain = SnakeBrain::new(snake.uid);

	//let chord = render_peek(world.peek_from(worm.uid));
	//chord.print();
	//chord.unfold().print();

	// for i in range(0, 100000) {
	/*for i in 0..5 {
		if worm_brain.act(&mut world) == Option::None {
			println!("");
			println!("Everything eaten after {} iterations.", i);
			break
		}
	}*/



	/*cortex.sense_vec(0, "pre-thal", &mut vec1);
	cortex.sense_vec(0, "post-thal", &mut vec1);*/







/*pub fn pe<T: Integer + Copy + Clone + NumCast + Default + Display + FromPrimitive + ToPrimitive, V>(label: &'static str, env: &Envoy<T>, scale: usize, 
				val_range: Option<(V, V)>, 
				idx_range: Option<(usize, usize)>
) {
	print!("\n{}: ", label);
	env.len();
	//env.print(scale, val_range, idx_range);
}*/


		//
		// 128:1 RATIO FOR PRINTING IS COOL (100% ACTIVITY)
		// 512:1 (25% ACTIVITY, 262144 len)
		//

		//	println!("\n tmp_out: ");
		//	cortex.sensory_segments[0].tmp_out.print(1000);


		/* SYNAPSE COL_OFS (SRC_OFS) */

		/*print!("\ncols.syns.src_ofs:");
		cortex.cortical_area.mcols.dens.syns.src_ofs.print_val_range(1 << 12, -128, 127);*/

		/*if false {
			print!("\nsoma.bsl_dst_dens.syns.src_col_xy_offs:");
			cortex.cortical_area.soma.bsl_dst_dens.syns.src_col_xy_offs.print(1 << 14);		// 16384

			print!("\ncols.bsl_prx_dens.syns.src_col_xy_offs:");
			cortex.cortical_area.mcols.bsl_prx_dens.syns.src_col_xy_offs.print(1 << 16);
		}*/

		/* SYNAPSE AXN_ROW_IDS */

		/*if false {
			print!("\nsoma.bsl_dst_dens.syns.src_slice_ids:");
			cortex.cortical_area.soma.bsl_dst_dens.syns.src_slice_ids.print(1 << 14);		// 16384
		}

		if false {
			print!("\ncols.bsl_prx_dens.syns.src_slice_ids:");
			cortex.cortical_area.mcols.bsl_prx_dens.syns.src_slice_ids.print(1 << 10);
		}*/


		/* SYNAPSE STRENGTHS */

		/*if false {		
			println!("\nsoma.bsl_dst_dens.syns.strengths: ");
			cortex.cortical_area.soma.bsl_dst_dens.syns.strengths.print_val_range(1 << 6, 17, 127);
		}

		if false {
			print!("\ncols.bsl_prx_dens.syns.strengths: ");
			cortex.cortical_area.mcols.bsl_prx_dens.syns.strengths.print_val_range(1 << 4, 17, 127);
		}*/

		/*if true {	
			print!("\nsoma.dst_dens.syns.states: ");
			cortex.cortical_area.soma.dst_dens.syns.states.print(1 << 14);
		}*/

		/*if true {
			print!("\ncols.bsl_prx_dens.syns.states: ");
			cortex.cortical_area.mcols.bsl_prx_dens.syns.states.print(1 << 10);
		}*/

		/* DENDRITE STATES */

		/*if true {
			print!("\nsoma.bsl_dst_dens.states: ");
			cortex.cortical_area.soma.bsl_dst_dens.states.print(1 << 10);
		}

		if true {
			print!("\ncols.bsl_prx_dens.states: ");
			cortex.cortical_area.mcols.bsl_prx_dens.states.print(1 << 6);
		}*/


		/* AUX VALS */

		/*if true {
			print!("\naux.chars_0: ");
			cortex.cortical_area.aux.chars_0.print(1 << 0);
		}

		if true {
			print!("\naux.chars_1: ");
			cortex.cortical_area.aux.chars_1.print(1 << 0);
		}*/



	/*if false {
		println!("\n cortical_area.somata.states: ");
		cortex.cortical_area.somata.states.print(1 << 8);
	}*/


	/*let mut t3: Envoy<i8> = Envoy::new(64, 0i8, &cortex.ocl);

	println!("t3.vec.len(): {}", t3.vec.len());


	let tkern = ocl::new_kernel(cortex.ocl.program, "test_int_shift");

	ocl::set_kernel_arg(0, t3.buf, tkern);
	ocl::set_kernel_arg(1, -96i8, tkern);

	ocl::enqueue_kernel(tkern, cortex.ocl.command_queue, t3.vec.len());

	t3.read();

	cmn::print_vec(&t3.vec, 1, true);*/


	/*##### Testing Axon Stuff #####
		let mut vec2: Vec<u8> = iter::repeat(0).take(1024 * 256).collect();

		let mut tar_idxs: Vec<usize> = iter::repeat(0).take(1024 * 256).collect();


		for i in range(0, vec2.len()) {

			tar_idxs[i] = ((cortex.sensory_segments[0].target_idxesses.target_column_bodies.vec[i] as usize) << 8 ) + cortex.sensory_segments[0].target_idxesses.target_column_dst_dens.syns.vec[i] as usize;

			//	print!("[{}: {}]", i, tar_idx);

			vec2[tar_idxs[i]] = vec1[i as usize >> 8us];
			

			//	*x = (tcb_vec[i] << 8) as u32 + tcs_vec[i] as u32;

		}

		//cmn::dup_check(&tar_idxs);

		//println!("First 3: {}, {}, {}", tar_idxs[0], tar_idxs[1], tar_idxs[2]);
		//println!("Last 3: {}, {}, {}", tar_idxs[tar_idxs.len() - 1], tar_idxs[tar_idxs.len() - 2], tar_idxs[tar_idxs.len() - 3]);

	###############****/

	



/*pub fn buffer_test() {
	println!("--- test4::buffer_test() ---");
	let mut cortex = cortex::Cortex::new();

	let mut vec1: Vec<u32> = Vec::new();
	for i in range(0, 1024) {
		vec1.push(i as u32);
	}

	let init_val = 0u32;
	let size = 50;

	let mut vec2: Vec<u32> = iter::repeat(init_val).take(size).collect();
	let buf2: ocl::cl_mem = ocl::new_write_buffer(&mut vec2, cortex.ocl.context);

	//cmn::print_vec(&vec2, 10);

	for i in range(0, 5) {
		for x in vec2.iter_mut() {
			*x += 1000;
		}
		//cmn::print_vec(&vec2, 10);

		cortex.ocl.enqueue_write_buffer(&mut vec2, buf2, cortex.ocl.command_queue);
	}

	

	_buffer_test(&cortex, &vec2, buf2, "buffer_test");

	for i in range(0, 5) {
		for x in vec2.iter_mut() {
			*x += 1000;
		}
		//cmn::print_vec(&vec2, 10);

		ocl::enqueue_write_buffer(&mut vec2, buf2, cortex.ocl.command_queue);
	}

	//cmn::print_vec(&vec2, 10);
	_buffer_test(&cortex, &vec2, buf2, "buffer_test");
	

	cortex.release_components();

	println!("--- test4::buffer_test() complete ---");

}

fn _buffer_test<T: Clone + Default + Integer>(
			cortex: &Cortex,
			test_source: &Vec<T>,
			test_source_buff: ocl::cl_mem, 
			test_kernel_name: &str, 

) {
	//print_vec_info(test_source, test_kernel_name);
	println!("Performing buffer test with {} iterations.", READBACK_TEST_ITERATIONS);

	let time_start = time::get_time().sec;
	//println!("Timer Started at: {:?}...", time::now().to_timespec());

	// Create Vec for output
	let mut test_out: Vec<T> = Vec::with_capacity(test_source.len());
	for i in range(0, test_out.capacity()) {
		test_out.push(Default::default());
	}


	// Create output buffer to read into output Vec
	let test_out_buff: ocl::cl_mem = ocl::new_read_buffer(&test_out, cortex.ocl.context);

	// Test Kernels simply clone the vector right now
	let test_out_kernel: ocl::cl_kernel = ocl::new_kernel(cortex.ocl.program, test_kernel_name);
	ocl::set_kernel_arg(0, test_source_buff, test_out_kernel);
	ocl::set_kernel_arg(1, test_out_buff, test_out_kernel);

	// Run Kernel then read from output buffer
	ocl::enqueue_kernel(test_out_kernel, cortex.ocl.command_queue, test_source.len());
	ocl::enqueue_read_buffer(&test_out, test_out_buff, cortex.ocl.command_queue);

	// Calculate sum total for output vector

	//println!("");
	//print_vec_info(&test_out, test_kernel_name);
	
	// Run again 5 times using output as source
	ocl::set_kernel_arg(0, test_out_buff, test_out_kernel);

	

	for i in range(0, READBACK_TEST_ITERATIONS) {
		//println!("... {}", i + 1);
		ocl::enqueue_kernel(test_out_kernel, cortex.ocl.command_queue, test_source.len());
		//ocl::enqueue_read_buffer(&test_out, test_out_buff, cortex.ocl.command_queue);
		//print_vec_info(&test_out, test_kernel_name);

	}

	//println!("Readback loops complete, reading buffer...");

	ocl::enqueue_read_buffer(&test_out, test_out_buff, cortex.ocl.command_queue);

	let time_stop = time::get_time().sec;
	let time_complete = time_stop - time_start;

	//print_vec_info(&test_out, test_kernel_name);

	println!("	{} Iterations complete in: {} sec.", READBACK_TEST_ITERATIONS, time_complete);

	//print_vec_info(test_source, test_kernel_name);
	print!("	");
	print_vec_info(&test_out, test_kernel_name);
	
	ocl::release_kernel(test_out_kernel);
}
 

pub fn print_vec_info<T: Clone + Default + Integer>(my_vec: &Vec<T>, info: &str) {
	let mut total = 0us;
	for x in range(0, my_vec.len()) {
		total += my_vec[x].to_uint().expect("test_4.print_vec_info()");
	}
	println!("[{}: total:{}; len:{}; avg:{}]", info, total, my_vec.len(), total/my_vec.len());

}
*/

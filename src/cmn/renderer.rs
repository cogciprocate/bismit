use cmn::{ self, CorticalDimensions, SliceDimensions, HexTilePlane, Sdr };
use map::{ SliceMap };

use std::char;
use std::iter;
use std::collections::{ BTreeMap };

pub struct Renderer {
	//dims: CorticalDimensions,
	axn_history: Vec<u8>,
	sst_history: Vec<u8>,
	//slices: SliceMap,
	aff_out_dims: SliceDimensions,
}

impl Renderer {
	pub fn new(area_dims: &CorticalDimensions) -> Renderer {		
		let dims = SliceDimensions::new(area_dims, None).unwrap();
		let sdr_len = (dims.columns()) as usize;

		Renderer { 
			aff_out_dims: dims,
			axn_history: iter::repeat(0).take(sdr_len).collect(),
			sst_history: iter::repeat(0).take(sdr_len).collect(),
			//slice_map: slice_map.clone(),
		}
	}

	// DRAW(): v_size-row-v, u_size-col-u
	// TODO: NEED TO MAKE SST_AXNS OPTIONAL 
	pub fn render(&mut self, out_axns: &Sdr, sst_axns_opt: Option<&Sdr>, dims_opt: Option<&SliceDimensions>, 
				input_status: &str, print_summary: bool) 
	{
		let dims = match dims_opt {
			Some(dims) => dims,
			None => &self.aff_out_dims,
		};

		let v_size = dims.v_size();
		let u_size = dims.u_size();
		assert!((v_size * u_size) as usize == out_axns.len());

		let sst_axns = match sst_axns_opt {
			Some(sst_axns) => sst_axns,
			None => out_axns,
		};

		let use_history = sst_axns_opt.is_some();

		let mut margin = String::with_capacity(v_size as usize + 10);
		//let mut margin: String = iter::repeat(' ').take(v_size as usize - 1).collect();

		let mut print_buf = String::with_capacity(256);

		let mut active_axns = 0usize;
		let mut active_ssts = 0usize;
		let mut failed_preds = 0usize;
		let mut corr_preds = 0usize;
		let mut anomalies = 0usize;
		let mut new_preds = 0usize;

		print!("\n");

		for v in 0..v_size {
			//let v = (v_size - 1) - v_mirror;
			print!("{}", margin);
			
			for u in 0..u_size {
				//let u = (u_size - 1) - u_mirror;
				let sdr_idx = ((v * u_size) + u) as usize;
				let sdr_val = out_axns[sdr_idx];
				let sdr_cmpd = (sdr_val >> 4) | (((sdr_val & 0x0F) != 0) as u8);
				//let sdr_cmpd = sdr_val;

				let axn_active = out_axns[sdr_idx] != 0;
				let sst_active = sst_axns[sdr_idx] != 0;
				let prediction = out_axns[sdr_idx] != sst_axns[sdr_idx];
				let new_prediction = prediction && (!sst_active); // RENAME (it's not necessarily a new pred)

				//let prev_active = vec_ff_prev[i] != Default::default();
				let prev_prediction = if use_history {
					cmn::new_pred(self.axn_history[sdr_idx], self.sst_history[sdr_idx])
				} else {
					false
				};

				if sst_active {	active_ssts += 1; }
				if new_prediction { new_preds += 1;	}
				if sst_active && !prev_prediction {	anomalies += 1; }

				if (prev_prediction && !new_prediction) && !sst_active {
					failed_preds += 1;
				} else if prev_prediction && sst_active {
					corr_preds += 1;
				}

				if sdr_cmpd == 0 {
					//print_buf.push('-');
					print_buf.push_str("--");
				} else {
					active_axns += 1;

					if prediction {
						print_buf.push_str(cmn::BGC_DGR);
					}

					if new_prediction {
						print_buf.push_str(cmn::C_MAG);
					} else {
						print_buf.push_str(cmn::C_BLU);
					}
					print_buf.push_str(&format!("{:02X}", sdr_val));
					//print_buf.push(char::from_digit(sdr_cmpd as u32, 16).unwrap()); // PRESUMABLY FASTER THAN format!()
				}

				print_buf.push_str(cmn::BGC_DEFAULT);
				print_buf.push_str(cmn::C_DEFAULT);
				//print_buf.push(' ');
			}

			margin.push(' ');
			//margin.pop();

			print!("{}\n", &print_buf);
			print_buf.clear();
		}

		if use_history {
			self.axn_history.clear();
			self.sst_history.clear();

			self.axn_history.push_all(out_axns);
			self.sst_history.push_all(sst_axns);
		}

		// for hst_i in 0..self.out_axns.len() {
		// 	self.axn_history[hst_i] = out_axns[hst_i];
		// 	self.sst_history[hst_i] = sst_axns[hst_i];
		// }
		let preds_total = (corr_preds + failed_preds) as f32;

		let pred_accy = if preds_total > 0f32 {
			(corr_preds as f32 / preds_total) * 100f32
		} else {
			0f32
		};

		print!("{}{}\n", cmn::C_DEFAULT, cmn::BGC_DEFAULT);

		if print_summary {
			println!("prev preds:{} (correct:{}, incorrect:{}, accuracy:{:.1}%), anomalies:{}, \
				new preds:{}, ssts active:{}, axns active:{}, input status:{}", 
				preds_total, corr_preds, failed_preds, pred_accy, 
				anomalies, new_preds, active_ssts, active_axns, input_status,
			);
		}
	}

	pub fn render_axon_space(&mut self, axn_space: &Sdr, slices: &SliceMap) {
		for slc_id in 0..slices.slc_count() {			
			//let axn_idz = cmn::axn_idz_2d(slc_id, col_count, hrz_demarc) as usize;
			let slc_dims = &slices.dims()[slc_id as usize];
			let axn_idz = slices.idz(slc_id) as usize;
			let axn_idn = axn_idz + slc_dims.columns() as usize;		
			let layer_name = slices.layer_name(slc_id);			

			print!("Axon slice '{}': slc_id: {}, axn_idz: {}", layer_name, slc_id, axn_idz);

			self.render(&axn_space[axn_idz..axn_idn], None, Some(&slc_dims), layer_name, false);
		}
	}
}

/*
pub fn render_sdr(
			vec_out: &Sdr, 
			vec_ff_opt: Option<&Sdr>, 
			vec_out_prev_opt: Option<&Sdr>, 
			vec_ff_prev_opt: Option<&Sdr>,
			slc_map: &BTreeMap<u8, &'static str>,
			print: bool,
			sdr_len: u32,
) -> f32 {
	let vec_ff = match vec_ff_opt {
		Some(v) => v,
		None => vec_out.clone(),
	};

	let vec_out_prev = match vec_out_prev_opt {
		Some(v) => v,
		None => vec_out.clone(),
	};

	let vec_ff_prev = match vec_ff_prev_opt {
		Some(v) => v,
		None => vec_out.clone(),
	};

	//println!("vec_ff.len(): {}, vec_out.len(): {}", vec_ff.len(), vec_out.len());

	assert!(vec_ff.len() == vec_out.len(), "cmn::render_sdr(): vec_ff.len() != vec_out.len(), Input vectors must be of equal length.");
	assert!(vec_out.len() == vec_out_prev.len(), "cmn::render_sdr(): vec_out.len() != vec_out_prev.len(), Input vectors must be of equal length.");
	assert!(vec_out.len() == vec_ff_prev.len(), "cmn::render_sdr(): vec_out.len() != vec_ff_prev.len(), Input vectors must be of equal length.");
	

	let mut active_cols = 0usize;
	let mut failed_preds = 0usize;
	let mut corr_preds = 0usize;
	let mut anomalies = 0usize;
	let mut new_preds = 0usize;
	let mut ttl_active = 0usize;

	let cortical_area_per_line = 64;
	let line_character_u_size = (cortical_area_per_line * (4 + 4 + 2 + 4 + 4 + 1)) + 8;	// 8 extra for funsies

	//println!("\n[{}{}{}]:", C_GRN, vec_ff.len(), C_DEFAULT);

	let mut out_line: String = String::with_capacity(line_character_u_size);
	let mut i_line = 0usize;
	let mut i_global = 0usize;
	let mut i_pattern = 0usize; // DEPRICATE
	let mut i_cort_area = 0u8;

	println!("");
	io::stdout().flush().ok();

	loop {
		if i_line >= vec_out.len() { break }

		out_line.clear();

		for i in i_line..(i_line + cortical_area_per_line) {
			let cur_active = vec_out[i] != Default::default();
			let col_active = vec_ff[i] != Default::default();
			let prediction = vec_out[i] != vec_ff[i];
			let new_prediction = prediction && (!col_active);

			//let prev_active = vec_ff_prev[i] != Default::default();
			let prev_prediction = new_pred(vec_out_prev[i], vec_ff_prev[i]);

			if col_active {
				active_cols += 1;
			}

			if new_prediction { 
				new_preds += 1;
			}

			if (prev_prediction && !new_prediction) && !col_active {
				failed_preds += 1;
			} else if prev_prediction && col_active {
				corr_preds += 1;
			}

			if col_active && !prev_prediction {
				anomalies += 1;
			}

			if print {
				if cur_active {
					if prediction {
						out_line.push_str(BGC_DGR);
					}

					if new_prediction {
						//assert!(new_pred(vec_out[i], vec_ff[i]));
						out_line.push_str(C_MAG);
					} else {
						out_line.push_str(C_BLU);
					}
					/*if corr_pred(vec_out[i], vec_ff[i], vec_out_prev[i], vec_ff_prev[i]) {
						corr_preds += 1;
					}*/
				} else {
					out_line.push_str(C_DEFAULT);
				}

				if cur_active {
					out_line.push_str(&format!("{:02X}", vec_out[i]));
					ttl_active += 1;
				} else {
					if (i & 0x07) == 0 || (i_global & 0x07) == 0 {				// || ((i_global & 0x0F) == 7) || ((i_global & 0x0F) == 8)
						out_line.push_str("  ");
					} else {
						out_line.push_str("--");
					}
				} 

				out_line.push_str(C_DEFAULT);
				out_line.push_str(BGC_DEFAULT);
				out_line.push_str(" ");
			}
		}


		if print {
			if ((i_line % sdr_len as usize) == 0) && (vec_ff.len() > sdr_len as usize) {
				let slc_id = (i_cort_area) as u8;

				let slc_name = match slc_map.get(&slc_id) {
					Some(&name) => name,
					None => "<render_sdr(): slc name not found in map>",
				};

				println!("\n[{}: {}]", slc_id, slc_name);
				i_cort_area += 1;
				i_pattern = 0; // DEPRICATE
			} else {
				i_pattern += 1; // DEPRICATE
			}
			
			println!("{}", out_line);
		}

		i_line += cortical_area_per_line;
		i_global += 1;
	}


	let preds_total = (corr_preds + failed_preds) as f32;

	let pred_accy = if preds_total > 0f32 {
		(corr_preds as f32 / preds_total) * 100f32
	} else {
		0f32
	};

	if print {
		if vec_out_prev_opt.is_some() {
			println!("\nprev preds:{} (correct:{}, incorrect:{}, accuracy:{:.1}%), anomalies:{}, cols active:{}, ttl active:{}, new_preds:{}", 
				preds_total, corr_preds, failed_preds, pred_accy, anomalies, active_cols, ttl_active, new_preds,);
		}
	}

	pred_accy
}
*/

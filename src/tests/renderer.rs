use ocl::{ CorticalDimensions };
use cmn;

use std::char;
use std::iter;

pub struct Renderer {
	dims: CorticalDimensions,
	axn_history: Vec<u8>,
	sst_history: Vec<u8>,
}

impl Renderer {
	pub fn new(dims: CorticalDimensions) -> Renderer {
		let sdr_len = (dims.width() * dims.height()) as usize;

		Renderer { 
			dims: dims,
			axn_history: iter::repeat(0).take(sdr_len).collect(),
			sst_history: iter::repeat(0).take(sdr_len).collect(),
		}
	}

	// DRAW(): height-row-v, width-col-u
	pub fn render(&mut self, axn_sdr: &[u8], sst_sdr: &[u8]) {
		let height = self.dims.height();
		let width = self.dims.width();
		assert!((height * width) as usize == axn_sdr.len());

		let mut margin = String::with_capacity(height as usize + 10);
		//let mut margin: String = iter::repeat(' ').take(height as usize - 1).collect();

		let mut print_buf = String::with_capacity(256);

		let mut active_axns = 0usize;
		let mut active_ssts = 0usize;
		let mut failed_preds = 0usize;
		let mut corr_preds = 0usize;
		let mut anomalies = 0usize;
		let mut new_preds = 0usize;

		print!("\n\n");

		for v in 0..height {
			//let v = (height - 1) - v_mirror;
			print!("{}", margin);
			
			for u in 0..width {
				//let u = (width - 1) - u_mirror;
				let sdr_idx = ((v * width) + u) as usize;
				let sdr_val = axn_sdr[sdr_idx];
				let sdr_cmpd = (sdr_val >> 4) | (((sdr_val & 0x0F) != 0) as u8);
				//let sdr_cmpd = sdr_val;

				let axn_active = axn_sdr[sdr_idx] != 0;
				let sst_active = sst_sdr[sdr_idx] != 0;
				let prediction = axn_sdr[sdr_idx] != sst_sdr[sdr_idx];
				let new_prediction = prediction && (!sst_active); // RENAME (it's not necessarily a new pred)

				//let prev_active = vec_ff_prev[i] != Default::default();
				let prev_prediction = cmn::new_pred(self.axn_history[sdr_idx], self.sst_history[sdr_idx]);

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
				print_buf.push(' ');
			}

			margin.push(' ');
			//margin.pop();

			print!("{}\n", &print_buf);
			print_buf.clear();
		}

		self.axn_history.clear();
		self.sst_history.clear();

		self.axn_history.push_all(axn_sdr);
		self.sst_history.push_all(sst_sdr);

		// for hst_i in 0..self.axn_sdr.len() {
		// 	self.axn_history[hst_i] = axn_sdr[hst_i];
		// 	self.sst_history[hst_i] = sst_sdr[hst_i];
		// }
		let preds_total = (corr_preds + failed_preds) as f32;

		let pred_accy = if preds_total > 0f32 {
			(corr_preds as f32 / preds_total) * 100f32
		} else {
			0f32
		};

		print!("{}{}\n", cmn::C_DEFAULT, cmn::BGC_DEFAULT);
		println!("\nprev preds:{} (correct:{}, incorrect:{}, accuracy:{:.1}%), anomalies:{}, \
			new preds:{}, ssts active:{}, axns active:{}", 
			preds_total, corr_preds, failed_preds, pred_accy, 
			anomalies, new_preds, active_ssts, active_axns,
		);
	}
}

/*
pub fn render_sdr(
			vec_out: &[u8], 
			vec_ff_opt: Option<&[u8]>, 
			vec_out_prev_opt: Option<&[u8]>, 
			vec_ff_prev_opt: Option<&[u8]>,
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
	let line_character_width = (cortical_area_per_line * (4 + 4 + 2 + 4 + 4 + 1)) + 8;	// 8 extra for funsies

	//println!("\n[{}{}{}]:", C_GRN, vec_ff.len(), C_DEFAULT);

	let mut out_line: String = String::with_capacity(line_character_width);
	let mut i_line = 0usize;
	let mut i_global = 0usize;
	let mut i_pattern = 0usize; // DEPRICATE
	let mut i_cort_area = 0u8;

	print!("\n");
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

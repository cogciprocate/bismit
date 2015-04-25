

static inline uint asp_to_col_ofs(uint asp_idx) {
	return (asp_idx - ASPINY_REACH) << ASPINY_SPAN_LOG2;
}

static inline uint asp_col_id_to_col_idx(uint const asp_idx, uint const asp_col_id) {
	return (asp_to_col_ofs(asp_idx) + (asp_col_id & (ASPINY_SPAN - 1)));
}

static inline uint axn_idx_wrap_2d(uchar row_z, int col_x) {
	int const row_width = get_global_size(1);
	int const row_count = get_global_size(0);
	//int const axn_len = mul24(row_width, row_count);	// COMPUTE THIS AHEAD OF TIME

	int axn_idx = mad24((int)row_z, row_width, col_x + SYNAPSE_REACH);
	
	return axn_idx;
	//return axn_idx + mul24((axn_idx < SYNAPSE_REACH), axn_len);
}

static inline void syns_learn( // VECTORIZE
				__global const uchar* const syn_states,
				uint const syn_idx_start,
				uint const syns_per_den_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		char syn_strength = syn_strengths[i];
		uchar syn_state = syn_states[i];

		uchar rnd_char = (rnd ^ i) & 0x7F;		
		char inc = (rnd_char > abs(syn_strength)); 
		//char dec = 0 - inc;
		syn_strengths[i] = (syn_strength - inc) + mul24((syn_state != 0), (inc << 1));

		/*if (syn_states[i]) {
			syn_strengths[i] += inc;
			//syn_strengths[i] = clamp(syn_strength + 1, -100, 100);
		} else {
			syn_strengths[i] -=	inc;
			//syn_strengths[i] = clamp(syn_strength - 1, -100, 100);
		}*/
	}
}




/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/





/*
	GENERAL OPTIMIZATION TODO:
		- Vectorize (pretty much everywhere)
		- Fit data into workgroups better for a few kernels
			- Keep data loading contiguous for the workgroup
		- Use Async copy
			event_t async_work_group_copy(__local T *dst, const __global T *src, size_t num_elements, event_t event)
			event_t async_work_group_copy(__global T *dst, const __local T *src, size_t num_elements, event_t event)
			void wait_group_events (int num_events, event_t *event_list)



	COL_SYNS_CYCLE():
		number of source rows can not exceed: 
			ROWS * (SYNAPSES_PER_CELL_PROXIMAL + SYNAPSE_WORKGROUP_SIZE)

	TODO:
		- Vectorize!
		- Col Inputs/Outputs probably need to be limited to one row.

	WATCH OUT FOR:
		- Bank conflicts once src_col_offs start to change
*/
//	__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void syns_cycle(
	__global const uchar* const axn_states,
	__global const char* const syn_src_col_offs,
	__global const uchar* const syn_src_row_ids,
	//__global const char* const syn_strengths,
	__private uint const syns_per_cell_l2,
	__global uchar* const syn_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const l_id = get_local_id(1); 
	uint const wg_id = get_group_id(1);
	uint const wg_size = get_local_size(1);
	
	uint const base_col_id = mul24(wg_id, wg_size);
	uint const base_cel_idx = mad24(row_id, row_width, base_col_id);
	uint const syn_row_width = row_width << syns_per_cell_l2;
	uint const init_syn_idx = (base_cel_idx << syns_per_cell_l2) + l_id;

	uint const syn_n = init_syn_idx + (wg_size << syns_per_cell_l2);
	uint syn_col_i = init_syn_idx;

	for (uint syn_idx = init_syn_idx; syn_idx < syn_n; syn_idx += wg_size) {
		syn_col_i -= mul24(syn_row_width, (uint)(syn_col_i >= syn_row_width));
		uint col_pos = syn_col_i >> syns_per_cell_l2;
		uint axn_idx = mad24((uint)syn_src_row_ids[syn_idx], row_width, (uint)(col_pos + syn_src_col_offs[syn_idx] + SYNAPSE_REACH));
		uchar axn_state = axn_states[axn_idx];

		

		syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);

		//char syn_strength = syn_strengths[syn_idx];
		//syn_states[syn_idx] = mul24((syn_strength >= 0), ((axn_state != 0) << 7) + (axn_state >> 1));

		syn_col_i += wg_size;
	}
}




/*	FOR LATER:
*
*	NEEDS REWRITE
*	OPTIMIZE FOR WORKGROUP
*	VECTORIZE
*
*/

/*	FOR NOW:
*
*	process synapse weights -> state
*	determine raw value for learning purposes (?) -> state_raw
*	IMPLEMENT LEARNING REATTACHMENT
*
*/
__kernel void den_cycle(
	__global const uchar* const syn_states,
	__global const char* const syn_strengths,
	__private uint const syns_per_den_l2,
	__private uint const den_threshold,
	__global uchar* const den_states_raw,
	__global uchar* const den_states
) {
	uint const row_id = get_global_id(0);
	uint const den_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const den_idx = mad24(row_id, row_width, den_id);
	uint const syn_ofs = den_idx << syns_per_den_l2;

	int syn_sum = 0;
	int syn_sum_raw = 0;

	int const n = (1 << syns_per_den_l2);

	for (int i = 0; i < n; i += 1) {
		char syn_strength = syn_strengths[syn_ofs + i];
		uchar syn_state = syn_states[syn_ofs + i];

		//syn_sum += 128;
		syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum);
		
		//syn_sum_raw += 128;
		syn_sum_raw += syn_state;
	}
	
	syn_sum = mul24((syn_sum > den_threshold), syn_sum);

	den_states_raw[den_idx] = clamp((syn_sum_raw >> 7), 0, 255);
	den_states[den_idx] = clamp((syn_sum >> 7), 0, 255);
}


__kernel void aspiny_cycle_pre(
	__global const uchar* const col_states,
	__global uchar* const asp_states,
	__global uchar* const asp_col_ids
	
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);
	uint const col_ofs = asp_pos << ASPINY_SPAN_LOG2;

	uchar col_states_vec[1 << (ASPINY_REACH_LOG2)];

	uchar winner_val = 0;
	uchar winner_id = 0;
	
	uchar val = 0;
	uchar id = 0;

		#pragma unroll
	for (uint i = 0; i < ASPINY_SPAN; i += 4) {
		vstore4(vload4((col_ofs + i) >> 2, col_states), 0, col_states_vec);

			#pragma unroll
		for (uint j = 0; j < 4; j++) {
			val = col_states_vec[j];
			id = j + i;

			if (val <= winner_val) {
				continue;
			} else {
				winner_val = val;
				winner_id = ((col_ofs + id) & 0xFF);
			}
		}
	}
	
	asp_states[asp_idx] = winner_val;
	asp_col_ids[asp_idx] = winner_id;		// | (winner_val & 0xF8);
}



/* 
TODO:
	- REWRITE (REVERSE) IF/ELSE BRANCHES TO OPTIMIZE BETTER
	- VECTORIZE

FUTURE IMPROVEMENTS:
	- MOVE TO 3D (2D INPUT SPACE)
		- USE 4X4 PRIMARY GRID AND 4X4 (16X16) PERIPH
			- MUST WIN 13/16 OR SOMETHING TO SURVIVE
		- OR USE 4X4 PRIMARY AND 3X3 (12X12) PERIPH
			- MUST WIN 10/12
*/
__kernel void aspiny_cycle_wins(
	__global uchar* const asp_states,
	__global uchar* const asp_wins
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);

	//uint const as_bitmask = (ASPINY_SPAN - 1);

	uchar asp_state = asp_states[asp_idx];
	uchar asp_win = asp_wins[asp_idx];

	int win_count = asp_win; // asp_wins[asp_idx];

	for (uint i = 0; i < ASPINY_SPAN; i++) {
		uint cur_comp_idx = (asp_idx - ASPINY_REACH) + i + (i > (ASPINY_REACH - 1));
		uchar cur_comp_state = asp_states[cur_comp_idx];
		uchar cur_comp_win = asp_wins[cur_comp_idx];

		if (asp_win == cur_comp_win) {
			if (asp_state > cur_comp_state) {
				win_count += 1;
			}

			/*if ((asp_state == cur_comp_state) && (asp_state > 0)) {		// OLD (TIEBREAK) VERSION
				if ((asp_idx & as_bitmask) == (asp_state & as_bitmask)) {
					win_count += 1;
				} else if ((cur_comp_idx & as_bitmask) != (asp_state & as_bitmask)) {
					win_count += ((asp_idx) < (cur_comp_idx));
				}
			} else if (asp_state > cur_comp_state) {
				win_count += 1;
			}*/
		} else if (asp_win > cur_comp_win) {
			win_count += 1;
		} else {
			win_count = 0;
			asp_state = 0;
			break;
		}
		
	}

	asp_wins[asp_idx] = win_count;
	asp_states[asp_idx] = asp_state;
}


__kernel void aspiny_cycle_post(
	__global uchar* const asp_wins,
	//__global uchar* const asp_col_ids,
	__global uchar* const asp_states
	//__global uchar* const col_states
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);
	//uint const col_ofs = asp_pos << ASPINY_SPAN_LOG2;

	//uchar asp_state = asp_states[asp_idx];
	uchar const asp_win = asp_wins[asp_idx];

	asp_states[asp_idx] = asp_win;

	asp_wins[asp_idx] = 0;
}



// VECTORIZE ME
// RENAME ME
// CLEAN ME UP
	//__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void col_post_inhib_unoptd (										
	__global const uchar* const asp_col_ids,
	__global const uchar* const asp_states,
	__global const uchar* const asp_wins,
	__private uint const col_axn_row_offset,
	__global uchar* const col_states,
	__global uchar* const axn_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = col_idx + mad24(col_axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (col_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;

	uchar const asp_state = asp_states[asp_idx];
	uchar const col_state = col_states[col_idx];

	int win = (asp_col_id_to_col_idx(asp_idx, (asp_col_ids[asp_idx])) == col_idx);
	win = (win && asp_state);

	col_states[col_idx] = mul24(col_state, (win > 0));
	axn_states[axn_idx] = mul24(col_state, (win > 0));
}



__kernel void pyr_activate(
				__global const uchar* const col_states,
				__global const uchar* const col_cels_status,
				__private uchar const axn_row_base,
				__global int* const aux_ints_0,
				__global uchar* const pyr_states,	
				__global uchar* const axn_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const pyr_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = mad24(axn_row_base + row_id, row_width, col_id + (uint)SYNAPSE_REACH);

	uchar const col_state = col_states[col_id];
	uchar const cc_status = col_cels_status[col_id];
	uchar pyr_state = pyr_states[pyr_idx];

	int corr_pred = (pyr_state && col_state);
	int anomaly = ((col_state != 0) && (cc_status == 0));

	pyr_state = ((corr_pred != 0) || (anomaly != 0)) && (col_state != 0);
	//pyr_state = (corr_pred | anomaly) && (col_state);
	//pyr_state = mul24(((corr_pred != 0) || (anomaly != 0)), col_state);
	
	//axn_states[axn_idx] = pyr_state;
	pyr_states[pyr_idx] = pyr_state;
	//aux_ints_0[pyr_idx] = 5;
	//aux_ints_0[pyr_idx] = pyr_state;
}



__kernel void col_learn(
	__global const uchar* const asp_col_ids,
	__global const uchar* const asp_states,
	__global const uchar* const syn_states,
	//__global const uchar* const col_states,
	__private uint const syns_per_den_l2,
	__private uint const rnd,
	__global int* const aux_ints_0,
	__global char* const syn_strengths
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);

	uint const col_idx = asp_col_id_to_col_idx(asp_idx, (asp_col_ids[asp_idx]));
	uint const syn_idx = col_idx << syns_per_den_l2;

	uchar asp_state = asp_states[asp_idx];

	if (asp_state) {
		syns_learn(syn_states, syn_idx, syns_per_den_l2, rnd, syn_strengths);
	}

	//aux_ints_0[asp_id] = (rn ^ syn_idx) >> 2;
}




/* NEEDS RESTRUCTURING AND OPTIMIZATION */
__kernel void cels_learn_unoptd(
	__global const uchar* const cel_states,
	__global const uchar* const cel_best_den_ids,
	__global const uchar* const den_states,
	__global const uchar* const syn_states,
	__private uint const syns_per_den_l2,
	__private uint const dens_per_cel_l2,
	__private uint const cels_per_grp,
	__private uint const rnd,
	//__global int* const aux_ints_1,
	__global char* const syn_strengths
) {
	uint const row_id = get_global_id(0);
	uint const col_grp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_grp_id = mad24(row_id, row_width, col_grp_id);
	//uint const den_ofs = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;

	uint const cel_idx_init = mul24(cel_grp_id, cels_per_grp);
	uint const cel_idx_n = cel_idx_init + cels_per_grp;

	//uint debug_output = 0;
 
	for (int c = cel_idx_init; c < cel_idx_n; c++) {
		if (cel_states[c] == 0) {
			continue;
		} else {
			uint den_idx_init = (c << dens_per_cel_l2);
			uint syn_idx = ((den_idx_init + cel_best_den_ids[c]) << syns_per_den_l2);
			syns_learn(syn_states, syn_idx, syns_per_den_l2, rnd, syn_strengths);
		}
	}
}

/* SYNS_REGROW()

	- [done] check for dead synapses (syn_strength < 127)
	- [partial] replace with new random src_col_offs and src_row_id
	- [partial] scan through synapses on that dendrite to check for duplicates
	- [changed] repeat if duplicate found
	- [done] abort if duplicate found

	FUTURE CORRECTIONS:
		- actually assign a new src_row
			- either generate it host side and use the same one for every 
				synapse or, better yet...
			- store a pre-generated array of randomly distributed columns 
				device side (64 - 256 will be enough) and use a peice of the
				random seed to pick one.
		- [partial] actually scan for duplicates

	FUTURE OPTIMIZATIONS:
		- pre-load synapse values into local memory
		- move this to a dendrite controlled kernel (den_regrow()?) and process
			a whole dendrite for each work item (possibly even a whole cell 
			later)

*/
__kernel void syns_regrow(
	__global char* const syn_strengths,
	__private uint const syns_per_den_l2,
	__private uint const rnd,
	__global int* const aux_ints_1,
	__global char* const syn_src_col_offs,
	__global uchar* const syn_src_row_ids
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const syn_idx = mad24(row_id, row_width, col_id);

	char const syn_strength = syn_strengths[syn_idx];

	//uchar rnd_row_id = 0;

	if (syn_strength > -100) {
		return;
	} else {
		char rnd_col_ofs = ((rnd ^ ((syn_idx << 5) ^ (syn_idx >> 3))) & 0xFF);
		//rnd_row_id = ((rnd >> 8) & 0xFF);		// CHOOSE FROM PRE-BUILT ARRAY

			// CHECK FOR DUPLICATES 

		uint base_syn_idx = (syn_idx >> syns_per_den_l2) << syns_per_den_l2;
		uint n = base_syn_idx + (1 << syns_per_den_l2);

		for (uint i = base_syn_idx; i < n; i++) {
			int dup = (rnd_col_ofs == syn_src_col_offs[syn_idx]);		// ADD && ROW CHECK
			//int dup_row = ^^^^^^

			if (!dup) {
				continue;
			} else {
				//	JUST EXIT IF OUR NEW RANDOM ADDR IS A DUPLICATE
				//	AND LET IT BE REASSIGNED NEXT REGROW CYCLE
				aux_ints_1[syn_idx] = syn_strength;
				return;	
			}
		}

		syn_strengths[syn_idx] = 0;	
		syn_src_col_offs[syn_idx] = rnd_col_ofs;
		//syn_src_row_ids[syn_idx] =

			//aux_ints_1[syn_idx] = syn_strength;
	}

	
	/*int dead_syn = (syn_strength <= -100);
	syn_src_col_offs[syn_idx] = mul24(dead_syn, (int)rnd_col_ofs);
	syn_strengths[syn_idx] = mul24(!(dead_syn), (int)syn_strength);*/

	//syn_src_row_ids[syn_idx] =

}



__kernel void pyr_cycle(
				__global const uchar* const den_states,
				__private uchar const axn_row_base,
				__global uchar* const pyr_best_dens,
				__global uchar* const pyr_states,
				__global uchar* const axn_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row_id, row_width, col_id);
	uint const den_ofs = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;
	uint const axn_idx = mad24(axn_row_base + row_id, row_width, col_id + (uint)SYNAPSE_REACH);

	uint den_sum = 0;

	uchar best_den_state = 0;
	uchar best_den_id = 0;
	//int active_dendrites = 0;

	//uint pyr_state = pyr_states[cel_idx];

		//#pragma unroll 
	for (uchar i = 0; i < DENDRITES_PER_CELL_DISTAL; i++) {
		uchar den_state = den_states[den_ofs + i];

		/*if (den_state > best_den_state) {
			best_den_id = i;
			best_den_state = den_state;

		}*/
		int den_state_biggest = (den_state > best_den_state);
		best_den_id = mul24(den_state_biggest, i);
		best_den_state = mul24(den_state_biggest, den_state);

		den_sum += den_state;
		//den_sum += (den_state != 0);
		//den_sum += (den_state > 0);
		//active_dendrites += (den_state > 0);
	}
	
	//den_sum = den_sum >> 4;

	pyr_best_dens[cel_idx] = best_den_id;
	pyr_states[cel_idx] = clamp(den_sum, 0u, 255u); 	// v.N1
	axn_states[axn_idx] = clamp(den_sum, 0u, 255u);

	//pyr_states[cel_idx] = clamp(den_sum, 0, 127);

	//pyr_states[cel_idx] = (den_sum >> 1);
	//pyr_states[cel_idx] = active_dendrites;
}




__kernel void col_output(
				__global const uchar* const col_states,	// CONVERT TO READING FROM AXON
				__global const uchar* const pyr_states,	// CONVERT TO READING FROM AXON
				//__private uchar const col_row_count,
				__private uchar const pyr_depth,
				__private uchar const pyr_axn_base_row,
				__private uchar const axn_row_output,
				//__private uchar const pyr_base_row,
				__global uchar* const col_cels_status,
				__global uchar* const axn_states
				
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	//uint const axn_idx_output = mad24(axn_row_output, row_width, col_id + (uint)SYNAPSE_REACH);
	uint const axn_idx_output = axn_idx_wrap_2d(axn_row_output, col_id);
	uint const col_idx = mad24(row_id, row_width, col_id);

	int col_state = col_states[col_idx];
	int output_total = 0;

	for (uint i = 0; i < pyr_depth; i++) {
		uint pyr_idx = mad24(i, row_width, col_id);			// v.N3
		//uint axn_idx_pyr = mad24(pyr_axn_base_row + i, row_width, col_id + (uint)SYNAPSE_REACH); 	// v.N1
		
		output_total = max(output_total, (int)pyr_states[pyr_idx]);		// v.N3
		//output_total += axn_states[axn_idx_pyr];							// v.N2
		//output_total = max(output_total, (int)axn_states[axn_idx_pyr]); 	// v.N1
		
		//output_total += (axn_states[axn_idx_pyr] > 0);
	}

	output_total = clamp(output_total, 0, 255);

	col_cels_status[col_idx] = output_total;

	axn_states[axn_idx_output] = clamp(output_total + col_state, 0, 255); 			// v.N2
	//axn_states[axn_idx_output] = clamp(max(output_total, col_state), 0, 255); 	// v.N1
	//axn_states[axn_idx_output] = clamp((output_total), 0, 255);
	//axn_states[axn_idx] = test;
}








/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/







/*
	OPTIMIZE FOR WORKGROUP
	VECTORIZE
*/
__kernel void den_dist_cycle_unused(
	__global const uchar* const syn_states,
	__private uint const syns_per_den_l2,
	__global uchar* const den_states
) {
	uint const row_id = get_global_id(0);
	uint const den_id = get_global_id(1);
	//uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const den_idx = mad24(row_id, row_width, den_id);
	//uint const syn4_per_den_l2 = syns_per_den_l2 - 2;
	//uint const syn_ofs = den_idx << syn4_per_den_l2;
	uint const syn_ofs = den_idx << syns_per_den_l2;

	int syn_sum = 0;
	uint const n = (1 << syns_per_den_l2);

	for (uint i = 0; i < n; i += 1) {
		uchar syn_state = syn_states[syn_ofs + i];
		syn_sum += syn_state;
	}

	syn_sum = mul24((syn_sum > DENDRITE_INITIAL_THRESHOLD_PROXIMAL), syn_sum);

	den_states[den_idx] = clamp((syn_sum >> 7), 0, 255);
	//den_states[den_idx] = mad24((den_total > 0), 128, clamp(den_total >> (syns_per_den_l2 + 1), 0, 127));
	//den_states[den_idx] = den_total; //(0, 1, 2, 3); 
	//den_states[den_idx] = (syn_sum >> syns_per_den_l2);

}

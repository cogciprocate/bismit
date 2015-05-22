// #define LTD_BIAS_LOG2	0
#define FLAG_SET(flag_set, mask)			((flag_set) |= (mask))
#define FLAG_CLEAR(flag_set, mask)			((flag_set) &= ~(mask))
#define FLAG_TEST(flag_set, mask)			(((flag_set) & (mask)) == (mask))


/*  bismit.cl: conventions

	idx: index, physical in-memory address
	idz: index[0], first element, starting element
	idn: index[len], element after final element, termination point

	id: identifier, but not a physical index

	fuz: fuzzyness, level of predictiveness

*/


static inline uint asp_to_spi_ofs(uint asp_idx) {
	return (asp_idx - ASPINY_REACH) << ASPINY_SPAN_LOG2;
}

static inline uint asp_spi_id_to_spi_idx(uint const asp_idx, uint const asp_spi_id) {
	return (asp_to_spi_ofs(asp_idx) + (asp_spi_id & (ASPINY_SPAN - 1)));
}


/* AXN_IDX_2D(): Axon Address Resolution
	- We must calculate the address for both the horizontal row and spatial (vertical) row case.
		- When calculating vertical rows: 
			- Simply multiply row_id * row width and add offset, cell spiumn id, and global padding (SYNAPSE_REACH).
		- When calculating horizontal rows:
			- Horizontal rows are always physically after spatial rows within axn_states so we must add that space first (HARF * row_width). That gets us to the beginning of horizontal row space after padding is added (padding = SYNAPSE_REACH).
			- We then multiply SYNAPSE_SPAN (which is SYNAPSE_REACH * 2) by the horizontal row_id to get to the correct horizontal row.
			- We must add padding + an extra SYNAPSE_REACH to get us to the middle of the row.
			- We then apply the offset (spi_ofs) to get to the exact axon_idx.
			- col_id is irrelevant and unused for horiz. rows.
		
	- As always, for performance reasons we calculate both cases and multiply by a bool rather than branch


	- [in progress] Accommodate horizontal axon rows, rows which are nonspatial and look the same from any spiumn in a region.
		- Rows above HORIZONTAL_AXON_ROW_DEMARCATION are considered horizontal and will be mapped to the rear of axn_states.
	- [incomplete] Specific unit tests
	- [incomplete] #define row_width 
*/
static inline uint axn_idx_2d(uchar row_id, uint row_width, uint col_id, char spi_ofs) {

	uint axn_idx_spt = mad24((uint)row_id, row_width, (uint)(col_id + spi_ofs + SYNAPSE_REACH));

	int hrow_id = row_id - HORIZONTAL_AXON_ROW_DEMARCATION;
	int hcol_id = mad24(hrow_id, SYNAPSE_SPAN, spi_ofs + SYNAPSE_REACH);
	uint axn_idx_hrz = mad24((uint)HORIZONTAL_AXON_ROW_DEMARCATION, row_width, (uint)(hcol_id + SYNAPSE_REACH));

	
	return mul24((uint)(hrow_id < 0), axn_idx_spt) + mul24((uint)(hrow_id >= 0), axn_idx_hrz);
}

/*static inline uint axn_idx_wrap_2d(uchar row_z, int spi_x) {
	int const row_width = get_global_size(1);
	int const row_count = get_global_size(0);
	//int const axn_len = mul24(row_width, row_count);	// COMPUTE THIS AHEAD OF TIME

	int axn_idx = mad24((int)row_z, row_width, spi_x + SYNAPSE_REACH);
	
	return axn_idx;
	//return axn_idx + mul24((axn_idx < SYNAPSE_REACH), axn_len);
}*/

static inline int rnd_inc(uint const rnd_a,	uint const rnd_b, char const syn_strength) {
		return ((rnd_a ^ rnd_b) & 0x7F) > abs(syn_strength);
}



// VECTORIZE
static inline void dst_syns__active__stp_ltd( 					// ANOMALY & CRYSTALLIZATION
				__global uchar const* const syn_states,
				uint const syn_idx_start,
				uint const syns_per_den_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global char* const syn_flag_sets,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		char syn_strength = syn_strengths[i];
		uchar syn_flag_set = syn_flag_sets[i] & ~SYN_STP_FLAG;
		int const inc = rnd_inc(rnd, i, syn_strength);
		//uchar const rnd_char = (rnd ^ i) & 0x7F;		
		//int const inc = (rnd_char > abs(syn_strength));
		int const syn_active = syn_state != 0;

		syn_flag_set |= mul24(syn_active, SYN_STP_FLAG);

		syn_flag_sets[i] = syn_flag_set;
		syn_strengths[i] = mul24(!syn_active, (syn_strength - inc));
	}
}


// VECTORIZE --- RE-STREAMLINE (REMOVE BRANCH)
static inline void cel_syns_trm( 			// TERMINATION
				__global uchar const* const syn_states,
				uint const syn_idx_start,
				uint const syns_per_cel_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global char* const syn_flag_sets,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_cel_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		char syn_strength = syn_strengths[i];
		uchar syn_flag_set = syn_flag_sets[i];
		int const inc = rnd_inc(rnd, i, syn_strength);
		//uchar const rnd_char = (rnd ^ i) & 0x7F;		
		//int const inc = (rnd_char > abs(syn_strength));
		int const syn_active = syn_state != 0;
		int const syn_prev_stp = (syn_flag_set & SYN_STP_FLAG) == SYN_STP_FLAG;

		if (syn_prev_stp) {
			if (syn_active) {
				syn_strength -= inc;			
			} else {
				syn_strength += inc;
			}
		}

		syn_strength -= mul24(syn_flag_set, inc);

		syn_flag_sets[i] = syn_flag_set & ~SYN_STP_FLAG;
		syn_strengths[i] = syn_strength;
	}
}


/*// VECTORIZE
static inline void dst_syns__active__set_prev_active( 
				__global uchar const* const syn_states,
				uint const syn_idx_start,
				uint const syns_per_den_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global char* const syn_flag_sets,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		//char syn_strength = syn_strengths[i];
		uchar syn_flag_set = syn_flag_sets[i];
		//uchar const rnd_char = (rnd ^ i) & 0x7F;		
		//int const inc = (rnd_char > abs(syn_strength));

		//syn_pba = ((syn_flag_set & SYN_CONCRETE_FLAG) == SYN_CONCRETE_FLAG)

		//syn_flag_set &= ~SYN_STP_FLAG;
		syn_flag_set &= ~SYN_CONCRETE_FLAG;

		syn_flag_set |= mul24((syn_state != 0), SYN_CONCRETE_FLAG);

		syn_flag_sets[i] = syn_flag_set;
		//syn_strengths[i] = mul24((syn_state == 0), (syn_strength - inc));
	}
}*/

/*// VECTORIZE
static inline void dst_syns__prev_active__ltd( 
				__global uchar const* const syn_states,
				uint const syn_idx_start,
				uint const syns_per_den_l2,
				uint const rnd,
				__global char* const syn_flag_sets,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		char syn_strength = syn_strengths[i];
		uchar syn_flag_set = syn_flag_sets[i];
		int const inc = rnd_inc(rnd, i, syn_strength);
		//uchar const rnd_char = (rnd ^ i) & 0x7F;		
		//int const inc = (rnd_char > abs(syn_strength));

		int const prev_active = (syn_flag_set & SYN_CONCRETE_FLAG) == SYN_CONCRETE_FLAG;

		if (prev_active) {
			syn_strength -= inc;
		} 

		syn_strengths[i] = syn_strength;
	}
}*/




// VECTORIZE --- RE-STREAMLINE (REMOVE BRANCH)
static inline void prx_syns__active__ltp_ltd( 
				__global uchar const* const syn_states,
				uint const syn_idx_start,
				uint const syns_per_den_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		char syn_strength = syn_strengths[i];
		uchar syn_state = syn_states[i];
		//int is_neg = (syn_strength < 0);	// NEGATIVE STRENGTH SYNAPSES GET A BONUS (LOOKS LIKE THIS MAY BE NO BUENO)

		uchar rnd_char = (rnd ^ i) & 0x7F;		
		int inc = (rnd_char > abs(syn_strength));

		if (syn_state == 0) {
			syn_strength -= inc;
		} else {
			//syn_strength += (inc + is_neg);	// NEGATIVE STRENGTH SYNAPSES GET A BONUS (COMMENT OUT BELOW IF USING)
			syn_strength += inc;
		}

		syn_strengths[i] = syn_strength;
		
		//syn_strengths[i] = (syn_strength - inc) + mul24((syn_state != 0), (inc << 1));
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
		- Fit data into workgroups better for several kernels
			- Keep data loading contiguous for the workgroup
		- Use Async copy
			event_t async_work_group_copy(__local T *dst, const __global T *src, size_t num_elements, event_t event)
			event_t async_work_group_copy(__global T *dst, const __local T *src, size_t num_elements, event_t event)
			void wait_group_events (int num_events, event_t *event_list)
		- Globalize wherever possible:
			- row_width
			- 

	CLEAN UP:
		- One day soon this beast of a .cl file will be split up.

*/



/* 
COL_SYNS_CYCLE():
		number of source rows can not exceed: 
			ROWS * (SYNAPSES_PER_CELL_PROXIMAL + SYNAPSE_WORKGROUP_SIZE)

	TODO:
		- Vectorize!
		- Col Inputs/Outputs probably need to be limited to one row.
			- This isn't feasable. Need to intelligently prefetch:
				- syns_cycle() will need knowledge of which axon ranges it's expected to read from

	WATCH OUT FOR:
		- Bank conflicts once src_spi_x_offs start to change
*/
//	__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void syns_cycle(
	__global uchar const* const axn_states,
	__global char const* const syn_src_spi_x_offs,
	__global uchar const* const syn_src_row_ids,
	//__global char const* const syn_strengths,
	__private uint const syns_per_cell_l2,
	__global int* const aux_ints_0,
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

	uint const base_syn_idx = (base_cel_idx << syns_per_cell_l2);
	uint const init_syn_idx = base_syn_idx + l_id;

	uint const syn_row_width = row_width << syns_per_cell_l2;
	uint const syns_per_wg = wg_size << syns_per_cell_l2;

	uint const syn_n = base_syn_idx + syns_per_wg;

	int syn_spi_i = (base_col_id << syns_per_cell_l2) + l_id;
	uint syn_idx = init_syn_idx;

	uint aux_idx = mad24(row_id, row_width, col_id); // DEBUG

	for (; syn_idx < syn_n; syn_idx += wg_size) {
		syn_spi_i -= mul24((int)syn_row_width, (syn_spi_i >= syn_row_width));
		int spi_pos = syn_spi_i >> syns_per_cell_l2;
		uint axn_idx = axn_idx_2d(syn_src_row_ids[syn_idx], row_width, spi_pos, syn_src_spi_x_offs[syn_idx]);
		//uint axn_idx = mad24((uint)syn_src_row_ids[syn_idx], row_width, (uint)(spi_pos + syn_src_spi_x_offs[syn_idx] + SYNAPSE_REACH));
		uchar axn_state = axn_states[axn_idx];
		
		//syn_states[syn_idx] = axn_state;
		syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);
		
		//char syn_strength = syn_strengths[syn_idx];
		//syn_states[syn_idx] = mul24((syn_strength >= 0), ((axn_state != 0) << 7) + (axn_state >> 1));

		//aux_ints_0[syn_idx] = spi_pos;

		syn_spi_i += wg_size;
	}

	//aux_ints_0[0] = HORIZONTAL_AXON_ROW_DEMARCATION;

	//uint aux_idx = mad24(row_id, row_width, col_id);
	//aux_ints_0[aux_idx] = l_id;
	//aux_ints_0[aux_idx] = syn_idx;
	//aux_ints_0[base_cel_idx] = 12321;
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
*	determine raw value for ltping purposes (?) -> state_raw
*	IMPLEMENT LEARNING REATTACHMENT
*
*/
__kernel void den_cycle(
	__global uchar const* const syn_states,
	__global char const* const syn_strengths,
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

		//uchar syn_state = mul24((syn_states[syn_ofs + i] > 0), 1); // ***** *
		uchar syn_state = syn_states[syn_ofs + i]; // ***** *

		//syn_sum += syn_state; // ***** **
		syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum); // ***** **
		
		syn_sum_raw += syn_state;
	}
	
	syn_sum = mul24((syn_sum > den_threshold), syn_sum);


	//den_states_raw[den_idx] = clamp(syn_sum_raw, 0, 255); // ***** ***
	//den_states[den_idx] = clamp(syn_sum, 0, 255); // ***** ****
	den_states_raw[den_idx] = clamp((syn_sum_raw >> 7), 0, 255); // ***** ***
	den_states[den_idx] = clamp((syn_sum >> 7), 0, 255); // ***** ****
}


__kernel void peak_spi_cycle_pre(
	__global uchar const* const spi_states,
	__global uchar* const asp_states,
	__global uchar* const asp_spi_ids
	
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);
	uint const spi_ofs = asp_pos << ASPINY_SPAN_LOG2;

	uchar spi_states_vec[1 << (ASPINY_REACH_LOG2)];

	uchar winner_val = 0;
	uchar winner_id = 0;
	
	uchar val = 0;
	uchar id = 0;

		#pragma unroll
	for (uint i = 0; i < ASPINY_SPAN; i += 4) {
		vstore4(vload4((spi_ofs + i) >> 2, spi_states), 0, spi_states_vec);

			#pragma unroll
		for (uint j = 0; j < 4; j++) {
			val = spi_states_vec[j];
			id = j + i;

			if (val <= winner_val) {
				continue;
			} else {
				winner_val = val;
				winner_id = ((spi_ofs + id) & 0xFF);
			}
		}
	}
	
	asp_states[asp_idx] = winner_val;
	asp_spi_ids[asp_idx] = winner_id;		// | (winner_val & 0xF8);
}



/* 
TODO:
	- REWRITE (REVERSE) IF/ELSE BRANCHES TO OPTIMIZE BETTER
	- VECTORIZE

FUTURE IMPROVEMENTS:
	- MOVE TO 3D (2D INPUT SPACE)
		- USE 4X4 PRIMARY GRID AND 4X4 (16X16) PERIPH
			- MUST WIN 13/16 (?) TO SURVIVE
		- OR USE 4X4 PRIMARY AND 3X3 (12X12) PERIPH
			- MUST WIN 10/12
*/
__kernel void peak_spi_cycle_wins(
	__global uchar* const asp_states,
	__global uchar* const asp_wins
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);

	uint const as_bitmask = (ASPINY_SPAN - 1);

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

			if ((asp_state == cur_comp_state) && (asp_state > 0)) {		// OLD (TIEBREAK) VERSION
				if ((asp_idx & as_bitmask) == (asp_state & as_bitmask)) {
					win_count += 1;
				} else if ((cur_comp_idx & as_bitmask) != (asp_state & as_bitmask)) {
					win_count += ((asp_idx) < (cur_comp_idx));
				}
			} else if (asp_state > cur_comp_state) {
				win_count += 1;
			}
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


__kernel void peak_spi_cycle_post(
	__global uchar* const asp_wins,
	//__global uchar* const asp_spi_ids,
	__global uchar* const asp_states
	//__global uchar* const spi_states
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);
	//uint const spi_ofs = asp_pos << ASPINY_SPAN_LOG2;

	//uchar asp_state = asp_states[asp_idx];
	uchar const asp_win = asp_wins[asp_idx];

	asp_states[asp_idx] = asp_win;

	asp_wins[asp_idx] = 0;
}



// VECTORIZE ME
// RENAME ME
// CLEAN ME UP
	//__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void spi_post_inhib_unoptd (										
	__global uchar const* const asp_spi_ids,
	__global uchar const* const asp_states,
	__global uchar const* const asp_wins,
	__private uint const spi_axn_row,
	__global uchar* const spi_states,
	__global uchar* const axn_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const spi_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = axn_idx_2d(spi_axn_row, row_width, col_id, 0);
	//uint const axn_idx = mad24(spi_axn_row, row_width, spi_idx + (uint)SYNAPSE_REACH);
	uint const asp_idx = (spi_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;

	uchar const asp_state = asp_states[asp_idx];
	uchar const spi_state = spi_states[spi_idx];

	int win = (asp_spi_id_to_spi_idx(asp_idx, (asp_spi_ids[asp_idx])) == spi_idx);
	win = (win && asp_state);

	//spi_states[spi_idx] = mul24(spi_state, (win > 0));
	//axn_states[axn_idx] = mul24(spi_state, (win > 0));

	spi_states[spi_idx] = mul24(spi_state, (win > 0));
	axn_states[axn_idx] = mul24(spi_state, (win > 0));
}



__kernel void pyr_activate(
				__global uchar const* const mcol_states, // COL
				__global uchar const* const mcol_pyr_fuz_flags, // COL
				__global uchar const* const mcol_best_col_den_states,
				__global uchar const* const pyr_best1_den_dst_ids,

				// ADD PYR BEST DEN STATE NOW THAT WE'VE ADDED IT (and to another kernel somewhere also)

				__global uchar const* const den_dst_states,
				__private uchar const pyr_axn_row_base,
				//__global int* const aux_ints_0,
				//__global uchar* const pyr_energies,
				__global uchar* const pyr_flag_sets,
				__global uchar* const pyr_fuzs,
				__global uchar* const axn_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const pyr_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = axn_idx_2d(pyr_axn_row_base + row_id, row_width, col_id, 0);

	uint const den_ofs = pyr_idx << DENDRITES_PER_CELL_DISTAL_LOG2;			// REPLACE
	uint const best1_den_idx = den_ofs + pyr_best1_den_dst_ids[pyr_idx];		// REPLACE

	uchar const best1_den_state = den_dst_states[best1_den_idx];				// CHANGE

	//uint const axn_idx = mad24(pyr_axn_row_base + row_id, row_width, col_id + (uint)SYNAPSE_REACH);
	uchar const mcol_best_col_den_state = mcol_best_col_den_states[col_id];
	uchar const mcol_state = mcol_states[col_id];
	uchar const mcol_pyr_fuz_flag = mcol_pyr_fuz_flags[col_id];
	uchar const pyr_fuz = pyr_fuzs[pyr_idx];
	uchar pyr_flag_set = pyr_flag_sets[pyr_idx];

	pyr_flag_set &= ~PYR_BEST_IN_COL_FLAG;
	pyr_flag_set |= mul24((mcol_best_col_den_state == best1_den_state), PYR_BEST_IN_COL_FLAG);

	int const mcol_active = mcol_state != 0;
	int const mcol_any_pred = mcol_pyr_fuz_flag != 0;
	int const pyr_fuzictive = (pyr_fuz != 0);

	int const crystal = pyr_fuzictive && mcol_active;
	int const anomaly = mcol_active && !mcol_any_pred;

	int const activate_axon = crystal || anomaly;
	//pyr_fuz = (crystal | anomaly) && (mcol_state);
	//pyr_fuz = mul24(((crystal != 0) || (anomaly != 0)), mcol_state);
 
	pyr_flag_sets[pyr_idx] = pyr_flag_set;
	axn_states[axn_idx] = (uchar)mad24(anomaly, (int)mcol_state, mul24(crystal, (int)pyr_fuz));

	//pyr_fuzs[pyr_idx] = pyr_fuz;
	//aux_ints_0[pyr_idx] = 5;
	//aux_ints_0[pyr_idx] = pyr_fuz;
}



__kernel void spi_ltp(
	__global uchar const* const asp_spi_ids,
	__global uchar const* const asp_states,
	__global uchar const* const syn_states,
	//__global uchar const* const spi_states,
	__private uint const syns_per_den_l2,
	__private uint const rnd,
	//__global int* const aux_ints_0,
	__global char* const syn_strengths
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);

	uint const spi_idx = asp_spi_id_to_spi_idx(asp_idx, (asp_spi_ids[asp_idx]));
	uint const syn_idx = spi_idx << syns_per_den_l2;

	uchar asp_state = asp_states[asp_idx];

	if (asp_state) {
		prx_syns__active__ltp_ltd(syn_states, syn_idx, syns_per_den_l2, rnd, syn_strengths);
	}

	//aux_ints_0[asp_id] = (rn ^ syn_idx) >> 2;
}



/* PYRS_LTP(): Pyramidal long term potentiation and depression - adjusting synapse strengths

	- For each pyramidal cell:
		- if cell axon is currently active:
			- cause learning to take place on it's most active dendrite
		- if cell axon is currently inactive:
			- check to see if the cell's axon was previously active (by checking flag_set)
				- if so, depress (reduce strengths of) any currently active synapses
					- NOTE: The reasoning here is that any synapses which are active just after (but not before) the cell was active are likely to be unrelated to it's prior activity. In other words, a rough implementation of LTD (simplified and optimized and theorized and ... oh who knows). 

	- TODO:
		- Vectorize (should be highly vectorizable)
		- reducing branching will be tough with this one
		- Tests (check that flag_set and prev_best1_den_id are robustly maintained)

		- Let's shit on these goddamn menace fucking constantly active goddamn fucking inputs
			- The root of the problem is the propensity to build on whatever other activity is happening from your neighbors. This activity breeds even more activity and it positively feeds back
			- If we take a dump on synaptic inputs which are active when we are inactive... it should shave some of the bullshit off
			- Constrain this to act in very few circumstances

		##########      CAN'T EQUATE AXON OUTPUT WITH PYR DEPOLS       #############


		- if pyr_prev_concrete 
			- if pyr_concrete
			- if pyr_fuz

		- if pyr_prev_pred
			- if pyr_concrete
			- if pyr_fuz

	- Misc Notes:

		- SYN(    -> STP) WHEN: (SYN_STATE > 0) AND (PYR_TANGIBLE) AND (PYR_BEST_IN_COLUMN)
		                    OR: (SYN_STATE > 0) AND (PYR_TANGIBLE) AND (PYR_PREV_PRED)

		- MAINTAIN STP STATE AS LONG AS: (SYN_STATE > 0) AND (PYR_ACTIVE)

		- SYN(STP -> LTP) ONLY WHEN: ((PYR_ACTIVE -> 0)) SAME TIME AS (SYN_STATE -> 0)




			
*/
__kernel void pyrs_ltp_unoptd(
	__global uchar const* const axn_states,
	__global uchar const* const pyr_fuzs,
	__global uchar const* const pyr_best1_den_ids,
	__global uchar const* const pyr_best2_den_ids,
	__global uchar const* const den_states,
	__global uchar const* const syn_states,
	__private uint const pyr_axn_idx_base, 
	__private uint const syns_per_den_l2,
	__private uint const dens_per_cel_l2,
	__private uint const pyrs_per_wi,
	__private uint const rnd,
	//__global int* const aux_ints_1,
	__global uchar* const syn_flag_sets,
	__global uchar* const pyr_flag_sets,
	//__global uchar* const pyr_prev_best1_den_ids,
	__global char* const syn_strengths
) {
	uint const row_id = get_global_id(0);
	uint const pg_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const pyr_grp_id = mad24(row_id, row_width, pg_id);
	//uint const den_ofs = pyr_idx << DENDRITES_PER_CELL_DISTAL_LOG2;

	//uint const axn_idx_base = mad24(pyr_axn_row_base + row_id, row_width, col_id + (uint)SYNAPSE_REACH);

	uint const pyr_idz = mul24(pyr_grp_id, pyrs_per_wi);
	uint const pyr_idx_n = pyr_idz + pyrs_per_wi;

	//uint const pyr_idz = mul24(pyr_grp_id, pyrs_per_wi);
	//uint const pyr_idx_n = pyr_idz + pyrs_per_wi;

	//uint debug_output = 0;
 
	for (uint i = pyr_idz; i < pyr_idx_n; i++) {
		uchar pyr_best1_den_id = pyr_best1_den_ids[i];
		uchar pyr_best2_den_id = pyr_best2_den_ids[i];
		//uchar pyr_prev_best1_den_id = pyr_prev_best1_den_ids[i];
		uchar pyr_flag_set = pyr_flag_sets[i];

		int pyr_concrete = axn_states[i + pyr_axn_idx_base] != 0;
		int pyr_fuzzy = pyr_fuzs[i] != 0;

		int pyr_prev_concrete = (pyr_flag_set & PYR_PREV_CONCRETE_FLAG) == PYR_PREV_CONCRETE_FLAG;
		//int pyr_prev_stp = (pyr_flag_set & PYR_PREV_STP_FLAG) == PYR_PREV_STP_FLAG;
		int pyr_prev_fuzzy = (pyr_flag_set & PYR_PREV_FUZZY_FLAG) == PYR_PREV_FUZZY_FLAG;
		int pyr_best_in_col = (pyr_flag_set & PYR_BEST_IN_COL_FLAG) == PYR_BEST_IN_COL_FLAG;

		uint den_idx_base = i << dens_per_cel_l2;

		uint pyr_syn_idz = ((den_idx_base) << syns_per_den_l2);	 // WHOLE CELL
		uint best1_den_syn_idz = (den_idx_base + pyr_best1_den_id) << syns_per_den_l2;
		uint best2_den_syn_idz = (den_idx_base + pyr_best2_den_id) << syns_per_den_l2;
		//uint prev_best1_den_syn_idz = (den_idx_base + pyr_prev_best1_den_id) << syns_per_den_l2;


		int pyr_ano = !pyr_prev_fuzzy && pyr_concrete;
		int pyr_cry = pyr_prev_fuzzy && pyr_concrete;
		int pyr_trm = pyr_prev_concrete && !pyr_concrete;


		if (pyr_concrete) {
			if (pyr_prev_fuzzy) { // PREVIOUS (CORRECT) PREDICTION (EVERY PYR IN COL): REINFORCE DEN + TRAIN NEW DEN
				// SAME AS ANO + TRAIN A SECOND REDUNDANT DENDRITE AS WELL
				dst_syns__active__stp_ltd(syn_states, best1_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
				dst_syns__active__stp_ltd(syn_states, best2_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);

			} else if (pyr_best_in_col) { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
				dst_syns__active__stp_ltd(syn_states, best1_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
				
			} else { // EVERYTHING ELSE: JUST SET PREV ACTIVE
				// NOT GOING TO WORRY ABOUT THIS -- ALLOW STP TO REFLECT PRIOR ACTIVITY
				// dst_syns__active__set_prev_active(syn_states, best1_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
			}

			//pyr_flag_set |= PYR_PREV_STP_FLAG;
			pyr_flag_set |= PYR_PREV_CONCRETE_FLAG;

		} else if (pyr_prev_concrete) { // TRM	
			cel_syns_trm(syn_states, pyr_syn_idz, syns_per_den_l2 + dens_per_cel_l2, rnd, syn_flag_sets, syn_strengths);
			//pyr_flag_set &= ~PYR_PREV_STP_FLAG;
			pyr_flag_set &= ~PYR_PREV_CONCRETE_FLAG;
		}


			/*uint syn_idx = ((den_idx_base) << syns_per_den_l2);	 // WHOLE CELL
			dst_syns_ltd_active(syn_states, syn_idx, (syns_per_den_l2 + dens_per_cel_l2), rnd, syn_flag_sets, syn_strengths);*/

			//pyr_flag_set &= ~PYR_PREV_FUZZY_FLAG;

		//pyr_prev_best1_den_id = pyr_best1_den_id;

		pyr_flag_set &= ~PYR_PREV_FUZZY_FLAG;
		pyr_flag_set |= mul24(pyr_fuzzy, PYR_PREV_FUZZY_FLAG);

		/*pyr_flag_set &= ~PYR_PREV_FUZZY_FLAG;
		pyr_flag_set |= mul24(pyr_fuzzy, PYR_PREV_FUZZY_FLAG);*/

		//pyr_prev_best1_den_ids[i] = pyr_prev_best1_den_id;
		pyr_flag_sets[i] = pyr_flag_set;


			// PROBABLY NOT GOOD (TEST LATER)
			/*if (pyr_prev_fuzzy && !pyr_fuzzy) {
				
				// PROBABLY NOT GOOD (TEST LATER)
				if ((best1_den_syn_idz != prev_best1_den_syn_idz) && !pyr_concrete) {
					dst_syns__prev_active__ltd(syn_states, best1_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
				}
				//uint syn_idx = ((den_idx_base) << syns_per_den_l2);	 // WHOLE CELL
				//dst_syns_ltd_active(syn_states, syn_idx, (syns_per_den_l2 + dens_per_cel_l2), rnd, syn_flag_sets, syn_strengths);
			}*/

	}
}


/* SYNS_REGROW()

	- [done] check for dead synapses (syn_strength < 127)
	- [partial] replace with new random src_spi_x_offs and src_row_id
	- [partial] scan through synapses on that dendrite to check for duplicates
	- [changed] repeat if duplicate found
	- [done] abort if duplicate found

	FUTURE CORRECTIONS:
		- actually assign a new src_row
			- either generate it host side and use the same one for every 
				synapse or, better yet...
			- store a pre-generated array of randomly distributed spiumns 
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
	//__global int* const aux_ints_1,
	__global char* const syn_src_spi_x_offs,
	__global uchar* const syn_src_row_ids
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const syn_idx = mad24(row_id, row_width, col_id);

	char const syn_strength = syn_strengths[syn_idx];

	//uchar rnd_row_id = 0;

	if (syn_strength > SYNAPSE_STRENGTH_FLOOR) {
		return;
	} else {
		char rnd_spi_ofs = clamp(-127, 127, (int)((rnd ^ ((syn_idx << 5) ^ (syn_idx >> 3))) & 0xFF));
		//rnd_row_id = ((rnd >> 8) & 0xFF);		// CHOOSE FROM PRE-BUILT ARRAY

			// CHECK FOR DUPLICATES 

		uint base_syn_idx = (syn_idx >> syns_per_den_l2) << syns_per_den_l2;
		uint n = base_syn_idx + (1 << syns_per_den_l2);

		for (uint i = base_syn_idx; i < n; i++) {
			int dup = (rnd_spi_ofs == syn_src_spi_x_offs[syn_idx]);		// ADD && ROW CHECK
			//int dup_row = ^^^^^^

			if (!dup) {
				continue;
			} else {
				//	JUST EXIT IF OUR NEW RANDOM ADDR IS A DUPLICATE
				//	AND LET IT BE REASSIGNED NEXT REGROW CYCLE
				//aux_ints_1[syn_idx] = syn_strength;
				return;	
			}
		}

		syn_strengths[syn_idx] = 0;	
		syn_src_spi_x_offs[syn_idx] = rnd_spi_ofs;
		//syn_src_row_ids[syn_idx] =

			//aux_ints_1[syn_idx] = syn_strength;
	}

	
	/*int dead_syn = (syn_strength <= -100);
	syn_src_spi_x_offs[syn_idx] = mul24(dead_syn, (int)rnd_spi_ofs);
	syn_strengths[syn_idx] = mul24(!(dead_syn), (int)syn_strength);*/

	//syn_src_row_ids[syn_idx] =

}


/*	PYR_CYCLE():

		- Vectorize
*/
__kernel void pyr_cycle(
				__global uchar const* const den_states,
				//__private uchar const pyr_axn_row_base,
				__global uchar* const pyr_energies,
				__global uchar* const pyr_best1_den_ids,
				__global uchar* const pyr_best1_den_states,
				__global uchar* const pyr_best2_den_ids,
				__global uchar* const pyr_best2_den_states,
				__global uchar* const pyr_fuzs
				//__global uchar* const axn_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const pyr_idx = mad24(row_id, row_width, col_id);
	uint const den_ofs = pyr_idx << DENDRITES_PER_CELL_DISTAL_LOG2;
	//uint const axn_idx = axn_idx_2d(pyr_axn_row_base + row_id, row_width, col_id);
	//uint const axn_idx = mad24(pyr_axn_row_base + row_id, row_width, col_id + (uint)SYNAPSE_REACH);
	uchar pyr_energy = pyr_energies[pyr_idx];

	//uint den_sum = 0;

	uchar best1_den_state = 0;
	uchar best1_den_id = 0;

	uchar best2_den_state = 0;
	uchar best2_den_id = 0;
	//int active_dendrites = 0;

	uchar den_state = 0;

	//uint pyr_fuz = pyr_fuzs[pyr_idx];

		//#pragma unroll 
	for (uchar i = 0; i < DENDRITES_PER_CELL_DISTAL; i++) {
		uchar den_state = den_states[den_ofs + i];
		int den_state_bigger = (den_state > best1_den_state);

		best2_den_id = mad24(den_state_bigger, best1_den_id, mul24(!den_state_bigger, best2_den_id));
		best2_den_state = mad24(den_state_bigger, best1_den_state, mul24(!den_state_bigger, best2_den_state));

		best1_den_id = mad24(den_state_bigger, i, mul24(!den_state_bigger, best1_den_id));
		best1_den_state = mad24(den_state_bigger, den_state, mul24(!den_state_bigger, best1_den_state));

		//best1_den_state = mul24(den_state_bigger, den_state);

		//den_sum += den_state;
		//den_sum += (den_state != 0);
		//den_sum += (den_state > 0);
		//active_dendrites += (den_state > 0);
	}
	
	//den_sum = den_sum >> 2;


	// EXPERIMENTAL ENERGY CODE
	if (best1_den_state > 0) {
		if (pyr_energy >= 9) {
			pyr_energy -= 9;
			den_state = best1_den_state;
		} else {
			pyr_energy += 1;
		}
	} else {
		if (pyr_energy < 255) {
			pyr_energy += 1;
		}
	}


	pyr_energies[pyr_idx] = pyr_energy;
	pyr_best1_den_ids[pyr_idx] = best1_den_id;
	pyr_best1_den_states[pyr_idx] = best1_den_state;
	pyr_best2_den_ids[pyr_idx] = best2_den_id;
	pyr_best2_den_states[pyr_idx] = best2_den_state;
	pyr_fuzs[pyr_idx] = den_state;


	//pyr_fuzs[pyr_idx] = clamp(den_sum, 0u, 255u); 	// v.N1
	//axn_states[axn_idx] = clamp(den_sum, 0u, 255u);

	//pyr_fuzs[pyr_idx] = clamp(den_sum, 0, 127);

	//pyr_fuzs[pyr_idx] = (den_sum >> 1);
	//pyr_fuzs[pyr_idx] = active_dendrites;
}


/*	COL_OUTPUT()
		- rename coming
*/
__kernel void col_output(
				__global uchar const* const spi_states,	// CONVERT TO READING FROM AXON
				__global uchar const* const pyr_fuzs,
				__global uchar const* const pyr_best1_den_states,
				//__private uchar const spi_row_count,
				__private uchar const pyr_depth,
				//__private uchar const pyr_axn_base_row,
				__private uchar const output_axn_row,
				//__private uchar const pyr_base_row,
				__global uchar* const mcol_pyr_fuz_flags,
				__global uchar* const mcol_best_col_den_states,
				__global uchar* const axn_states
				
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const output_axn_idx = axn_idx_2d(output_axn_row + row_id, row_width, col_id, 0);
	//uint const output_axn_idx = mad24(output_axn_row + row_id, row_width, col_id + (uint)SYNAPSE_REACH);
	//uint const axn_idx_output = axn_idx_wrap_2d(axn_row_output, col_id);
	uint const col_idx = mad24(row_id, row_width, col_id);

	int spi_state = spi_states[col_idx];
	uchar max_den_state = 0;
	int col_pyr_fuz_total = 0;

	for (uint i = 0; i < pyr_depth; i++) {
		// POTENTIALLY FALSE ASSUMPTION HERE ABOUT PYR CELLS ALL BEING INVOLVED IN OUTPUT
		uint pyr_idx = mad24(i, row_width, col_id);	

		uchar pyr_best1_den_state = pyr_best1_den_states[pyr_idx];
		uchar pyr_fuz = pyr_fuzs[pyr_idx];

		max_den_state = max(max_den_state, pyr_best1_den_state);
		
		col_pyr_fuz_total = max(col_pyr_fuz_total, (int)pyr_fuz);

		//col_pyr_fuz_total += axn_states[axn_idx_pyr];						
		//col_pyr_fuz_total = max(col_pyr_fuz_total, (int)axn_states[axn_idx_pyr]); 
		
		//col_pyr_fuz_total += (axn_states[axn_idx_pyr] > 0);
	}


	mcol_pyr_fuz_flags[col_idx] = clamp(col_pyr_fuz_total, 0, 255);
	mcol_best_col_den_states[col_idx] = max_den_state;



	//axn_states[output_axn_idx] = clamp(col_pyr_fuz_total, 0, 255);
	//axn_states[output_axn_idx] = clamp(spi_state, 0, 255);
	axn_states[output_axn_idx] = clamp(col_pyr_fuz_total + spi_state, 0, 255); // *****
}









/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=========================== SCRAPS & BONES BELOW ==============================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/




// VECTORIZE
/*static inline void syns_ltd_unused_bak( 
				__global uchar const* const syn_states,
				uint const syn_idx_start,
				uint const syns_per_den_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		char syn_strength = syn_strengths[i];
		uchar syn_state = syn_states[i];
		//int is_neg = (syn_strength < 0);	// NEGATIVE STRENGTH SYNAPSES GET A BONUS (LOOKS LIKE THIS MAY BE NO BUENO)

		uchar rnd_char = ((rnd ^ i) & 0x7F) >> LTD_BIAS_LOG2;		
		int inc = (rnd_char > abs(syn_strength));

		if (syn_state == 0) {
			continue;
		} else {
			syn_strength -= inc;
		}

		syn_strengths[i] = syn_strength;
		
		//syn_strengths[i] = (syn_strength - inc) + mul24((syn_state != 0), inc + inc);
	}
}*/




__kernel void pyrs_ltp_unoptd_bak(
	__global uchar const* const axn_states,
	//__global uchar const* const pyr_fuzs,
	__global uchar const* const pyr_best1_den_ids,
	__global uchar const* const den_states,
	__global uchar const* const syn_states,
	__private uint const pyr_axn_idx_base, 
	__private uint const syns_per_den_l2,
	__private uint const dens_per_cel_l2,
	__private uint const pyrs_per_wi,
	__private uint const rnd,
	//__global int* const aux_ints_1,
	__global uchar* const pyr_flag_sets,
	__global uchar* const pyr_prev_best1_den_ids,
	__global char* const syn_strengths
) {
	uint const row_id = get_global_id(0);
	uint const pg_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const pyr_grp_idx = mad24(row_id, row_width, pg_id);
	//uint const den_ofs = pyr_idx << DENDRITES_PER_CELL_DISTAL_LOG2;

	//uint const axn_idx_base = mad24(pyr_axn_row_base + row_id, row_width, col_id + (uint)SYNAPSE_REACH);

	uint const pyr_idz = mul24(pyr_grp_idx, pyrs_per_wi);
	uint const pyr_idx_n = pyr_idz + pyrs_per_wi;	

	//uint const pyr_idz = mul24(pyr_grp_id, pyrs_per_wi);
	//uint const pyr_idx_n = pyr_idz + pyrs_per_wi;

	//uint debug_output = 0;
 
	for (uint i = pyr_idz; i < pyr_idx_n; i++) {
		uchar pyr_best1_den_id = pyr_best1_den_ids[i];
		uchar pyr_prev_best1_den_id = pyr_prev_best1_den_ids[i];
		uchar pyr_flag_set = pyr_flag_sets[i];

		uint den_idx_init = (i << dens_per_cel_l2);
		uint syn_idx = ((den_idx_init + pyr_best1_den_id) << syns_per_den_l2);

		int pyr_prev_concrete = (pyr_flag_set & PYR_PREV_CONCRETE_FLAG) == PYR_PREV_CONCRETE_FLAG;
		int pyr_best_in_col = (pyr_flag_set & PYR_BEST_IN_COL_FLAG) == PYR_BEST_IN_COL_FLAG;
		int pyr_prev_stp = (pyr_flag_set & PYR_PREV_STP_FLAG) == PYR_PREV_STP_FLAG;


		if (axn_states[i + pyr_axn_idx_base] == 0) {
			if (pyr_prev_stp) {
			//if (pyr_prev_concrete && (pyr_prev_best1_den_id == pyr_best1_den_id)) {

				// 	
				//  NOT SURE WHAT WE'RE GOING TO DO WITH THIS
				// 	DEFINITELY REDUCES OVERALL PREDICTIONS
				// 	ALSO APPEARS TO REDUCE SUPERFLUOUS ONES
				//	CELL SPECIFIC POST-ACTIVATION LEARNING SHOULD REDUCE THIS
				// 	SURVEY SAYS LEAVE IT IN FOR NOW
				//
				//syns_ltd(syn_states, syn_idx, syns_per_den_l2, rnd, syn_strengths);

			}

			pyr_flag_set &= ~PYR_PREV_CONCRETE_FLAG;
			pyr_flag_set &= ~PYR_PREV_STP_FLAG;
			continue;
		} else {
			if (pyr_best_in_col) {
				//syns_ltp_ltd(syn_states, syn_idx, syns_per_den_l2, rnd, syn_strengths);
				
				pyr_prev_best1_den_id = pyr_best1_den_id;
				pyr_flag_set |= PYR_PREV_STP_FLAG;
			}

			pyr_flag_set |= PYR_PREV_CONCRETE_FLAG;
		}

		pyr_prev_best1_den_ids[i] = pyr_prev_best1_den_id;
		pyr_flag_sets[i] = pyr_flag_set;
	}
}




/*
	OPTIMIZE FOR WORKGROUP
	VECTORIZE
*/
__kernel void den_dist_cycle_unused_old(
	__global uchar const* const syn_states,
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

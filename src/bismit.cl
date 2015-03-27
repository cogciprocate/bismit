//#define DENDRITES_PER_CELL_DISTAL_LOG2						4
//static int const DENDRITES_PER_CELL_DISTAL =				1 << DENDRITES_PER_CELL_DISTAL_LOG2;

// SYNAPSES_PER_DENDRITE_DISTAL_LOG2: [MAX: 8]
//#define SYNAPSES_PER_DENDRITE_DISTAL_LOG2					4
//static int const SYNAPSES_PER_DENDRITE_DISTAL =			1 << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

//static uint const SYNAPSES_PER_CELL_DISTAL_LOG2 =			DENDRITES_PER_CELL_DISTAL_LOG2 + SYNAPSES_PER_DENDRITE_DISTAL_LOG2;
//static uint const SYNAPSES_PER_CELL_DISTAL =				1 << (DENDRITES_PER_CELL_DISTAL_LOG2 + SYNAPSES_PER_DENDRITE_DISTAL_LOG2);

//#define SYNAPSE_STRENGTH_DEFAULT_DISTAL_LOG2				4
//static int const SYNAPSE_STRENGTH_DEFAULT_DISTAL =			1 << SYNAPSE_STRENGTH_DEFAULT_DISTAL_LOG2;

//#define DENDRITES_PER_CELL_PROXIMAL_LOG2					0
//static uint const DENDRITES_PER_CELL_PROXIMAL =			1 << DENDRITES_PER_CELL_PROXIMAL_LOG2;

// SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2: [MAX 8]
//#define SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2					7

//static uint const SYNAPSES_PER_DENDRITE_PROXIMAL =			1 << SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;

//static int const SYNAPSES_PER_CELL_PROXIMAL_LOG2 =			DENDRITES_PER_CELL_PROXIMAL_LOG2 + SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;
//static int const SYNAPSES_PER_CELL_PROXIMAL =				1 << (DENDRITES_PER_CELL_PROXIMAL_LOG2 + SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2);

//#define SYNAPSE_STRENGTH_DEFAULT_PROXIMAL_LOG2				4
//static uint const SYNAPSE_STRENGTH_DEFAULT_PROXIMAL =		1 << SYNAPSE_STRENGTH_DEFAULT_PROXIMAL_LOG2;

//#define SYNAPSE_STRENGTH_MAX				127

//#define COLUMNS_PER_HYPERCOLUMN_LOG2		6

//#define SYNAPSE_REACH						128
//static uint const SYNAPSE_SPAN = 			SYNAPSE_REACH * 2;

//#define SYNAPSE_WORKGROUP_SIZE				256
//#define AXONS_WORKGROUP_SIZE 				256

//#define ASPINY_REACH_LOG2					2
/*#define ASPINY_REACH 						4
#define ASPINY_SPAN_LOG2 					3
#define ASPINY_SPAN 						8*/
//static int const ASPINY_REACH =				1 << ASPINY_REACH_LOG2;
//static int const ASPINY_SPAN_LOG2 =			ASPINY_REACH_LOG2 + 1;
//static int const ASPINY_SPAN =				1 << (ASPINY_REACH_LOG2 + 1);

//#define COLUMN_DOMINANCE_FLOOR				47	//47

/*
static inline uint xos_rng(uint seed) {
	uint rnd = seed + 1181783497;
	rnd ^= (rnd << 23);
	rnd ^= (rnd >> 17);
	rnd ^= seed ^ (rnd >> 26);

	return rnd + seed;
}
*/

static inline uint asp_to_col_ofs(uint asp_idx) {
	return (asp_idx - ASPINY_REACH) << ASPINY_SPAN_LOG2;
}

static inline uint asp_col_id_to_col_idx(uint asp_idx, uint asp_col_id) {
	return (asp_to_col_ofs(asp_idx) + (asp_col_id & (ASPINY_SPAN - 1)));
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
	COL_SYNS_CYCLE():
		number of source rows can not exceed: 
			ROWS * (SYNAPSES_PER_CELL_PROXIMAL + SYNAPSE_WORKGROUP_SIZE)

	TODO:
		- Vectorize!
		- Col Inputs/Outputs probably need to be limited to one row.

	WATCH OUT FOR:
		- Bank conflicts once src_ofs start to change
*/
//	__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void syns_cycle(
	__global uchar* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__private uint const syns_per_cell_l2,
	/*__global int* const aux_ints_0,
	__global int* const aux_ints_1,*/
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

	//uint const cels_per_iter = wg_size >> syns_per_cell_l2;
	//uint const iters_per_cel = 0;		// syns_per_cell / wg_size

	//uint const init_cel_idx = base_cel_idx + (l_id >> syns_per_cell_l2);
	uint const init_syn_idx = (base_cel_idx << syns_per_cell_l2) + l_id;
	//uint const init_axn_idx = mad24((uint)syn_src_row_ids[init_syn_idx], row_width, (uint)(0 + syn_src_ofs[init_syn_idx]));
	
	/*uint cel_idx = init_cel_idx;
	uint axn_idx = init_axn_idx;*/

	uint syn_n = init_syn_idx + (wg_size << syns_per_cell_l2);
	//uint n = syns_per_cell_l2;

	//aux_ints_0[col_id] = base_cel_idx;
	//uint syn_idx = init_syn_idx;

	//uint q = 0;

	//uint row_i = 0;
	uint syn_col_i = init_syn_idx;

	for (uint syn_idx = init_syn_idx; syn_idx < syn_n; syn_idx += wg_size) {
		//row_i += (syn_i >= syn_row_width);
		syn_col_i -= mul24(syn_row_width, (uint)(syn_col_i >= syn_row_width));

		uint col_pos = syn_col_i >> syns_per_cell_l2;
		//uint cel_idx_dup = init_cel_idx + i;
		uint axn_idx = (mad24((uint)syn_src_row_ids[syn_idx], row_width, (uint)(col_pos + syn_src_ofs[syn_idx]))) + SYNAPSE_REACH;

		uchar axn_state = axn_states[axn_idx];

		syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);
		//syn_states[syn_idx] = 5;
		//syn_states[syn_idx] = (axn_idx - 3200) >> 2;

		//syn_idx += wg_size;
		/*q++;
		if (q < 65536) {
			continue;
		} else {
			aux_ints_0[cel_idx] = 9999;
			syn_states[syn_idx] = 99;
			break;
		}*/
		syn_col_i += wg_size;
	}
}




/*
	NEEDS REWRITE
	OPTIMIZE FOR WORKGROUP
	VECTORIZE
*/
__kernel void den_prox_cycle(
	__global uchar* const syn_states,
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
	uint n = (1 << syns_per_den_l2);

	for (uint i = 0; i < n; i += 1) {
		uchar syn_state = syn_states[syn_ofs + i];
		syn_sum += mul24((syn_state > 128), (syn_state) & (0x7F));
	}

	//den_states[den_idx] = (syn_sum >> syns_per_den_l2);
	den_states[den_idx] = syn_sum >> (syns_per_den_l2 - 1);
	//den_states[den_idx] = mad24((den_total > 0), 128, clamp(den_total >> (syns_per_den_l2 + 1), 0, 127));
	//den_states[den_idx] = den_total; //(0, 1, 2, 3); 
}




__kernel void aspiny_cycle_pre(
	__global uchar* const col_states,
	__global uchar* const asp_states,
	__global uchar* const asp_col_ids
	
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	//uint const asp_idx = asp_pos + (1 << ASPINY_REACH_LOG2);
	uint const asp_idx = (asp_pos + ASPINY_REACH);
	uint const col_ofs = asp_pos << ASPINY_SPAN_LOG2;

	uchar col_states_vec[1 << (ASPINY_REACH_LOG2)]; // = {0, 0, 0, 0};

	uchar winner_val = 0;
	uchar winner_id = 0;
	
	uchar val = 0;
	uchar id = 0;

	//uint n = ASPINY_REACH >> 2;

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
	//__global uchar* const asp_col_ids,
	__global uchar* const asp_wins
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);
	//uint const col_ofs = asp_pos << ASPINY_SPAN_LOG2;

	uint const as_bitmask = (ASPINY_SPAN - 1);

	uchar asp_state = asp_states[asp_idx];
	uchar asp_win = asp_wins[asp_idx];

	int win_count = asp_win; // asp_wins[asp_idx];

	for (uint i = 0; i < ASPINY_SPAN; i++) {
		uint cur_comp_idx = (asp_idx - ASPINY_REACH) + i + (i > (ASPINY_REACH - 1));
		uchar cur_comp_state = asp_states[cur_comp_idx];
		uchar cur_comp_win = asp_wins[cur_comp_idx];

		if (asp_win == cur_comp_win) {
			if ((asp_state == cur_comp_state) && (asp_state > 0)) {
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

	//asp_wins[asp_idx] = win_count;
	//asp_wins[asp_idx] = asp_state >> ASPINY_SPAN_LOG2;
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
	uchar asp_win = asp_wins[asp_idx];

	asp_states[asp_idx] = asp_win;

	asp_wins[asp_idx] = 0;
}



// VECTORIZE ME
// RENAME ME
// CLEAN ME UP
	//__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void col_post_inhib_unoptd (										
	__global uchar* const asp_col_ids,
	__global uchar* const asp_states,
	__global uchar* const asp_wins,
	__global uchar* const col_states,
	//__global uchar* const col_cel_status,
	__global int* const aux_ints_0,
	__global int* const aux_ints_1,
	//__global uchar* const pyr_states,
	//__private uchar const pyr_height,
	//__private uchar const pyr_base_row,
	__private uint const col_axn_row_offset,
	__global uchar* const axn_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = col_idx + mad24(col_axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (col_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;

	uchar asp_state = asp_states[asp_idx];
	//uchar asp_win = asp_wins[asp_idx];
	uchar col_state = col_states[col_idx];

	int win = (asp_col_id_to_col_idx(asp_idx, (asp_col_ids[asp_idx])) == col_idx);
	win = (win && asp_state);

	//if (win > 0) {
	/*int column_predictions = 0;

	for (uint i = 0; i < pyr_height; i++) {
		uint pyr_idx = mad24((uint)i, row_width, col_id);
		column_predictions += (pyr_states[pyr_idx] > 0);
	}*/

	/*for (uint i = 0; i < pyr_height; i++) {
		uint pyr_idx = mad24((uint)i, row_width, col_id);
		int pyr_state = pyr_states[pyr_idx];
		uint cc_status = col_cel_status[col_idx];
		//pyr_state += (((pyr_state > 0) || (column_predictions == 0)) && win) << 7;

		if (cc_status) {

		}

		pyr_states[pyr_idx] = clamp(pyr_state, 0, 254); // CLAMP SHOULDN'T BE NEEDED

		//aux_ints_0[pyr_idx] = column_predictions;
	}*/
	//}

	col_states[col_idx] = mul24(col_state, (win > 0));
	axn_states[axn_idx] = mul24(col_state, (win > 0));
	//aux_ints_0[col_idx] = mul24(activity, (win > 0));
	//aux_ints_1[col_idx] = mul24((int)(asp_idx & 0x0F), (win >= COLUMN_DOMINANCE_FLOOR));
}




__kernel void pyr_activate(
				__global uchar* const col_states,
				__global uchar* const col_cel_status,
				
				//__private uchar const col_row_count,
				__private uchar const pyr_row_count,
				//__private uchar const axn_output_row,
				//__private uchar const pyr_base_row,
				//__global uchar* const axn_states
				__global uchar* const pyr_states
				
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	//uint const axn_idx = mad24(axn_output_row, row_width, col_id + (uint)SYNAPSE_REACH);
	//uint const col_idx = mad24(row_id, row_width, col_id);
	uint pyr_idx = mad24(row_id, row_width, col_id);

	uchar col_state = col_states[col_id];
	uchar cc_status = col_cel_status[col_id];

	
}




/*__kernel void col_pyr_activate(
				__global uchar* const col_states,
				__global uchar* const col_cel_status,
				__global uchar* const pyr_states,
				//__private uchar const col_row_count,
				__private uchar const pyr_row_count,
				//__private uchar const axn_output_row,
				//__private uchar const pyr_base_row,
				//__global uchar* const axn_states
				
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	//uint const axn_idx = mad24(axn_output_row, row_width, col_id + (uint)SYNAPSE_REACH);
	uint const col_idx = mad24(row_id, row_width, col_id);

	uchar col_state = col_states[col_idx];

	int output_total = 0;

	for (uint i = 0; i < pyr_row_count; i++) {
		uint pyr_idx = mad24(i, row_width, col_id);
		output_total += pyr_states[pyr_idx];
		//output_total += 1;
	}

	//axn_states[axn_idx] = clamp(output_total, 0, 255);
	//axn_states[axn_idx] = test;
}*/



/*
	OPTIMIZE FOR WORKGROUP
	VECTORIZE
*/
__kernel void den_dist_cycle(
	__global uchar* const syn_states,
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
	uint n = (1 << syns_per_den_l2);

	for (uint i = 0; i < n; i += 1) {
		uchar syn_state = syn_states[syn_ofs + i];
		syn_sum += syn_state;
	}

	uchar den_state = clamp((syn_sum >> 7), 0, 255);

	den_states[den_idx] = mul24((den_state > DENDRITE_INITIAL_THRESHOLD), den_state);
	//den_states[den_idx] = mad24((den_total > 0), 128, clamp(den_total >> (syns_per_den_l2 + 1), 0, 127));
	//den_states[den_idx] = den_total; //(0, 1, 2, 3); 
	//den_states[den_idx] = (syn_sum >> syns_per_den_l2);

}


__kernel void pyr_cycle_dens(
				__global uchar* const den_states,
				__private uchar const pyr_axn_row_offs,
				__global uchar* const pyr_states
				//__global uchar* const axn_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row_id, row_width, col_id);
	uint const den_grp = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;
	//uint const axn_idx = mad24(pyr_base_axn_row + row_id, row_width, col_id + (uint)SYNAPSE_REACH);

	int den_sum = 0;

	//int active_dendrites = 0;

	//uint pyr_state = pyr_states[cel_idx];

		#pragma unroll 
	for (uint i = 0; i < DENDRITES_PER_CELL_DISTAL; i++) {
		uchar den_state = den_states[den_grp + i];
		den_sum += den_state;
		//active_dendrites += (den_state > 0);
	}
	
	//den_sum >>= DENDRITES_PER_CELL_DISTAL_LOG2;

	//pyr_states[cel_idx] = (den_sum >> 1);
	pyr_states[cel_idx] = clamp(den_sum, 0, 255);
	//pyr_states[cel_idx] = active_dendrites;
}


__kernel void col_output(
				__global uchar* const col_states,
				__global uchar* const pyr_states,
				__private uchar const col_row_count,
				__private uchar const pyr_row_count,
				__private uchar const axn_output_row,
				//__private uchar const pyr_base_row,
				__global uchar* const col_cel_status,
				__global uchar* const axn_states
				
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const axn_idx = mad24(axn_output_row, row_width, col_id + (uint)SYNAPSE_REACH);
	uint const col_idx = mad24(row_id, row_width, col_id);

	int col_state = col_states[col_idx];

	int output_total = 0;

	for (uint i = 0; i < pyr_row_count; i++) {
		uint pyr_idx = mad24(i, row_width, col_id);
		output_total += pyr_states[pyr_idx];
		//output_total += 1;
	}

	col_cel_status[col_idx] = clamp(output_total, 0, 255);
	axn_states[axn_idx] = clamp(max(output_total, col_state), 0, 255);
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





/*__kernel void dens_cycle_origish_with_vectors(
	__global uchar* const syn_states,
	__private uint const syns_per_den_l2,
	__global uchar* const den_states
) {
	uint const row_id = get_global_id(0);
	uint const den_id = get_global_id(1);
	//uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const den_idx = mad24(row_id, row_width, den_id);
	uint const syn4_per_den_l2 = syns_per_den_l2 - 2;
	uint const syn_ofs = den_idx << syn4_per_den_l2;

	int4 syn_sum = (int4)(0, 0, 0, 0);
	uint n = 1 << syn4_per_den_l2;

	for (uint i = 0; i < n; i += 1) {
		syn_sum += convert_int4(vload4((syn_ofs + i), syn_states));
		//syn_sum += syn_state.s0;
	}

	int den_total = syn_sum.s0 + syn_sum.s1 + syn_sum.s2 + syn_sum.s3;

	den_states[den_idx] = mad24((den_total > 0), 128, clamp(den_total >> (syns_per_den_l2 + 1), 0, 127));
	//den_states[den_idx] = den_total; //(0, 1, 2, 3); 
}*/



/*__kernel void dens_cycle_WITH_VECTOR(
	__global uchar* const syn_states,
	__private uint const syns_per_den_l2,
	__global uchar* const den_states
) {
	uint const row_id = get_global_id(0);
	uint const den_id = get_global_id(1);
	//uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const den_idx = mad24(row_id, row_width, den_id);
	uint const syn4_per_den_l2 = syns_per_den_l2 - 2;
	uint const syn_ofs = den_idx << syn4_per_den_l2;

	int4 syn_sum = (int4)(0, 0, 0, 0);
	uint n = 1 << syn4_per_den_l2;

	for (uint i = 0; i < n; i += 1) {
		syn_sum += convert_int4(vload4((syn_ofs + i), syn_states));
		//syn_sum += syn_state.s0;
	}

	int den_total = syn_sum.s0 + syn_sum.s1 + syn_sum.s2 + syn_sum.s3;

	den_states[den_idx] = mad24((den_total > 0), 128, clamp(den_total >> (syns_per_den_l2 + 1), 0, 127));
	//den_states[den_idx] = den_total; //(0, 1, 2, 3); 
}*/


/*__kernel void aspiny_cycle_wins_old(
	__global uchar* const asp_states,
	__global uchar* const asp_col_ids,
	__global uchar* const asp_wins,
	__private uint const win_floor,
	__private uint const mode
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);
	//uint const col_ofs = asp_pos << ASPINY_SPAN_LOG2;

	uchar asp_state = asp_states[asp_idx];
	uchar asp_win = asp_wins[asp_idx];

	int win_count = asp_win; // asp_wins[asp_idx];

	for (uint i = 0; i < ASPINY_SPAN; i++) {

		uint cur_asp_idx = (asp_idx - ASPINY_REACH) + i + (i > (ASPINY_REACH - 1));
		uchar cur_comp_state = asp_states[cur_asp_idx];
		uchar cur_comp_wins = asp_wins[cur_asp_idx];

		//if (((asp_idx & 0x07) == (asp_state & 0x07)))

		if (mode == 0) {
			if (asp_state < cur_comp_state) {
				continue;
			} else if (asp_state == cur_comp_state) {
				if ((asp_idx & 0x01) ^ (cur_asp_idx & 0x01)) {
					win_count += ((asp_idx) < (cur_asp_idx));
				}
			} else {
				win_count += 1;
			}
		} else if (mode == 1) {
			if (asp_win < cur_comp_wins) {
				continue;
			} else if (asp_win == cur_comp_wins) {
				if ((asp_idx & 0x01) ^ (cur_asp_idx & 0x01)) {
					win_count += ((asp_idx) < (cur_asp_idx));
				}
			} else {
				win_count += 1;
			}
		} else if (mode == 2) {
			if (cur_comp_wins > 0) {
				//asp_states[asp_idx] = 0;
				//asp_states[asp_idx] = 0;
				win_count = 0;
				break;
			} else {
				win_count += 1;
			}
		}
	}
	if (mode == 0) {
		asp_wins[asp_idx] = mul24(win_count, (asp_state >> ASPINY_SPAN_LOG2));
	} else if (mode == 1) {
		asp_wins[asp_idx] = mul24(win_count, (win_count >= win_floor));
	} else if (mode == 2) {
		asp_wins[asp_idx] = mul24(win_count, (win_count >= win_floor));
	}


	//asp_wins[asp_idx] = mul24(win_count, (win_count >= win_floor));
	//asp_wins[asp_idx] = win_count;
	//asp_wins[asp_idx] = asp_state >> ASPINY_SPAN_LOG2;
}*/




/*__kernel void syns_cycle_1_1(
	__global uchar* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__private uint const dens_per_wg,
	__global int* const aux_ints_0,
	__global int* const aux_ints_1,
	__global uchar* const syn_states
) {
	uint const row_id = get_global_id(0);
	uint const den_grp_id = get_global_id(1);
	uint const l_id = get_local_id(1); 
	uint const wg_id = get_group_id(1);
	uint const wg_size = get_local_size(1);
	uint const row_width = mul24(get_global_size(1), dens_per_wg);

	uint const base_col_id = mul24(wg_id, mul24(wg_size, dens_per_wg));
	
	//uint 
	//uint const syns_per_wg = SYNAPSES_PER_DENDRITE_PROXIMAL * dens_per_wg;

	uint dens_per_cell = (1 << DENDRITES_PER_CELL_PROXIMAL_LOG2);




	uint const base_syn_idx = mul24(base_col_id, (uint)(1 << SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2)) + l_id; 

	uint const base_cel_idx = mad24(row_id, row_width, base_col_id);

	uint const base_axn_idx = mad24((uint)syn_src_row_ids[base_syn_idx], row_width, (uint)(0 + syn_src_ofs[base_syn_idx]));




	uint syn_idx = base_syn_idx;
	uint den_idx = 0;
	uint cel_idx = 0;
	uint axn_idx = 0;

	uint syn_i = 0;
	uint den_i = 0;
	uint cel_i = 0;
	uint axn_i = 0;

	uint n = den_grp_id + (SYNAPSE_REACH << 1);

	for (uint i = den_grp_id; i < n; i += dens_per_wg) {

		syn_idx = base_syn_idx + syn_i;

		//cel_idx = 
		axn_idx = mad24((uint)syn_src_row_ids[syn_idx], row_width, (uint)(i + syn_src_ofs[syn_idx]));



		syn_states[syn_idx] = axn_states[axn_idx];


		den_i += dens_per_wg;
		syn_i += SYNAPSE_WORKGROUP_SIZE;
		//syn_idx += SYNAPSE_WORKGROUP_SIZE;
	}
}*/




/*
	__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void syns_cycle_old(
	__global uchar* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__global uchar* const syn_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row_id, row_width, col_id);
	
	uint syn_idx = ((cel_idx - l_id) << SYNAPSES_PER_CELL_PROXIMAL_LOG2) + l_id;

	uint n = SYNAPSE_WORKGROUP_SIZE + col_id;
	uint axn_idx;

	for (uint i = col_id; i < n; i++) {
		axn_idx = mad24((uint)syn_src_row_ids[syn_idx], row_width, (uint)(i + syn_src_ofs[syn_idx]));
		syn_states[syn_idx] = axn_states[axn_idx];
		syn_idx += SYNAPSE_WORKGROUP_SIZE;
	}
}
*/




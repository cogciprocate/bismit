#define DENDRITES_PER_CELL_DISTAL_LOG2						4
static uint const DENDRITES_PER_CELL_DISTAL =				1 << DENDRITES_PER_CELL_DISTAL_LOG2;

#define SYNAPSES_PER_DENDRITE_DISTAL_LOG2					4
static uint const SYNAPSES_PER_DENDRITE_DISTAL =			1 << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

//static uint const SYNAPSES_PER_CELL_DISTAL_LOG2 =			DENDRITES_PER_CELL_DISTAL_LOG2 + SYNAPSES_PER_DENDRITE_DISTAL_LOG2;
//static uint const SYNAPSES_PER_CELL_DISTAL =				1 << (DENDRITES_PER_CELL_DISTAL_LOG2 + SYNAPSES_PER_DENDRITE_DISTAL_LOG2);

#define SYNAPSE_STRENGTH_DEFAULT_DISTAL_LOG2				4
static uint const SYNAPSE_STRENGTH_DEFAULT_DISTAL =			1 << SYNAPSE_STRENGTH_DEFAULT_DISTAL_LOG2;


#define DENDRITES_PER_CELL_PROXIMAL_LOG2					0
//static uint const DENDRITES_PER_CELL_PROXIMAL =			1 << DENDRITES_PER_CELL_PROXIMAL_LOG2;

#define SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2					8
//static uint const SYNAPSES_PER_DENDRITE_PROXIMAL =		1 << SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;

static uint const SYNAPSES_PER_CELL_PROXIMAL_LOG2 =			DENDRITES_PER_CELL_PROXIMAL_LOG2 + SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;
static uint const SYNAPSES_PER_CELL_PROXIMAL =				1 << (DENDRITES_PER_CELL_PROXIMAL_LOG2 + SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2);

#define SYNAPSE_STRENGTH_DEFAULT_PROXIMAL_LOG2				4
//static uint const SYNAPSE_STRENGTH_DEFAULT_PROXIMAL =		1 << SYNAPSE_STRENGTH_DEFAULT_PROXIMAL_LOG2;


#define SYNAPSE_STRENGTH_MAX				127

#define COLUMNS_PER_HYPERCOLUMN_LOG2		6

#define SYNAPSE_REACH						128
//static uint const SYNAPSE_SPAN = 			SYNAPSE_REACH * 2;

#define SYNAPSE_WORKGROUP_SIZE				256
#define AXONS_WORKGROUP_SIZE 				256

#define ASPINY_REACH_LOG2					2
static uint const ASPINY_REACH =			1 << ASPINY_REACH_LOG2;
static uint const ASPINY_SPAN_LOG2 =		ASPINY_REACH_LOG2 + 1;
static uint const ASPINY_SPAN =			1 << (ASPINY_REACH_LOG2 + 1);





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

	WATCH OUT FOR:
		- Bank conflicts once src_ofs start to change
*/
	__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void col_syns_cycle(
	__global char* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__global char* const syn_states
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


__kernel void col_cycle(
	__global char* const syn_states,
	__global char* const col_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	//uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const syn4_per_cel_l2 = SYNAPSES_PER_CELL_PROXIMAL_LOG2 - 2;
	uint const syn_ofs = col_idx << syn4_per_cel_l2;

	char4 syn_state = (char4)(0, 0, 0, 0);
	int4 syn_sum = (int4)(0, 0, 0, 0);
	uint n = 1 << syn4_per_cel_l2;

	for (uint i = 0; i < n; i += 1) {
		syn_state = vload4((syn_ofs + i), syn_states);
		syn_sum += convert_int4(syn_state);
		//syn_sum += syn_state.s0;
	}

	int col_total = syn_sum.s0 + syn_sum.s1 + syn_sum.s2 + syn_sum.s3;

	col_states[col_idx] = (char)(col_total >> SYNAPSES_PER_CELL_PROXIMAL_LOG2);
	//col_states[col_idx] = cel_idx >> 2; //(0, 1, 2, 3); 
}


__kernel void aspiny_cycle(
	__global char* const col_states,
	__global uchar* const asp_ids,
	__global char* const asp_vals
) {
	uint const row_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const asp_pos = mad24(row_id, row_width, asp_id);
	//uint const asp_idx = asp_pos + (1 << ASPINY_REACH_LOG2);
	uint const asp_idx = (asp_pos + ASPINY_REACH);
	uint const col_ofs = asp_pos << ASPINY_SPAN_LOG2;

	char col_states_vec[1 << (ASPINY_REACH_LOG2)]; // = {0, 0, 0, 0};

	char winner_val = 0;
	char winner_id = 0;
	
	char val = 0;

	//uint n = ASPINY_REACH >> 2;

		#pragma unroll
	for (uint i = 0; i < ASPINY_SPAN; i += 4) {
		vstore4(vload4((col_ofs + i) >> 2, col_states), 0, col_states_vec);

			#pragma unroll
		for (uint j = 0; j < 4; j++) {
			val = col_states_vec[j];

			if (val <= winner_val) {
				continue;
			} else {
				winner_val = val;
				winner_id = j + i;
			}
		}
	}
	
	asp_vals[asp_idx] = winner_val;
	asp_ids[asp_idx] = winner_id;		// | (winner_val & 0xF8);
}



//	 HAVE TO WIN WITHIN YOUR GROUP OF 8 --- *THEN* WIN AMONG THE GROUP OF ~64

uint asp_to_col_ofs(uint asp_idx) {
	return (asp_idx - ASPINY_REACH) << ASPINY_SPAN_LOG2;
	//current_col_idx_ghetto = ((i - ASPINY_REACH) << ASPINY_SPAN_LOG2) + asp_ids[i];
}


	__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void axns_cycle_unoptd (
	__global uchar* const asp_ids,
	__global char* const asp_vals,
	__global char* const col_states,
	__global char* const axn_states,
	__global int* const aux_ints_0,
	__global int* const aux_ints_1,
	//__local uchar* wiv_local,
	__private uint const axn_row_offset
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = col_idx + mad24(axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (col_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;


	//uint practical_aspiny_reach = (ASPINY_REACH - 1);
	/*uint low_compare_asp_idx = asp_idx - 4;
	uint high_compare_asp_idx = asp_idx + 4;*/


	int const this_fucking_col_state = col_states[col_idx];
	uint const this_fucking_col_idx = col_idx;
	int winner_close = false;
	bool winner_far = false;
	uint max_asp_idx = 0;
	int max_col_state = 0;
	uint current_col_idx = 0;
	int const asp_offs = asp_idx - 4;

	if (col_states[col_idx] == asp_vals[asp_idx]) {
		winner_close = true;

		max_col_state = col_states[col_idx];
		max_asp_idx = asp_idx;
	}

	uint win_count = 0;

	if (winner_close > 0) {

		for (int i = -3; i <= 3; i++) {
			if (max_col_state < asp_vals[asp_idx + i]) {
				max_col_state = asp_vals[asp_idx + i];
				max_asp_idx = asp_idx + i;
			}
		}

		//aux_ints_0[col_idx + (i + 10) * 10] = max_col_state;

		/*if (win_count >= 5) {
			aux_ints_0[col_idx] = this_fucking_col_state;	//	max_col_state;
			aux_ints_1[col_idx] = win_count;
		}*/

	}

	if (this_fucking_col_state == max_col_state) {
		aux_ints_0[col_idx] = max_col_state;
	}

	/*if (win_count >= 8) {
		aux_ints_0[col_idx] = max_col_state;
		aux_ints_1[col_idx] = win_count;
	}*/


	/*if (winner_close) {

		for (int i = 0; i < 8; i++) {
			current_col_idx = asp_to_col_ofs(i) + asp_ids[i];

			if (col_states[col_idx] > col_states[current_col_idx]) {
				local_max_col_idx = col_idx;
				winner_far += 1;

			} else if (col_states[col_idx] == col_states[current_col_idx]) {

				if ((col_states[current_col_idx] & 0x01) ^ (col_states[col_idx] & 0x01)) {
					winner_far += 1;
					
				} else {
					winner_far -= 1;
				}

			} else {
				winner_far -= 1;
			}
		}
	}*/

	

	/*if (i == 4) {
				aux_ints_0[(col_idx - 4) + i] = asp_vals[() + i] + 100;
			} else {
				//aux_ints_0[(col_idx - 4) + i] = asp_vals[(asp_idx - 4) + i];
			}*/
	
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



	__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void axns_cycle_unoptd_4_0 (
	__global uchar* const asp_ids,
	__global char* const asp_vals,
	__global char* const col_states,
	__global char* const axn_states,
	__global int* const aux_ints_0,
	__global int* const aux_ints_1,
	//__local uchar* wiv_local,
	__private uint const axn_row_offset
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = col_idx + mad24(axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (col_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;


	//uint practical_aspiny_reach = (ASPINY_REACH - 1);
	/*uint low_compare_asp_idx = asp_idx - 4;
	uint high_compare_asp_idx = asp_idx + 4;*/


	int const this_fucking_col_state = col_states[col_idx];
	uint const this_fucking_col_idx = col_idx;
	int winner_close = false;
	bool winner_far = false;
	uint max_asp_idx = 0;
	int max_col_state = 0;
	uint current_col_idx = 0;
	int const asp_offs = asp_idx - 4;

	if (col_states[col_idx] == asp_vals[asp_idx]) {
		winner_close = true;

		max_col_state = col_states[col_idx];
		max_asp_idx = asp_idx;
	}

	uint win_count = 0;

	if (winner_close > 0) {

		for (int i = -3; i <= 3; i++) {
			if (max_col_state < asp_vals[asp_idx + i]) {
				max_col_state = asp_vals[asp_idx + i];
				max_asp_idx = asp_idx + i;
			}
		}

		//aux_ints_0[col_idx + (i + 10) * 10] = max_col_state;

		/*if (win_count >= 5) {
			aux_ints_0[col_idx] = this_fucking_col_state;	//	max_col_state;
			aux_ints_1[col_idx] = win_count;
		}*/

	}

	if (this_fucking_col_state == max_col_state) {
		aux_ints_0[col_idx] = max_col_state;
	}

	/*if (win_count >= 8) {
		aux_ints_0[col_idx] = max_col_state;
		aux_ints_1[col_idx] = win_count;
	}*/


	/*if (winner_close) {

		for (int i = 0; i < 8; i++) {
			current_col_idx = asp_to_col_ofs(i) + asp_ids[i];

			if (col_states[col_idx] > col_states[current_col_idx]) {
				local_max_col_idx = col_idx;
				winner_far += 1;

			} else if (col_states[col_idx] == col_states[current_col_idx]) {

				if ((col_states[current_col_idx] & 0x01) ^ (col_states[col_idx] & 0x01)) {
					winner_far += 1;
					
				} else {
					winner_far -= 1;
				}

			} else {
				winner_far -= 1;
			}
		}
	}*/

	

	/*if (i == 4) {
				aux_ints_0[(col_idx - 4) + i] = asp_vals[() + i] + 100;
			} else {
				//aux_ints_0[(col_idx - 4) + i] = asp_vals[(asp_idx - 4) + i];
			}*/
	
}



	__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void axns_cycle_unoptd_3_0_working (
	__global uchar* const asp_ids,
	__global char* const asp_vals,
	__global char* const col_states,
	__global char* const axn_states,
	//__local uchar* wiv_local,
	__private uint const axn_row_offset
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = col_idx + mad24(axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (col_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;


	uint practical_aspiny_reach = (ASPINY_REACH - 1);
	uint low_compare_asp_idx = asp_idx - (practical_aspiny_reach);
	uint high_compare_asp_idx = asp_idx + (practical_aspiny_reach);

	int winner_close = 0;
	int winner_far = 2;

	uint local_max_col_idx = 0;
	uint current_col_idx = 0;

	if ((asp_to_col_ofs(asp_idx)) == col_idx) {
		winner_close += 1;
	}

	if (winner_close) {

		for (uint i = low_compare_asp_idx; i <= high_compare_asp_idx; i++) {
			current_col_idx = asp_to_col_ofs(i) + asp_ids[i];

			if (col_states[col_idx] > col_states[current_col_idx]) {
				local_max_col_idx = current_col_idx;
				winner_far += 1;
			} else if (col_states[col_idx] == col_states[current_col_idx]) {

				if ((col_states[local_max_col_idx] & 0x01) ^ (col_states[col_idx] & 0x01)) {
					winner_far += 1;
					local_max_col_idx = current_col_idx;
				} else {
					winner_far -= 1;
				}
			} else {
				winner_far -= 1;
			}
		}
	}



	if ((winner_close > 0) && (winner_far > 0)) {
		//axn_states[axn_idx] = col_states[local_max_col_idx];			// * asp_vals[asp_idx];
		//axn_states[axn_idx] = winner_far;
		axn_states[axn_idx] = winner_far + winner_close + 10;
	}

	//axn_states[axn_idx] = winner_close;

}




	__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void axns_cycle_unoptd_incomplete_2_0 (
	__global uchar* asp_ids,
	__global char* asp_vals,
	__global char* col_states,
	__global char* axn_states,
	//__local uchar* wiv_local,
	__private uint const axn_row_offset
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = col_idx + mad24(axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (col_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;

	//uint col_winner_pos = asp_ids[asp_idx] & 0x07;
	//uint cel_pos = col_idx & 0x07;

	uint practical_aspiny_reach = (ASPINY_REACH - 1);
	uint low_compare_asp_idx = asp_idx - (practical_aspiny_reach);
	uint high_compare_asp_idx = asp_idx + (practical_aspiny_reach);

	//char col_val_local_max = 0;
	//uchar col_id_local_max = 0;

	uint winner_pts = 0;
	
	uint current_col_idx_ghetto = 0;

	//char asp_winner_val = 0;

	uint x = 0;


	bool winner_close = false;
	bool winner_far = false;

	if (asp_to_col_ofs(asp_idx) + asp_ids[asp_idx] == col_idx) {
		winner_close = true;
	}

		//#pragma unroll
	for (uint i = low_compare_asp_idx; i <= high_compare_asp_idx; i++) {

		current_col_idx_ghetto = asp_to_col_ofs(i) + asp_ids[i];

		//asp_winner_val = asp_vals[i];

		/*if (col_states[current_col_idx_ghetto] != asp_vals[i]) {
			axn_states[axn_idx] = -99;
			continue;
		}*/

		/*if (col_states[col_idx] >= col_states[current_col_idx_ghetto]) {
			col_val_local_max = col_states[current_col_idx_ghetto];
			col_id_local_max = current_col_idx_ghetto;
			winner_pts += 1;
		} else {
			continue;
		}*/

		if (col_states[col_idx] >= col_states[current_col_idx_ghetto]) {
			winner_pts += 1;
		}

		/*if (current_col_idx_ghetto == col_idx) {
			winner_pts += 1;
			col_id_local_max = x;
		}*/


		/*if (col_states[current_col_idx_ghetto] > col_val_local_max) {
			col_val_local_max = col_states[current_col_idx_ghetto];
			col_id_local_max = current_col_idx_ghetto;
		} else if (col_states[current_col_idx_ghetto] == col_val_local_max) {
			if (current_col_idx_ghetto < col_idx) {
				col_val_local_max = col_states[current_col_idx_ghetto];
				col_id_local_max = current_col_idx_ghetto;
			}
		}*/
		x++;
	}


	//axn_states[axn_idx] = winner_pts;

	if (winner_close) {
		axn_states[axn_idx] = 50;
	}




	/*if (winner_pts == 7) {
		axn_states[axn_idx] = winner_pts;
	}*/

	//axn_states[axn_idx] = col_id_local_max;

	//axn_states[axn_idx] = current_col_idx_ghetto & 0xFF;

	/*if (col_id_local_max == col_idx) {
		//axn_states[axn_idx] = col_val_local_max;
		axn_states[axn_idx] = current_col_idx_ghetto >> 4;
	} else {
		//axn_states[axn_idx] = 0;
	}*/



	//axn_states[axn_idx] = col_val_local_max;

	//axn_states[axn_idx] = asp_vals[high_compare_asp_idx]; //col_id_local_max >> 4;

	


	/*if (cel_pos != col_winner_pos) {
		axn_states[axn_idx] = 0;
	} else {
		axn_states[axn_idx] = winner_vals[asp_idx];		//	asp_ids[asp_idx] + 64;
	}*/
}



	__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void axns_cycle_unoptd_incomplete_1_0 (
	__global uchar* asp_ids,
	__global char* asp_vals,
	__global char* col_states,
	__global char* axn_states,
	//__local uchar* wiv_local,
	__private uint const axn_row_offset
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = col_idx + mad24(axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (col_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;

	//uint col_winner_pos = asp_ids[asp_idx] & 0x07;
	//uint cel_pos = col_idx & 0x07;

	uint practical_aspiny_reach = (ASPINY_REACH - 1);
	uint low_compare_asp_idx = asp_idx - (practical_aspiny_reach);
	uint high_compare_asp_idx = asp_idx + (practical_aspiny_reach);

	char col_val_local_max = 0;
	uchar col_id_local_max = 0;

	uint winner_pts = 0;
	
	uint current_col_idx_ghetto = 0;

	char asp_winner_val = 0;

	uint x = 0;

		//#pragma unroll
	for (uint i = low_compare_asp_idx; i <= high_compare_asp_idx; i++) {

		current_col_idx_ghetto = ((i - ASPINY_REACH) << ASPINY_SPAN_LOG2) + asp_ids[i];

		//asp_winner_val = asp_vals[i];

		/*if (col_states[current_col_idx_ghetto] != asp_vals[i]) {
			axn_states[axn_idx] = -99;
			continue;
		}*/

		/*if (col_states[col_idx] >= col_states[current_col_idx_ghetto]) {
			col_val_local_max = col_states[current_col_idx_ghetto];
			col_id_local_max = current_col_idx_ghetto;
			winner_pts += 1;
		} else {
			continue;
		}*/

		if (col_states[col_idx] >= col_states[current_col_idx_ghetto]) {
			winner_pts += 1;
		}

		/*if (current_col_idx_ghetto == col_idx) {
			winner_pts += 1;
			col_id_local_max = x;
		}*/


		/*if (col_states[current_col_idx_ghetto] > col_val_local_max) {
			col_val_local_max = col_states[current_col_idx_ghetto];
			col_id_local_max = current_col_idx_ghetto;
		} else if (col_states[current_col_idx_ghetto] == col_val_local_max) {
			if (current_col_idx_ghetto < col_idx) {
				col_val_local_max = col_states[current_col_idx_ghetto];
				col_id_local_max = current_col_idx_ghetto;
			}
		}*/
		x++;
	}


	axn_states[axn_idx] = winner_pts;




	/*if (winner_pts == 7) {
		axn_states[axn_idx] = winner_pts;
	}*/

	//axn_states[axn_idx] = col_id_local_max;

	//axn_states[axn_idx] = current_col_idx_ghetto & 0xFF;

	/*if (col_id_local_max == col_idx) {
		//axn_states[axn_idx] = col_val_local_max;
		axn_states[axn_idx] = current_col_idx_ghetto >> 4;
	} else {
		//axn_states[axn_idx] = 0;
	}*/



	//axn_states[axn_idx] = col_val_local_max;

	//axn_states[axn_idx] = asp_vals[high_compare_asp_idx]; //col_id_local_max >> 4;

	


	/*if (cel_pos != col_winner_pos) {
		axn_states[axn_idx] = 0;
	} else {
		axn_states[axn_idx] = winner_vals[asp_idx];		//	asp_ids[asp_idx] + 64;
	}*/
}









/*	__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void axns_cycle_unoptd_working (
	__global uchar* asp_ids,
	__global char* col_states,
	__global char* axn_states,
	__local uchar* wiv_local,
	__private uint const axn_row_offset
	//__global char* 
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = cel_idx + mad24(axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (cel_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;
	//uint const axn_idx = mad24((row_id + axn_row_offset), row_width, col_id) + SYNAPSE_REACH;
	//uint const axn_idx = mad24(row_id, row_width, col_id);

	uint col_winner_pos = asp_ids[asp_idx] & 0x07; // GOOD
	uint cel_pos = cel_idx & 0x07;	// GOOD
	//uchar cel_pos = cel_idx & 0x07;


	//axn_states[axn_idx] = (asp_pos == cel_pos);
	if (cel_pos != col_winner_pos) {
		axn_states[axn_idx] = 0;
	} else {
		axn_states[axn_idx] = asp_ids[asp_idx] + 64;
		//axn_states[axn_idx] = clamp(asp_ids[asp_idx] << 2, 0, 127);
		//axn_states[axn_idx] = col_winner_pos;
		//axn_states[axn_idx] = asp_ids[asp_idx];
	}

	//axn_states[axn_idx] = (asp_ids[asp_idx] & 0x07);
	//axn_states[axn_idx] = asp_ids[asp_idx & 0xF8];
	//axn_states[axn_idx] = asp_ids[asp_idx];
	//axn_states[axn_idx] = get_local_size(1);
	//axn_states[axn_idx] = get_local_size(1) >> 2; 

}*/






__kernel void col_cycle_old(
	__global char* const syn_states,
	__global char* const col_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	//uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row_id, row_width, col_id);
	uint const syn_ofs = cel_idx << SYNAPSES_PER_CELL_PROXIMAL_LOG2;

	int syn_sum = 0;

	for (uint i = 0; i < SYNAPSES_PER_CELL_PROXIMAL; i += 1) {
		syn_sum += syn_states[syn_ofs + i];
	}

	col_states[cel_idx] = (char)(syn_sum >> SYNAPSES_PER_CELL_PROXIMAL_LOG2);
}






__kernel void soma_cycle_pre(
				__global char* const prx_den_states,
				__global char* const som_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row_id, row_width, col_id);
	uint const den_grp = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;

	short den_sum = 0;

		#pragma unroll 
	for (uint i = 0; i < DENDRITES_PER_CELL_DISTAL; i++) {
		den_sum += prx_den_states[den_grp + i];
	}

	den_sum >>= DENDRITES_PER_CELL_DISTAL_LOG2;

	som_states[cel_idx] = (char)den_sum;
}


	//	#WORK SIZE: Synapses per Region
__kernel void syns_cycle(
				__global char* const axn_states,
				__global uchar* const syn_axn_row_ids,
				__global char* const syn_axn_col_offs,
				__global char* const syn_strs,
				__global char* const syn_states
				//__private const uint axn_row_width
) {
	uint const row_id = get_global_id(0);		//	y (height)
	uint const col_id = get_global_id(1);		//	x (width)
	uint const syn_id = get_global_id(2);		//	z (depth)
	//uint const syn_row_width = get_global_size(2) * get_global_size(0);
	uint const width = get_global_size(1);
	uint const depth = get_global_size(2);
	//uint const col_pos = syn_pos >> SYNAPSES_PER_CELL_LOG2;



	/* [(row_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	uint const syns_idx = mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));
	uint const axns_idx = mad24(
		syn_axn_row_ids[syns_idx], 
		width, 
		syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH
	);
	/*
	uint const syns_idx = (row_id * depth * width) + (col_id * depth) + syn_id;
	uint const axns_idx = (syn_axn_row_ids[syns_idx] * width) + syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH;
	*/
	int syn_state = (int)syn_strs[syns_idx] * (int)axn_states[axns_idx];
	syn_states[syns_idx] = (char)clamp((int)(syn_state >> SYNAPSE_STRENGTH_DEFAULT_DISTAL_LOG2), (int)0, (int)127);
	//syn_states[syns_idx] =	(syn_strs[syns_idx] > 0);
}


__kernel void dens_cycle_new(
				__global char* const syn_states,
				__global char* const den_thrs,
				__global char* const den_states,
				__private uchar const boost_log2
) {
	uint const gid = get_global_id(0);
	uint const syn_grp = gid << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	short syn_sum = 0;

		#pragma unroll 
	for (uint i = 0; i < SYNAPSES_PER_DENDRITE_DISTAL; i++) {
		syn_sum += syn_states[syn_grp + i];
	}

	syn_sum = syn_sum << boost_log2;

	char den_val = (char)clamp((short)(syn_sum >> SYNAPSES_PER_DENDRITE_DISTAL_LOG2), (short)-128, (short)127);

	den_states[gid] = den_val; // * (den_val > den_thrs[gid] || den_val < 0);

	/*if (den_val > den_thrs[gid]) {
		den_states[gid] = den_val;			// DENDRITE_ACTIVE;
	}*/
}

__kernel void dens_cycle(
				__global char* const syn_states,
				__global char* const den_thrs,
				__global char* const den_states,
				__private uchar const boost_log2
) {
	uint const gid = get_global_id(0);
	uint const syn_grp = gid << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	short syn_sum = 0;

		#pragma unroll 
	for (uint i = 0; i < SYNAPSES_PER_DENDRITE_DISTAL; i++) {
		syn_sum += syn_states[syn_grp + i];
	}

	syn_sum = syn_sum << boost_log2;

	char den_val = (char)clamp((short)(syn_sum >> SYNAPSES_PER_DENDRITE_DISTAL_LOG2), (short)-128, (short)127);

	den_states[gid] = den_val; // * (den_val > den_thrs[gid] || den_val < 0);

	/*if (den_val > den_thrs[gid]) {
		den_states[gid] = den_val;			// DENDRITE_ACTIVE;
	}*/
}


__kernel void soma_cycle_post(
				__global char* const dst_den_states,
				//__global char* const prx_den_states,
				__global char* const som_states,
				__private uint const cell_row_offset		// Change this to __attribute__ or something
) {
	uint const row = get_global_id(0);
	uint const col = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row, row_width, col);
	//uint const som_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
	uint const den_grp = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;
	//int cel_grp = gid << CELLS_PER_COLUMN_LOG2;

	short den_sum = 0;

		#pragma unroll 
	for (uint i = 0; i < DENDRITES_PER_CELL_DISTAL; i++) {
		den_sum += dst_den_states[den_grp + i];
		//den_sum = (char)add_sat((char)den_sum, (char)dst_den_states[den_grp + i]);
	}

	den_sum = den_sum >> DENDRITES_PER_CELL_DISTAL_LOG2;

	//short prx_den_state = clamp((short)((short)prx_den_states[cel_idx] << SYNAPSES_PER_CELL_LOG2), (short)-128, (short)127);
	//som_states[som_idx] = (char)clamp((short)(den_sum + prx_den_state), (short)0, (short)127);

	som_states[cel_idx] = (char)clamp((short)(den_sum), (short)0, (short)127);
	//som_states[cel_idx] = (char)clamp((short)(den_sum + prx_den_states[cel_idx]), (short)0, (short)127);

	//barrier(CLK_LOCAL_MEM_FENCE);


	/*if (den_mix) {
		som_states[som_idx] |= CELL_PREDICTIVE;
	} else {
		som_states[som_idx] = 0;
	}
*/
}


__kernel void soma_inhib(
	__global char* const src_vals,
	__global uchar* const dst_ids,
	__global char* const dst_vals
) {
	uint const row = get_global_id(0);
	uint const col = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const hcol_idx = mad24(row, row_width, col);
	//uint const wg_width = get_local_size(1);

	uchar const hcol_size_log2 = COLUMNS_PER_HYPERCOLUMN_LOG2;
	//uchar const sub_grp_size_log2 = hcol_size_log2 >> 1;

	uint const src_vals_ofs = hcol_idx << hcol_size_log2;
	
	char hcol_max_val = 0;
	char hcol_max_pos = 0;
	//char sub_grp_max_val = 0;
	//char sub_grp_max_pos = 0;
	
	short pos = 0;
	char val = 0;

		#pragma unroll 
	for (uint i = 0; i < 1 << hcol_size_log2; i++) {
		val = src_vals[src_vals_ofs + pos];

		if (val > hcol_max_val) {
			hcol_max_val = val;
			hcol_max_pos = pos;
		}

		pos += 1;
	}
	dst_vals[hcol_idx] = hcol_max_val;
	dst_ids[hcol_idx] = hcol_max_pos;
	//dst_ids[hcol_idx] = pos;
}



__kernel void axns_cycle_old(
				__global char* const som_states,
				//__global char* const hcol_max_vals,
				__global uchar* const hcol_max_ids,
				__global char* const axn_states,
				__private uint const cell_row_offset		// Change this to __attribute__ or something
) {
	uint const row = get_global_id(0);
	uint const col = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row, row_width, col);
	uint const axn_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
	uint const hcol_idx = cel_idx >> COLUMNS_PER_HYPERCOLUMN_LOG2;
	uint const hcol_max_idx = (hcol_idx << COLUMNS_PER_HYPERCOLUMN_LOG2) + hcol_max_ids[hcol_idx];

	char axn_state = 0;

	if (cel_idx == hcol_max_idx) {
		axn_state = som_states[cel_idx];
	} 

	axn_states[axn_idx] = axn_state;

}


__kernel void syns_learn(
				__global uchar* const hcol_max_ids,
				__global char* const hcol_max_vals,
				__global char* const syn_states,
				__global char* const den_states,
				__global char* const syn_strengths,
				__global char* const rand_ofs

) {
	uint const row = get_global_id(0);
	uint const col = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const hcol_idx = mad24(row, row_width, col);

	uchar const hcol_size_log2 = COLUMNS_PER_HYPERCOLUMN_LOG2;
	uint const cel_ofs = hcol_idx << hcol_size_log2;
	uchar const hcol_max_id = hcol_max_ids[hcol_idx];
	uint const cel_idx = cel_ofs + hcol_max_id;

	uint const den_ofs = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;
	//uint const syn_ofs = den_ofs << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	uint pseudo_rand = (col + row + (uint)hcol_max_ids) & 0x00FF;

	uint rand1 = (uint)rand_ofs[pseudo_rand];
	uint rand2 = (uint)rand_ofs[rand1];

	uint rand_den_idx = den_ofs + (rand1 & 0x000F);
	uint rand_syn_idx = (rand_den_idx << SYNAPSES_PER_DENDRITE_DISTAL_LOG2) + (rand2 & 0x000F);


	syn_strengths[rand_syn_idx] += 
		(syn_states[rand_syn_idx] > den_states[rand_den_idx]) * 
		(den_states[rand_den_idx] > hcol_max_vals[hcol_idx]) *
		(syn_strengths[rand_syn_idx] < SYNAPSE_STRENGTH_MAX)
	;

}

__kernel void syns_decay(
	__global char* const syn_strs
) {

	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const syn_id = get_global_id(2);
	//uint const syn_row_width = get_global_size(2) * get_global_size(0);
	uint const width = get_global_size(1);
	uint const depth = get_global_size(2);
	//uint const col_pos = syn_pos >> SYNAPSES_PER_CELL_LOG2;

	/* [(row_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	uint const syn_idx = mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));

	syn_strs[syn_idx] -= (1 * (syn_strs[syn_idx] > -128)); 
}


__kernel void syns_regrow(
	__global char* const syn_strs,
	__global char* const rand_ofs,
	__global char* const syn_ofs
) {

	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const syn_id = get_global_id(2);
	//uint const syn_row_width = get_global_size(2) * get_global_size(0);
	uint const width = get_global_size(1);
	uint const depth = get_global_size(2);
	//uint const col_pos = syn_pos >> SYNAPSES_PER_CELL_LOG2;

	/* [(row_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	uint const syn_idx = mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));

	if (syn_strs[syn_id] <= -127) {
		syn_ofs[syn_idx] = rand_ofs[syn_id]; 
		syn_strs[syn_idx] = SYNAPSE_STRENGTH_DEFAULT_DISTAL; 
	}
}

























__kernel void test_cell_axn_stable(__global uchar *axn, __global uchar *ax_out) {
	int i = get_global_id(0);
	uchar ax = axn[i] + 2;
	axn[i] = mul_hi(ax, (uchar)128) * 2;
	ax_out[i] = axn[i];
}




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

#define SYNAPSE_REACH						128
//static uint const SYNAPSE_SPAN = 			SYNAPSE_REACH * 2;

#define SYNAPSE_WORKGROUP_SIZE				256
#define AXONS_WORKGROUP_SIZE 				256

//#define ASPINY_REACH_LOG2					2
/*#define ASPINY_REACH 						4
#define ASPINY_SPAN_LOG2 					3
#define ASPINY_SPAN 						8*/
static int const ASPINY_REACH =				1 << ASPINY_REACH_LOG2;
static int const ASPINY_SPAN_LOG2 =			ASPINY_REACH_LOG2 + 1;
static int const ASPINY_SPAN =				1 << (ASPINY_REACH_LOG2 + 1);

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
	//return (asp_idx - ASPINY_REACH) << ASPINY_SPAN_LOG2;
	return (asp_idx - ASPINY_REACH) << ASPINY_SPAN_LOG2;
	//current_col_idx_ghetto = ((i - ASPINY_REACH) << ASPINY_SPAN_LOG2) + asp_col_ids[i];
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

	WATCH OUT FOR:
		- Bank conflicts once src_ofs start to change
*/
//	__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void syns_cycle(
	__global uchar* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__private uint const syns_per_cell_l2,
	__global int* const aux_ints_0,
	__global int* const aux_ints_1,
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

	uint q = 0;

	for (uint syn_idx = init_syn_idx; syn_idx < syn_n; syn_idx += wg_size) {
		uint cel_idx = syn_idx >> syns_per_cell_l2;
		//uint cel_idx_dup = init_cel_idx + i;
		uint axn_idx = (mad24((uint)syn_src_row_ids[syn_idx], row_width, (uint)(cel_idx + syn_src_ofs[syn_idx]))) + SYNAPSE_REACH;

		syn_states[syn_idx] = axn_states[axn_idx];
		//syn_states[syn_idx] = base_cel_idx >> 2;
		//syn_states[syn_idx] = (axn_idx - 3200) >> 2;

		//syn_idx += wg_size;
		q++;
		if (q < 65536) {
			continue;
		} else {
			aux_ints_0[cel_idx] = 9999;
			syn_states[syn_idx] = 99;
			break;
		}
	}
}



__kernel void col_cycle(
	__global uchar* const syn_states,
	__global uchar* const col_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	//uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const syn4_per_cel_l2 = SYNAPSES_PER_CELL_PROXIMAL_LOG2 - 2;
	uint const syn_ofs = col_idx << syn4_per_cel_l2;

	uchar4 syn_state = (uchar4)(0, 0, 0, 0);
	int4 syn_sum = (int4)(0, 0, 0, 0);
	uint n = 1 << syn4_per_cel_l2;

	for (uint i = 0; i < n; i += 1) {
		syn_state = vload4((syn_ofs + i), syn_states);
		syn_sum += convert_int4(syn_state);
		//syn_sum += syn_state.s0;
	}

	int col_total = syn_sum.s0 + syn_sum.s1 + syn_sum.s2 + syn_sum.s3;

	col_states[col_idx] = (uchar)(col_total >> SYNAPSES_PER_CELL_PROXIMAL_LOG2);
	//col_states[col_idx] = col_total; //(0, 1, 2, 3); 
}


__kernel void aspiny_cycle(
	__global uchar* const col_states,
	__global uchar* const asp_col_ids,
	__global uchar* const asp_states
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



// RENAME ME
	//__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void axns_cycle_unoptd (										
	__global uchar* const asp_col_ids,
	__global uchar* const asp_states,
	__global uchar* const col_states,
	__global uchar* const axn_states,
	__global int* const aux_ints_0,
	__global int* const aux_ints_1,
	__private uint const axn_row_offset
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = col_idx + mad24(axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (col_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;

	uchar col_state = col_states[col_idx];

	int win_count = (asp_col_id_to_col_idx(asp_idx, (asp_col_ids[asp_idx])) == col_idx);
	//int tie_count = 0;

	//int aux_int_1_ofs = (col_idx * 100);

	if (win_count) {		

		for (uint i = 0; i < ASPINY_SPAN; i++) {

			uint cur_asp_idx = (asp_idx - ASPINY_REACH) + i + (i > (ASPINY_REACH - 1));
			uchar cur_comp_state = asp_states[cur_asp_idx];

			uint cur_comp_dist = abs((int)asp_idx - (int)cur_asp_idx);
			int inhib_power = (ASPINY_SPAN + 1) - (cur_comp_dist);

			if (col_state < cur_comp_state) {
				continue;

			} else if (col_state == cur_comp_state) {
				//win_count += inhib_power;
				//tie_count += inhib_power;

				//if (((asp_idx & 0x07) == 4)) {
				//if (((asp_idx & 0x01) ^ (cur_asp_idx & 0x01))) {
				if (((asp_idx & 0x07) == (col_state & 0x07))) {
					win_count += inhib_power;
					//win_count += mul24((inhib_power << 1), ((asp_idx) < (cur_asp_idx)));
					//win_count += 1;

				} else {
					//win_count += inhib_power;
					//win_count += mul24((inhib_power << 1), ((asp_idx) > (cur_asp_idx)));
					//win_count += 1;
				}

			} else {
				win_count += inhib_power;
				//win_count += 1;
			}
		}
	}

	//col_states[col_idx] = mul24(col_state, (win_count >= COLUMN_DOMINANCE_FLOOR));
	axn_states[axn_idx] = mul24(col_state, (win_count >= COLUMN_DOMINANCE_FLOOR));
	//aux_ints_0[col_idx] = mul24(win_count, (win_count >= COLUMN_DOMINANCE_FLOOR));
	//aux_ints_1[col_idx] = mul24((int)(asp_idx & 0x0F), (win_count >= COLUMN_DOMINANCE_FLOOR));
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





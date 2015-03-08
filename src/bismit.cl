#define DENDRITES_PER_CELL_DISTAL_LOG2			4
#define DENDRITES_PER_CELL_DISTAL				1 << DENDRITES_PER_CELL_DISTAL_LOG2

#define SYNAPSES_PER_DENDRITE_DISTAL_LOG2		4
#define SYNAPSES_PER_DENDRITE_DISTAL			1 << SYNAPSES_PER_DENDRITE_DISTAL_LOG2

#define SYNAPSES_PER_CELL_DISTAL_LOG2			DENDRITES_PER_CELL_DISTAL_LOG2 + SYNAPSES_PER_DENDRITE_DISTAL_LOG2
#define SYNAPSES_PER_CELL_DISTAL				1 << SYNAPSES_PER_CELL_DISTAL_LOG2

#define SYNAPSE_STRENGTH_DEFAULT_DISTAL_LOG2	4
#define SYNAPSE_STRENGTH_DEFAULT_DISTAL			1 << SYNAPSE_STRENGTH_DEFAULT_DISTAL_LOG2


#define DENDRITES_PER_CELL_PROXIMAL_LOG2		0
#define DENDRITES_PER_CELL_PROXIMAL				1 << DENDRITES_PER_CELL_PROXIMAL_LOG2

#define SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2		8
#define SYNAPSES_PER_DENDRITE_PROXIMAL			1 << SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2

#define SYNAPSES_PER_CELL_PROXIMAL_LOG2			DENDRITES_PER_CELL_PROXIMAL_LOG2 + SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2
#define SYNAPSES_PER_CELL_PROXIMAL				DENDRITES_PER_CELL_PROXIMAL * SYNAPSES_PER_DENDRITE_PROXIMAL

#define SYNAPSE_STRENGTH_DEFAULT_LOG2_PROXIMAL	4
#define SYNAPSE_STRENGTH_DEFAULT_PROXIMAL		1 << SYNAPSE_STRENGTH_DEFAULT_PROXIMAL_LOG2


#define SYNAPSE_STRENGTH_MAX			127

#define COLUMNS_PER_HYPERCOLUMN_LOG2	6

//#define COLUMNS_PER_SEGMENT 			64 * 16

//#define SYNAPSES_PER_LAYER				SYNAPSES_PER_CELL * COLUMNS_PER_SEGMENT

//#define CELLS_PER_COLUMN				16
//#define CELLS_PER_COLUMN_LOG2			4

#define SYNAPSE_REACH					128
#define SYNAPSE_SPAN					SYNAPSE_REACH * 2

#define SYNAPSE_WORKGROUP_SIZE			256

/*
#define DENDRITE_ACTIVE					0x01
#define COLUMN_ACTIVE_INPUT				0x10
#define SOMA_ACTIVE_OUTPUT				0x01
#define CELL_PREDICTIVE					0x01
#define CELL_ACTIVE						0x10
*/


/*
 Kernel Preferred work group size multiple:	 	64
 Max compute units:				 				32
 Max work items dimensions:						3
 Max work group size:				 				256

 Remember to inline functions
*/

/*
	COL_SYNS_CYCLE():
		number of source rows can not exceed: 
			ROWS * (SYNAPSES_PER_CELL_PROXIMAL + SYNAPSE_WORKGROUP_SIZE)

	TODO:
		- Vectorize!
*/
	__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)));
__kernel void col_syns_cycle(
	__global char* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__global char* const syn_states
) {
	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const lid = get_local_id(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row_id, row_width, col_id);
	size_t const syns_per_cell_log2 = SYNAPSES_PER_CELL_PROXIMAL_LOG2;	// Have to do this because of glitches with #define
	
	//size_t axn_ofs = col_id;
	size_t syn_idx = ((cel_idx - lid) << syns_per_cell_log2) + lid;

	size_t end = SYNAPSE_WORKGROUP_SIZE + col_id;
	size_t axn_idx;

	for (size_t i = col_id; i < end; i++) {
		axn_idx = mad24((size_t)syn_src_row_ids[syn_idx], row_width, (size_t)(i + syn_src_ofs[syn_idx]));
		syn_states[syn_idx] = axn_states[axn_idx];
		syn_idx += SYNAPSE_WORKGROUP_SIZE;
	}
}

__kernel void col_cycle(
	__global char* const syn_states,
	__global char* const col_states
) {
	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const l_id = get_local_id(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row_id, row_width, col_id);
	size_t const syns_per_cell_log2 = SYNAPSES_PER_CELL_PROXIMAL_LOG2;
	size_t const syn_ofs = cel_idx << SYNAPSES_PER_CELL_PROXIMAL_LOG2;

	int syn_sum = 0;

	for (size_t i = 0; i < SYNAPSES_PER_CELL_PROXIMAL; i += 1) {
		syn_sum += syn_states[syn_ofs + i];
	}

	col_states[cel_idx] = (char)(syn_sum >> SYNAPSES_PER_CELL_PROXIMAL_LOG2);
}
































__kernel void soma_cycle_pre(
				__global char* const prx_den_states,
				__global char* const som_states
) {
	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row_id, row_width, col_id);
	size_t const den_grp = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;

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
	size_t const row_id = get_global_id(0);		//	y (height)
	size_t const col_id = get_global_id(1);		//	x (width)
	size_t const syn_id = get_global_id(2);		//	z (depth)
	//size_t const syn_row_width = get_global_size(2) * get_global_size(0);
	size_t const width = get_global_size(1);
	size_t const depth = get_global_size(2);
	//size_t const col_pos = syn_pos >> SYNAPSES_PER_CELL_LOG2;



	/* [(row_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	size_t const syns_idx = mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));
	size_t const axns_idx = mad24(
		syn_axn_row_ids[syns_idx], 
		width, 
		syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH
	);
	/*
	size_t const syns_idx = (row_id * depth * width) + (col_id * depth) + syn_id;
	size_t const axns_idx = (syn_axn_row_ids[syns_idx] * width) + syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH;
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
	size_t const gid = get_global_id(0);
	size_t const syn_grp = gid << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

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
	size_t const gid = get_global_id(0);
	size_t const syn_grp = gid << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

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
	size_t const row = get_global_id(0);
	size_t const col = get_global_id(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row, row_width, col);
	//size_t const som_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
	size_t const den_grp = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;
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
	size_t const row = get_global_id(0);
	size_t const col = get_global_id(1);
	size_t const row_width = get_global_size(1);
	size_t const hcol_idx = mad24(row, row_width, col);
	//size_t const wg_width = get_local_size(1);

	uchar const hcol_size_log2 = COLUMNS_PER_HYPERCOLUMN_LOG2;
	//uchar const sub_grp_size_log2 = hcol_size_log2 >> 1;

	size_t const src_vals_ofs = hcol_idx << hcol_size_log2;
	
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


/*#pragma unroll 
	for (uint i = 0; i < 1 << sub_grp_size_log2; i++) {
		sub_grp_max_val = 0;
		sub_grp_max_pos = 0;

		#pragma unroll 
		for (uint j = 0; j < 1 << sub_grp_size_log2; j++) {
			val = src_vals[src_vals_ofs + pos];

			if (val > sub_grp_max_val) {
				sub_grp_max_val = val;
				sub_grp_max_pos = pos;
			}
			pos += 1;
		}

		if (sub_grp_max_val > hcol_max_val) {
			hcol_max_val = sub_grp_max_val;
			hcol_max_pos = pos - 1;
		}

	}*/


__kernel void axns_cycle(
				__global char* const som_states,
				//__global char* const hcol_max_vals,
				__global uchar* const hcol_max_ids,
				__global char* const axn_states,
				__private uint const cell_row_offset		// Change this to __attribute__ or something
) {
	size_t const row = get_global_id(0);
	size_t const col = get_global_id(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row, row_width, col);
	size_t const axn_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
	size_t const hcol_idx = cel_idx >> COLUMNS_PER_HYPERCOLUMN_LOG2;
	size_t const hcol_max_idx = (hcol_idx << COLUMNS_PER_HYPERCOLUMN_LOG2) + hcol_max_ids[hcol_idx];

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
	size_t const row = get_global_id(0);
	size_t const col = get_global_id(1);
	size_t const row_width = get_global_size(1);
	size_t const hcol_idx = mad24(row, row_width, col);

	uchar const hcol_size_log2 = COLUMNS_PER_HYPERCOLUMN_LOG2;
	size_t const cel_ofs = hcol_idx << hcol_size_log2;
	uchar const hcol_max_id = hcol_max_ids[hcol_idx];
	size_t const cel_idx = cel_ofs + hcol_max_id;

	size_t const den_ofs = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;
	//size_t const syn_ofs = den_ofs << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	size_t pseudo_rand = (col + row + (size_t)hcol_max_ids) & 0x00FF;

	size_t rand1 = (size_t)rand_ofs[pseudo_rand];
	size_t rand2 = (size_t)rand_ofs[rand1];

	size_t rand_den_idx = den_ofs + (rand1 & 0x000F);
	size_t rand_syn_idx = (rand_den_idx << SYNAPSES_PER_DENDRITE_DISTAL_LOG2) + (rand2 & 0x000F);


	syn_strengths[rand_syn_idx] += 
		(syn_states[rand_syn_idx] > den_states[rand_den_idx]) * 
		(den_states[rand_den_idx] > hcol_max_vals[hcol_idx]) *
		(syn_strengths[rand_syn_idx] < SYNAPSE_STRENGTH_MAX)
	;

	//char syn_strength;
	//char syn_state;

	/*
		LET'S AVERAGE THE SYNAPSES FOR ALL THE DENDRITES AND BOOST THE TOP 10 - 40%
		BETTER YET: FIND THE BEST DENDRITES AND BOOST THE TOP SYNAPSES JUST FOR THEM
		BETTER YET: COME UP WITH MULTIPLE STRATEGIES FOR BOOSTING STRENGTHS AND TRY THEM ALL OUT
		DON'T BOOST MORE THAN PROBABLY 3 - 6 SYNAPSES
	*/

	/*
	short den_states_sum = 5;
	char den_states_avg = 0;

	for (uchar d = 0; d < DENDRITES_PER_CELL; d++) {
		den_states_sum += den_states[den_ofs + d];
	}

	den_states_avg = (char)(den_states_sum >> DENDRITES_PER_CELL_LOG2);
	*/

	//syn_strengths[syn_ofs + 0] = den_states_avg + 100; 


	/*for (uchar s = 0; s < SYNAPSES_PER_DENDRITE; s++) {
		//syn_strengths[syn_idx] += (syn_states[syn_idx] > den_states[den_idx]) * (syn_strengths[syn_idx] < 63);
		//syn_strengths[syn_idx] += (syn_states[syn_idx] > den_states[den_idx]) * (syn_strengths[syn_idx] < 64);
		//syn_strengths[syn_idx] += 1;
		
		
		syn_strength = syn_strengths[syn_idx];
		syn state = syn_states[syn_idx];
		if ((syn_strength > den_states[den_idx]) && (syn_strength < 63)) {
			syn_strengths[syn_idx] += 1;
		}
		

		syn_idx++;
	}*/


	/*for (uchar d = 0; d < DENDRITES_PER_CELL; d++) {
		for (uchar s = 0; s < SYNAPSES_PER_DENDRITE; s++) {
			syn_strengths[syn_idx] += (syn_states[syn_idx] > den_states[den_idx]) * (syn_strengths[syn_idx] < 64);
			//syn_strengths[syn_idx] += 1;
			syn_idx++;
		}
		den_idx ++;
	}*/
}

__kernel void syns_decay(
	__global char* const syn_strs
) {

	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const syn_id = get_global_id(2);
	//size_t const syn_row_width = get_global_size(2) * get_global_size(0);
	size_t const width = get_global_size(1);
	size_t const depth = get_global_size(2);
	//size_t const col_pos = syn_pos >> SYNAPSES_PER_CELL_LOG2;

	/* [(row_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	size_t const syn_idx = mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));

	syn_strs[syn_idx] -= (1 * (syn_strs[syn_idx] > -128)); 
}


__kernel void syns_regrow(
	__global char* const syn_strs,
	__global char* const rand_ofs,
	__global char* const syn_ofs
) {

	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const syn_id = get_global_id(2);
	//size_t const syn_row_width = get_global_size(2) * get_global_size(0);
	size_t const width = get_global_size(1);
	size_t const depth = get_global_size(2);
	//size_t const col_pos = syn_pos >> SYNAPSES_PER_CELL_LOG2;

	/* [(row_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	size_t const syn_idx = mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));

	if (syn_strs[syn_id] <= -127) {
		syn_ofs[syn_idx] = rand_ofs[syn_id]; 
		syn_strs[syn_idx] = SYNAPSE_STRENGTH_DEFAULT_DISTAL; 
	}
}























/*Bullshit Below




	__kernel void read_char_array(__global uchar *values, __global uchar *output) {
		int gid = get_global_id(0);
		output[gid] = values[gid];
	}

	__kernel void read_uint_array(__global uint *values, __global uint *output) {
		int gid = get_global_id(0);
		output[gid] = values[gid];
	}

	__kernel void get_target_cols(__global ushort *target_cols) {
		int gid = get_global_id(0);
		output[gid] = values[gid];
	}



__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void col_syns_cycle_2_0(
	__global char* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__global char* const syn_states,
	__private uchar const src_axn_row
	//__local char* const axn_cache
) {

	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const lid = get_local_id(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row_id, row_width, col_id);
	//size_t const axn_zero = lid + SYNAPSE_REACH;
	size_t const depth_log2 = SYNAPSES_PER_CELL_PROXIMAL_LOG2;
	
	//__local char axn_cache[SYNAPSE_WORKGROUP_SIZE + SYNAPSE_SPAN]; // ADD HEIGHT AS A CONSTANT AT SOME POINT
	//__local size_t axn_cache_width;
	//__local size_t axn_cache_height;

	//size_t axn_ofs = lid << 1;
	size_t axn_idx = mad24(src_axn_row, row_width, add_sat(col_id, (size_t)SYNAPSE_REACH));
	//axn_cache[axn_ofs - 1] = axn_states[axn_idx + axn_ofs - 1];
	//axn_cache[axn_ofs] = axn_states[axn_idx + axn_ofs];


	if (lid == 0) {
		size_t const wg_size = get_local_size(1);
		axn_cache_width = add_sat((size_t)SYNAPSE_SPAN, wg_size);
		axn_cache_height = 1; // *** FIX (should be based on size of src_axn_rows or whatever it becomes) ***
		size_t const axn_ofs = mad24(src_axn_row, row_width, col_id + SYNAPSE_REACH);
		size_t axn_idx = axn_ofs;

		#pragma unroll
		for (size_t i = 0; i < axn_cache_width; i++) {
			axn_cache[i] = axn_states[axn_idx];
			axn_idx += 1;
		}
	}

	//barrier(CLK_LOCAL_MEM_FENCE);

	//size_t syn_idx = cel_idx << SYNAPSES_PER_CELL_PROXIMAL_LOG2;
	//size_t axn_cache_idx;

	//size_t spc =  << SYNAPSE_WORKGROUP_SIZE;

		// START AT THE FIRST CELL OF THE WORKGROUP
		// INCREMENT SYN_IDX BY ONE WHOLE WORKGROUP (1 workgroup = 1 cell) AT A TIME
	size_t syn_idx = ((cel_idx - lid) << depth_log2) + lid;
	int end = SYNAPSE_WORKGROUP_SIZE + lid;

	//#pragma unroll
	for (int i = 0; i < SYNAPSE_WORKGROUP_SIZE; i += 1) {
		//axn_cache_idx = axn_zero + syn_src_ofs[syn_idx];

		//syn_states[syn_idx] = axn_cache[axn_cache_idx];
		syn_states[syn_idx] = axn_states[axn_idx + syn_src_ofs[syn_idx]];
		syn_idx += SYNAPSE_WORKGROUP_SIZE;
		axn_idx += 1;
	}
}



__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void col_syns_cycle_1_0(
	__global char* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__global char* const syn_states,
	__private uchar const src_axn_row
	//__local char* const axn_cache
) {

	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const lid = get_local_id(1);
	size_t const wg_size = get_local_size(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row_id, row_width, col_id);
	size_t const axn_zero = lid + SYNAPSE_REACH;
	size_t const depth_log2 = SYNAPSES_PER_CELL_PROXIMAL_LOG2;
	
	__local char axn_cache[SYNAPSE_WORKGROUP_SIZE + SYNAPSE_SPAN]; // ADD HEIGHT AS A CONSTANT AT SOME POINT
	__local size_t axn_cache_width;
	__local size_t axn_cache_height;

	if (lid == 0) {
		axn_cache_width = add_sat((size_t)SYNAPSE_SPAN, wg_size);
		axn_cache_height = 1; // *** FIX (should be based on size of src_axn_rows or whatever it becomes) ***
		size_t const axn_ofs = mad24(src_axn_row, row_width, col_id + SYNAPSE_REACH);
		size_t axn_idx = axn_ofs;

		#pragma unroll
		for (size_t i = 0; i < axn_cache_width; i++) {
			axn_cache[i] = axn_states[axn_idx];
			axn_idx += 1;
		}
	}

	barrier(CLK_LOCAL_MEM_FENCE);

	size_t syn_idx = cel_idx << depth_log2;
	size_t axn_idx;

	#pragma unroll
	for (int i = 0; i < SYNAPSES_PER_CELL_PROXIMAL; i++) {
		axn_idx = axn_zero + syn_src_ofs[syn_idx];

		syn_states[syn_idx] = axn_cache[axn_idx];
		syn_idx += 1;
	}
}

__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void col_syns_cycle_crashes(
	__global char* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__global char* const syn_states
) {
	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const l_id = get_local_id(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row_id, row_width, col_id);
	
	size_t axn_ofs = col_id + (size_t)SYNAPSE_REACH;
	size_t syn_idx = ((cel_idx - l_id) << (size_t)SYNAPSES_PER_CELL_PROXIMAL_LOG2) + l_id;

	size_t n = (size_t)SYNAPSES_PER_CELL_PROXIMAL + axn_ofs;
	size_t axn_idx;

	for (size_t i = axn_ofs; i < n; i += 1) {
		axn_idx = mad24((size_t)syn_src_row_ids[syn_idx], row_width, (size_t)(i + syn_src_ofs[syn_idx]));
		syn_states[syn_idx] = axn_states[axn_idx];
		syn_idx += SYNAPSE_WORKGROUP_SIZE;
	}
}



/* MUL_HI TEST STABLE

*/

__kernel void test_cell_axn_stable(__global uchar *axn, __global uchar *ax_out) {
	int i = get_global_id(0);
	uchar ax = axn[i] + 2;
	axn[i] = mul_hi(ax, (uchar)128) * 2;
	ax_out[i] = axn[i];
}

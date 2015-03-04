#define DENDRITES_PER_NEURON_LOG2		4
#define DENDRITES_PER_NEURON			1 << DENDRITES_PER_NEURON_LOG2

#define SYNAPSES_PER_DENDRITE_LOG2		4
#define SYNAPSES_PER_DENDRITE			1 << SYNAPSES_PER_DENDRITE_LOG2

#define SYNAPSES_PER_NEURON_LOG2		8
#define SYNAPSES_PER_NEURON				1 << SYNAPSES_PER_NEURON_LOG2 // SYNAPSES_PER_DENDRITE * DENDRITES_PER_NEURON

#define SYNAPSE_STRENGTH_DEFAULT		16
#define SYNAPSE_STRENGTH_DEFAULT_LOG2	4

#define SYNAPSE_STRENGTH_MAX			32

#define COLUMNS_PER_HYPERCOLUMN_LOG2	6

#define COLUMNS_PER_SEGMENT 			64 * 16

#define SYNAPSES_PER_LAYER				SYNAPSES_PER_NEURON * COLUMNS_PER_SEGMENT

#define CELLS_PER_COLUMN				16
#define CELLS_PER_COLUMN_LOG2			4

#define SYNAPSE_REACH					128
#define MAX_SYNAPSE_RANGE				SYNAPSE_REACH * 2

/*
#define DENDRITE_ACTIVE					0x01
#define COLUMN_ACTIVE_INPUT				0x10
#define SOMA_ACTIVE_OUTPUT				0x01
#define CELL_PREDICTIVE					0x01
#define CELL_ACTIVE						0x10
*/

//#define WORKGROUP_SIZE					64

/*
** Kernel Preferred work group size multiple:	 	64
** Max compute units:				 				32
** Max work items dimensions:						3
** Max work group size:				 				256
**
** Remember to inline functions
*/

//__global char axn_states;

/* 
	SYNS_CYCLE(): INTEGRATION WITH OTHER 'CYCLE' FUNCTIONS - FUTURE OPTIMIZATION NOTES

	- preserve original function for comparison
	- load 256 cells (2^16 synapses) into each workgroup (256 synapses per work item)
	- load data from all 512 necessary axons into local (workgroup) memory
	- perform summation and weighing for cell dendrites and cell somata in a serially iterative manner
	- pre-calculate workgroup-sharable constants such as axon lyr offset using compiler flags
	

*/
	//	#WORK SIZE: Synapses per Region
__kernel void syns_cycle(
				__global char* const axn_states,
				__global uchar* const syn_axn_lyr_ids,
				__global char* const syn_axn_col_offs,
				__global char* const syn_strs,
				__global char* const syn_states
				//__private const uint axn_lyr_width
) {
	size_t const lyr_id = get_global_id(0);		//	y (height)
	size_t const col_id = get_global_id(1);		//	x (width)
	size_t const syn_id = get_global_id(2);		//	z (depth)
	//size_t const syn_lyr_width = get_global_size(2) * get_global_size(0);
	size_t const width = get_global_size(1);
	size_t const depth = get_global_size(2);
	//size_t const col_pos = syn_pos >> SYNAPSES_PER_NEURON_LOG2;



	/* [(lyr_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	size_t const syns_idx = mad24(mul24(lyr_id, depth), width, mad24(col_id, depth, syn_id));
	size_t const axns_idx = mad24(
		syn_axn_lyr_ids[syns_idx], 
		width, 
		syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH
	);
	/*
	size_t const syns_idx = (lyr_id * depth * width) + (col_id * depth) + syn_id;
	size_t const axns_idx = (syn_axn_lyr_ids[syns_idx] * width) + syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH;
	*/
	int syn_state = (int)syn_strs[syns_idx] * (int)axn_states[axns_idx];
	syn_states[syns_idx] = (char)clamp((int)(syn_state >> SYNAPSE_STRENGTH_DEFAULT_LOG2), (int)0, (int)127);
	//syn_states[syns_idx] =	(syn_strs[syns_idx] > 0);
}


__kernel void dens_cycle(
				__global char* const syn_states,
				__global char* const den_thrs,
				__global char* const den_states,
				__private uchar const boost_log2
) {
	size_t const gid = get_global_id(0);
	size_t const syn_grp = gid << SYNAPSES_PER_DENDRITE_LOG2;

	short syn_sum = 0;

	#pragma unroll 
	for (uint i = 0; i < SYNAPSES_PER_DENDRITE; i++) {
		syn_sum += syn_states[syn_grp + i];
	}

	syn_sum = syn_sum << boost_log2;

	char den_val = (char)clamp((short)(syn_sum >> SYNAPSES_PER_DENDRITE_LOG2), (short)-128, (short)127);

	den_states[gid] = den_val; // * (den_val > den_thrs[gid] || den_val < 0);

	/*if (den_val > den_thrs[gid]) {
		den_states[gid] = den_val;			// DENDRITE_ACTIVE;
	}*/
}

__kernel void soma_cycle(
				__global char* const dst_den_states,
				__global char* const prx_den_states,
				__global char* const som_states,
				__private uint const cell_lyr_offset		// Change this to __attribute__ or something
) {
	size_t const lyr = get_global_id(0);
	size_t const col = get_global_id(1);
	size_t const lyr_width = get_global_size(1);
	size_t const cel_idx = mad24(lyr, lyr_width, col);
	//size_t const som_idx = mad24((lyr + cell_lyr_offset), lyr_width, col) + SYNAPSE_REACH;
	size_t const den_grp = cel_idx << DENDRITES_PER_NEURON_LOG2;
	//int cel_grp = gid << CELLS_PER_COLUMN_LOG2;

	short den_sum = 0;

	#pragma unroll 
	for (uint i = 0; i < DENDRITES_PER_NEURON; i++) {
		den_sum += dst_den_states[den_grp + i];
		//den_sum = (char)add_sat((char)den_sum, (char)dst_den_states[den_grp + i]);
	}

	den_sum = den_sum >> DENDRITES_PER_NEURON_LOG2;

	//short prx_den_state = clamp((short)((short)prx_den_states[cel_idx] << SYNAPSES_PER_NEURON_LOG2), (short)-128, (short)127);
	//som_states[som_idx] = (char)clamp((short)(den_sum + prx_den_state), (short)0, (short)127);

	som_states[cel_idx] = (char)clamp((short)(den_sum + prx_den_states[cel_idx]), (short)0, (short)127);
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
	size_t const lyr = get_global_id(0);
	size_t const col = get_global_id(1);
	size_t const lyr_width = get_global_size(1);
	size_t const hcol_idx = mad24(lyr, lyr_width, col);
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
				__private uint const cell_lyr_offset		// Change this to __attribute__ or something
) {
	size_t const lyr = get_global_id(0);
	size_t const col = get_global_id(1);
	size_t const lyr_width = get_global_size(1);
	size_t const cel_idx = mad24(lyr, lyr_width, col);
	size_t const axn_idx = mad24((lyr + cell_lyr_offset), lyr_width, col) + SYNAPSE_REACH;
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
	size_t const lyr = get_global_id(0);
	size_t const col = get_global_id(1);
	size_t const lyr_width = get_global_size(1);
	size_t const hcol_idx = mad24(lyr, lyr_width, col);

	uchar const hcol_size_log2 = COLUMNS_PER_HYPERCOLUMN_LOG2;
	size_t const cel_ofs = hcol_idx << hcol_size_log2;
	uchar const hcol_max_id = hcol_max_ids[hcol_idx];
	size_t const cel_idx = cel_ofs + hcol_max_id;

	size_t const den_ofs = cel_idx << DENDRITES_PER_NEURON_LOG2;
	size_t const syn_ofs = den_ofs << SYNAPSES_PER_DENDRITE_LOG2;

	size_t pseudo_rand = (col + lyr + (size_t)hcol_max_ids) & 0x00FF;

	size_t rand1 = (size_t)rand_ofs[pseudo_rand];
	size_t rand2 = (size_t)rand_ofs[rand1];

	size_t rand_den_idx = den_ofs + (rand1 & 0x000F);
	size_t rand_syn_idx = (rand_den_idx << SYNAPSES_PER_DENDRITE_LOG2) + (rand2 & 0x000F);


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

	for (uchar d = 0; d < DENDRITES_PER_NEURON; d++) {
		den_states_sum += den_states[den_ofs + d];
	}

	den_states_avg = (char)(den_states_sum >> DENDRITES_PER_NEURON_LOG2);
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


	/*for (uchar d = 0; d < DENDRITES_PER_NEURON; d++) {
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

	size_t const lyr_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const syn_id = get_global_id(2);
	//size_t const syn_lyr_width = get_global_size(2) * get_global_size(0);
	size_t const width = get_global_size(1);
	size_t const depth = get_global_size(2);
	//size_t const col_pos = syn_pos >> SYNAPSES_PER_NEURON_LOG2;

	/* [(lyr_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	size_t const syn_idx = mad24(mul24(lyr_id, depth), width, mad24(col_id, depth, syn_id));

	syn_strs[syn_idx] -= (1 * (syn_strs[syn_idx] > -128)); 
}


__kernel void syns_regrow(
	__global char* const syn_strs,
	__global char* const rand_ofs,
	__global char* const syn_ofs
) {

	size_t const lyr_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const syn_id = get_global_id(2);
	//size_t const syn_lyr_width = get_global_size(2) * get_global_size(0);
	size_t const width = get_global_size(1);
	size_t const depth = get_global_size(2);
	//size_t const col_pos = syn_pos >> SYNAPSES_PER_NEURON_LOG2;

	/* [(lyr_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	size_t const syn_idx = mad24(mul24(lyr_id, depth), width, mad24(col_id, depth, syn_id));

	if (syn_strs[syn_id] <= -127) {
		syn_ofs[syn_idx] = rand_ofs[syn_id]; 
		syn_strs[syn_idx] = SYNAPSE_STRENGTH_DEFAULT; 
	}
}























/* Bullshit Below




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
*/




__kernel void inhib_3_0(
	__global char* const src_vals,
	__global char* const dst_vals
) {
	size_t lyr = get_global_id(0);
	size_t col = get_global_id(1);
	size_t lyr_width = get_global_size(1);
	size_t grp_idx = mad24(lyr, lyr_width, col);
	size_t wg_width = get_local_size(1);

	//__local char best_of_8[32]; // wg_size = 256; 256 / 8 = 32

	const uchar grp_size_log2 = 3;

	size_t src_grp = grp_idx << grp_size_log2;
	char grp_max = 0;

	#pragma unroll 
	for (uint i = 0; i < 1 << grp_size_log2; i++) {
		grp_max = max(src_vals[src_grp + i], grp_max);
		//dst_vals[src_grp + i] = 5;
	}

	dst_vals[grp_idx] = grp_max; //		grp_max;

}

__kernel void inhib_2_0(
	__global char* const som_states,
	__global char* const axn_states,
	__private const uint axn_out_ofs,
	__global int* const aux_vals
) {
	size_t lyr = get_global_id(0);
	size_t col = get_global_id(1);
	size_t lyr_width = get_global_size(1);
	size_t grp_idx = mad24(lyr, lyr_width, col);
	size_t wg_width = get_local_size(1);

	//__local char best_of_8[32]; // wg_size = 256; 256 / 8 = 32

	const uchar grp_size_log2 = 4;

	size_t som_grp = grp_idx << grp_size_log2;
	char grp_max = 0;

	#pragma unroll 
	for (uint i = 0; i < 1 << grp_size_log2; i++) {
		grp_max = max(som_states[som_grp + i], grp_max);
	}

	aux_vals[grp_idx] = grp_max;
	//aux_vals[(grp_idx << 2) + col] = 69;

}



__kernel void inhib_1_0(		// FUCK IT. LET'S DUPLICATE WORK FOR NOW. I'M DRUNK.
				__global char* const axn_states,
				__private const uint cell_lyr_offset,		// Change this to __attribute__ (or macro) or something
				__private const uint axn_inhib_tmp_ofs,
				__private const uint axn_inhib_tmp_2_ofs
) {
	size_t lyr = get_global_id(0);
	size_t col = get_global_id(1);
	size_t lyr_width = get_global_size(1);
	size_t cel_idx = mad24(lyr, lyr_width, col);
	size_t axn_idx = mad24((lyr + cell_lyr_offset), lyr_width, col) + SYNAPSE_REACH;
	//size_t den_grp = cel_idx << DENDRITES_PER_NEURON_LOG2;

	size_t group_size = 16;
	size_t axn_grp = (size_t)axn_idx & (size_t)(0xFFFFFFFF - group_size); // groups of 16;

	char axn_state_max = 1;
	size_t axn_idx_max = 2;

	#pragma unroll 
	for (uint i = 0; i < group_size; i++) {
		size_t idx = axn_grp + i;
		if (axn_states[idx] > axn_state_max) {
			axn_state_max = axn_states[idx];
			axn_idx_max = idx;
		}
	}

	/* NO CLUE WHAT I'M DOING -- START THIS OVER */
	axn_states[axn_inhib_tmp_ofs + cel_idx + SYNAPSE_REACH] = axn_state_max;
	axn_states[axn_inhib_tmp_2_ofs + cel_idx + SYNAPSE_REACH] = axn_idx_max;
}





__kernel void get_synapse_values(__global uchar *values, __global uchar *output) {
	int gid = get_global_id(0);
	output[gid] = values[gid];
}


__kernel void buffer_test(__global uint *synapse, __global uint *syn_out) {
	int i = get_global_id(0);
	synapse[i] += 1;
	syn_out[i] = synapse[i];
}

__kernel void test_synapse(__global uchar *synapse, __global uchar *syn_out) {
	int i = get_global_id(0);
	synapse[i] += 1;
	syn_out[i] = synapse[i];
}


__kernel void test_cell_axn(__global uchar *axn, __global uchar *ax_out) {
	int i = get_global_id(0);
	//uchar ax = axn[i] + 2;
	//axn[i] = mul_hi(ax, (uchar)128) * 2;
	ax_out[i] = axn[i];
}



__kernel void hello(__global float *input, __global float *output) {
	size_t id = get_global_id(0);
	output[id] = input[id] * input[id];
}

__kernel void test_int_shift(__global char *test_out, __private char input) {
	uint gid = get_global_id(0);
	test_out[gid] = input >> 4;
}


	// #WG: common::COLUMN_SYNAPSES_PER_SEGMENT
__kernel void sense(
				__global const char *src_vals,  // CHANGE TO _states
				__global char * const tar_vals,
				__global const short *tar_som_idxs,
				__global const char *tar_syn_idxs,
				__private const char dup_factor_shift
) {
	size_t gid = get_global_id(0);
	size_t tar_idx = mad24(tar_som_idxs[gid], SYNAPSES_PER_NEURON, tar_syn_idxs[gid]);

	tar_vals[tar_idx] = src_vals[gid >> dup_factor_shift];
	
}

	// #WG: common::COLUMN_DENDRITES_PER_SEGMENT
__kernel void cycle_col_dens(
				__global const uchar *syn_states,
				__global const uchar *syn_strs,
				__global const uchar *den_thrs,
				__global uchar * const den_states
) {
	size_t gid = get_global_id(0);
	size_t syn_grp = gid << SYNAPSES_PER_DENDRITE_LOG2;

	uchar den_val = 0;

	for (uint i = 0; i < SYNAPSES_PER_DENDRITE; i++) {
		den_val += mul_hi(syn_states[syn_grp + i], syn_strs[syn_grp + i]);
	}

	if (den_val > den_thrs[gid]) {
		den_states[gid] = den_val;	
	}
}


	// #WG: common::COLUMNS_PER_SEGMENT
/*__kernel void cycle_col_soms(
				__global const uchar *den_states,
				__global uchar * const som_vals,
				__global uchar *cel_states
) {
	size_t gid = get_global_id(0);
	size_t den_grp = gid << DENDRITES_PER_NEURON_LOG2;
	size_t cel_grp = gid << CELLS_PER_COLUMN_LOG2;

	uchar den_mix = 0;

	for (uint i = 0; i < DENDRITES_PER_NEURON; i++) {
		den_mix |= den_states[den_grp + i];
	}

	if (den_mix) {
		som_vals[gid] = COLUMN_ACTIVE_INPUT;

		for (uint i = 0; i < CELLS_PER_COLUMN; i++) {
			cel_states[cel_grp + i] |= CELL_ACTIVE;
		}
	}
}*/



/*
	// #WG common::CELL_SYNAPSES_PER_SEGMENT
__kernel void cycle_cel_syns_1(
				__global uchar *src_states,
				__global ushort *syn_src_idxs,
				__global uchar *syn_strs,
				__global uchar *syn_states,
				__private uint layer_group_offset,
				__private uint layers_per_layer_group
) {
	int gid = get_global_id(0);
	//int layer_id = gid << 

	syn_states[gid] = mul_hi(src_states[syn_src_idxs[gid]], syn_strs[gid]);
}

__kernel void cycle_cel_syns_2(
				__global uchar *src_states,
				__global ushort *syn_src_idxs,
				__global uchar *syn_strs,
				__global uchar *syn_states,
				__private uint layer_group_offset,
				__private uint layers_per_layer_group
) {
	int gid = get_global_id(0);
	int lgid = layer_group_offset + gid;

	// uchar myself = src_states[lgid];

	ushort src_idx = syn_src_idxs[lgid];
	uchar src_state = src_states[src_idx];

	syn_states[lgid] = mul_hi(src_state, syn_strs[lgid]);
}

__kernel void cycle_cel_syns_2_left(
				__global uchar *src_states,
				__global ushort *syn_src_idxs,
				__global uchar *syn_strs,
				__global uchar *syn_states,
				__private uint layer_group_offset,
				__private uint layers_per_layer_group
) {
	int gid = get_global_id(0);
	int lgid = layer_group_offset + gid;

	uchar myself = src_states[lgid];

	ushort src_idx = syn_src_idxs[lgid]
	uchar src_state = src_states[src_idx];

	syn_states[lgid] = mul_hi(src_state, syn_strs[lgid]);
}

__kernel void cycle_cel_syns_2_right(
				__global uchar *src_states,
				__global ushort *syn_src_idxs,
				__global uchar *syn_strs,
				__global uchar *syn_states,
				__private uint layer_group_offset,
				__private uint layers_per_layer_group
) {
	int gid = get_global_id(0);
	int lgid = layer_group_offset + gid;

	uchar myself = src_states[lgid];

	ushort src_idx = syn_src_idxs[lgid]
	uchar src_state = src_states[src_idx];

	syn_states[lgid] = mul_hi(src_state, syn_strs[lgid]);
}

*/


/* MUL_HI TEST STABLE
*
*
*
*
*
*/

__kernel void test_cell_axn_stable(__global uchar *axn, __global uchar *ax_out) {
	int i = get_global_id(0);
	uchar ax = axn[i] + 2;
	axn[i] = mul_hi(ax, (uchar)128) * 2;
	ax_out[i] = axn[i];
}

#define DENDRITES_PER_NEURON			16
#define DENDRITES_PER_NEURON_LOG2		4

#define SYNAPSES_PER_DENDRITE			16
#define SYNAPSES_PER_DENDRITE_LOG2		4

#define SYNAPSES_PER_NEURON				256 // SYNAPSES_PER_DENDRITE * DENDRITES_PER_NEURON
#define SYNAPSES_PER_NEURON_LOG2		8

#define COLUMNS_PER_SEGMENT 			64 * 16

#define SYNAPSES_PER_LAYER				SYNAPSES_PER_NEURON * COLUMNS_PER_SEGMENT

#define CELLS_PER_COLUMN				16
#define CELLS_PER_COLUMN_LOG2			4

#define SYNAPSE_REACH					128
#define MAX_SYNAPSE_RANGE				SYNAPSE_REACH * 2

#define DENDRITE_ACTIVE					0x01
#define COLUMN_ACTIVE_INPUT				0x10
#define SOMA_ACTIVE_OUTPUT				0x01
#define CELL_PREDICTIVE					0x01
#define CELL_ACTIVE						0x10

#define WORKGROUP_SIZE					64

#define COLUMNS_PER_HYPERCOLUMN_LOG2	6

/*
** Kernel Preferred work group size multiple:	 	64
** Max compute units:				 				32
** Max work items dimensions:						3
** Max work group size:				 				256
**
** Remember to inline functions
*/

//__global char axn_states;


	//	#WORK SIZE: Synapses per Region
__kernel void syns_cycle(
				__global const char* axn_states,
				__global const uchar* syn_axn_row_ids,
				__global const char* syn_axn_col_offs,
				__global const char* syn_strs,
				__global char* const syn_states
				//__private const uint axn_row_width
) {
	size_t row_id = get_global_id(0);		//	y (height)
	size_t col_id = get_global_id(1);		//	x (width)
	size_t syn_id = get_global_id(2);		//	z (depth)
	//size_t syn_row_width = get_global_size(2) * get_global_size(0);
	size_t width = get_global_size(1);
	size_t depth = get_global_size(2);
	//size_t col_pos = syn_pos >> SYNAPSES_PER_NEURON_LOG2;



	// [(row_id * depth)] * [width] + [(col_id * depth) + syn_id];
	/*size_t syns_idx =mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));
	size_t axns_idx = mad24(
		syn_axn_row_ids[syns_idx], 
		width, 
		syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH
	);*/

	size_t syns_idx = (row_id * depth * width) + (col_id * depth) + syn_id;
	size_t axns_idx = (syn_axn_row_ids[syns_idx] * width) + syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH;
	
	syn_states[syns_idx] =	mul_hi(axn_states[axns_idx], syn_strs[syns_idx]) ;
}


__kernel void dens_cycle(
				__global const char* syn_states,
				__global const char* den_thrs,
				__global char* const den_states,
				__private const uchar boost_log2
) {
	size_t gid = get_global_id(0);
	size_t syn_grp = gid << SYNAPSES_PER_DENDRITE_LOG2;

	short syn_sum = 0;

	#pragma unroll 
	for (uint i = 0; i < SYNAPSES_PER_DENDRITE; i++) {
		syn_sum += syn_states[syn_grp + i];
	}

	syn_sum = syn_sum << boost_log2;

	char den_val = (char)clamp((short)(syn_sum >> 4), (short)-128, (short)127);

	den_states[gid] = den_val;

	/*if (den_val > den_thrs[gid]) {
		den_states[gid] = den_val;			// DENDRITE_ACTIVE;
	}*/
}

__kernel void soma_cycle(
				__global const char* dst_den_states,
				__global const char* prx_den_states,
				__global char* const som_states,
				__private const uint cell_row_offset		// Change this to __attribute__ or something
) {
	size_t row = get_global_id(0);
	size_t col = get_global_id(1);
	size_t row_width = get_global_size(1);
	size_t cel_idx = mad24(row, row_width, col);
	//size_t som_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
	size_t den_grp = cel_idx << DENDRITES_PER_NEURON_LOG2;
	//int cel_grp = gid << CELLS_PER_COLUMN_LOG2;

	short den_sum = 0;
	//short den_mix = 0;

	#pragma unroll 
	for (uint i = 0; i < DENDRITES_PER_NEURON; i++) {
		den_sum += dst_den_states[den_grp + i];
		//den_mix = (char)add_sat((char)den_mix, (char)dst_den_states[den_grp + i]);
	}

	den_sum = clamp(den_sum, (short)0, (short)127);

	//short prx_den_state = clamp((short)((short)prx_den_states[cel_idx] << SYNAPSES_PER_NEURON_LOG2), (short)-128, (short)127);
	//som_states[som_idx] = (char)clamp((short)(den_sum + prx_den_state), (short)0, (short)127);
	som_states[cel_idx] = (char)clamp((short)(den_sum + prx_den_states[cel_idx]), (short)0, (short)127);

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
	__global char* const dst_vals,
	__global uchar* const dst_poss
) {
	size_t row = get_global_id(0);
	size_t col = get_global_id(1);
	size_t row_width = get_global_size(1);
	size_t hcol_idx = mad24(row, row_width, col);
	size_t wg_width = get_local_size(1);

	const uchar hcol_size_log2 = COLUMNS_PER_HYPERCOLUMN_LOG2;
	const uchar sub_grp_size_log2 = hcol_size_log2 >> 1;

	const size_t src_vals_ofs = hcol_idx << hcol_size_log2;
	
	char hcol_max_val = 0;
	char hcol_max_pos = 0;
	char sub_grp_max_val = 0;
	char sub_grp_max_pos = 0;
	
	short pos = 0;
	char val = 0;

	#pragma unroll 
	for (uint i = 0; i < 1 << sub_grp_size_log2; i++) {

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
			hcol_max_pos = sub_grp_max_pos;
		}
		sub_grp_max_val = 0;
		sub_grp_max_pos = 0;
	}
	dst_vals[hcol_idx] = hcol_max_val;
	dst_poss[hcol_idx] = hcol_max_pos;
}


__kernel void cycle_axns(
				__global char* const som_states,
				__global char* const hcol_max_vals,
				__global uchar* const hcol_max_poss,
				__global char* const axn_states,
				__private const uint cell_row_offset		// Change this to __attribute__ or something
) {
	size_t row = get_global_id(0);
	size_t col = get_global_id(1);
	size_t row_width = get_global_size(1);
	size_t cel_idx = mad24(row, row_width, col);
	size_t axn_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
	size_t hcol_idx = cel_idx >> COLUMNS_PER_HYPERCOLUMN_LOG2;
	size_t hcol_max_idx = (hcol_idx << COLUMNS_PER_HYPERCOLUMN_LOG2) + hcol_max_poss[hcol_idx];

	//char axn_state = 0;

	if (cel_idx != hcol_max_idx) {
		axn_states[axn_idx] = 0;
	} else {
		axn_states[axn_idx] = som_states[cel_idx];
	}

	//axn_states[axn_idx] = axn_state;

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
	size_t row = get_global_id(0);
	size_t col = get_global_id(1);
	size_t row_width = get_global_size(1);
	size_t grp_idx = mad24(row, row_width, col);
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
	size_t row = get_global_id(0);
	size_t col = get_global_id(1);
	size_t row_width = get_global_size(1);
	size_t grp_idx = mad24(row, row_width, col);
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
				__private const uint cell_row_offset,		// Change this to __attribute__ (or macro) or something
				__private const uint axn_inhib_tmp_ofs,
				__private const uint axn_inhib_tmp_2_ofs
) {
	size_t row = get_global_id(0);
	size_t col = get_global_id(1);
	size_t row_width = get_global_size(1);
	size_t cel_idx = mad24(row, row_width, col);
	size_t axn_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
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
		den_states[gid] = DENDRITE_ACTIVE;	
	}
}


	// #WG: common::COLUMNS_PER_SEGMENT
__kernel void cycle_col_soms(
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
}



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

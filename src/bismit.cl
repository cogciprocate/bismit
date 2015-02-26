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
__kernel void cycle_syns(
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


__kernel void cycle_dens(
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

__kernel void cycle_axns(
				__global const char* dst_den_states,
				__global const char* prx_den_states,
				__global char* const axn_states,
				__private const uint cell_row_offset		// Change this to __attribute__ or something
) {
	size_t row = get_global_id(0);
	size_t col = get_global_id(1);
	size_t row_width = get_global_size(1);
	size_t cel_idx = mad24(row, row_width, col);
	size_t axn_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
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
	//axn_states[axn_idx] = (char)clamp((short)(den_sum + prx_den_state), (short)0, (short)127);
	axn_states[axn_idx] = (char)clamp((short)(den_sum + prx_den_states[cel_idx]), (short)0, (short)127);

	/*if (den_mix) {
		axn_states[axn_idx] |= CELL_PREDICTIVE;
	} else {
		axn_states[axn_idx] = 0;
	}
*/
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

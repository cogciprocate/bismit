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

#define DENDRITE_ACTIVE					0x01
#define COLUMN_ACTIVE_INPUT				0x10
#define SOMA_ACTIVE_OUTPUT				0x01
#define CELL_PREDICTIVE					0x01
#define CELL_ACTIVE						0x10


	// #WG: common::COLUMN_SYNAPSES_PER_SEGMENT
__kernel void sense(
				__global char *src_vals,  // CHANGE TO _states
				__global char *tar_vals,
				__global short *tar_som_idxs,
				__global char *tar_syn_idxs,
				__private char dup_factor_shift
) {
	size_t gid = get_global_id(0);
	size_t tar_idx = mad24(tar_som_idxs[gid], SYNAPSES_PER_NEURON, tar_syn_idxs[gid]);

	tar_vals[tar_idx] = src_vals[gid >> dup_factor_shift];
	
}

	// #WG: common::COLUMN_DENDRITES_PER_SEGMENT
__kernel void cycle_col_dens(
				__global uchar *syn_vals,
				__global uchar *syn_strs,
				__global uchar *den_thrs,
				__global uchar *den_vals
) {
	size_t gid = get_global_id(0);
	size_t syn_grp = gid << SYNAPSES_PER_DENDRITE_LOG2;

	uchar den_val = 0;

	for (uint i = 0; i < SYNAPSES_PER_DENDRITE; i++) {
		den_val += mul_hi(syn_vals[syn_grp + i], syn_strs[syn_grp + i]);
	}

	if (den_val > den_thrs[gid]) {
		den_vals[gid] = DENDRITE_ACTIVE;	
	}
}


	// #WG: common::COLUMNS_PER_SEGMENT
__kernel void cycle_col_soms(
				__global uchar *den_vals,
				__global uchar *som_vals,
				__global uchar *cel_states
) {
	size_t gid = get_global_id(0);
	size_t den_grp = gid << DENDRITES_PER_NEURON_LOG2;
	size_t cel_grp = gid << CELLS_PER_COLUMN_LOG2;

	uchar den_mix = 0;

	for (uint i = 0; i < DENDRITES_PER_NEURON; i++) {
		den_mix |= den_vals[den_grp + i];
	}

	if (den_mix) {
		som_vals[gid] = COLUMN_ACTIVE_INPUT;

		for (uint i = 0; i < CELLS_PER_COLUMN; i++) {
			cel_states[cel_grp + i] |= CELL_ACTIVE;
		}
	}
}

	// #WORKGROUP SIZE: common::SYNAPSES_PER_LAYER
__kernel void cycle_cel_syns(
				__global char *src_vals,
				__global short *syn_src_idxs,
				__global char *syn_strs,
				__global char *syn_vals,
				__private uint src_offset,
				__private uint syn_offset,
				__private uint gid_offset_factor,
				__private char boost_factor
				//__private uint layer_current
) {
	size_t gid = get_global_id(0);
	size_t ogid = syn_offset + gid;

	int src_idx = syn_src_idxs[ogid] + src_offset + (gid_offset_factor * gid);
	char src_val = mad_sat(src_vals[src_idx], boost_factor, (char)0);
	//int src_idx = syn_src_idxs[gid];
	//char src_val = src_vals[src_idx];

	syn_vals[ogid] = mul_hi(src_val, syn_strs[ogid]);
	//syn_vals[gid] = src_val;
}

__kernel void cycle_cel_dens(
				__global char *syn_vals,
				__global char *den_thrs,
				__global char *den_vals
) {
	size_t gid = get_global_id(0);
	size_t syn_grp = gid << SYNAPSES_PER_DENDRITE_LOG2;

	short den_sum = 0;

	for (uint i = 0; i < SYNAPSES_PER_DENDRITE; i++) {
		den_sum += syn_vals[syn_grp + i];
	}

	uchar den_val = (uchar)(den_sum >> 4);

	if (den_sum > (short)den_thrs[gid]) {
		den_vals[gid] = den_val;			// DENDRITE_ACTIVE;
	}
}

__kernel void cycle_cel_axons(
				__global uchar *den_vals,
				__global uchar *som_states
) {
	size_t gid = get_global_id(0);
	size_t den_grp = gid << DENDRITES_PER_NEURON_LOG2;
	//int cel_grp = gid << CELLS_PER_COLUMN_LOG2;

	uchar den_mix = 0;

	for (uint i = 0; i < DENDRITES_PER_NEURON; i++) {
		den_mix |= den_vals[den_grp + i];
	}

	if (den_mix) {
		som_states[gid] |= CELL_PREDICTIVE;
	} else {
		som_states[gid] = 0;
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





/* MUL_HI TEST STABLE
*
*
*
*
*
*/

__kernel void test_cell_axon_stable(__global uchar *axon, __global uchar *ax_out) {
	int i = get_global_id(0);
	uchar ax = axon[i] + 2;
	axon[i] = mul_hi(ax, (uchar)128) * 2;
	ax_out[i] = axon[i];
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


__kernel void test_cell_axon(__global uchar *axon, __global uchar *ax_out) {
	int i = get_global_id(0);
	//uchar ax = axon[i] + 2;
	//axon[i] = mul_hi(ax, (uchar)128) * 2;
	ax_out[i] = axon[i];
}



__kernel void hello(__global float *input, __global float *output) {
	size_t id = get_global_id(0);
	output[id] = input[id] * input[id];
}

__kernel void test_int_shift(__global char *test_out, __private char input) {
	uint gid = get_global_id(0);
	test_out[gid] = input >> 4;
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

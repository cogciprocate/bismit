#define DENDRITES_PER_NEURON			16
#define DENDRITES_PER_NEURON_LOG2		4

#define SYNAPSES_PER_DENDRITE			16
#define SYNAPSES_PER_DENDRITE_LOG2		4

#define SYNAPSES_PER_NEURON				256 // SYNAPSES_PER_DENDRITE * DENDRITES_PER_NEURON
#define SYNAPSES_PER_NEURON_LOG2		8

#define CELLS_PER_COLUMN				16
#define CELLS_PER_COLUMN_LOG2			4

#define DENDRITE_ACTIVE					0xFF
#define COLUMN_ACTIVE_INPUT				0x10
#define SOMA_ACTIVE_OUTPUT				0x01
#define CELL_PREDICTIVE					0x01
#define CELL_ACTIVE						0x10

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

	// #WG: common::COLUMN_SYNAPSES_PER_SEGMENT
__kernel void sense(
				__global uchar *source_vals,  // CHANGE TO _states
				__global uchar *tar_vals,
				__global ushort *tar_som_addrs,
				__global uchar *tar_syn_addrs,
				__private uchar dup_factor_shift
) {
	uint gid = get_global_id(0);
	uint tar_addr = mad24(tar_som_addrs[gid], SYNAPSES_PER_NEURON, tar_syn_addrs[gid]);

	tar_vals[tar_addr] = source_vals[gid >> dup_factor_shift];
	
}

	// #WG: common::COLUMN_DENDRITES_PER_SEGMENT
__kernel void cycle_dens(
				__global uchar *syn_vals,
				__global uchar *syn_strs,
				__global uchar *den_thrs,
				__global uchar *den_vals
) {
	int gid = get_global_id(0);
	int syn_grp = gid << SYNAPSES_PER_DENDRITE_LOG2;

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
	int gid = get_global_id(0);
	int den_grp = gid << DENDRITES_PER_NEURON_LOG2;
	int cel_grp = gid << CELLS_PER_COLUMN_LOG2;

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


	// #WG common::CELL_SYNAPSES_PER_SEGMENT
__kernel void cycle_cel_syns(
				__global uchar *source_states,  	// 1/256
				__global uchar *tar_states,  		// 1
				__global ushort *tar_som_addrs,		// 1
				__global uchar *tar_syn_addrs,		// 1
				__private uchar dup_factor_shift	// (8)

) {
	int gid = get_global_id(0);

	uint tar_addr = mad24(tar_som_addrs[gid], SYNAPSES_PER_NEURON, tar_syn_addrs[gid]);

	tar_states[tar_addr] =  source_states[gid >> dup_factor_shift];	
}

__kernel void cycle_cel_soms(
				__global uchar *den_vals,
				__global uchar *som_states
) {
	int gid = get_global_id(0);
	int den_grp = gid << DENDRITES_PER_NEURON_LOG2;
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

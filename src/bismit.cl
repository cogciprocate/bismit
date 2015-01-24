#define DENDRITES_PER_NEURON			16
#define SYNAPSES_PER_DENDRITE			16

#define SYNAPSES_PER_NEURON				256 // SYNAPSES_PER_DENDRITE * DENDRITES_PER_NEURON	// DENDRITES_PER_NEURON * SYNAPSES_PER_DENDRITE

__kernel void my_kernel_func(__global float *a, __global float *b, __global float *c) {

	
	int i = get_global_id(0);
	c[i] = a[i] - b[i];
}

__kernel void hello_kernel(__global char16 *msg) {
	*msg = (char16)('H', 'e', 'l', 'l', 'o', ' ', 'k', 'e', 'r', 'n', 'e', 'l', '!', '!', '!', '\0');
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

__kernel void sense(
				__global uchar *source_vals, 
				__global uchar *tar_vals,
				__global ushort *tar_bod_addrs,
				__global uchar *tar_syn_addrs,
				__private uchar dup_factor_shift,
				__global uint *tmp_out
) {
	uint gid = get_global_id(0);
	uint tar_addr = mad24(tar_bod_addrs[gid], SYNAPSES_PER_NEURON, tar_syn_addrs[gid]);

	tar_vals[tar_addr] = source_vals[gid >> dup_factor_shift];
	
	tmp_out[gid] = tar_addr;
}

__kernel void cycle_col_dens(
				__global uchar *syn_vals,
				__global uchar *syn_strs,
				__global uchar *den_thrs,
				__global uchar *den_vals
) {
	int gid = get_global_id(0);
	int syn_grp = gid << 4;

	uchar den_val = 0;

	for (uint i = 0; i < 16; i++) {
		den_val += mul_hi(syn_vals[syn_grp + i], syn_strs[syn_grp + i]);
	}

	if (den_val > den_thrs[gid]) {
		den_vals[gid] = 0xFF;
	}
}

__kernel void cycle_cols(
				__global uchar *den_vals
				//__global uchar *syn_strs,
				//__global uchar *den_thrs,
				//__global uchar *den_vals
) {
	int gid = get_global_id(0);
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


/*
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


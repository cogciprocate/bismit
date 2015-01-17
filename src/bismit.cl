#define DENDRITES_PER_NEURON			16
#define SYNAPSES_PER_DENDRITE			16

#define SYNAPSES_PER_NEURON				256 	// DENDRITES_PER_NEURON * SYNAPSES_PER_DENDRITE

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
				__global uchar *values, 
				__global uchar *tar_syn_vals, 
				__global ushort *tar_cols, 
				__global uchar *tar_col_syns,
				__global uint *out
) {
	int gid = get_global_id(0);
	//uint tar_syn = mad24(tar_cols[gid], SYNAPSES_PER_NEURON, tar_col_syns[gid]);
	//tar_syn_vals[tar_syn] = values[gid];

	out[gid] = (uint)tar_col_syns[gid];
}




__kernel void get_synapse_values(__global uchar *values, __global uchar *output) {
	int gid = get_global_id(0);
	output[gid] = values[gid];
}


/*
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

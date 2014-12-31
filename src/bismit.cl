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
	axon[i] = mul_hi(axon[i], (uchar)16);
	ax_out[i] = axon[i];
}



__kernel void hello(__global float *input, __global float *output) {
	size_t id = get_global_id(0);
	output[id] = input[id] * input[id];
}

__kernel void sense(__global char *peek_chord) {
	int gid = get_global_id(0);
	peek_chord[gid] += 2;
	//syn_out[gid] = synapse[gid];
}

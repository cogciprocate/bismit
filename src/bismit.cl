__kernel void my_kernel_func(__global float *a, __global float *b, __global float *c) {

	
	int i = get_global_id(0);
	c[i] = a[i] - b[i];
}

__kernel void hello_kernel(__global char16 *msg) {
	*msg = (char16)('H', 'e', 'l', 'l', 'o', ' ', 'k', 'e', 'r', 'n', 'e', 'l', '!', '!', '!', '\0');
}

__kernel void test_synapse(__global char *synapse, __global char *syn_out) {
	int i = get_global_id(0);
	syn_out[i] = synapse[i];
}

__kernel void test_axon(__global short *axon, __global short *ax_out) {
	int i = get_global_id(0);
	ax_out[i] = axon[i];
}

//__kernel void hypercolumn input(__global column_states) {}

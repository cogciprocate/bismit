

__kernel void retina(
				__global uchar const* const cel_states,
				__private uchar const cel_base_axn_slc,		// <<<<< DEPRICATE: USE A GLOBAL OFFSET
				__global uchar* const axn_states
) {
	uint const slc_id = get_global_id(0);	// <<<<< TODO: USE A GLOBAL OFFSET
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const cel_idx = cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id);
	uint const axn_idx = axn_idx_3d_safe(slc_id + cel_base_axn_slc, v_size, v_id, 0, u_size, u_id, 0);

	uchar const cel_state = cel_states[cel_idx];

	axn_states[axn_idx] = cel_state;
}

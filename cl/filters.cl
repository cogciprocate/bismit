

__kernel void retina(
				__global uchar const* const axn_states,
				__global char const* const syn_src_col_u_offs,
				__global char const* const syn_src_col_v_offs,
				__global uchar const* const syn_src_slc_ids,
				__private uint const cel_idz,
				__private uchar const syns_per_tuft_l2,
				__global int* const aux_ints_0,
				__global uchar* const syn_states
) {
	uint const slc_id = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);

	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const syn_idz = (cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id) + cel_idz) << syns_per_tuft_l2;
	uint const syn_idn = syn_idz + (1 << syns_per_tuft_l2);

	for (uint syn_idx = syn_idz; syn_idx < syn_idn; syn_idx++) {
		uchar src_slc_id = syn_src_slc_ids[syn_idx];
		char v_ofs = syn_src_col_v_offs[syn_idx];
		char u_ofs = syn_src_col_u_offs[syn_idx];

		//uint axn_idx = axn_idx_3d_safe(src_slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs);
		//uchar axn_state = axn_states[axn_idx];
		uchar axn_state = axn_state_3d_safe(src_slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs, axn_states);
	
		syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);
	}
}

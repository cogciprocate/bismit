
static inline int get_neighbors_avg(uchar const slc_id, uint const v_size, uint const v_id, 
			uint const u_size, uint const u_id, __global uchar const* const cel_states, 
			uint const cel_idx, int const radius)
{
	int const radius_pos = radius; // (4:61), (7:XXX), (9:271)
	int const radius_neg = 0 - radius_pos;

	//int const center_state = cel_states[cel_idx];
	int neighbors_sum = 0; // includes cell itself
	int neighbor_count = 0;

	for (int v_ofs = radius_neg; v_ofs <= radius_pos; v_ofs++) {
		int v_neg = 0 - v_ofs;
		int u_z = max(radius_neg, v_neg - radius_pos);
		int u_m = min(radius_pos, v_neg + radius_pos);

		for (int u_ofs = u_z; u_ofs <= u_m; u_ofs++) {
			neighbors_sum += cel_state_3d_safe(slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs, cel_states);
			neighbor_count += 1;
		}
	}	

	return neighbors_sum / neighbor_count;
}


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
	//uint const axn_idx = axn_idx_3d_safe(slc_id + cel_base_axn_slc, v_size, v_id, 0, u_size, u_id, 0);
	uint const axn_idx = cel_idx_3d_unsafe(slc_id + cel_base_axn_slc, v_size, v_id, u_size, u_id);

	int const center_state = cel_states[cel_idx];

	// int const radius_pos = 4; // (4:61), (7:XXX), (9:271)
	// int const radius_neg = 0 - radius_pos;

	// int const center_state = cel_states[cel_idx];
	// int neighbors_sum = 0; // includes cell itself
	// int neighbor_count = 0;

	// for (int v_ofs = radius_neg; v_ofs <= radius_pos; v_ofs++) {
	// 	int v_neg = 0 - v_ofs;
	// 	int u_z = max(radius_neg, v_neg - radius_pos);
	// 	int u_m = min(radius_pos, v_neg + radius_pos);

	// 	for (int u_ofs = u_z; u_ofs <= u_m; u_ofs++) {
	// 		neighbors_sum += cel_state_3d_safe(slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs, cel_states);
	// 		neighbor_count += 1;
	// 	}
	// }	

	// int const neighbor_avg = neighbors_sum / neighbor_count;	

	int const is_off_cen_cel = cel_idx & 1;
	int const is_large_cel = ((cel_idx & 3) >> 1) ^ is_off_cen_cel;

	int radius = is_large_cel ? 4 : 3;

	int const neighbors_avg = get_neighbors_avg(slc_id, v_size, v_id, u_size, u_id, cel_states,
		cel_idx, radius);

	uchar cel_state = 0;

	if (is_off_cen_cel) {
		int cst = center_state + RETNAL_THRESHOLD;
		cel_state = mul24(neighbors_avg - cst, neighbors_avg > cst);
	} else {
		int nat = neighbors_avg + RETNAL_THRESHOLD;
		cel_state = mul24(center_state - nat, center_state > nat);
	}

	axn_states[axn_idx] = cel_state;	
}

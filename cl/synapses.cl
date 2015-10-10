

static inline uchar syn_fire(uchar const axn_state) {
	return ((axn_state != 0) << 7) + (axn_state >> (SYNAPSE_AXON_BIAS_LOG2));
}

static inline uchar4 syn_fire_vec4(uchar4 const axn_state) {
	return (convert_uchar4(axn_state != (uchar)0) & (uchar4)0x80) 
		| (axn_state >> (uchar4)(SYNAPSE_AXON_BIAS_LOG2));
}

// SYNS_CYCLE_SIMPLE(): Simple synapse cycling without workgroup optimization or vectorization
__kernel void syns_cycle_simple(
				__global uchar const* const axn_states,
				__global char const* const syn_src_col_u_offs,
				__global char const* const syn_src_col_v_offs,
				__global uchar const* const syn_src_slc_ids,
				__private uint const cel_idz,
				__private uchar const syns_per_tuft_l2,
				__global int* const aux_ints_0,
				__global uchar* const syn_states) 
{
	uint const slc_id_lyr = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);

	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const syn_idz = (cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id) + cel_idz) << syns_per_tuft_l2;
	uint const syn_idn = syn_idz + (1 << syns_per_tuft_l2);

	for (uint syn_idx = syn_idz; syn_idx < syn_idn; syn_idx++) {
		uchar src_slc_id = syn_src_slc_ids[syn_idx];
		char v_ofs = syn_src_col_v_offs[syn_idx];
		char u_ofs = syn_src_col_u_offs[syn_idx];

		//uint axn_idx = axn_idx_3d_safe(src_slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs);
		//uchar axn_state = axn_states[axn_idx];
		uchar axn_state = axn_state_3d_safe(src_slc_id, v_id, v_ofs, u_id, u_ofs, axn_states);
	
		// syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);
		syn_states[syn_idx] = syn_fire(axn_state);
	}
}


// SYNS_CYCLE_SIMPLE_VEC4(): Simple synapse cycling with vectorization
__kernel void syns_cycle_simple_vec4(
				__global uchar const* const axn_states,
				__global char4 const* const syn_src_col_u_offs,
				__global char4 const* const syn_src_col_v_offs,
				__global uchar4 const* const syn_src_slc_ids,
				__private uint const cel_idz,
				__private uchar const syns_per_tuft_l2,
				__global int* const aux_ints_0,
				__global uchar4* const syn_states) 
{
	uint const slc_id_lyr = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);

	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const syn4_idz = ((cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id) + cel_idz) 
		<< (syns_per_tuft_l2 - 2)); // DIVIDED BY 4 BECAUSE VECTORS
	uint const syn4_idn = syn4_idz + (1 << (syns_per_tuft_l2 - 2)); // DIVIDED BY 4 BECAUSE VECTORS

	for (uint syn4_idx = syn4_idz; syn4_idx < syn4_idn; syn4_idx++) {
		uchar4 src_slc_id = syn_src_slc_ids[syn4_idx];
		char4 v_ofs = syn_src_col_v_offs[syn4_idx];
		char4 u_ofs = syn_src_col_u_offs[syn4_idx];

		uchar4 axn_state = axn_state_3d_safe_vec4(
			src_slc_id,
			(int4)(int)v_id,
			v_ofs, 
			(int4)(int)u_id, 
			u_ofs, 
			axn_states);

		// syn_states[syn4_idx] = (convert_uchar4(axn_state != (uchar)0) & (uchar4)0x80) | (axn_state >> (uchar4)1);
		syn_states[syn4_idx] = syn_fire_vec4(axn_state);
	}
}



//SYNS_CYCLE_WG_OPT(): Cycle synapses with workgroup optimized writes
__kernel void syns_cycle_wow(
				__global uchar const* const axn_states,
				__global char const* const syn_src_col_u_offs,
				__global char const* const syn_src_col_v_offs,
				__global uchar const* const syn_src_slc_ids,
				__private uint const cel_idz,
				__private uchar const syns_per_tuft_l2,
				__global int* const aux_ints_0,
				__global uchar* const syn_states) 
{
	uint const slc_id_lyr = get_global_id(0);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const v_work_size = get_local_size(1);
	uint const u_work_size = get_local_size(2);

	/* <<<<< SHOULD PROBABLY DO THIS USING GET_NUM_GROUPS()... >>>>> */

	// // BASE DIM_ID (COORDINATE) FOR CURRENT SLICE (GLOBAL ID ON THE INITIAL EDGE OF THE SLICE)
	// uint const v_id_slc_base = mul24(v_size, slc_id_lyr);
	// uint const u_id_slc_base = mul24(u_size, slc_id_lyr);

	// // DIM_ID WITHIN CURRENT SLICE
	// uint const v_id_slc = v_id_global - v_id_slc_base;
	// uint const u_id_slc = u_id_global - u_id_slc_base;

	// // BASE DIM_ID FOR CURRENT WORKGROUP
	// uint const v_id_base = v_id_slc - get_local_id(1);
	// uint const u_id_base = u_id_slc - get_local_id(2);
	/* <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> */

	// BASE DIM_ID FOR CURRENT WORKGROUP
	uint const v_id_base = get_global_id(1) - get_local_id(1);
	uint const u_id_base = get_global_id(2) - get_local_id(2);
			

	uint const syns_per_tuft = 1 << syns_per_tuft_l2;
	uint const syns_per_wg = mul24(v_work_size, u_work_size);

	uint syns_per_iter = syns_per_wg; 	// PRECALCULATE -- MAKE CONST
	uint u_per_iter = 0;	// PRECALCULATE -- MAKE CONST
	uint v_per_iter = 0; 	// PRECALCULATE -- MAKE CONST
	
	while (syns_per_iter >= syns_per_tuft) { // PRECALCULATE
		u_per_iter += 1;
		syns_per_iter -= syns_per_tuft;
	}

	while (u_per_iter >= u_work_size) { // PRECALCULATE
		v_per_iter += 1;
		u_per_iter -= u_work_size;
	}


	int cur_syn_ofs = mad24(get_local_id(1), u_work_size, get_local_id(2));
	int cur_u_wg = 0;
	int cur_v_wg = 0;
	
	while (cur_syn_ofs >= syns_per_tuft) {
		cur_u_wg += 1;
		cur_syn_ofs -= syns_per_tuft;
	}

	while (cur_u_wg >= u_work_size) {
		cur_v_wg += 1;
		cur_u_wg -= u_work_size;
	}

	for (uint i = 0; i < syns_per_tuft; i += 1) {
		int cur_syn_ofs_is_oob = (cur_syn_ofs >= syns_per_tuft);
		cur_u_wg += cur_syn_ofs_is_oob;
		cur_syn_ofs -= mul24(cur_syn_ofs_is_oob, (int)syns_per_tuft);

		int cur_u_wg_is_oob = (cur_u_wg >= u_work_size);
		cur_v_wg += cur_u_wg_is_oob;
		cur_u_wg -= mul24(cur_u_wg_is_oob, (int)u_work_size);

		uint v_id = v_id_base + cur_v_wg;
		uint u_id = u_id_base + cur_u_wg;

		uint syn_idx = ((cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id) + cel_idz) 
			<< syns_per_tuft_l2) + cur_syn_ofs;

		char v_ofs = syn_src_col_v_offs[syn_idx];
		char u_ofs = syn_src_col_u_offs[syn_idx];
		uchar src_slc_id = syn_src_slc_ids[syn_idx];

		uchar axn_state = axn_state_3d_safe(src_slc_id, v_id, v_ofs, u_id, u_ofs, axn_states);

		// syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);
		syn_states[syn_idx] = syn_fire(axn_state);

		// ### DO NOT REMOVE ###
		// if ((slc_id_lyr == 1) && (get_global_id(1) == 6) && (get_global_id(2) == 6) && (cel_idz == 0)) {
		// 	aux_ints_0[i] = v_id_base;
		// }

		cur_syn_ofs += syns_per_iter;
		cur_u_wg += u_per_iter;
		cur_v_wg += v_per_iter;

	}
}


// SYNS_CYCLE_WG_OPT_VEC4(): Cycle synapses with workgroup optimized writes and vectorization
__kernel void syns_cycle_wow_vec4(
				__global uchar const* const axn_states,
				__global char4 const* const syn_src_col_u_offs,
				__global char4 const* const syn_src_col_v_offs,
				__global uchar4 const* const syn_src_slc_ids,
				__private uint const cel_idz,
				__private uchar const syns_per_tuft_l2,
				__global int* const aux_ints_0,
				__global uchar4* const syn_states) 
{
	uint const slc_id_lyr = get_global_id(0);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const v_work_size = get_local_size(1);
	uint const u_work_size = get_local_size(2);

	uint const v_id_base = get_global_id(1) - get_local_id(1);
	uint const u_id_base = get_global_id(2) - get_local_id(2);

	uint const syn4s_per_tuft = (1 << (syns_per_tuft_l2)) >> 2; // VEC4'D
	uint const syn4s_per_wg = mul24(v_work_size, u_work_size); // DON'T DIVIDE ME (DOING SAME SYN4S AS SYNS)

	uint syn4s_per_iter = syn4s_per_wg; 	// PRECALCULATE -- MAKE CONST
	uint u_per_iter = 0;	// PRECALCULATE -- MAKE CONST
	uint v_per_iter = 0; 	// PRECALCULATE -- MAKE CONST
	
	while (syn4s_per_iter >= syn4s_per_tuft) { // PRECALCULATE
		u_per_iter += 1;
		syn4s_per_iter -= syn4s_per_tuft;
	}

	while (u_per_iter >= u_work_size) { // PRECALCULATE
		v_per_iter += 1;
		u_per_iter -= u_work_size;
	}


	int cur_syn4_ofs = mad24(get_local_id(1), u_work_size, get_local_id(2));
	int cur_u_wg = 0;
	int cur_v_wg = 0;
	
	while (cur_syn4_ofs >= syn4s_per_tuft) {
		cur_u_wg += 1;
		cur_syn4_ofs -= syn4s_per_tuft;
	}

	while (cur_u_wg >= u_work_size) {
		cur_v_wg += 1;
		cur_u_wg -= u_work_size;
	}

	for (uint i = 0; i < syn4s_per_tuft; i++) {
		int cur_syn4_ofs_is_oob = (cur_syn4_ofs >= syn4s_per_tuft);
		cur_u_wg += cur_syn4_ofs_is_oob;
		cur_syn4_ofs -= mul24(cur_syn4_ofs_is_oob, (int)syn4s_per_tuft);

		int cur_u_wg_is_oob = (cur_u_wg >= u_work_size);
		cur_v_wg += cur_u_wg_is_oob;
		cur_u_wg -= mul24(cur_u_wg_is_oob, (int)u_work_size);

		uint v_id = v_id_base + cur_v_wg;
		uint u_id = u_id_base + cur_u_wg;

		uint syn4_idx = (((cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id) + cel_idz) 
			<< syns_per_tuft_l2) >> 2) + cur_syn4_ofs; // VEC4'D IDX

		char4 v_ofs = syn_src_col_v_offs[syn4_idx];
		char4 u_ofs = syn_src_col_u_offs[syn4_idx];
		uchar4 src_slc_id = syn_src_slc_ids[syn4_idx];

		// uchar axn_state = axn_state_3d_safe(src_slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs, axn_states);
		// syn_states[syn4_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);

		uchar4 axn_state = axn_state_3d_safe_vec4(
			src_slc_id, 
			(int4)(int)v_id, 
			v_ofs, 
			(int4)(int)u_id, 
			u_ofs, 
			axn_states);

		// syn_states[syn4_idx] = (convert_uchar4(axn_state != (uchar)0) & (uchar4)0x80) | (axn_state >> (uchar4)1);
		syn_states[syn4_idx] = syn_fire_vec4(axn_state);


		// ### DO NOT REMOVE ###
		// if ((slc_id_lyr == 1) && (get_global_id(1) == 6) && (get_global_id(2) == 6) && (cel_idz == 0)) {
		// 	aux_ints_0[i] = cur_u_wg;
		// }

		cur_syn4_ofs += syn4s_per_iter;
		cur_u_wg += u_per_iter;
		cur_v_wg += v_per_iter;

	}
}






// RENAME ME
	//__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void axns_cycle_unoptd_5_0 (										
	__global uchar* const asp_col_ids,
	__global char* const asp_states,
	__global char* const col_states,
	__global char* const axn_states,
	__global int* const aux_ints_0,
	__global int* const aux_ints_1,
	//__local uchar* wiv_local,
	__private uint const axn_row_offset
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	//uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = col_idx + mad24(axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (col_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;

	char col_state = col_states[col_idx];

	uint competetor_ids[ASPINY_SPAN] = { 0, 0, 0, 0, 0, 0, 0, 0 };
	char competetor_states[ASPINY_SPAN] = { 0, 0, 0, 0, 0, 0, 0, 0 }; // { 0, 1, 2, 3, 4, 5, 6, 7 };

	//int const winner_close = ;
	int win_count = (asp_col_id_to_col_idx(asp_idx, (asp_col_ids[asp_idx])) == col_idx);
	//int cci = 0;
	//int cur_asp_idx = 0;
	for (uint i = 0; i < ASPINY_SPAN; i++) {
		//uint cur_asp_idx = (asp_idx - ASPINY_REACH) + i + (i > 3);

		//competetor_ids[i] = asp_col_id_to_col_idx(cur_asp_idx, competetor_ids[i]);

		//aux_ints_1[(col_idx << 3) + i] = cur_asp_idx;			//competetor_ids[i];
		aux_ints_1[(col_idx << 3) + i] = 0;
		//aux_ints_1[col_idx] = cur_asp_idx;
	}

	if (win_count) {

		vstore4(vload4(0, &asp_states[asp_idx - ASPINY_REACH]), 0, &competetor_states[0]);
		vstore4(vload4(0, &asp_states[asp_idx + 1]), 0, &competetor_states[ASPINY_REACH]);

		vstore4(convert_uint4(vload4(0, &asp_col_ids[asp_idx - ASPINY_REACH])), 0, &competetor_ids[0]);
		vstore4(convert_uint4(vload4(0, &asp_col_ids[asp_idx + 1])), 0, &competetor_ids[ASPINY_REACH]);

		for (uint i = 0; i < ASPINY_SPAN; i++) {
			uint cur_asp_idx = (asp_idx - ASPINY_REACH) + i + (i > 3);

			//competetor_ids[i] = asp_col_id_to_col_idx(cur_asp_idx, competetor_ids[i]);

			//aux_ints_1[(col_idx << 3) + i] = competetor_ids[i] + 1;
			aux_ints_1[(col_idx << 3) + i] = asp_col_ids[cur_asp_idx];
			//aux_ints_1[(col_idx << 3) + i] = cur_asp_idx;			
			//aux_ints_1[(col_idx << 3) + i] = asp_idx;
			//aux_ints_1[col_idx] = cur_asp_idx;
		}


		for (uint i = 0; i < ASPINY_SPAN; i++) {

			
			if (col_state < competetor_states[i]) {
				continue;

			} else if (col_state == competetor_states[i]) {

				if ((col_idx & 0x3F) == (competetor_ids[i] & 0x3F)) {
						// SHOULD BE UNREACHABLE
					win_count += 99;
				}

				//if (((col_idx & 0x01) ^ (competetor_ids[i] & 0x01))) {
				if (((col_idx & 0x3F) < (competetor_ids[i] & 0x3F))) {
					win_count += ((col_idx & 0x3F) < (competetor_ids[i] & 0x3F));
					//win_count += 1;
					//aux_ints_1[(col_idx << 3) + i] = (competetor_ids[i] & 0xFF);

				} else {
					win_count += ((col_idx & 0x3F) > (competetor_ids[i] & 0x3F));
					//win_count += 1;
					//aux_ints_1[(col_idx << 3) + i] = (competetor_ids[i] & 0x3F) + 2000;
				}
				//win_count += ((col_idx & 0x1F) < (competetor_ids[i] & 0x1F)) | (low_6(col_idx) == 32);

			} else {
				win_count += 1;
			}

			/*win_count += (
				(col_state > competetor_states[i]) | (
				(col_idx & 0x1F) < (competetor_ids[i] & 0x1F)) |
				(low_6(col_idx) == 32)
			);*/

		}
	}

	col_states[col_idx] = mul24(col_state, (win_count >= COLUMN_DOMINANCE_FLOOR));

	//col_state = mad24(64, (col_state > 0), (col_state >> 1));
	axn_states[axn_idx] = mul24(col_state, (win_count > 0));
	//axn_states[axn_idx] = mul24(col_state, (win_count >= COLUMN_DOMINANCE_FLOOR));
	//aux_ints_0[col_idx] = mul24(col_state, (win_count >= COLUMN_DOMINANCE_FLOOR));
	//aux_ints_0[col_idx] = mul24((int)asp_idx, (win_count > 0));
	//aux_ints_1[col_idx] = mul24(win_count, (win_count >= COLUMN_DOMINANCE_FLOOR));
	//aux_ints_1[col_idx] = asp_col_id_to_col_idx(asp_idx, (asp_col_ids[asp_idx]));



}











	__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void axns_cycle_unoptd_buggy_phantom_value_version (
	__global uchar* const asp_col_ids,
	__global char* const asp_states,
	__global char* const col_states,
	__global char* const axn_states,
	//__global int* const aux_ints_0,
	//__global int* const aux_ints_1,
	//__local uchar* wiv_local,
	__private uint const axn_row_offset
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	//uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const col_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = col_idx + mad24(axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (col_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;

	uint rnd = col_id + 1181783497276652981;
	rnd ^= (rnd << 23);
	rnd ^= (rnd >> 17);
	rnd ^= asp_idx & (asp_idx >> 26);

	rnd = rnd & 0x00FF;


	//uint practical_aspiny_reach = (ASPINY_REACH - 1);
	//uint low_compare_asp_idx = asp_idx - 4;
	//uint high_compare_asp_idx = asp_idx + 4;


	char const col_state = col_states[col_idx];

	bool winner_close = false;
	//bool winner_far = false;
	//uint max_asp_idx = 0;
	//int max_col_state = 0;
	uint current_col_idx = 0;
	int const asp_offs = asp_idx - 4;

	if (col_states[col_idx] >= asp_states[asp_idx]) {
		winner_close = true;

	}

	uint best_idx = 101;
	uint best_idx_diff = 101;

	int win_count = 0;

	uint curr_asp_id_low_6 = 100;


	int competetor_states[8] = { 99, 99, 99, 99, 99, 99, 99, 99 };
	int competetor_ids[8] = { 99, 99, 99, 99, 99, 99, 99, 99 };


	for (int i = 0; i <= 8; i++) {

		if (i == 4) {
			continue;
		}

		int curr_asp_idx = (asp_idx - 4) + i;
		int curr_competetor_idx = 99;

		if (i < 4) {
			curr_competetor_idx = i;

		} else if (i > 4) {
			curr_competetor_idx = i - 1;
		}

		competetor_states[curr_competetor_idx] = asp_states[curr_asp_idx];
		competetor_ids[curr_competetor_idx] = asp_col_ids[curr_asp_idx];
	}

	char best = 0;

	if (winner_close) {

		for (uint i = 0; i < 8; i++) {

			if (col_state > competetor_states[i]) {
				win_count += 1;
				best = col_state;
				continue;

			} else if (col_state == competetor_states[i]) {
				uint competetor_id_low = low_6(competetor_ids[i]);

				if (low_6(col_idx) == 0) {
					win_count += 1;
				} else {
					win_count = 0;
					break;
				}
			} else {
				win_count = 0;
				break;
			}
		}
	}


	//aux_ints_0[col_idx] = col_states[col_idx];
	//aux_ints_0[2006] = col_states[6];

	//axn_states[axn_idx] = col_states[col_idx];

	if (win_count) {
		if (col_states[col_idx] < 127) {
			axn_states[axn_idx] = col_state - 20;	
		} else {
			axn_states[axn_idx] = col_states[col_idx];
		}
		

		//aux_ints_0[col_idx] = col_states[col_idx];		
		//aux_ints_1[col_idx] = low_6(asp_to_col_ofs(asp_idx + 1) + asp_col_ids[asp_idx + 1]);
		//aux_ints_1[col_idx] = asp_to_col_ofs(asp_idx + 0); // + asp_col_ids[asp_idx + 1];

		//aux_ints_1[col_idx] = win_count;
		//aux_ints_1[col_idx] = competetor_ids[6];
	}
}






/*	__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void axns_cycle_unoptd_working (
	__global uchar* asp_col_ids,
	__global char* col_states,
	__global char* axn_states,
	__local uchar* wiv_local,
	__private uint const axn_row_offset
	//__global char* 
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row_id, row_width, col_id);
	uint const axn_idx = cel_idx + mad24(axn_row_offset, row_width, (uint)SYNAPSE_REACH);
	uint const asp_idx = (cel_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;
	//uint const axn_idx = mad24((row_id + axn_row_offset), row_width, col_id) + SYNAPSE_REACH;
	//uint const axn_idx = mad24(row_id, row_width, col_id);

	uint col_winner_pos = asp_col_ids[asp_idx] & 0x07; // GOOD
	uint cel_pos = cel_idx & 0x07;	// GOOD
	//uchar cel_pos = cel_idx & 0x07;


	//axn_states[axn_idx] = (asp_pos == cel_pos);
	if (cel_pos != col_winner_pos) {
		axn_states[axn_idx] = 0;
	} else {
		axn_states[axn_idx] = asp_col_ids[asp_idx] + 64;
		//axn_states[axn_idx] = clamp(asp_col_ids[asp_idx] << 2, 0, 127);
		//axn_states[axn_idx] = col_winner_pos;
		//axn_states[axn_idx] = asp_col_ids[asp_idx];
	}

	//axn_states[axn_idx] = (asp_col_ids[asp_idx] & 0x07);
	//axn_states[axn_idx] = asp_col_ids[asp_idx & 0xF8];
	//axn_states[axn_idx] = asp_col_ids[asp_idx];
	//axn_states[axn_idx] = get_local_size(1);
	//axn_states[axn_idx] = get_local_size(1) >> 2; 

}*/






__kernel void col_cycle_old(
	__global char* const syn_states,
	__global char* const col_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	//uint const l_id = get_local_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row_id, row_width, col_id);
	uint const syn_ofs = cel_idx << SYNAPSES_PER_CELL_PROXIMAL_LOG2;

	int syn_sum = 0;

	for (uint i = 0; i < SYNAPSES_PER_CELL_PROXIMAL; i += 1) {
		syn_sum += syn_states[syn_ofs + i];
	}

	col_states[cel_idx] = (char)(syn_sum >> SYNAPSES_PER_CELL_PROXIMAL_LOG2);
}






__kernel void soma_cycle_pre(
				__global char* const prx_den_states,
				__global char* const som_states
) {
	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row_id, row_width, col_id);
	uint const den_grp = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;

	short den_sum = 0;

		#pragma unroll 
	for (uint i = 0; i < DENDRITES_PER_CELL_DISTAL; i++) {
		den_sum += prx_den_states[den_grp + i];
	}

	den_sum >>= DENDRITES_PER_CELL_DISTAL_LOG2;

	som_states[cel_idx] = (char)den_sum;
}


	//	#WORK SIZE: Synapses per Region
__kernel void syns_cycle(
				__global char* const axn_states,
				__global uchar* const syn_axn_row_ids,
				__global char* const syn_axn_col_offs,
				__global char* const syn_strs,
				__global char* const syn_states
				//__private const uint axn_row_width
) {
	uint const row_id = get_global_id(0);		//	y (height)
	uint const col_id = get_global_id(1);		//	x (width)
	uint const syn_id = get_global_id(2);		//	z (depth)
	//uint const syn_row_width = get_global_size(2) * get_global_size(0);
	uint const width = get_global_size(1);
	uint const depth = get_global_size(2);
	//uint const col_pos = syn_pos >> SYNAPSES_PER_CELL_LOG2;



	/* [(row_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	uint const syns_idx = mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));
	uint const axns_idx = mad24(
		syn_axn_row_ids[syns_idx], 
		width, 
		syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH
	);
	/*
	uint const syns_idx = (row_id * depth * width) + (col_id * depth) + syn_id;
	uint const axns_idx = (syn_axn_row_ids[syns_idx] * width) + syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH;
	*/
	int syn_state = (int)syn_strs[syns_idx] * (int)axn_states[axns_idx];
	syn_states[syns_idx] = (char)clamp((int)(syn_state >> SYNAPSE_STRENGTH_DEFAULT_DISTAL_LOG2), (int)0, (int)127);
	//syn_states[syns_idx] =	(syn_strs[syns_idx] > 0);
}


__kernel void dens_cycle_new(
				__global char* const syn_states,
				__global char* const den_thrs,
				__global char* const den_states,
				__private uchar const boost_log2
) {
	uint const gid = get_global_id(0);
	uint const syn_grp = gid << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	short syn_sum = 0;

		#pragma unroll 
	for (uint i = 0; i < SYNAPSES_PER_DENDRITE_DISTAL; i++) {
		syn_sum += syn_states[syn_grp + i];
	}

	syn_sum = syn_sum << boost_log2;

	char den_val = (char)clamp((short)(syn_sum >> SYNAPSES_PER_DENDRITE_DISTAL_LOG2), (short)-128, (short)127);

	den_states[gid] = den_val; // * (den_val > den_thrs[gid] || den_val < 0);

	/*if (den_val > den_thrs[gid]) {
		den_states[gid] = den_val;			// DENDRITE_ACTIVE;
	}*/
}

__kernel void dens_cycle(
				__global char* const syn_states,
				__global char* const den_thrs,
				__global char* const den_states,
				__private uchar const boost_log2
) {
	uint const gid = get_global_id(0);
	uint const syn_grp = gid << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	short syn_sum = 0;

		#pragma unroll 
	for (uint i = 0; i < SYNAPSES_PER_DENDRITE_DISTAL; i++) {
		syn_sum += syn_states[syn_grp + i];
	}

	syn_sum = syn_sum << boost_log2;

	char den_val = (char)clamp((short)(syn_sum >> SYNAPSES_PER_DENDRITE_DISTAL_LOG2), (short)-128, (short)127);

	den_states[gid] = den_val; // * (den_val > den_thrs[gid] || den_val < 0);

	/*if (den_val > den_thrs[gid]) {
		den_states[gid] = den_val;			// DENDRITE_ACTIVE;
	}*/
}


__kernel void soma_cycle_post(
				__global char* const dst_den_states,
				//__global char* const prx_den_states,
				__global char* const som_states,
				__private uint const cell_row_offset		// Change this to __attribute__ or something
) {
	uint const row = get_global_id(0);
	uint const col = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row, row_width, col);
	//uint const som_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
	uint const den_grp = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;
	//int cel_grp = gid << CELLS_PER_COLUMN_LOG2;

	short den_sum = 0;

		#pragma unroll 
	for (uint i = 0; i < DENDRITES_PER_CELL_DISTAL; i++) {
		den_sum += dst_den_states[den_grp + i];
		//den_sum = (char)add_sat((char)den_sum, (char)dst_den_states[den_grp + i]);
	}

	den_sum = den_sum >> DENDRITES_PER_CELL_DISTAL_LOG2;

	//short prx_den_state = clamp((short)((short)prx_den_states[cel_idx] << SYNAPSES_PER_CELL_LOG2), (short)-128, (short)127);
	//som_states[som_idx] = (char)clamp((short)(den_sum + prx_den_state), (short)0, (short)127);

	som_states[cel_idx] = (char)clamp((short)(den_sum), (short)0, (short)127);
	//som_states[cel_idx] = (char)clamp((short)(den_sum + prx_den_states[cel_idx]), (short)0, (short)127);

	//barrier(CLK_LOCAL_MEM_FENCE);


	/*if (den_mix) {
		som_states[som_idx] |= CELL_PREDICTIVE;
	} else {
		som_states[som_idx] = 0;
	}
*/
}


__kernel void soma_inhib(
	__global char* const src_vals,
	__global uchar* const dst_ids,
	__global char* const dst_vals
) {
	uint const row = get_global_id(0);
	uint const col = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const hcol_idx = mad24(row, row_width, col);
	//uint const wg_width = get_local_size(1);

	uchar const hcol_size_log2 = COLUMNS_PER_HYPERCOLUMN_LOG2;
	//uchar const sub_grp_size_log2 = hcol_size_log2 >> 1;

	uint const src_vals_ofs = hcol_idx << hcol_size_log2;
	
	char hcol_max_val = 0;
	char hcol_max_pos = 0;
	//char sub_grp_max_val = 0;
	//char sub_grp_max_pos = 0;
	
	short pos = 0;
	char val = 0;

		#pragma unroll 
	for (uint i = 0; i < 1 << hcol_size_log2; i++) {
		val = src_vals[src_vals_ofs + pos];

		if (val > hcol_max_val) {
			hcol_max_val = val;
			hcol_max_pos = pos;
		}

		pos += 1;
	}
	dst_vals[hcol_idx] = hcol_max_val;
	dst_ids[hcol_idx] = hcol_max_pos;
	//dst_ids[hcol_idx] = pos;
}



__kernel void axns_cycle_old(
				__global char* const som_states,
				//__global char* const hcol_max_vals,
				__global uchar* const hcol_max_ids,
				__global char* const axn_states,
				__private uint const cell_row_offset		// Change this to __attribute__ or something
) {
	uint const row = get_global_id(0);
	uint const col = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const cel_idx = mad24(row, row_width, col);
	uint const axn_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
	uint const hcol_idx = cel_idx >> COLUMNS_PER_HYPERCOLUMN_LOG2;
	uint const hcol_max_idx = (hcol_idx << COLUMNS_PER_HYPERCOLUMN_LOG2) + hcol_max_ids[hcol_idx];

	char axn_state = 0;

	if (cel_idx == hcol_max_idx) {
		axn_state = som_states[cel_idx];
	} 

	axn_states[axn_idx] = axn_state;

}


__kernel void syns_learn(
				__global uchar* const hcol_max_ids,
				__global char* const hcol_max_vals,
				__global char* const syn_states,
				__global char* const den_states,
				__global char* const syn_strengths,
				__global char* const rand_ofs

) {
	uint const row = get_global_id(0);
	uint const col = get_global_id(1);
	uint const row_width = get_global_size(1);
	uint const hcol_idx = mad24(row, row_width, col);

	uchar const hcol_size_log2 = COLUMNS_PER_HYPERCOLUMN_LOG2;
	uint const cel_ofs = hcol_idx << hcol_size_log2;
	uchar const hcol_max_id = hcol_max_ids[hcol_idx];
	uint const cel_idx = cel_ofs + hcol_max_id;

	uint const den_ofs = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;
	//uint const syn_ofs = den_ofs << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	uint pseudo_rand = (col + row + (uint)hcol_max_ids) & 0x00FF;

	uint rand1 = (uint)rand_ofs[pseudo_rand];
	uint rand2 = (uint)rand_ofs[rand1];

	uint rand_den_idx = den_ofs + (rand1 & 0x000F);
	uint rand_syn_idx = (rand_den_idx << SYNAPSES_PER_DENDRITE_DISTAL_LOG2) + (rand2 & 0x000F);


	syn_strengths[rand_syn_idx] += 
		(syn_states[rand_syn_idx] > den_states[rand_den_idx]) * 
		(den_states[rand_den_idx] > hcol_max_vals[hcol_idx]) *
		(syn_strengths[rand_syn_idx] < SYNAPSE_STRENGTH_MAX)
	;

}

__kernel void syns_decay(
	__global char* const syn_strs
) {

	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const syn_id = get_global_id(2);
	//uint const syn_row_width = get_global_size(2) * get_global_size(0);
	uint const width = get_global_size(1);
	uint const depth = get_global_size(2);
	//uint const col_pos = syn_pos >> SYNAPSES_PER_CELL_LOG2;

	/* [(row_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	uint const syn_idx = mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));

	syn_strs[syn_idx] -= (1 * (syn_strs[syn_idx] > -128)); 
}


__kernel void syns_regrow(
	__global char* const syn_strs,
	__global char* const rand_ofs,
	__global char* const syn_ofs
) {

	uint const row_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const syn_id = get_global_id(2);
	//uint const syn_row_width = get_global_size(2) * get_global_size(0);
	uint const width = get_global_size(1);
	uint const depth = get_global_size(2);
	//uint const col_pos = syn_pos >> SYNAPSES_PER_CELL_LOG2;

	/* [(row_id * depth)] * [width] + [(col_id * depth) + syn_id]; */
	uint const syn_idx = mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));

	if (syn_strs[syn_id] <= -127) {
		syn_ofs[syn_idx] = rand_ofs[syn_id]; 
		syn_strs[syn_idx] = SYNAPSE_STRENGTH_DEFAULT_DISTAL; 
	}
}









__kernel void test_cell_axn_stable(__global uchar *axn, __global uchar *ax_out) {
	int i = get_global_id(0);
	uchar ax = axn[i] + 2;
	axn[i] = mul_hi(ax, (uchar)128) * 2;
	ax_out[i] = axn[i];
}






Bullshit Below

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


#define COLUMNS_PER_SEGMENT 			64 * 16
#define SYNAPSES_PER_LAYER				SYNAPSES_PER_CELL * COLUMNS_PER_SEGMENT
#define CELLS_PER_COLUMN				16
#define CELLS_PER_COLUMN_LOG2			4

#define DENDRITE_ACTIVE					0x01
#define COLUMN_ACTIVE_INPUT				0x10
#define SOMA_ACTIVE_OUTPUT				0x01
#define CELL_PREDICTIVE					0x01
#define CELL_ACTIVE						0x10




 Kernel Preferred work group size multiple:	 	64
 Max compute units:				 				32
 Max work items dimensions:						3
 Max work group size:				 				256

 Remember to inline functions





 __kernel void syns_learn(
				__global uchar* const hcol_max_ids,
				__global char* const hcol_max_vals,
				__global char* const syn_states,
				__global char* const den_states,
				__global char* const syn_strengths,
				__global char* const rand_ofs

) {
	size_t const row = get_global_id(0);
	size_t const col = get_global_id(1);
	size_t const row_width = get_global_size(1);
	size_t const hcol_idx = mad24(row, row_width, col);

	uchar const hcol_size_log2 = COLUMNS_PER_HYPERCOLUMN_LOG2;
	size_t const cel_ofs = hcol_idx << hcol_size_log2;
	uchar const hcol_max_id = hcol_max_ids[hcol_idx];
	size_t const cel_idx = cel_ofs + hcol_max_id;

	size_t const den_ofs = cel_idx << DENDRITES_PER_CELL_DISTAL_LOG2;
	//size_t const syn_ofs = den_ofs << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	size_t pseudo_rand = (col + row + (size_t)hcol_max_ids) & 0x00FF;

	size_t rand1 = (size_t)rand_ofs[pseudo_rand];
	size_t rand2 = (size_t)rand_ofs[rand1];

	size_t rand_den_idx = den_ofs + (rand1 & 0x000F);
	size_t rand_syn_idx = (rand_den_idx << SYNAPSES_PER_DENDRITE_DISTAL_LOG2) + (rand2 & 0x000F);


	syn_strengths[rand_syn_idx] += 
		(syn_states[rand_syn_idx] > den_states[rand_den_idx]) * 
		(den_states[rand_den_idx] > hcol_max_vals[hcol_idx]) *
		(syn_strengths[rand_syn_idx] < SYNAPSE_STRENGTH_MAX)
	;

	//char syn_strength;
	//char syn_state;

	/*
		LET'S AVERAGE THE SYNAPSES FOR ALL THE DENDRITES AND BOOST THE TOP 10 - 40%
		BETTER YET: FIND THE BEST DENDRITES AND BOOST THE TOP SYNAPSES JUST FOR THEM
		BETTER YET: COME UP WITH MULTIPLE STRATEGIES FOR BOOSTING STRENGTHS AND TRY THEM ALL OUT
		DON'T BOOST MORE THAN PROBABLY 3 - 6 SYNAPSES
	*/

	/*
	short den_states_sum = 5;
	char den_states_avg = 0;

	for (uchar d = 0; d < DENDRITES_PER_CELL; d++) {
		den_states_sum += den_states[den_ofs + d];
	}

	den_states_avg = (char)(den_states_sum >> DENDRITES_PER_CELL_LOG2);
	*/

	//syn_strengths[syn_ofs + 0] = den_states_avg + 100; 


	/*for (uchar s = 0; s < SYNAPSES_PER_DENDRITE; s++) {
		//syn_strengths[syn_idx] += (syn_states[syn_idx] > den_states[den_idx]) * (syn_strengths[syn_idx] < 63);
		//syn_strengths[syn_idx] += (syn_states[syn_idx] > den_states[den_idx]) * (syn_strengths[syn_idx] < 64);
		//syn_strengths[syn_idx] += 1;
		
		
		syn_strength = syn_strengths[syn_idx];
		syn state = syn_states[syn_idx];
		if ((syn_strength > den_states[den_idx]) && (syn_strength < 63)) {
			syn_strengths[syn_idx] += 1;
		}
		

		syn_idx++;
	}*/


	/*for (uchar d = 0; d < DENDRITES_PER_CELL; d++) {
		for (uchar s = 0; s < SYNAPSES_PER_DENDRITE; s++) {
			syn_strengths[syn_idx] += (syn_states[syn_idx] > den_states[den_idx]) * (syn_strengths[syn_idx] < 64);
			//syn_strengths[syn_idx] += 1;
			syn_idx++;
		}
		den_idx ++;
	}*/
}



__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void col_syns_cycle_2_0(
	__global char* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__global char* const syn_states,
	__private uchar const src_axn_row
	//__local char* const axn_cache
) {

	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const lid = get_local_id(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row_id, row_width, col_id);
	//size_t const axn_zero = lid + SYNAPSE_REACH;
	size_t const depth_log2 = SYNAPSES_PER_CELL_PROXIMAL_LOG2;
	
	//__local char axn_cache[SYNAPSE_WORKGROUP_SIZE + SYNAPSE_SPAN]; // ADD HEIGHT AS A CONSTANT AT SOME POINT
	//__local size_t axn_cache_width;
	//__local size_t axn_cache_height;

	//size_t axn_ofs = lid << 1;
	size_t axn_idx = mad24(src_axn_row, row_width, add_sat(col_id, (size_t)SYNAPSE_REACH));
	//axn_cache[axn_ofs - 1] = axn_states[axn_idx + axn_ofs - 1];
	//axn_cache[axn_ofs] = axn_states[axn_idx + axn_ofs];


	if (lid == 0) {
		size_t const wg_size = get_local_size(1);
		axn_cache_width = add_sat((size_t)SYNAPSE_SPAN, wg_size);
		axn_cache_height = 1; // *** FIX (should be based on size of src_axn_rows or whatever it becomes) ***
		size_t const axn_ofs = mad24(src_axn_row, row_width, col_id + SYNAPSE_REACH);
		size_t axn_idx = axn_ofs;

		#pragma unroll
		for (size_t i = 0; i < axn_cache_width; i++) {
			axn_cache[i] = axn_states[axn_idx];
			axn_idx += 1;
		}
	}

	//barrier(CLK_LOCAL_MEM_FENCE);

	//size_t syn_idx = cel_idx << SYNAPSES_PER_CELL_PROXIMAL_LOG2;
	//size_t axn_cache_idx;

	//size_t spc =  << SYNAPSE_WORKGROUP_SIZE;

		// START AT THE FIRST CELL OF THE WORKGROUP
		// INCREMENT SYN_IDX BY ONE WHOLE WORKGROUP (1 workgroup = 1 cell) AT A TIME
	size_t syn_idx = ((cel_idx - lid) << depth_log2) + lid;
	int end = SYNAPSE_WORKGROUP_SIZE + lid;

	//#pragma unroll
	for (int i = 0; i < SYNAPSE_WORKGROUP_SIZE; i += 1) {
		//axn_cache_idx = axn_zero + syn_src_ofs[syn_idx];

		//syn_states[syn_idx] = axn_cache[axn_cache_idx];
		syn_states[syn_idx] = axn_states[axn_idx + syn_src_ofs[syn_idx]];
		syn_idx += SYNAPSE_WORKGROUP_SIZE;
		axn_idx += 1;
	}
}



__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void col_syns_cycle_1_0(
	__global char* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__global char* const syn_states,
	__private uchar const src_axn_row
	//__local char* const axn_cache
) {

	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const lid = get_local_id(1);
	size_t const wg_size = get_local_size(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row_id, row_width, col_id);
	size_t const axn_zero = lid + SYNAPSE_REACH;
	size_t const depth_log2 = SYNAPSES_PER_CELL_PROXIMAL_LOG2;
	
	__local char axn_cache[SYNAPSE_WORKGROUP_SIZE + SYNAPSE_SPAN]; // ADD HEIGHT AS A CONSTANT AT SOME POINT
	__local size_t axn_cache_width;
	__local size_t axn_cache_height;

	if (lid == 0) {
		axn_cache_width = add_sat((size_t)SYNAPSE_SPAN, wg_size);
		axn_cache_height = 1; // *** FIX (should be based on size of src_axn_rows or whatever it becomes) ***
		size_t const axn_ofs = mad24(src_axn_row, row_width, col_id + SYNAPSE_REACH);
		size_t axn_idx = axn_ofs;

		#pragma unroll
		for (size_t i = 0; i < axn_cache_width; i++) {
			axn_cache[i] = axn_states[axn_idx];
			axn_idx += 1;
		}
	}

	barrier(CLK_LOCAL_MEM_FENCE);

	size_t syn_idx = cel_idx << depth_log2;
	size_t axn_idx;

	#pragma unroll
	for (int i = 0; i < SYNAPSES_PER_CELL_PROXIMAL; i++) {
		axn_idx = axn_zero + syn_src_ofs[syn_idx];

		syn_states[syn_idx] = axn_cache[axn_idx];
		syn_idx += 1;
	}
}

__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void col_syns_cycle_crashes(
	__global char* const axn_states,
	__global char* const syn_src_ofs,
	__global uchar* const syn_src_row_ids,
	__global char* const syn_states
) {
	size_t const row_id = get_global_id(0);
	size_t const col_id = get_global_id(1);
	size_t const l_id = get_local_id(1);
	size_t const row_width = get_global_size(1);
	size_t const cel_idx = mad24(row_id, row_width, col_id);
	
	size_t axn_ofs = col_id + (size_t)SYNAPSE_REACH;
	size_t syn_idx = ((cel_idx - l_id) << (size_t)SYNAPSES_PER_CELL_PROXIMAL_LOG2) + l_id;

	size_t n = (size_t)SYNAPSES_PER_CELL_PROXIMAL + axn_ofs;
	size_t axn_idx;

	for (size_t i = axn_ofs; i < n; i += 1) {
		axn_idx = mad24((size_t)syn_src_row_ids[syn_idx], row_width, (size_t)(i + syn_src_ofs[syn_idx]));
		syn_states[syn_idx] = axn_states[axn_idx];
		syn_idx += SYNAPSE_WORKGROUP_SIZE;
	}
}



__kernel void syns_cycle_opt(
				__global char* const axn_states,
				__global uchar* const syn_axn_row_ids,
				__global char* const syn_axn_col_offs,
				__global char* const syn_strs,
				__global char* const syn_states
				//__private const uint axn_row_width
) {
	size_t const row_id = get_global_id(0);		//	y (height)
	size_t const col_id = get_global_id(1);		//	x (width)
	size_t const syn_id = get_global_id(2);		//	z (depth)
	//size_t const syn_row_width = get_global_size(2) * get_global_size(0);
	size_t const width = get_global_size(1);
	size_t const depth = get_global_size(2);
	//size_t const col_pos = syn_pos >> SYNAPSES_PER_CELL_LOG2;



		// [(row_id * depth)] * [width] + [(col_id * depth) + syn_id];
	size_t const syns_idx = mad24(mul24(row_id, depth), width, mad24(col_id, depth, syn_id));
	size_t const axns_idx = mad24(
		syn_axn_row_ids[syns_idx], 
		width, 
		syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH
	);
	
	//size_t const syns_idx = (row_id * depth * width) + (col_id * depth) + syn_id;
	//size_t const axns_idx = (syn_axn_row_ids[syns_idx] * width) + syn_axn_col_offs[syns_idx] + col_id + SYNAPSE_REACH;
	
	int syn_state = (int)syn_strs[syns_idx] * (int)axn_states[axns_idx];
	syn_states[syns_idx] = (char)clamp((int)(syn_state >> SYNAPSE_STRENGTH_DEFAULT_LOG2), (int)0, (int)127);
	//syn_states[syns_idx] =	(syn_strs[syns_idx] > 0);
}





__kernel void inhib_3_0(
	__global char* const src_vals,
	__global char* const dst_vals
) {
	size_t row = get_global_id(0);
	size_t col = get_global_id(1);
	size_t row_width = get_global_size(1);
	size_t grp_idx = mad24(row, row_width, col);
	size_t wg_width = get_local_size(1);

	//__local char best_of_8[32]; // wg_size = 256; 256 / 8 = 32

	const uchar grp_size_log2 = 3;

	size_t src_grp = grp_idx << grp_size_log2;
	char grp_max = 0;

	#pragma unroll 
	for (uint i = 0; i < 1 << grp_size_log2; i++) {
		grp_max = max(src_vals[src_grp + i], grp_max);
		//dst_vals[src_grp + i] = 5;
	}

	dst_vals[grp_idx] = grp_max; //		grp_max;

}

__kernel void inhib_2_0(
	__global char* const som_states,
	__global char* const axn_states,
	__private const uint axn_out_ofs,
	__global int* const aux_vals
) {
	size_t row = get_global_id(0);
	size_t col = get_global_id(1);
	size_t row_width = get_global_size(1);
	size_t grp_idx = mad24(row, row_width, col);
	size_t wg_width = get_local_size(1);

	//__local char best_of_8[32]; // wg_size = 256; 256 / 8 = 32

	const uchar grp_size_log2 = 4;

	size_t som_grp = grp_idx << grp_size_log2;
	char grp_max = 0;

	#pragma unroll 
	for (uint i = 0; i < 1 << grp_size_log2; i++) {
		grp_max = max(som_states[som_grp + i], grp_max);
	}

	aux_vals[grp_idx] = grp_max;
	//aux_vals[(grp_idx << 2) + col] = 69;

}



__kernel void inhib_1_0(		// FUCK IT. LET'S DUPLICATE WORK FOR NOW. I'M DRUNK.
				__global char* const axn_states,
				__private const uint cell_row_offset,		// Change this to __attribute__ (or macro) or something
				__private const uint axn_inhib_tmp_ofs,
				__private const uint axn_inhib_tmp_2_ofs
) {
	size_t row = get_global_id(0);
	size_t col = get_global_id(1);
	size_t row_width = get_global_size(1);
	size_t cel_idx = mad24(row, row_width, col);
	size_t axn_idx = mad24((row + cell_row_offset), row_width, col) + SYNAPSE_REACH;
	//size_t den_grp = cel_idx << DENDRITES_PER_CELL_LOG2;

	size_t group_size = 16;
	size_t axn_grp = (size_t)axn_idx & (size_t)(0xFFFFFFFF - group_size); // groups of 16;

	char axn_state_max = 1;
	size_t axn_idx_max = 2;

	#pragma unroll 
	for (uint i = 0; i < group_size; i++) {
		size_t idx = axn_grp + i;
		if (axn_states[idx] > axn_state_max) {
			axn_state_max = axn_states[idx];
			axn_idx_max = idx;
		}
	}

	// NO CLUE WHAT I'M DOING -- START THIS OVER 
	axn_states[axn_inhib_tmp_ofs + cel_idx + SYNAPSE_REACH] = axn_state_max;
	axn_states[axn_inhib_tmp_2_ofs + cel_idx + SYNAPSE_REACH] = axn_idx_max;
}




	// #WG: common::COLUMN_SYNAPSES_PER_SEGMENT
__kernel void sense(
				__global const char *src_vals,  // CHANGE TO _states
				__global char * const tar_vals,
				__global const short *tar_som_idxs,
				__global const char *tar_syn_idxs,
				__private const char dup_factor_shift
) {
	size_t gid = get_global_id(0);
	size_t tar_idx = mad24(tar_som_idxs[gid], SYNAPSES_PER_CELL_DISTAL, tar_syn_idxs[gid]);

	tar_vals[tar_idx] = src_vals[gid >> dup_factor_shift];
	
}

	// #WG: common::COLUMN_DENDRITES_PER_SEGMENT
__kernel void cycle_col_dens(
				__global const uchar *syn_states,
				__global const uchar *syn_strs,
				__global const uchar *den_thrs,
				__global uchar * const den_states
) {
	size_t gid = get_global_id(0);
	size_t syn_grp = gid << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	uchar den_val = 0;

	for (uint i = 0; i < SYNAPSES_PER_DENDRITE_DISTAL; i++) {
		den_val += mul_hi(syn_states[syn_grp + i], syn_strs[syn_grp + i]);
	}

	if (den_val > den_thrs[gid]) {
		den_states[gid] = den_val;	
	}
}


	// #WG: common::COLUMNS_PER_SEGMENT
__kernel void cycle_col_soms(
				__global const uchar *den_states,
				__global uchar * const som_vals,
				__global uchar *cel_states
) {
	size_t gid = get_global_id(0);
	size_t den_grp = gid << DENDRITES_PER_CELL_LOG2;
	size_t cel_grp = gid << CELLS_PER_COLUMN_LOG2;

	uchar den_mix = 0;

	for (uint i = 0; i < DENDRITES_PER_CELL; i++) {
		den_mix |= den_states[den_grp + i];
	}

	if (den_mix) {
		som_vals[gid] = COLUMN_ACTIVE_INPUT;

		for (uint i = 0; i < CELLS_PER_COLUMN; i++) {
			cel_states[cel_grp + i] |= CELL_ACTIVE;
		}
	}
}




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


__kernel void test_cell_axn(__global uchar *axn, __global uchar *ax_out) {
	int i = get_global_id(0);
	//uchar ax = axn[i] + 2;
	//axn[i] = mul_hi(ax, (uchar)128) * 2;
	ax_out[i] = axn[i];
}



__kernel void hello(__global float *input, __global float *output) {
	size_t id = get_global_id(0);
	output[id] = input[id] * input[id];
}

__kernel void test_int_shift(__global char *test_out, __private char input) {
	uint gid = get_global_id(0);
	test_out[gid] = input >> 4;
}*/

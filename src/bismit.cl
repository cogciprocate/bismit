#define LTD_BIAS_LOG2					0
#define LTP_BIAS_LOG2					0
#define ENERGY_SETTING 					4
#define ENERGY_RECHARGE					1

#define FLAG_ON(flag_set, mask)			((flag_set) |= (mask))
#define FLAG_OFF(flag_set, mask)		((flag_set) &= ~(mask))
#define FLAG_TEST(flag_set, mask)		(((flag_set) & (mask)) == (mask))

#define ENERGY_LEVEL_MIN				9		
#define ENERGY_LEVEL_MAX				255
#define ENERGY_REGEN_AMOUNT				1

#define OLD_INHIB

//  bismit.cl: CONVENTIONS
//
// 		idx: current index (of a loop, workgroup, queue, etc.)
//			- almost always a physical in-memory address
//
// 		idz: index[0], first element, starting element
//
// 		idn: index[len], element after final element, termination point
//			- ex.: for(int i = 0, i < idn, i++)
//
// 		idm: index[max], final (valid) element just before idn (idn - 1 = idm)
//			- ex.: for(int i = 0, i <= idm, i++)
//
//
// 		id: identifier, but not a physical array index
//
//		y_id // DEPRICATING
//		x_id // DEPRICATING
//
//		slc_id
//		w_id
//		v_id
//		u_id
//
// 		fuz: fuzzyness, level of predictiveness
//
// 		***** High Priority Comment, Temporary Code Change
// 		<<<<< Medium Priority Comment, To Do
// 		##### Debug Message Prefix



// DEPRICATE
static inline uint asp_to_sst_ofs(uint asp_idx) {
	return (asp_idx - ASPINY_REACH) << ASPINY_SPAN_LOG2;
}

// DEPRICATE
static inline uint asp_sst_id_to_sst_idx(uint const asp_idx, uint const asp_sst_id) {
	return (asp_to_sst_ofs(asp_idx) + (asp_sst_id & (ASPINY_SPAN - 1)));
}

// DEPRICATE
static inline char split_v_ofs(char src_uv_ofs) {
	return ((char)(src_uv_ofs & 0xF0)) >> 4;
}

// DEPRICATE
static inline char split_u_ofs(char src_uv_ofs) {
	return ((char)((src_uv_ofs & 0x0F) << 4)) >> 4;
}

// DEPRICATE
static inline uint col_id_3d(uint v_id, uint u_id) {
	return mad24(v_id, get_global_size(1), u_id);
}


static inline int dim_is_safe(uint dim_size, uint dim_id, char dim_ofs) {
	int dim_ttl = (int)dim_id + dim_ofs;
	return (dim_ttl >= 0) & (dim_ttl < (int)dim_size);
}


// CEL_IDX_3D: LINEAR INDEX OF A CELL
static inline uint cel_idx_3d_unsafe(uint slc_id, uint v_size, uint v_id, uint u_size, uint u_id) {

	// int v_ofs_is_safe = dim_is_safe(v_size, v_id, v_ofs);	// USE ELSEWHERE
	// int u_ofs_is_safe = dim_is_safe(u_size, u_id, u_ofs);	// USE ELSEWHERE
	// int cel_idx_is_safe = v_ofs_is_safe && u_ofs_is_safe;	// USE ELSEWHERE

	return mad24(slc_id, mul24(v_size, u_size), mad24(v_id, u_size, u_id));	
}


// 	SAFE_CEL_STATE_3D(): 'Safe' Cell State Resolution
// 		- If id + ofs are out of cortical bounds, zero is returned
//			- otherwise resolved state is returned 
//		- Intended primarily for use by the inhibition-related kernel(s)
static inline uchar safe_cel_state_3d(
				uchar slc_id, uint v_size, uint v_id, char v_ofs, 
				uint u_size, uint u_id, char u_ofs, 
				__global uchar const* const cel_states
) {
	int v_ofs_is_safe = dim_is_safe(v_size, v_id, v_ofs);
	int u_ofs_is_safe = dim_is_safe(u_size, u_id, u_ofs);
	int cel_idx_is_safe = v_ofs_is_safe & u_ofs_is_safe;

	//uint safe_v_id = mad24((uint)v_ofs_is_safe, (uint)v_ofs, v_id); 	// UNNECESSARY
	//uint safe_u_id = mad24((uint)u_ofs_is_safe, (uint)u_ofs, u_id);	// UNNECESSARY

	uint cel_idx = cel_idx_3d_unsafe(slc_id, v_size, (int)v_id + v_ofs, u_size, (int)u_id + u_ofs);

	return mul24(cel_idx_is_safe, cel_states[cel_idx]);
}

// 	CEL_IDX_3D_SAFE(): [WORK IN PROGRESS]: For whatever that means... 
// 		if out of bounds, return the edge for now...
/*
	static inline uint cel_idx_3d_safe_wip(uint slc_id, uint v_size, uint v_id, int v_ofs, uint u_size, uint u_id, int u_ofs) {
		//int v_ofs_is_safe = dim_is_safe(v_size, v_id, v_ofs);
		//int u_ofs_is_safe = dim_is_safe(u_size, u_id, u_ofs);
		//int cel_idx_is_safe = v_ofs_is_safe && u_ofs_is_safe;

		return mad24(slc_id, mul24(v_size, u_size), mad24(v_id, u_size, u_id));	
	}
*/

// 	SAFE_DIM_OFS(): Ensure that a dimensional (x,y,z) id does not exceed it's global cortical boundary
//		- Can be depricated if synapses guarantee that their offsets are safe upon growth/regrowth
static inline char safe_dim_ofs(uint dim_size, uint dim_id, char dim_ofs) {
	int dim_ttl = (int)dim_id + dim_ofs;

	return dim_ofs + mul24(dim_ttl < 0, (0 - dim_ttl) << 1)
		- mul24(dim_ttl >= (int)dim_size, (int)(dim_ttl - (dim_size - 1)) << 1);
}

__kernel void test_safe_dim_ofs(
				__global uint const* const dim_ids,
				__global char const* const dim_offs,
				__private uint const dim_size,
				__global char* const safe_dim_offs
) {
	uint id = get_global_id(0);

	char safe_do = safe_dim_ofs(dim_size, dim_ids[id], dim_offs[id]);

	safe_dim_offs[id] = safe_do;
}


// AXN_IDX_2D(): Axon Address Resolution
// 	- We must calculate the address for both the horizontal slc and spatial (vertical) slc case.
// 		- When calculating vertical slcs: 
// 		- Simply multiply slc_id * slc dims.width and add offset, cell column id, and global padding (SYNAPSE_REACH_LIN).
// 		- When calculating horizontal slcs:
// 		- Horizontal slcs are always physically after spatial slcs within axn_states so we 
//		must add that space first (HARF * slc_columns). That gets us to the beginning of horizontal 
//		slc space after padding is added (padding = SYNAPSE_REACH_LIN).
// 		- We then multiply SYNAPSE_SPAN_LIN (which is SYNAPSE_REACH_LIN * 2) by the horizontal
//		slc_id to get to the correct horizontal slc.
// 		- We must add padding + an extra SYNAPSE_REACH_LIN to get us to the middle of the slc.
// 		- We then apply the offset (col_ofs) to get to the exact axon_idx.
// 		- col_id is irrelevant and unused for horiz. slcs.
//		
// 		- As always, for performance reasons, rather than branch we calculate both cases and multiply by a bool 
//
//
// 	- [complete] Accommodate horizontal axon slcs, slcs which are nonspatial and look the same from any column in a region.
// 	- Rows above HORIZONTAL_AXON_ROW_DEMARCATION are considered horizontal and will be mapped to the rear of axn_states.
// 	- [incomplete] Specific unit tests
// 	- [incomplete] #define slc_columns 
//
static inline uint axn_idx_2d(uchar slc_id, uint slc_columns, uint col_id, int col_ofs) {
	uint axn_idx_spt = mad24((uint)slc_id, slc_columns, (uint)(col_id + col_ofs + SYNAPSE_REACH_LIN));
	int hslc_id = slc_id - HORIZONTAL_AXON_ROW_DEMARCATION;
	int hcol_id = mad24(hslc_id, SYNAPSE_SPAN_LIN, col_ofs + SYNAPSE_REACH_LIN);
	uint axn_idx_hrz = mad24((uint)HORIZONTAL_AXON_ROW_DEMARCATION, slc_columns, (uint)(hcol_id + SYNAPSE_REACH_LIN));
	
	return mul24((uint)(hslc_id < 0), axn_idx_spt) + mul24((uint)(hslc_id >= 0), axn_idx_hrz);
}

// 	AXN_IDX_3D(): Axon Address Resolution
// 		- slc_id assumed to be a valid axon slice
// 		- safe_dim_ofs can be depricated later (see function comments)
//
//		- TODO: Make dendrite reach properly reflective, taking all three planar axis into consideration
//			- currently dendrites are all reflecting clockwise
// 		
//		- TODO: Deal with corner cases
//			- synapses in the acute corners are going to have to have double reflection into a 120-deg space
//
static inline uint axn_idx_3d(uchar slc_id, uint v_size, uint v_id, char v_ofs, uint u_size, uint u_id, char u_ofs) {
	char safe_v_ofs = safe_dim_ofs(v_size, v_id, v_ofs);
	char safe_u_ofs = safe_dim_ofs(u_size, u_id, u_ofs);

	uint uv_size = mul24(v_size, u_size);
	uint uv_id = mad24(v_id, u_size, u_id);
	int uv_ofs = mad24((int)safe_v_ofs, (int)u_size, (int)safe_u_ofs);

	uint axn_idx_spt = mad24((uint)slc_id, uv_size, (uint)(uv_id + uv_ofs));

	return axn_idx_2d(slc_id, uv_size, uv_id, uv_ofs);
}



//
//	CONVERT TO 3D
//		- slc_id: source index (synapse
//		- slc_columns
//
//
/*static inline uint axn_idx_3d(uchar slc_id, uint slc_columns, uint col_id, char hw_ofs) {

	uint area_width = 99; // ***** <<<<< OBVIOUSLY, SUPPLY THIS TO THE FUNCTION

	char width_ofs = hw_ofs & 0x0F;
	char height_ofs = (hw_ofs & 0xF0) >> 4;

	int slice_offset = mad24((int)height_ofs, (int)area_width, (int)width_ofs);

	uint normalized_slice_offset = 99;

	uint offset_physical = col_id + normalized_slice_offset;


	//uint offset_physical = (uint)(col_id + col_ofs + SYNAPSE_REACH_LIN);

	uint axn_idx_physical = mad24((uint)slc_id, slc_columns, phv_ofs);

	int hslc_id = slc_id - HORIZONTAL_AXON_ROW_DEMARCATION;

	int hcol_id = mad24(hslc_id, SYNAPSE_SPAN_LIN, offset_physical + SYNAPSE_REACH_LIN);

	uint axn_idx_hrz = mad24((uint)HORIZONTAL_AXON_ROW_DEMARCATION, slc_columns, (uint)(hcol_id + SYNAPSE_REACH_LIN));

	
	return mul24((uint)(hslc_id < 0), axn_idx_physical) + mul24((uint)(hslc_id >= 0), axn_idx_hrz);
}
*/

static inline int rnd_inc(uint const rnd_a,	uint const rnd_b, char const syn_strength) {
		return ((rnd_a ^ rnd_b) & 0x7F) > abs(syn_strength);
}



// VECTORIZE
static inline void dst_syns__active__stp_ltd( 					// ANOMALY & CRYSTALLIZATION
				__global uchar const* const syn_states,
				uint const syn_idx_start,	// (syn_idz)
				uint const syns_per_den_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global uchar* const syn_flag_sets,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		char syn_strength = syn_strengths[i];
		uchar syn_flag_set = syn_flag_sets[i];

		int const inc = rnd_inc(rnd, i, syn_strength);
		int const syn_active = syn_state != 0;
		syn_flag_set &= ~SYN_STP_FLAG;

		syn_flag_set |= mul24(syn_active, SYN_STP_FLAG);
		syn_strength -= mul24(!syn_active, inc << LTD_BIAS_LOG2);

		syn_flag_sets[i] = syn_flag_set;
		syn_strengths[i] = syn_strength;
	}
}


// VECTORIZE --- RE-STREAMLINE (REMOVE BRANCH)
static inline void cel_syns_trm( 			// TERMINATION
				__global uchar const* const syn_states,
				uint const syn_idx_start,	// (syn_idz)
				uint const syns_per_tuft_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global uchar* const syn_flag_sets,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_tuft_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		char syn_strength = syn_strengths[i];
		uchar syn_flag_set = syn_flag_sets[i];
		int const inc = rnd_inc(rnd, i, syn_strength);
		//uchar const rnd_char = (rnd ^ i) & 0x7F;		
		//int const inc = (rnd_char > abs(syn_strength));
		int const syn_prev_stp = (syn_flag_set & SYN_STP_FLAG) == SYN_STP_FLAG;
		int const syn_active = syn_state != 0;

		syn_strength += mul24(syn_prev_stp && !syn_active, inc << LTP_BIAS_LOG2);
		syn_strength -= mul24(syn_prev_stp && syn_active, inc << LTD_BIAS_LOG2);

		/*if (syn_prev_stp) {
			if (syn_active) {
				syn_strength -= (inc << LTD_BIAS_LOG2);			
			} else {
				syn_strength += (inc << LTP_BIAS_LOG2);
			}
		}*/

		//syn_strength -= mul24(syn_flag_set, inc);

		syn_flag_set &= ~SYN_STP_FLAG;

		syn_flag_sets[i] = syn_flag_set;
		syn_strengths[i] = syn_strength;
	}
}



// VECTORIZE --- RE-STREAMLINE (REMOVE BRANCH)
static inline void prx_syns__active__ltp_ltd( 
				__global uchar const* const syn_states,
				uint const syn_idx_start,
				uint const syns_per_den_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		char syn_strength = syn_strengths[i];
		//int is_neg = (syn_strength < 0);	// NEGATIVE STRENGTH SYNAPSES GET A BONUS (LOOKS LIKE THIS MAY BE NO BUENO)
		int const inc = rnd_inc(rnd, i, syn_strength);
		/*uchar rnd_char = (rnd ^ i) & 0x7F;		
		int inc = (rnd_char > abs(syn_strength));*/
		int const syn_active = syn_state != 0;

		syn_strength += mul24(syn_active, inc);
		syn_strength -= mul24(!syn_active, inc);


		// if (syn_state == 0) {
		// 	syn_strength -= inc;
		// } else {
		// 	//syn_strength += (inc + is_neg);	// NEGATIVE STRENGTH SYNAPSES GET A BONUS (COMMENT OUT BELOW IF USING)
		// 	syn_strength += inc;
		// }

		syn_strengths[i] = syn_strength;
		//syn_strengths[i] = (syn_strength - inc) + mul24((syn_state != 0), (inc << 1));
	}

}






/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/



// GENERAL OPTIMIZATION TODO:
// 	- Vectorize (pretty much everywhere)
// 	- Fit data into workgroups better for several kernels
// 		- Keep data loading contiguous for the workgroup
// 	- Use Async copy
// 		event_t async_work_group_copy(__local T *dst, const __global T *src, size_t num_elements, event_t event)
// 		event_t async_work_group_copy(__global T *dst, const __local T *src, size_t num_elements, event_t event)
// 		void wait_group_events (int num_events, event_t *event_list)
// 	- Globalize wherever possible:
// 		- slc_columns
// 		- 
//
// CLEAN UP:
// 	- One day soon this beast of a .cl file will be split up.


// SYNS_CYCLE_SIMPLE(): Simple synapse cycling with non-workgroup-optimized writes
__kernel void syns_cycle_simple(
				__global uchar const* const axn_states,
				__global char const* const syn_src_col_u_offs,
				__global char const* const syn_src_col_v_offs,
				__global uchar const* const syn_src_slc_ids,
				//__global char const* const syn_strengths,
				//__private uint const syn_tuft_i,
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

	uint const col_id = col_id_3d(v_id, u_id);

	uint const syn_idz = (cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id) + cel_idz) << syns_per_tuft_l2;
	uint const syn_idn = syn_idz + (1 << syns_per_tuft_l2);

	for (uint syn_idx = syn_idz; syn_idx < syn_idn; syn_idx++) {
		uchar src_slc_id = syn_src_slc_ids[syn_idx];

		//char src_uv_ofs = syn_src_col_v_offs[syn_idx];
		// char v_ofs = split_v_ofs(src_uv_ofs);
		// char u_ofs = split_u_ofs(src_uv_ofs);
		// uint axn_idx = axn_idx_3d(src_slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs);
		char v_ofs = syn_src_col_v_offs[syn_idx];
		char u_ofs = syn_src_col_u_offs[syn_idx];
		uint axn_idx = axn_idx_3d(src_slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs);

		uchar axn_state = axn_states[axn_idx];
	
		syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);

		// if ((syn_idx & 0xFF) == 0) {
		// 	aux_ints_0[syn_idx] = axn_idx;
		// }

	}
}




// SYNS_CYCLE():
// 	number of source slcs can not exceed: 
// 		ROWS * (SYNAPSES_PER_CELL_PROXIMAL + SYNAPSE_WORKGROUP_SIZE)
//
// TODO:
// 	- Vectorize!
// 	- Col Inputs/Outputs probably need to be limited to one slc.
// 		- This isn't feasable. Need to intelligently prefetch:
// 			- syns_cycle() will need knowledge of which axon ranges it's expected to read from
//
// WATCH OUT FOR:
// 	- Bank conflicts once src_col_uv_offs start to change


//	__attribute__((reqd_work_group_size(1, SYNAPSE_WORKGROUP_SIZE, 1)))
__kernel void syns_cycle_2d_workgroup_optimized(
				__global uchar const* const axn_states,
				__global char const* const syn_src_col_v_offs,
				__global uchar const* const syn_src_slc_ids,
				//__global char const* const syn_strengths,
				__private uint const syn_tuft_i,
				__private uchar const syns_per_tuft_l2,
				__global int* const aux_ints_0,
				__global uchar* const syn_states
) {
	//uint const slc_id = get_global_id(0);
	//uint const col_id = get_global_id(1);

	uint const slc_columns = mul24(get_global_size(1), get_global_size(2)); // PRECOMPUTE or depricate
	uint const layer_total_per_tuft = mul24(slc_columns, get_global_size(0)); // PRECOMPUTE
	uint const wg_size = mul24(get_local_size(1), get_local_size(2)); // PRECOMPUTE or depricate
	uint const base_cel_tuft_ofs = mul24(syn_tuft_i, layer_total_per_tuft); // PRECOMPUTE

	uint const slc_id = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);

	uint const col_id = mad24(v_id, get_global_size(1), u_id); // FOR DEBUG PURPOSES
	uint auu_idx = mad24(slc_id, slc_columns, col_id); // FOR DEBUG PURPOSES
	
	uint const wg_id = mad24(get_group_id(1), get_num_groups(2), get_group_id(2));
	uint const l_id = mad24(get_local_id(1), get_local_size(2), get_local_id(2));
	
	uint const base_col_id = mul24(wg_id, wg_size);
	uint const tuft_cel_idx = mad24(slc_id, slc_columns, base_col_id);
	uint const base_cel_idx = base_cel_tuft_ofs + tuft_cel_idx;
	//uint const base_cel_idx = tuft_cel_idx;
	uint const base_syn_idx = (base_cel_idx << syns_per_tuft_l2);
	uint const init_syn_idx = base_syn_idx + l_id;

	uint const syns_per_slc = slc_columns << syns_per_tuft_l2;
	uint const syns_per_wg = wg_size << syns_per_tuft_l2;


	int syn_col_i = (base_col_id << syns_per_tuft_l2) + l_id;
	uint syn_idx = init_syn_idx;
	uint const syn_n = base_syn_idx + syns_per_wg;

	for (; syn_idx < syn_n; syn_idx += wg_size) {
		syn_col_i -= mul24((int)syns_per_slc, (syn_col_i >= syns_per_slc));
		int col_pos = syn_col_i >> syns_per_tuft_l2;
		uint axn_idx = axn_idx_2d(syn_src_slc_ids[syn_idx], slc_columns, col_pos, syn_src_col_v_offs[syn_idx]);
		uchar axn_state = axn_states[axn_idx];

		//aux_ints_0[syn_idx - l_id] = axn_idx;
		
		syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);
		
		syn_col_i += wg_size;


		//syn_states[syn_idx] = axn_state;
		//char syn_strength = syn_strengths[syn_idx];
		//syn_states[syn_idx] = mul24((syn_strength >= 0), ((axn_state != 0) << 7) + (axn_state >> 1));
		//aux_ints_0[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);
		//aux_ints_0[syn_idx] = axn_idx;
	}

	//aux_ints_0[0] = HORIZONTAL_AXON_ROW_DEMARCATION;

	//uint auu_idx = mad24(slc_id, slc_columns, col_id);
	//aux_ints_0[init_syn_idx] = base_col_id;
	//aux_ints_0[init_syn_idx] = 1;
	//aux_ints_0[base_cel_idx] = 12321;
}



/*__kernel void syns_cycle_original(
				__global uchar const* const axn_states,
				__global char const* const syn_src_col_v_offs,
				__global uchar const* const syn_src_slc_ids,
				//__global char const* const syn_strengths,
				__private uchar const syns_per_tuft_l2,
				//__global int* const aux_ints_0,
				__global uchar* const syn_states
) {
	//uint const slc_id = get_global_id(0);
	//uint const col_id = get_global_id(1);

	uint const slc_id = get_global_id(0);
	uint const u_id = get_global_id(1);
	uint const v_id = get_global_id(2);

	uint const col_id = mul24(u_id, v_id);

	uint const slc_columns = get_global_size(1);
	uint const l_id = get_local_id(1); 
	uint const wg_id = get_group_id(1);
	uint const wg_size = get_local_size(1);
	
	uint const base_col_id = mul24(wg_id, wg_size);
	uint const base_cel_idx = mad24(slc_id, slc_columns, base_col_id);

	uint const base_syn_idx = (base_cel_idx << syns_per_tuft_l2);
	uint const init_syn_idx = base_syn_idx + l_id;

	uint const syns_per_slc = slc_columns << syns_per_tuft_l2;
	uint const syns_per_wg = wg_size << syns_per_tuft_l2;

	uint const syn_n = base_syn_idx + syns_per_wg;

	int syn_sst_i = (base_col_id << syns_per_tuft_l2) + l_id;
	uint syn_idx = init_syn_idx;

	//uint auu_idx = mad24(slc_id, slc_columns, col_id); // DEBUG

	for (; syn_idx < syn_n; syn_idx += wg_size) {
		syn_sst_i -= mul24((int)syns_per_slc, (syn_sst_i >= syns_per_slc));
		int sst_pos = syn_sst_i >> syns_per_tuft_l2;
		uint axn_idx = axn_idx_2d(syn_src_slc_ids[syn_idx], slc_columns, sst_pos, syn_src_col_v_offs[syn_idx]);
		uchar axn_state = axn_states[axn_idx];
		
		syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);
		
		syn_sst_i += wg_size;


		//syn_states[syn_idx] = axn_state;
		//char syn_strength = syn_strengths[syn_idx];
		//syn_states[syn_idx] = mul24((syn_strength >= 0), ((axn_state != 0) << 7) + (axn_state >> 1));
		//aux_ints_0[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);
	}

	//aux_ints_0[0] = HORIZONTAL_AXON_ROW_DEMARCATION;

	//uint auu_idx = mad24(slc_id, slc_columns, col_id);
	//aux_ints_0[auu_idx] = l_id;
	//aux_ints_0[auu_idx] = syn_idx;
	//aux_ints_0[base_cel_idx] = 12321;
}*/



/*	FOR LATER:
 *
 *	NEEDS REWRITE
 *	OPTIMIZE FOR WORKGROUP
 *	VECTORIZE
 *
 */

/*	FOR NOW:
 *
 *	process synapse weights -> state
 *	determine raw value for ltping purposes (?) -> state_raw
 *	IMPLEMENT LEARNING REATTACHMENT
 *
 */
__kernel void den_cycle(
				__global uchar const* const syn_states,
				__global char const* const syn_strengths,
				__private uchar const syns_per_den_l2,
				__private uint const den_threshold,
				__global uchar* const den_energies,
				__global uchar* const den_states_raw,
				//__global int* const aux_ints_1,
				__global uchar* const den_states
) {
	uint const den_idx = get_global_id(0);
	//uint const u_id = get_global_id(1);
	//uint const v_id = get_global_id(2);
	//uint const slc_columns = mul24(get_global_size(1), get_global_size(2));
	//uint const den_idx = mad24(slc_id, slc_columns, mul24(u_id, v_id));
	uint const syn_idz = den_idx << syns_per_den_l2;

	/*uint const slc_id = get_global_id(0);
	uint const u_id = get_global_id(1);
	uint const v_id = get_global_id(2);
	uint const slc_columns = mul24(get_global_size(1), get_global_size(2));
	uint const den_idx = mad24(slc_id, slc_columns, mul24(u_id, v_id));
	uint const syn_idz = den_idx << syns_per_den_l2;*/

	uchar den_energy = den_energies[den_idx];

	int syn_sum = 0;
	int syn_sum_raw = 0;

	int const n = (1 << syns_per_den_l2);

	for (int i = 0; i < n; i += 1) {
		char syn_strength = syn_strengths[syn_idz + i];

		//uchar syn_state = mul24((syn_states[syn_idz + i] > 0), 1);
		uchar syn_state = syn_states[syn_idz + i]; 

		//syn_sum += syn_state;
		syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum); 
		
		syn_sum_raw += syn_state;
	}
	
	syn_sum = mul24((syn_sum > den_threshold), syn_sum);


	if (syn_sum != 0) {
		if (den_energy >= ENERGY_LEVEL_MIN) {
			den_energy -= ENERGY_LEVEL_MIN;
			//output_state = best1_den_state; 	// NEW
		} else {
			den_energy += ENERGY_REGEN_AMOUNT;
			syn_sum = 0; 						// NEW
		}
	} else {
		if (den_energy < ENERGY_LEVEL_MAX) {
			den_energy += ENERGY_REGEN_AMOUNT;
		}
	}


	den_states_raw[den_idx] = clamp((syn_sum_raw >> 7), 0, 255); 
	den_states[den_idx] = clamp((syn_sum >> 7), 0, 255); 

	//den_states_raw[den_idx] = clamp(syn_sum_raw, 0, 255); 	// UNUSED
	//den_states[den_idx] = clamp(syn_sum, 0, 255);	 			// UNUSED

	//aux_ints_1[den_idx] = clamp((syn_sum >> 7), 0, 255); 		// DEBUGGING
}



__kernel void den_cycle_old(
				__global uchar const* const syn_states,
				__global char const* const syn_strengths,
				__private uchar const syns_per_den_l2,
				__private uint const den_threshold,
				__global uchar* const den_energies,
				__global uchar* const den_states_raw,
				//__global int* const aux_ints_1,
				__global uchar* const den_states
) {
	uint const slc_id = get_global_id(0);
	uint const den_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const den_idx = mad24(slc_id, slc_columns, den_id);
	uint const syn_idz = den_idx << syns_per_den_l2;

	uchar den_energy = den_energies[den_idx];

	int syn_sum = 0;
	int syn_sum_raw = 0;

	int const n = (1 << syns_per_den_l2);

	for (int i = 0; i < n; i += 1) {
		char syn_strength = syn_strengths[syn_idz + i];

		//uchar syn_state = mul24((syn_states[syn_idz + i] > 0), 1);
		uchar syn_state = syn_states[syn_idz + i]; 

		//syn_sum += syn_state;
		syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum); 
		
		syn_sum_raw += syn_state;
	}
	
	syn_sum = mul24((syn_sum > den_threshold), syn_sum);


	if (syn_sum != 0) {
		if (den_energy >= ENERGY_LEVEL_MIN) {
			den_energy -= ENERGY_LEVEL_MIN;
			//output_state = best1_den_state; 	// NEW
		} else {
			den_energy += ENERGY_REGEN_AMOUNT;
			syn_sum = 0; 						// NEW
		}
	} else {
		if (den_energy < ENERGY_LEVEL_MAX) {
			den_energy += ENERGY_REGEN_AMOUNT;
		}
	}

	safe_cel_state_3d(5, 5, 5, 5, 5, 5, 5, den_states);



	den_states_raw[den_idx] = clamp((syn_sum_raw >> 7), 0, 255); 
	den_states[den_idx] = clamp((syn_sum >> 7), 0, 255); 

	//den_states_raw[den_idx] = clamp(syn_sum_raw, 0, 255); 	// UNUSED
	//den_states[den_idx] = clamp(syn_sum, 0, 255);	 			// UNUSED

	//aux_ints_1[den_idx] = clamp((syn_sum >> 7), 0, 255); 		// DEBUGGING
}


/* 	//##################### DEN_CYCLE(): WORKSPACE ##############################

// EXPERIMENTAL ENERGY CODE -- WORKING -- (FROM PYR_CYCLE)
	if (input_state > 0) {
		if (energy >= 9) {
			energy -= 9;
			output_state = best1_den_state;
		} else {
			energy += 1;
		}
	} else {
		if (energy < 255) {
			energy += 1;
		}
	}

// EXPERIMENTAL ENERGY CODE (FROM DEN_CYCLE)

	if (input_state > 0) {
		if (energy >= ENERGY_DRAIN) {
			energy -= ENERGY_DRAIN;
		} else {
			output_state = 0;
			energy += ENERGY_RECHARGE;
		}
	} else {
		if (energy < 255) {
			energy += ENERGY_RECHARGE;
		}
	}

//###############################################################

__kernel void den_cycle_original(
				__global uchar const* const syn_states,
				__global char const* const syn_strengths,
				__private uchar const syns_per_den_l2,
				__private uint const den_threshold,
				__global uchar* const den_energies,
				__global uchar* const den_states_raw,
				//__global int* const aux_ints_1,
				__global uchar* const den_states
) {
	uint const slc_id = get_global_id(0);
	uint const den_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const den_idx = mad24(slc_id, slc_columns, den_id);
	uint const syn_idz = den_idx << syns_per_den_l2;

	int syn_sum = 0;
	int syn_sum_raw = 0;

	int const n = (1 << syns_per_den_l2);

	for (int i = 0; i < n; i += 1) {
		char syn_strength = syn_strengths[syn_idz + i];

		//uchar syn_state = mul24((syn_states[syn_idz + i] > 0), 1);
		uchar syn_state = syn_states[syn_idz + i]; 

		//syn_sum += syn_state;
		syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum); 
		
		syn_sum_raw += syn_state;
	}
	
	syn_sum = mul24((syn_sum > den_threshold), syn_sum);



	//den_states_raw[den_idx] = clamp(syn_sum_raw, 0, 255); // UNUSED
	//den_states[den_idx] = clamp(syn_sum, 0, 255); // UNUSED

	den_states_raw[den_idx] = clamp((syn_sum_raw >> 7), 0, 255); 
	den_states[den_idx] = clamp((syn_sum >> 7), 0, 255); 


	//aux_ints_1[den_idx] = clamp((syn_sum >> 7), 0, 255);
}
*/ //################################ END #######################################



/*__kernel void inhib_simple_original(
				__global uchar const* const cel_states,
				//__global uchar* const iinn_states,
				//__global uchar* const iinn_cel_ids

				// GET SST BASE AXON LAYER

				__private uchar const cel_base_axn_slc,		// <<<<< DEPRICATE: USE A GLOBAL OFFSET

				__global int* const aux_ints_1,
				__global uchar* const axn_states
) {
	uint const slc_id = get_global_id(0);	// <<<<< TODO: USE A GLOBAL OFFSET
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);

	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const cel_idx = cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id);
	uint const axn_idx = axn_idx_3d(slc_id + cel_base_axn_slc, v_size, v_id, 0, u_size, u_id, 0);

	uchar const cel_state = cel_states[cel_idx];

	int const radius_pos = 4; // 61 Cells
	int const radius_neg = 0 - radius_pos;

	int i_am_the_big_dog = 1;
	//int biggest_dog = 0;

	//uint dumb_iter = 0;

	for (int u = radius_neg; u <= radius_pos; u++) {
		int u_neg = 0 - u;
		int v_z = max(radius_neg, u_neg - radius_pos);
		int v_m = min(radius_pos, u_neg + radius_pos);

		for (int v = v_z; v <= v_m; v++) {

			int neighbor_state = safe_cel_state_3d(slc_id, v_size, v_id, v, u_size, u_id, u, cel_states);			

			// if (neighbor_state < cel_state) {
			// 	i_am_the_big_dog = 0;
			// }

			i_am_the_big_dog &= (neighbor_state <= cel_state);


			// if ((v_id == 8) && (u_id == 8)) {
			// 	aux_ints_1[dumb_iter] = safe_cel_state_3d(slc_id, 
			// 		v_size, v_id, v, u_size, u_id, u, cel_states);
			// }

			//dumb_iter += 1;

			if (cel_idx == 384) {
				aux_ints_1[axn_idx] = neighbor_state;
			}
		}
	}

	//axn_states[axn_idx] = mul24(i_am_the_big_dog, (int)cel_state);
	axn_states[axn_idx] = cel_states[cel_idx];

}*/


// 	INHIB_SIMPLE(): Cell Inhibition - reads from soma, writes to axon
//		- If any nearby cells are more active (have a higher soma 'state')
//			- cell will not 'fire'
//			- otherwise, write soma (cel_states[cel_idx]) to axon (axn_states[axn_idx])
//
//		- Overly simplistic algorithm 
// 			- Distance should be taken into account when state is considered
//			- Search area broadened
// 		- Horribly unoptimized, Should:
//			- Cache values for an area in local (workgroup) memory
//			- be vectorized
//			- use a few other hex grid tricks (see written notes 03-Jun)
__kernel void inhib_simple(
				__global uchar const* const cel_states,
				//__global uchar* const iinn_states,
				//__global uchar* const iinn_cel_ids

				// GET SST BASE AXON LAYER

				__private uchar const cel_base_axn_slc,		// <<<<< DEPRICATE: USE A GLOBAL OFFSET

				__global int* const aux_ints_1,
				__global uchar* const axn_states
) {
	uint const slc_id = get_global_id(0);	// <<<<< TODO: USE A GLOBAL OFFSET
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);

	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const cel_idx = cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id);
	uint const axn_idx = axn_idx_3d(slc_id + cel_base_axn_slc, v_size, v_id, 0, u_size, u_id, 0);

	uchar const cel_state = cel_states[cel_idx];

	int const radius_pos = 4; // 61 Cells
	int const radius_neg = 0 - radius_pos;

	int unsuppressed = cel_state > 0;
	//uchar biggest_neighbor = 0;

	//uint dumb_iter = 0;

	for (int v = radius_neg; v <= radius_pos; v++) {
		int v_neg = 0 - v;
		int u_z = max(radius_neg, v_neg - radius_pos);
		int u_m = min(radius_pos, v_neg + radius_pos);

		for (int u = u_z; u <= u_m; u++) {

			uchar neighbor_state = safe_cel_state_3d(slc_id, v_size, v_id, v, u_size, u_id, u, cel_states);	// ORIGINAL		
			///uchar neighbor_state = cel_states[cel_idx_3d_unsafe(slc_id, v_size, v_id + v, u_size, u_id + u)]; // DEBUG


			// STREAMLINE ME
			/*if ((neighbor_state > cel_state) && (cel_state > 0)) {
				unsuppressed = 0;
			}*/


			//unsuppressed |= (neighbor_state > cel_state); // DEBUG
			unsuppressed &= (neighbor_state <= cel_state); // ORIGINAL

			// DEBUG
			// if (neighbor_state > 0) {
			// 	biggest_neighbor = neighbor_state;
			// }


			// [DEBUG]: PICK ONLY A FEW POINTS
			/*if (((v_id == 10) && (u_id == 10)) 
				|| ((v_id == 20) && (u_id == 20)) 
				|| ((v_id == 30) && (u_id == 30))
				|| ((v_id == 40) && (u_id == 40))) {
				uint unsafe_target_axn_idx = axn_idx_3d(slc_id + cel_base_axn_slc, v_size, v_id, v, u_size, u_id, u);

				//aux_ints_1[dumb_iter] = safe_cel_state_3d(slc_id, 
				//	v_size, v_id, v, u_size, u_id, u, cel_states);
				//aux_ints_1[unsafe_target_axn_idx] = 1;
				//axn_states[unsafe_target_axn_idx] = neighbor_state;
				axn_states[unsafe_target_axn_idx] = 1 + unsuppressed;
			}
			
			dumb_iter += 1;
			*/

			

			/*if (cel_idx == 384) {
				aux_ints_1[axn_idx + mad24(v, 100, u)] = neighbor_state + 1;
			}*/
		}
	}

	axn_states[axn_idx] = mul24((uint)unsuppressed, (uint)cel_state); // ORIGINAL *****
	//axn_states[axn_idx] = cel_states[cel_idx]; // DEBUG *****
	//axn_states[axn_idx] = unsuppressed; // DEBUG
	//axn_states[axn_idx] = biggest_dog; // DEBUG

}



// <<<<< FOLLOWING SECTION SLATED FOR REMOVAL/DEPRICATION >>>>>

__kernel void peak_sst_cycle_pre(
				__global uchar const* const sst_states,
				__global uchar* const asp_states,
				__global uchar* const asp_sst_ids
	
) {
	uint const slc_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const asp_pos = mad24(slc_id, slc_columns, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);
	uint const sst_ofs = asp_pos << ASPINY_SPAN_LOG2;

	uchar sst_states_vec[1 << (ASPINY_REACH_LOG2)];

	uchar winner_val = 0;
	uchar winner_id = 0;
	
	uchar val = 0;
	uchar id = 0;

		#pragma unroll
	for (uint i = 0; i < ASPINY_SPAN; i += 4) {
		vstore4(vload4((sst_ofs + i) >> 2, sst_states), 0, sst_states_vec);

			#pragma unroll
		for (uint j = 0; j < 4; j++) {
			val = sst_states_vec[j];
			id = j + i;

			if (val <= winner_val) {
				continue;
			} else {
				winner_val = val;
				winner_id = ((sst_ofs + id) & 0xFF);
			}
		}
	}
	
	asp_states[asp_idx] = winner_val;
	asp_sst_ids[asp_idx] = winner_id;		// | (winner_val & 0xF8);
}



/* 
TODO:
	- REWRITE (REVERSE) IF/ELSE BRANCHES TO OPTIMIZE BETTER
	- VECTORIZE

FUTURE IMPROVEMENTS:
	- MOVE TO 3D (2D INPUT SPACE)
		- USE 4X4 PRIMARY GRID AND 4X4 (16X16) PERIPH
			- MUST WIN 13/16 (?) TO SURVIVE
		- OR USE 4X4 PRIMARY AND 3X3 (12X12) PERIPH
			- MUST WIN 10/12
*/
__kernel void peak_sst_cycle_wins(
				__global uchar* const asp_states,
				__global uchar* const asp_wins
) {
	uint const slc_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const asp_pos = mad24(slc_id, slc_columns, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);

	uint const as_bitmask = (ASPINY_SPAN - 1);

	uchar asp_state = asp_states[asp_idx];
	uchar asp_win = asp_wins[asp_idx];

	int win_count = asp_win; // asp_wins[asp_idx];

	for (uint i = 0; i < ASPINY_SPAN; i++) {
		uint cur_comp_idx = (asp_idx - ASPINY_REACH) + i + (i > (ASPINY_REACH - 1));
		uchar cur_comp_state = asp_states[cur_comp_idx];
		uchar cur_comp_win = asp_wins[cur_comp_idx];

		if (asp_win == cur_comp_win) {
			if (asp_state > cur_comp_state) {
				win_count += 1;
			}

			if ((asp_state == cur_comp_state) && (asp_state > 0)) {		// OLD (TIEBREAK) VERSION
				if ((asp_idx & as_bitmask) == (asp_state & as_bitmask)) {
					win_count += 1;
				} else if ((cur_comp_idx & as_bitmask) != (asp_state & as_bitmask)) {
					win_count += ((asp_idx) < (cur_comp_idx));
				}
			} else if (asp_state > cur_comp_state) {
				win_count += 1;
			}
		} else if (asp_win > cur_comp_win) {
			win_count += 1;
		} else {
			win_count = 0;
			asp_state = 0;
			break;
		}
		
	}

	asp_wins[asp_idx] = win_count;
	asp_states[asp_idx] = asp_state;
}


__kernel void peak_sst_cycle_post(
				__global uchar* const asp_wins,
				//__global uchar* const asp_sst_ids,
				__global uchar* const asp_states
				//__global uchar* const sst_states
) {
	uint const slc_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const asp_pos = mad24(slc_id, slc_columns, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);
	//uint const sst_ofs = asp_pos << ASPINY_SPAN_LOG2;

	//uchar asp_state = asp_states[asp_idx];
	uchar const asp_win = asp_wins[asp_idx];

	asp_states[asp_idx] = asp_win;
	asp_wins[asp_idx] = 0;
}



// VECTORIZE ME
// RENAME ME
// CLEAN ME UP
	//__attribute__((reqd_work_group_size(1, AXONS_WORKGROUP_SIZE, 1)))
__kernel void sst_post_inhib_unoptd (										
				__global uchar const* const asp_sst_ids,
				__global uchar const* const asp_states,
				__global uchar const* const asp_wins,
				__private uchar const sst_axn_slc,
				__global uchar* const sst_states,
				__global uchar* const axn_states
) {
	uint const slc_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const sst_idx = mad24(slc_id, slc_columns, col_id);
	uint const axn_idx = axn_idx_2d(sst_axn_slc, slc_columns, col_id, 0);
	//uint const axn_idx = mad24(sst_axn_slc, slc_columns, sst_idx + (uint)SYNAPSE_REACH_LIN);
	uint const asp_idx = (sst_idx >> ASPINY_SPAN_LOG2) + ASPINY_REACH;

	uchar const asp_state = asp_states[asp_idx];
	uchar const sst_state = sst_states[sst_idx];

	int win = (asp_sst_id_to_sst_idx(asp_idx, (asp_sst_ids[asp_idx])) == sst_idx);
	win = (win && asp_state);

	//sst_states[sst_idx] = mul24(sst_state, (win > 0));
	//axn_states[axn_idx] = 128;

	//sst_states[sst_idx] = mul24(sst_state, (win > 0));
	axn_states[axn_idx] = mul24(sst_state, (win > 0));
}

// <<<<< END REMOVE/DEPRICATE SECTION >>>>>







// SST_LTP(): Long term potentiation for Spiny Stellate Cells
__kernel void sst_ltp(
				__global uchar const* const axn_states,
				__global uchar const* const syn_states,
				//__global uchar const* const sst_states,
				__private uint const cel_axn_idz,
				__private uchar const syns_per_tuft_l2,
				//__private uint const cels_per_tuft,
				__private uint const rnd,
				//__global int* const aux_ints_0,
				__global char* const syn_strengths
) {
	uint const slc_id = get_global_id(0);
	uint const col_tuft_id = get_global_id(1);
	uint const tuft_size = get_global_size(1);
	uint const cel_tuft_id = mad24(slc_id, tuft_size, col_tuft_id);

	uint cels_per_tuft = get_local_size(1);

	uint const cel_idz = mul24(cel_tuft_id, cels_per_tuft);
	uint const cel_idn = cel_idz + cels_per_tuft;

	for (uint cel_idx = cel_idz; cel_idx < cel_idn; cel_idx++) {
		uchar axn_state = axn_states[cel_axn_idz + cel_idx];
		
		if (axn_state) {
			uint syn_idz = cel_idx << syns_per_tuft_l2;	
			prx_syns__active__ltp_ltd(syn_states, syn_idz, syns_per_tuft_l2, rnd, syn_strengths);
		}

		//aux_ints_0[cel_idx] = axn_state;
	}
}


// PYR_ACTIVATE(): CONVERT TO 1 WORK_DIM
__kernel void pyr_activate(
				//__global uchar const* const mcol_states, // COL
				__global uchar const* const mcol_pyr_pred_flags, // COL
				__global uchar const* const mcol_best_pyr_den_states,
				__global uchar const* const pyr_best_den_ids,
				// ADD PYR BEST DEN STATE NOW THAT WE'VE ADDED IT (and to another kernel somewhere also)
				__global uchar const* const den_states,

				__private uint const ssts_axn_idz,
				__private uchar const pyr_axn_slc_base,
				__private uchar const dens_per_tuft_l2,
				
				//__global uchar* const pyr_energies,
				__global uchar* const pyr_flag_sets,
				__global uchar* const pyr_preds,
				//__global int* const aux_ints_0,
				__global uchar* const axn_states
) {
	uint const slc_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const pyr_idx = mad24(slc_id, slc_columns, col_id);
	uint const axn_idx = axn_idx_2d(pyr_axn_slc_base + slc_id, slc_columns, col_id, 0);

	uint const den_ofs = pyr_idx << dens_per_tuft_l2;			// REPLACE
	uint const best_den_idx = den_ofs + pyr_best_den_ids[pyr_idx];		// REPLACE

	uchar const best_den_state = den_states[best_den_idx];				// CHANGE

	//uint const axn_idx = mad24(pyr_axn_slc_base + slc_id, slc_columns, col_id + (uint)SYNAPSE_REACH_LIN);
	uchar const mcol_best_col_den_state = mcol_best_pyr_den_states[col_id];
	uchar const sst_axn_state = axn_states[ssts_axn_idz + col_id];
	//uchar const mcol_state = mcol_states[col_id];
	uchar const mcol_pyr_pred_flag = mcol_pyr_pred_flags[col_id];
	uchar const pyr_pred = pyr_preds[pyr_idx];
	uchar pyr_flag_set = pyr_flag_sets[pyr_idx];

	//aux_ints_0[pyr_idx] = pyr_flag_set;

	int const mcol_active = sst_axn_state != 0;
	//int const mcol_active = mcol_state != 0;
	int const mcol_any_pred = mcol_pyr_pred_flag != 0;
	int const pyr_predictive = (pyr_pred != 0);

	int const crystal = pyr_predictive && mcol_active;
	int const anomaly = mcol_active && !mcol_any_pred;

	//int const activate_axon = crystal || anomaly;
	//pyr_pred = (crystal | anomaly) && (mcol_state);
	//pyr_pred = mul24(((crystal != 0) || (anomaly != 0)), mcol_state);
	pyr_flag_set &= ~PYR_BEST_IN_COL_FLAG;
	
	pyr_flag_set |= mul24(mcol_best_col_den_state == best_den_state, PYR_BEST_IN_COL_FLAG);
	//pyr_flag_set |= mul24((mcol_best_col_den_state == best_den_state) && pyr_predictive, PYR_BEST_IN_COL_FLAG);


	// SHOULDN'T BE ACTIVATING IF OTHER PYRS IN COLUMN ARE PREDICTIVE

	axn_states[axn_idx] = (uchar)mad24(anomaly, (int)sst_axn_state, mul24(crystal, (int)pyr_pred));
	//axn_states[axn_idx] = (uchar)mad24(anomaly, (int)mcol_state, mul24(crystal, (int)pyr_pred));

	pyr_flag_sets[pyr_idx] = pyr_flag_set;

	//pyr_preds[pyr_idx] = pyr_pred;

	//aux_ints_0[pyr_idx] = 5;
	//aux_ints_0[pyr_idx] = pyr_pred;
}


// PYRS_LTP(): Pyramidal long term potentiation and depression - adjusting synapse strengths
//
//	- For each pyramidal cell:
//		- if cell axon is currently active:
//			- cause learning to take place on it's most active dendrite
//		- if cell axon is currently inactive:
//			- check to see if the cell's axon was previously active (by checking flag_set)
//				- if so, depress (reduce strengths of) any currently active synapses
//					- NOTE: The reasoning here is that any synapses which are active just after (but not before) the cell was active are likely to be unrelated to it's prior activity. In other words, a rough implementation of LTD (simplified and optimized and theorized and ... oh who knows). 
//
//	- TODO:
//		- Vectorize (should be highly vectorizable)
//		- reducing branching will be tough with this one
//		- Tests (check that flag_set and prev_best_den_id are robustly maintained)
//
//		- Let's shit on these goddamn menace fucking constantly active goddamn fucking inputs
//			- The root of the problem is the propensity to build on whatever other activity is happening from your neighbors. This activity breeds even more activity and it positively feeds back
//			- If we take a dump on synaptic inputs which are active when we are inactive... it should shave some of the bullshit off
//			- Constrain this to act in very few circumstances
//
//		##########      CAN'T EQUATE AXON OUTPUT WITH PYR DEPOLS       #############
//
//
//		- if pyr_prev_concrete 
//			- if pyr_concrete
//			- if pyr_pred
//
//		- if pyr_prev_pred
//			- if pyr_concrete
//			- if pyr_pred
//
//	- Misc Notes:
//
//		- SYN(    -> STP) WHEN: (SYN_STATE > 0) AND (PYR_TANGIBLE) AND (PYR_BEST_IN_COLUMN)
//		                    OR: (SYN_STATE > 0) AND (PYR_TANGIBLE) AND (PYR_PREV_PRED)
//
//		- MAINTAIN STP STATE AS LONG AS: (SYN_STATE > 0) AND (PYR_ACTIVE)
//
//		- SYN(STP -> LTP) ONLY WHEN: ((PYR_ACTIVE -> 0)) SAME TIME AS (SYN_STATE -> 0)
//			
// 	TODO: CONVERT TO 1 WORK_DIM
__kernel void pyrs_ltp_unoptd(
				__global uchar const* const axn_states,
				__global uchar const* const pyr_preds,
				__global uchar const* const pyr_best_den_ids,
				//__global uchar const* const pyr_best2_den_ids, // <<<<< SLATED FOR REMOVAL
				__global uchar const* const den_states,
				__global uchar const* const syn_states,
				__private uint const pyr_axn_idx_base, 
				__private uint const syns_per_den_l2,
				__private uint const dens_per_tuft_l2,
				__private uint const pyrs_per_wi,
				__private uint const rnd,
				__global uchar* const syn_flag_sets,
				__global uchar* const pyr_flag_sets,
				//__global uchar* const pyr_prev_best_den_ids,
				//__global int* const aux_ints_0,
				//__global int* const aux_ints_1,
				__global char* const syn_strengths
) {
	uint const slc_id = get_global_id(0);
	uint const col_tuft_id = get_global_id(1);
	uint const tufts_per_slc = get_global_size(1);
	uint const pyr_tuft_id = mad24(slc_id, tufts_per_slc, col_tuft_id);
	//uint const den_ofs = pyr_idx << DENDRITES_PER_CELL_DISTAL_LOG2;

	//uint const axn_idx_base = mad24(pyr_axn_slc_base + slc_id, slc_columns, col_id + (uint)SYNAPSE_REACH_LIN);

	uint const pyr_idz = mul24(pyr_tuft_id, pyrs_per_wi);
	uint const pyr_idn = pyr_idz + pyrs_per_wi;

	//uint const pyr_idz = mul24(pyr_tuft_id, pyrs_per_wi);
	//uint const pyr_idx_n = pyr_idz + pyrs_per_wi;

	//uint debug_output = 0;
 
	for (uint i = pyr_idz; i < pyr_idn; i++) {
		uchar pyr_best_den_id = pyr_best_den_ids[i];
		//uchar pyr_best2_den_id = pyr_best2_den_ids[i]; // <<<<< SLATED FOR REMOVAL
		//uchar pyr_prev_best_den_id = pyr_prev_best_den_ids[i];
		uchar pyr_flag_set = pyr_flag_sets[i];

		int pyr_concrete = axn_states[i + pyr_axn_idx_base] != 0;
		int pyr_fuzzy = pyr_preds[i] != 0;

		int pyr_prev_concrete = (pyr_flag_set & PYR_PREV_CONCRETE_FLAG) == PYR_PREV_CONCRETE_FLAG;
		//int pyr_prev_stp = (pyr_flag_set & PYR_PREV_STP_FLAG) == PYR_PREV_STP_FLAG;
		int pyr_prev_fuzzy = (pyr_flag_set & PYR_PREV_FUZZY_FLAG) == PYR_PREV_FUZZY_FLAG;
		int pyr_best_in_col = (pyr_flag_set & PYR_BEST_IN_COL_FLAG) == PYR_BEST_IN_COL_FLAG;

		uint den_idx_base = i << dens_per_tuft_l2;

		uint pyr_syn_idz = ((den_idx_base) << syns_per_den_l2);	 // WHOLE CELL
		uint best_den_syn_idz = (den_idx_base + pyr_best_den_id) << syns_per_den_l2;
		//uint best2_den_syn_idz = (den_idx_base + pyr_best2_den_id) << syns_per_den_l2; // <<<<< SLATED FOR REMOVAL
		//uint prev_best_den_syn_idz = (den_idx_base + pyr_prev_best_den_id) << syns_per_den_l2;


		//int pyr_ano = !pyr_prev_fuzzy && pyr_concrete;
		//int pyr_cry = pyr_prev_fuzzy && pyr_concrete;
		//int pyr_trm = pyr_prev_concrete && !pyr_concrete;

		//aux_ints_1[i] = pyr_flag_set;


		//uchar learned_what = 0;


		if (pyr_concrete) {
			if (pyr_prev_fuzzy) { // PREVIOUS (CORRECT) PREDICTION (EVERY PYR IN COL): REINFORCE DEN + TRAIN NEW DEN
				// SAME AS ANO + TRAIN A SECOND REDUNDANT DENDRITE AS WELL (OR DON'T)
				
				dst_syns__active__stp_ltd(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
				
				//dst_syns__active__stp_ltd(syn_states, best2_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);

				//learned_what = 1;

			} else if (pyr_best_in_col) { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN 
			//} else { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
				
				dst_syns__active__stp_ltd(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);


				/*int iz = best_den_syn_idz;
				int in = iz + (1 << syns_per_den_l2);
				for (; iz < in; iz ++) {
					aux_ints_0[iz] = 99;
				}*/

				//learned_what = 2;

			//} else { // EVERYTHING ELSE: JUST SET PREV ACTIVE
				// NOT GOING TO WORRY ABOUT THIS -- ALLOW STP TO REFLECT PRIOR ACTIVITY
				// dst_syns__active__set_prev_active(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
			}

			//pyr_flag_set |= PYR_PREV_STP_FLAG;
			pyr_flag_set |= PYR_PREV_CONCRETE_FLAG;

		} else if (pyr_prev_concrete) { // TRM	

			cel_syns_trm(syn_states, pyr_syn_idz, syns_per_den_l2 + dens_per_tuft_l2, rnd, syn_flag_sets, syn_strengths);

			//learned_what = 3;

			//pyr_flag_set &= ~PYR_PREV_STP_FLAG;
			pyr_flag_set &= ~PYR_PREV_CONCRETE_FLAG;
		}


		/*uint syn_idx = ((den_idx_base) << syns_per_den_l2);	 // WHOLE CELL
		dst_syns_ltd_active(syn_states, syn_idx, (syns_per_den_l2 + dens_per_tuft_l2), rnd, syn_flag_sets, syn_strengths);*/

		//pyr_flag_set &= ~PYR_PREV_FUZZY_FLAG;

		//pyr_prev_best_den_id = pyr_best_den_id;
		//pyr_prev_best_den_ids[i] = pyr_prev_best_den_id;
		

		pyr_flag_set &= ~PYR_PREV_FUZZY_FLAG;
		pyr_flag_set |= mul24(pyr_fuzzy, PYR_PREV_FUZZY_FLAG);

		pyr_flag_sets[i] = pyr_flag_set;
		

		// TOTAL PYRS:
		//aux_ints_1[i] = tufts_per_slc * pyrs_per_wi * get_global_size(0);

		//aux_ints_1[i] = mul24(pyr_concrete, (int)pyr_axn_idx_base);
		//aux_ints_1[i] = mul24(pyr_concrete, axn_states[i + pyr_axn_idx_base]);
		//aux_ints_1[i] = (learned_what * 10) + (pyr_flag_set & 0xFF);

		//aux_ints_1[i] = aux_ints_1[i] + (10 * learned_what);
		//aux_ints_1[i] = pyr_fuzzy;
		//aux_ints_1[i] = pyr_axn_idx_base;
		//aux_ints_1[i] = learned_what;

	}
}




__kernel void pyr_cycle(
				__global uchar const* const den_states,
				__global uchar const* const den_states_raw,
				__private uint const den_tufts_per_cel,
				__private uchar const dens_per_tuft_l2,
				//__private uchar const pyr_axn_slc_base,
				//__global uchar* const pyr_energies,	// <<<<< SLATED FOR REMOVAL
				__global uchar* const pyr_best_den_ids,
				__global uchar* const pyr_best_den_states,
				//__global uchar* const pyr_best2_den_ids,
				//__global uchar* const pyr_best2_den_states,
				__global uchar* const pyr_preds
				//__global uchar* const axn_states
) {
	//uint const slc_id = get_global_id(0);
	//uint const col_id = get_global_id(1);
	//uint const slc_columns = get_global_size(1);
	//uint const pyr_idx = mad24(slc_id, slc_columns, col_id);

	uint const pyr_idx = get_global_id(0);
	
	//uint const axn_idx = axn_idx_2d(pyr_axn_slc_base + slc_id, slc_columns, col_id);
	//uint const axn_idx = mad24(pyr_axn_slc_base + slc_id, slc_columns, col_id + (uint)SYNAPSE_REACH_LIN);
	//uchar pyr_energy = pyr_energies[pyr_idx];	// <<<<< SLATED FOR REMOVAL

	//uint den_sum = 0;

	uchar best_den_state = 0;
	uchar best_den_id = 0;

	//uchar best2_den_state = 0;
	//uchar best2_den_id = 0;
	//int active_dendrites = 0;

	uchar pyr_state = 0;


	//uint pyr_pred = pyr_preds[pyr_idx];

	for (uint den_tuft = 0; den_tuft < den_tufts_per_cel; den_tuft++) {
		uint const den_idz = mad24(den_tuft, get_global_size(0), pyr_idx) << dens_per_tuft_l2;
 
		for (uint den_idx = 0; den_idx < (1 << dens_per_tuft_l2); den_idx++) {
			uchar den_state = den_states[den_idz + den_idx];
			int den_state_bigger = (den_state > best_den_state);

			//best2_den_id = mad24(den_state_bigger, best_den_id, mul24(!den_state_bigger, best2_den_id));
			//best2_den_state = mad24(den_state_bigger, best_den_state, mul24(!den_state_bigger, best2_den_state));

			best_den_id = mad24(den_state_bigger, (int)den_idx, mul24(!den_state_bigger, best_den_id));
			best_den_state = mad24(den_state_bigger, den_state, mul24(!den_state_bigger, best_den_state));

			//best_den_state = mul24(den_state_bigger, den_state);

			//den_sum += den_state;
			//den_sum += (den_state != 0);
			//den_sum += (den_state > 0);
			//active_dendrites += (den_state > 0);
		}
	}
		
	// EXPERIMENTAL ENERGY CODE  // <<<<< SLATED FOR REMOVAL
	
	/*if (best_den_state > 0) {
		if (pyr_energy >= 9) {
			pyr_energy -= 9;
			pyr_state = best_den_state;
		} else {
			pyr_energy += 1;
		}
	} else {
		if (pyr_energy < 255) {
			pyr_energy += 1;
		}
	}*/

	pyr_state = best_den_state; // YEAH... I KNOW...


	//pyr_energies[pyr_idx] = pyr_energy;	 // <<<<< SLATED FOR REMOVAL
	pyr_best_den_ids[pyr_idx] = best_den_id;
	pyr_best_den_states[pyr_idx] = best_den_state;
	//pyr_best2_den_ids[pyr_idx] = best2_den_id;
	//pyr_best2_den_states[pyr_idx] = best2_den_state;
	pyr_preds[pyr_idx] = pyr_state;


	//pyr_preds[pyr_idx] = clamp(den_sum, 0u, 255u); 	// v.N1
	//axn_states[axn_idx] = clamp(den_sum, 0u, 255u);

	//pyr_preds[pyr_idx] = clamp(den_sum, 0, 127);

	//pyr_preds[pyr_idx] = (den_sum >> 1);
	//pyr_preds[pyr_idx] = active_dendrites;
}



//	COL_OUTPUT()
//		- rename coming
//
// 
__kernel void col_output(
				//__global uchar const* const sst_states,	// [done] CONVERT TO READING FROM AXON

				__global uchar const* const pyr_preds,
				__global uchar const* const pyr_best_den_states,
				//__private uchar const sst_slc_count,
				__private uint const sst_axn_idz,
				__private uchar const pyr_depth,
				//__private uchar const pyr_axn_base_slc,
				__private uchar const output_axn_slc,
				//__private uchar const pyr_base_slc,
				__global uchar* const mcol_pyr_pred_flags,
				__global uchar* const mcol_best_pyr_den_states,
				__global uchar* const axn_states
) {
	uint const slc_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const output_axn_idx = axn_idx_2d(output_axn_slc + slc_id, slc_columns, col_id, 0);
	//uint const output_axn_idx = mad24(output_axn_slc + slc_id, slc_columns, col_id + (uint)SYNAPSE_REACH_LIN);
	//uint const axn_idx_output = axn_idx_wrap_2d(axn_slc_output, col_id);
	uint const col_idx = mad24(slc_id, slc_columns, col_id);

	//int sst_state = sst_states[col_idx];
	int sst_axn_state = axn_states[sst_axn_idz + col_idx];
	uchar max_den_state = 0;
	int col_pyr_pred_total = 0;

	for (uint i = 0; i < pyr_depth; i++) {
		// POTENTIALLY FALSE ASSUMPTION HERE ABOUT PYR CELLS ALL BEING INVOLVED IN OUTPUT
		uint pyr_idx = mad24(i, slc_columns, col_id);	

		uchar pyr_best_den_state = pyr_best_den_states[pyr_idx];
		uchar pyr_pred = pyr_preds[pyr_idx];

		max_den_state = max(max_den_state, pyr_best_den_state);
		
		col_pyr_pred_total = max(col_pyr_pred_total, (int)pyr_pred);

		//col_pyr_pred_total += axn_states[axn_idx_pyr];						
		//col_pyr_pred_total = max(col_pyr_pred_total, (int)axn_states[axn_idx_pyr]); 
		
		//col_pyr_pred_total += (axn_states[axn_idx_pyr] > 0);
	}


	mcol_pyr_pred_flags[col_idx] = clamp(col_pyr_pred_total, 0, 255); // <<<<< FIX ME TO BE A FLAG
	mcol_best_pyr_den_states[col_idx] = max_den_state;



	//axn_states[output_axn_idx] = clamp(col_pyr_pred_total, 0, 255);
	//axn_states[output_axn_idx] = clamp(sst_state, 0, 255);
	axn_states[output_axn_idx] = clamp(col_pyr_pred_total + sst_axn_state, 0, 255);
}









/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
========================== EXPERIMENTAL AND UNUSED ============================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/




__kernel void cel_cycle(
				__global char const* const syn_strengths,
				__global uchar const* const axn_states,
				__private uint const den_threshold,
				__private uint const den_tufts_per_cel,
				__private uchar const dens_per_tuft_l2,
				__private uchar const syns_per_den_l2,
				__global uchar* const syn_states,
				__global uchar* const cel_energies,
				__global uchar* const cel_best_den_ids,
				__global uchar* const cel_states
) {
	uint const cel_idx = get_global_id(0);

	uchar best_den_state = 0;
	uchar best_den_id = 0;

	uchar cel_energy = cel_energies[cel_idx];

	for (uint den_tuft = 0; den_tuft < den_tufts_per_cel; den_tuft++) {
		uint const den_idz = mad24(den_tuft, den_tufts_per_cel, cel_idx) << dens_per_tuft_l2;

		for (uint den_idx = 0; den_idx < (1 << dens_per_tuft_l2); den_idx++) {
			uint const syn_idz = den_idx << syns_per_den_l2;
			int syn_sum = 0;
			int const n = (1 << syns_per_den_l2);

			for (int i = 0; i < n; i += 1) {
				char syn_strength = syn_strengths[syn_idz + i];
				
				uchar axn_state = 0; // THE BIG FISH: axn_states[axn_idx];

				uchar syn_state = ((axn_state != 0) << 7) + (axn_state >> 1); 

				syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum); 				
			}

			syn_sum = clamp((syn_sum >> 7), 0, 255);

			syn_sum = mul24((syn_sum > den_threshold), syn_sum);

			int den_state_is_bigger = (syn_sum > best_den_state);

			best_den_id = mad24(den_state_is_bigger, (int)den_idx, mul24(!den_state_is_bigger, best_den_id));
			best_den_state = mad24(den_state_is_bigger, syn_sum, mul24(!den_state_is_bigger, best_den_state));
		}
	}

	if (best_den_state != 0) {
		if (cel_energy >= ENERGY_LEVEL_MIN) {
			cel_energy -= ENERGY_LEVEL_MIN;
			//output_state = best1_den_state; 	// NEW
		} else {
			cel_energy += ENERGY_REGEN_AMOUNT;
			best_den_state = 0; 						// NEW
		}
	} else {
		if (cel_energy < ENERGY_LEVEL_MAX) {
			cel_energy += ENERGY_REGEN_AMOUNT;
		}
	}

	cel_best_den_ids[cel_idx] = best_den_id;
	//cel_best_den_states[cel_idx] = best_den_state;
	cel_states[cel_idx] = best_den_state;
}





/*static inline void dst_syns__active__stp_std_experimental( // ANOMALY & CRYSTALLIZATION
				__global uchar const* const syn_states,
				uint const syn_idx_start,	// (syn_idz)
				uint const syns_per_den_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global uchar* const syn_flag_sets,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		char syn_strength = syn_strengths[i];
		uchar syn_flag_set = syn_flag_sets[i];

		int const inc = rnd_inc(rnd, i, syn_strength);
		int const syn_active = syn_state != 0;
		syn_flag_set &= ~SYN_STP_FLAG;

		syn_flag_set |= mul24(syn_active, SYN_STP_FLAG);
		syn_flag_set |= mul24(!syn_active, SYN_STD_FLAG);
		//syn_strength -= mul24(!syn_active, inc << LTD_BIAS_LOG2);

		syn_flag_sets[i] = syn_flag_set;
		syn_strengths[i] = syn_strength;
	}
}*/




/*static inline void cel_syns_trm_experimental( // TERMINATION
				__global uchar const* const syn_states,
				uint const syn_idx_start,	// (syn_idz)
				uint const syns_per_tuft_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global uchar* const syn_flag_sets,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_tuft_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		char syn_strength = syn_strengths[i];
		uchar syn_flag_set = syn_flag_sets[i];
		int const inc = rnd_inc(rnd, i, syn_strength);
		//uchar const rnd_char = (rnd ^ i) & 0x7F;		
		//int const inc = (rnd_char > abs(syn_strength));
		int const syn_prev_stp = (syn_flag_set & SYN_STP_FLAG) == SYN_STP_FLAG;
		//int const syn_prev_std = (syn_flag_set & SYN_STD_FLAG) == SYN_STD_FLAG;
		int const syn_active = syn_state != 0;

		syn_strength += mul24(syn_prev_stp && !syn_active, inc << LTP_BIAS_LOG2);
		syn_strength -= mul24(syn_prev_stp || syn_active, inc);

		// if (syn_prev_stp) {
		// 	if (!syn_active) {
		// 		syn_strength += (inc << LTP_BIAS_LOG2);
		// 	}
		// 	if (syn_active || syn_prev_std) {
		// 		syn_strength -= (inc << LTD_BIAS_LOG2);			
		// 	} 
		// }

		syn_flag_set &= ~(SYN_STP_FLAG + SYN_STD_FLAG);

		syn_flag_sets[i] = syn_flag_set;
		syn_strengths[i] = syn_strength;
	}
}*/




__kernel void pyr_cycle_experimental( // EXPERIMENTAL VERSION
				__global uchar const* const den_states,
				__global uchar const* const den_states_raw,
				//__private uchar const pyr_axn_slc_base,
				__private uchar const dens_per_tuft_l2,
				__global uchar* const pyr_energies,
				__global uchar* const pyr_best_den_ids,
				__global uchar* const pyr_best_den_states,
				__global uchar* const pyr_best2_den_ids,
				__global uchar* const pyr_best2_den_states,
				__global uchar* const pyr_preds
				//__global uchar* const axn_states
) {
	uint const slc_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const pyr_idx = mad24(slc_id, slc_columns, col_id);
	uint const den_ofs = pyr_idx << dens_per_tuft_l2;
	//uint const axn_idx = axn_idx_2d(pyr_axn_slc_base + slc_id, slc_columns, col_id);
	//uint const axn_idx = mad24(pyr_axn_slc_base + slc_id, slc_columns, col_id + (uint)SYNAPSE_REACH_LIN);
	//uchar pyr_energy = pyr_energies[pyr_idx];

	//uint den_sum = 0;

	uchar best_den_state = 0;
	uchar best_den_id = 0;

	uchar best2_den_state = 0;
	uchar best2_den_id = 0;
	//int active_dendrites = 0;

	uchar den_max = 0;

	//uint pyr_pred = pyr_preds[pyr_idx];

		//#pragma unroll 
	for (uchar i = 0; i < (1 << dens_per_tuft_l2); i++) {
		uchar den_state = den_states[den_ofs + i];
		uchar den_state_raw = den_states_raw[den_ofs + i];

		int den_state_bigger = (den_state_raw > best_den_state);

		best2_den_id = mad24(den_state_bigger, best_den_id, mul24(!den_state_bigger, best2_den_id));
		best2_den_state = mad24(den_state_bigger, best_den_state, mul24(!den_state_bigger, best2_den_state));

		best_den_id = mad24(den_state_bigger, i, mul24(!den_state_bigger, best_den_id));
		best_den_state = mad24(den_state_bigger, den_state_raw, mul24(!den_state_bigger, best_den_state));

		//best_den_state = mul24(den_state_bigger, den_state);

		den_max = max(den_max, den_state);

		//den_sum += den_state;
		//den_sum += (den_state != 0);
		//den_sum += (den_state > 0);
		//active_dendrites += (den_state > 0);
	}
	
	//den_sum = den_sum >> 2;


	
	pyr_best_den_ids[pyr_idx] = best_den_id;
	pyr_best_den_states[pyr_idx] = best_den_state;
	pyr_best2_den_ids[pyr_idx] = best2_den_id;
	pyr_best2_den_states[pyr_idx] = best2_den_state;

	//pyr_preds[pyr_idx] = den_max; // ***** FUCKED WITH THIS
	pyr_preds[pyr_idx] = best_den_state; // ***** ^



	//pyr_preds[pyr_idx] = clamp(den_sum, 0u, 255u); 	// v.N1
	//axn_states[axn_idx] = clamp(den_sum, 0u, 255u);

	//pyr_preds[pyr_idx] = clamp(den_sum, 0, 127);

	//pyr_preds[pyr_idx] = (den_sum >> 1);
	//pyr_preds[pyr_idx] = active_dendrites;
}



__kernel void den_cycle_experimental(
				__global uchar const* const syn_states,
				__global char const* const syn_strengths,
				__private uint const syns_per_den_l2,
				__private uint const den_threshold,
				__global uchar* const den_energies,
				__global uchar* const den_states_raw,
				__global uchar* const den_states
) {
	uint const slc_id = get_global_id(0);
	uint const den_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const den_idx = mad24(slc_id, slc_columns, den_id);
	uint const syn_idz = den_idx << syns_per_den_l2;

	uchar den_energy = den_energies[den_idx];

	int syn_sum = 0;
	int syn_sum_raw = 0;

	int const n = (1 << syns_per_den_l2);

	for (int i = 0; i < n; i += 1) {
		char syn_strength = syn_strengths[syn_idz + i];

		//uchar syn_state = mul24((syn_states[syn_idz + i] > 0), 1); 
		uchar syn_state = syn_states[syn_idz + i]; 

		//syn_sum += syn_state; 
		syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum); 
		
		syn_sum_raw += syn_state;
	}
	
	syn_sum = mul24((syn_sum > den_threshold), syn_sum);

	//uchar den_state = clamp((syn_sum >> 7), 0, 255);
	
	// EXPERIMENTAL ENERGY CODE
	/*if (syn_sum > 0) {
		if (den_energy >= ENERGY_DRAIN) {
			//den_state = syn_sum;
			den_energy -= ENERGY_DRAIN;
		} else {
			den_state = 0;
			den_energy += ENERGY_RECHARGE;
		}
	} else {
		if (den_energy < 255) {
			den_energy += ENERGY_RECHARGE;
		}
	}*/


	den_energies[den_idx] = den_energy;
	den_states_raw[den_idx] = clamp((syn_sum_raw >> 7), 0, 255);
	den_states[den_idx] = clamp((syn_sum >> 7), 0, 255); 
}



__kernel void sst_ltp_old(
				__global uchar const* const asp_sst_ids,
				__global uchar const* const asp_states,
				__global uchar const* const syn_states,
				//__global uchar const* const sst_states,
				__private uint const syns_per_den_l2,
				__private uint const rnd,
				//__global int* const aux_ints_0,
				__global char* const syn_strengths
) {
	uint const slc_id = get_global_id(0);
	uint const asp_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const asp_pos = mad24(slc_id, slc_columns, asp_id);
	uint const asp_idx = (asp_pos + ASPINY_REACH);

	uint const sst_idx = asp_sst_id_to_sst_idx(asp_idx, (asp_sst_ids[asp_idx]));
	uint const syn_idx = sst_idx << syns_per_den_l2;

	uchar asp_state = asp_states[asp_idx];

	if (asp_state) {
		prx_syns__active__ltp_ltd(syn_states, syn_idx, syns_per_den_l2, rnd, syn_strengths);
	}

	//aux_ints_0[asp_id] = (rn ^ syn_idx) >> 2;
}




/* SYNS_REGROW()

	- [done] check for dead synapses (syn_strength < 127)
	- [partial] replace with new random src_col_uv_offs and src_slc_id
	- [partial] scan through synapses on that dendrite to check for duplicates
	- [changed] repeat if duplicate found
	- [done] abort if duplicate found

	FUTURE CORRECTIONS:
		- actually assign a new src_slc
			- either generate it host side and use the same one for every 
				synapse or, better yet...
			- store a pre-generated array of randomly distributed columns 
				device side (64 - 256 will be enough) and use a piece of the
				random seed to pick one.
		- [partial] actually scan for duplicates

	FUTURE OPTIMIZATIONS:
		- pre-load synapse values into local memory
		- move this to a dendrite controlled kernel (den_regrow()?) and process
			a whole dendrite for each work item (possibly even a whole cell 
			later)
*/
__kernel void syns_regrow_deprec(
				__global char* const syn_strengths,
				__private uint const syns_per_den_l2,
				__private uint const rnd,
				//__global int* const aux_ints_1,
				__global char* const syn_src_col_v_offs,
				__global uchar* const syn_src_slc_ids
) {
	uint const slc_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const syn_idx = mad24(slc_id, slc_columns, col_id);

	char const syn_strength = syn_strengths[syn_idx];

	//uchar rnd_slc_id = 0;

	if (syn_strength > SYNAPSE_STRENGTH_FLOOR) {
		return;
	} else {
		char rnd_col_ofs = clamp(-127, 127, (int)((rnd ^ ((syn_idx << 5) ^ (syn_idx >> 3))) & 0xFF));
		//rnd_slc_id = ((rnd >> 8) & 0xFF);		// CHOOSE FROM PRE-BUILT ARRAY

			// CHECK FOR DUPLICATES 

		uint base_syn_idx = (syn_idx >> syns_per_den_l2) << syns_per_den_l2;
		uint n = base_syn_idx + (1 << syns_per_den_l2);

		for (uint i = base_syn_idx; i < n; i++) {
			int dup = (rnd_col_ofs == syn_src_col_v_offs[syn_idx]);		// ADD && ROW CHECK
			//int dup_slc = ^^^^^^

			if (!dup) {
				continue;
			} else {
				//	JUST EXIT IF OUR NEW RANDOM ADDR IS A DUPLICATE
				//	AND LET IT BE REASSIGNED NEXT REGROW CYCLE
				//aux_ints_1[syn_idx] = syn_strength;
				return;	
			}
		}

		syn_strengths[syn_idx] = 0;	
		syn_src_col_v_offs[syn_idx] = rnd_col_ofs;
		//syn_src_slc_ids[syn_idx] =

			//aux_ints_1[syn_idx] = syn_strength;
	}

	
	/*int dead_syn = (syn_strength <= -100);
	syn_src_col_v_offs[syn_idx] = mul24(dead_syn, (int)rnd_col_ofs);
	syn_strengths[syn_idx] = mul24(!(dead_syn), (int)syn_strength);*/

	//syn_src_slc_ids[syn_idx] =

}




// VECTORIZE
/*static inline void syns_ltd_unused_bak( 
				__global uchar const* const syn_states,
				uint const syn_idx_start,
				uint const syns_per_den_l2, // MAKE THIS A CONSTANT SOMEDAY
				uint const rnd,
				__global char* const syn_strengths
) {
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		char syn_strength = syn_strengths[i];
		uchar syn_state = syn_states[i];
		//int is_neg = (syn_strength < 0);	// NEGATIVE STRENGTH SYNAPSES GET A BONUS (LOOKS LIKE THIS MAY BE NO BUENO)

		uchar rnd_char = ((rnd ^ i) & 0x7F) >> LTD_BIAS_LOG2;		
		int inc = (rnd_char > abs(syn_strength));

		if (syn_state == 0) {
			continue;
		} else {
			syn_strength -= inc;
		}

		syn_strengths[i] = syn_strength;
		
		//syn_strengths[i] = (syn_strength - inc) + mul24((syn_state != 0), inc + inc);
	}
}*/




/*__kernel void pyrs_ltp_unoptd_bak(
	__global uchar const* const axn_states,
	//__global uchar const* const pyr_preds,
	__global uchar const* const pyr_best1_den_ids,
	__global uchar const* const den_states,
	__global uchar const* const syn_states,
	__private uint const pyr_axn_idx_base, 
	__private uint const syns_per_den_l2,
	__private uint const dens_per_tuft_l2,
	__private uint const pyrs_per_wi,
	__private uint const rnd,
	//__global int* const aux_ints_1,
	__global uchar* const pyr_flag_sets,
	__global uchar* const pyr_prev_best1_den_ids,
	__global char* const syn_strengths
) {
	uint const slc_id = get_global_id(0);
	uint const col_tuft_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const pyr_tuft_idx = mad24(slc_id, slc_columns, col_tuft_id);
	//uint const den_ofs = pyr_idx << DENDRITES_PER_CELL_DISTAL_LOG2;

	//uint const axn_idx_base = mad24(pyr_axn_slc_base + slc_id, slc_columns, col_id + (uint)SYNAPSE_REACH_LIN);

	uint const pyr_idz = mul24(pyr_tuft_idx, pyrs_per_wi);
	uint const pyr_idx_n = pyr_idz + pyrs_per_wi;	

	//uint const pyr_idz = mul24(pyr_tuft_id, pyrs_per_wi);
	//uint const pyr_idx_n = pyr_idz + pyrs_per_wi;

	//uint debug_output = 0;
 
	for (uint i = pyr_idz; i < pyr_idx_n; i++) {
		uchar pyr_best1_den_id = pyr_best1_den_ids[i];
		uchar pyr_prev_best1_den_id = pyr_prev_best1_den_ids[i];
		uchar pyr_flag_set = pyr_flag_sets[i];

		uint den_idx_init = (i << dens_per_tuft_l2);
		uint syn_idx = ((den_idx_init + pyr_best1_den_id) << syns_per_den_l2);

		int pyr_prev_concrete = (pyr_flag_set & PYR_PREV_CONCRETE_FLAG) == PYR_PREV_CONCRETE_FLAG;
		int pyr_best_in_col = (pyr_flag_set & PYR_BEST_IN_COL_FLAG) == PYR_BEST_IN_COL_FLAG;
		int pyr_prev_stp = (pyr_flag_set & PYR_PREV_STP_FLAG) == PYR_PREV_STP_FLAG;


		if (axn_states[i + pyr_axn_idx_base] == 0) {
			if (pyr_prev_stp) {
			//if (pyr_prev_concrete && (pyr_prev_best1_den_id == pyr_best1_den_id)) {

				// 	
				//  NOT SURE WHAT WE'RE GOING TO DO WITH THIS
				// 	DEFINITELY REDUCES OVERALL PREDICTIONS
				// 	ALSO APPEARS TO REDUCE SUPERFLUOUS ONES
				//	CELL SPECIFIC POST-ACTIVATION LEARNING SHOULD REDUCE THIS
				// 	SURVEY SAYS LEAVE IT IN FOR NOW
				//
				//syns_ltd(syn_states, syn_idx, syns_per_den_l2, rnd, syn_strengths);

			}

			pyr_flag_set &= ~PYR_PREV_CONCRETE_FLAG;
			pyr_flag_set &= ~PYR_PREV_STP_FLAG;
			continue;
		} else {
			if (pyr_best_in_col) {
				//syns_ltp_ltd(syn_states, syn_idx, syns_per_den_l2, rnd, syn_strengths);
				
				pyr_prev_best1_den_id = pyr_best1_den_id;
				pyr_flag_set |= PYR_PREV_STP_FLAG;
			}

			pyr_flag_set |= PYR_PREV_CONCRETE_FLAG;
		}

		pyr_prev_best1_den_ids[i] = pyr_prev_best1_den_id;
		pyr_flag_sets[i] = pyr_flag_set;
	}
}*/




/*
	OPTIMIZE FOR WORKGROUP
	VECTORIZE
*/
/*__kernel void den_dist_cycle_unused_old(
	__global uchar const* const syn_states,
	__private uint const syns_per_den_l2,
	__global uchar* const den_states
) {
	uint const slc_id = get_global_id(0);
	uint const den_id = get_global_id(1);
	//uint const l_id = get_local_id(1);
	uint const slc_columns = get_global_size(1);
	uint const den_idx = mad24(slc_id, slc_columns, den_id);
	//uint const syn4_per_den_l2 = syns_per_den_l2 - 2;
	//uint const syn_idz = den_idx << syn4_per_den_l2;
	uint const syn_idz = den_idx << syns_per_den_l2;

	int syn_sum = 0;
	uint const n = (1 << syns_per_den_l2);

	for (uint i = 0; i < n; i += 1) {
		uchar syn_state = syn_states[syn_idz + i];
		syn_sum += syn_state;
	}

	syn_sum = mul24((syn_sum > DENDRITE_INITIAL_THRESHOLD_PROXIMAL), syn_sum);

	den_states[den_idx] = clamp((syn_sum >> 7), 0, 255);
	//den_states[den_idx] = mad24((den_total > 0), 128, clamp(den_total >> (syns_per_den_l2 + 1), 0, 127));
	//den_states[den_idx] = den_total; //(0, 1, 2, 3); 
	//den_states[den_idx] = (syn_sum >> syns_per_den_l2);

}*/

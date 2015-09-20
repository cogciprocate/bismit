
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

// INHIB_RADIUS: A CELL'S SPHERE OF INFLUENCE
#define INHIB_RADIUS					4
// INHIB_INFL_CENTER_OFFSET: MOVES CENTER OF INHIBITION CURVE NEARER(-) OR FURTHER(+) FROM CELL
#define INHIB_INFL_CENTER_OFFSET		1 
// INHIB_INFL_HORIZ_OFFSET: STRETCHES EDGE OF INHIBITION CURVE NEARER(-) OR FURTHER(+) FROM CELL
#define INHIB_INFL_HORIZ_OFFSET			3

#define RETNAL_THRESHOLD 				48

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
// 		##### Debug / Informational Message
//
//
// 		ASSUMPTIONS BEING MADE: (add assert!s)
//			syns_per_tuft > 4
// 			u_size and v_size (global) are multiples of 8







/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
================================== FUNCTIONS ==================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/



// 	W_COORD():
static inline int w_ofs(int v_ofs, int u_ofs) {
	return (0 - v_ofs) - u_ofs;
}



/*

static inline int w_id(int 

// 	CEL_DIST(): Distance between two cells (cubic coordinates)
static inline uint cel_dist(int v_1, int u_1, int v_2, int u_2) {
	int w_1 = w_coord(v_1, u_1); 
	int w_2 = w_coord(v_2, u_2); 

	//return (abs_diff(u_1, u_2) + abs_diff(v_1, v_2) + abs_diff(w_1, w_2)) >> 1;
	//return max(max(abs_diff(u_1, u_2), abs_diff(v_1, v_2)), abs_diff(w_1, w_2));
	//return abs_diff(w_1, w_2);
	return max(max(abs_diff(u_1, u_2), abs_diff(v_1, v_2)), (uint)0);
}

*/


// DIM_IS_SAFE(): BOUNDS CHECK FOR A SINGLE DIMENSION OF A CELLULAR COORDINATE
static inline int dim_is_safe(uint dim_size, uint dim_id, char dim_ofs) {
	int dim_ttl = (int)dim_id + dim_ofs;
	return (dim_ttl >= 0) & (dim_ttl < (int)dim_size);
}

// DIM_IS_SAFE_VEC4(): BOUNDS CHECK FOR A SINGLE DIMENSION OF A CELLULAR COORDINATE
static inline int4 dim_is_safe_vec4(int4 dim_size, int4 dim_id, int4 dim_ofs) {
	int4 dim_ttl = dim_id + dim_ofs;
	return (dim_ttl >= 0) & (dim_ttl < dim_size);
}


// AXN_IDX_HRZ(): Axon index for a horizontal axon
//		- If axon is not horizontal, return 0
static inline uint axn_idx_hrz(uchar slc_id, uint v_size, char v_ofs, uint u_size, char u_ofs) {
		// HRZ_SCT_ID: Id of horizontal section (basically a mini-slice_id)
		int hrz_sct_id = slc_id - HORIZONTAL_AXON_ROW_DEMARCATION;

		// 	IDX_HRZ_SCT: Axon position within horizontal section
		// 		- SYNAPSE_REACH_LIN := Dead center of section
		// 		- SYNAPSE_SPAN_LIN used in lieu of u_size because indexes are bounded 
		//			by the horizontal section rather than the entire slice.
		uint idx_hrz_sct = SYNAPSE_REACH_LIN + mad24((int)v_ofs, (int)SYNAPSE_SPAN_LIN, (int)u_ofs);

		// HRZ_AXN_ID: Position within slice
		uint hrz_axn_id = mad24(hrz_sct_id,  SYNAPSE_SPAN_LIN, (int)idx_hrz_sct);

		// AXN_IDX: Physical index within axon space (array)
		int axn_idx = mad24((uint)HORIZONTAL_AXON_ROW_DEMARCATION, mul24(u_size, v_size), hrz_axn_id);

		// Let's see if our address is even a horizontal one...
		int slc_id_is_hrz = hrz_sct_id >= 0;

		// If this isn't a horizontal address, return 0, which cannot be a horizontal address
		//		- unless, of course, there are only horizontal axons in this space...
		return mul24(slc_id_is_hrz, axn_idx);
}

// AXN_IDX_HRZ_VEC4(): Axon index for a horizontal axon
static inline int4 axn_idx_hrz_vec4(int4 slc_id, int4 v_size, int4 v_ofs, int4 u_size, int4 u_ofs) {
		int4 hrz_sct_id = slc_id - (int4)HORIZONTAL_AXON_ROW_DEMARCATION;
		int4 idx_hrz_sct = (int4)SYNAPSE_REACH_LIN + mad24(v_ofs, (int4)SYNAPSE_SPAN_LIN, u_ofs);
		int4 hrz_axn_id = mad24(hrz_sct_id,  (int4)SYNAPSE_SPAN_LIN, idx_hrz_sct);
		int4 axn_idx = mad24((int4)HORIZONTAL_AXON_ROW_DEMARCATION, mul24(u_size, v_size), hrz_axn_id);
		int4 slc_id_is_hrz = hrz_sct_id >= 0;
		return (slc_id_is_hrz & axn_idx);
}





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
				__global char* const safe_dim_offs)
{
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
static inline uint axn_idx_3d_safe(uchar slc_id, uint v_size, uint v_id, char v_ofs, uint u_size, uint u_id, char u_ofs) {
	char safe_v_ofs = safe_dim_ofs(v_size, v_id, v_ofs);
	char safe_u_ofs = safe_dim_ofs(u_size, u_id, u_ofs);

	uint uv_size = mul24(v_size, u_size);
	uint uv_id = mad24(v_id, u_size, u_id);
	int uv_ofs = mad24((int)safe_v_ofs, (int)u_size, (int)safe_u_ofs);

	//uint axn_idx_spt = mad24((uint)slc_id, uv_size, (uint)(uv_id + uv_ofs));

	return axn_idx_2d(slc_id, uv_size, uv_id, uv_ofs);
}


// CEL_IDX_3D_UNSAFE(): LINEAR INDEX OF A CELL
static inline uint cel_idx_3d_unsafe(uint slc_id, uint v_size, uint v_id, uint u_size, uint u_id) {
	return mad24(slc_id, mul24(v_size, u_size), mad24(v_id, u_size, u_id));	
}

// CEL_IDX_3D_UNSAFE_VEC4(): LINEAR INDEX OF A CELL
static inline int4 cel_idx_3d_unsafe_vec4(int4 slc_id, int4 v_size, int4 v_id, int4 u_size, int4 u_id) {
	return mad24(slc_id, mul24(v_size, u_size), mad24(v_id, u_size, u_id));	
}


// AXN_STATE_3D_SAFE():
static inline uchar axn_state_3d_safe(uchar slc_id, 
				uint v_size, uint v_id, char v_ofs, 
				uint u_size, uint u_id, char u_ofs,
				__global uchar const* const axn_states) 
{
	uint idx_hrz = axn_idx_hrz(slc_id, v_size, v_ofs, u_size, u_ofs);
	uint idx_spt = cel_idx_3d_unsafe(slc_id, v_size, (int)v_id + (int)v_ofs, u_size, (int)u_id + (int)u_ofs);
	int idx_is_hrz = idx_hrz != 0;
	uint axn_idx = mad24((uint)idx_is_hrz, idx_hrz, mul24((uint)!idx_is_hrz, idx_spt));
	int idx_is_safe = dim_is_safe(v_size, v_id, v_ofs) & dim_is_safe(u_size, u_id, u_ofs);
	return mul24(idx_is_safe, axn_states[axn_idx + SYNAPSE_REACH_LIN]);
}

// AXN_STATE_3D_SAFE_VEC4():
static inline uchar4 axn_state_3d_safe_vec4(uchar4 slc_id_uchar4, 
				uint v_size_scl, int4 v_id, char4 v_ofs_char4, 
				uint u_size_scl, int4 u_id, char4 u_ofs_char4,
				__global uchar const* const axn_states) 
{
	int4 v_size = (int4)((int)v_size_scl);
	int4 u_size = (int4)((int)u_size_scl);
	int4 slc_id = convert_int4(slc_id_uchar4);
	int4 v_ofs = convert_int4(v_ofs_char4);
	int4 u_ofs = convert_int4(u_ofs_char4);

	int4 idx_hrz = axn_idx_hrz_vec4(slc_id, v_size, v_ofs, u_size, u_ofs);
	int4 idx_spt = cel_idx_3d_unsafe_vec4(slc_id, v_size, (int4)v_id + (int4)v_ofs, u_size, (int4)u_id + (int4)u_ofs);
	int4 idx_is_hrz = idx_hrz != 0;

	int4 axn_idx = (idx_is_hrz & idx_hrz) | (~idx_is_hrz & idx_spt);
	int4 idx_is_safe = dim_is_safe_vec4(v_size, v_id, v_ofs) & dim_is_safe_vec4(u_size, u_id, u_ofs);

	uchar4 axn_state = (uchar4)(
		((uchar)idx_is_safe.s0 & axn_states[axn_idx.s0 + SYNAPSE_REACH_LIN]),
		((uchar)idx_is_safe.s1 & axn_states[axn_idx.s1 + SYNAPSE_REACH_LIN]),
		((uchar)idx_is_safe.s2 & axn_states[axn_idx.s2 + SYNAPSE_REACH_LIN]),
		((uchar)idx_is_safe.s3 & axn_states[axn_idx.s3 + SYNAPSE_REACH_LIN])
	);
	//uchar4 axn_state = (uchar4)((uchar)idx_is_safe.s0, (uchar)idx_is_safe.s1, (uchar)idx_is_safe.s2, (uchar)idx_is_safe.s3) ;
	return axn_state;
}


// 	SAFE_CEL_STATE_3D(): 'Safe' Cell State Resolution
// 		- If id + ofs are out of cortical bounds, zero is returned
//			- otherwise resolved state is returned 
//		- Intended primarily for use by the inhibition-related kernel(s)
static inline uchar cel_state_3d_safe(uchar slc_id, 
				uint v_size, uint v_id, char v_ofs, 
				uint u_size, uint u_id, char u_ofs, 
				__global uchar const* const cel_states) 
{
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
	static inline uint cel_idx_3d_safe_wip(uint slc_id, uint v_size, 
				uint v_id, int v_ofs, uint u_size, uint u_id, int u_ofs
	) {
		//int v_ofs_is_safe = dim_is_safe(v_size, v_id, v_ofs);
		//int u_ofs_is_safe = dim_is_safe(u_size, u_id, u_ofs);
		//int cel_idx_is_safe = v_ofs_is_safe && u_ofs_is_safe;

		return mad24(slc_id, mul24(v_size, u_size), mad24(v_id, u_size, u_id));	
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
				__global char* const syn_strengths) 
{
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
				__global char* const syn_strengths) 
{
	uint const n = syn_idx_start + (1 << syns_per_tuft_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		char syn_strength = syn_strengths[i];
		uchar syn_flag_set = syn_flag_sets[i];
		int const inc = rnd_inc(rnd, i, syn_strength);
		int const syn_prev_stp = (syn_flag_set & SYN_STP_FLAG) == SYN_STP_FLAG;
		int const syn_active = syn_state != 0;

		syn_strength += mul24(syn_prev_stp && !syn_active, inc << LTP_BIAS_LOG2);
		syn_strength -= mul24(syn_prev_stp && syn_active, inc << LTD_BIAS_LOG2);

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
				__global char* const syn_strengths) 
{
	uint const n = syn_idx_start + (1 << syns_per_den_l2);

	for (uint i = syn_idx_start; i < n; i++) {
		uchar const syn_state = syn_states[i];
		char syn_strength = syn_strengths[i];
		int const inc = rnd_inc(rnd, i, syn_strength);
		int const syn_active = syn_state != 0;

		syn_strength += mul24(syn_active, inc);
		syn_strength -= mul24(!syn_active, inc);

		syn_strengths[i] = syn_strength;
	}

}


static inline int square(int x) {
	return mul24(x, x);
}


static inline uint calc_syn_idz(uint const tuft_id, uint const cel_count, uint const cel_id, 
				uchar const syns_per_tuft_l2) 
{
	uint const syn_tuft_ofs = mul24(tuft_id, cel_count) << syns_per_tuft_l2;
	return syn_tuft_ofs + (cel_id << syns_per_tuft_l2);
}






/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=================================== KERNELS ===================================
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
	uint const slc_id = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);

	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const syn4_idz = ((cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id) + cel_idz) 
		<< (syns_per_tuft_l2 - 2)); // DIVIDED BY 4 BECAUSE VECTORS
	uint const syn4_idn = syn4_idz + (1 << (syns_per_tuft_l2 - 2)); // DIVIDED BY 4 BECAUSE VECTORS

	for (uint syn4_idx = syn4_idz; syn4_idx < syn4_idn; syn4_idx++) {
		uchar4 src_slc_id = syn_src_slc_ids[syn4_idx];
		char4 v_ofs = syn_src_col_v_offs[syn4_idx];
		char4 u_ofs = syn_src_col_u_offs[syn4_idx];

		uchar4 axn_state = axn_state_3d_safe_vec4(
			src_slc_id, v_size, (int4)((int)v_id), v_ofs, u_size, (int4)((int)u_id), u_ofs, axn_states);

		syn_states[syn4_idx] = (convert_uchar4(axn_state != (uchar)0) & (uchar4)0x80) | (axn_state >> (uchar4)1);
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
	uint const slc_id = get_global_id(0);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const v_work_size = get_local_size(1);
	uint const u_work_size = get_local_size(2);

	/* <<<<< SHOULD PROBABLY DO THIS USING GET_NUM_GROUPS()... >>>>> */

	// // BASE DIM_ID (COORDINATE) FOR CURRENT SLICE (GLOBAL ID ON THE INITIAL EDGE OF THE SLICE)
	// uint const v_id_slc_base = mul24(v_size, slc_id);
	// uint const u_id_slc_base = mul24(u_size, slc_id);

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

		uint syn_idx = ((cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id) + cel_idz) 
			<< syns_per_tuft_l2) + cur_syn_ofs;

		char v_ofs = syn_src_col_v_offs[syn_idx];
		char u_ofs = syn_src_col_u_offs[syn_idx];
		uchar src_slc_id = syn_src_slc_ids[syn_idx];

		uchar axn_state = axn_state_3d_safe(src_slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs, axn_states);
		syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);

		if ((slc_id == 1) && (get_global_id(1) == 6) && (get_global_id(2) == 6) && (cel_idz == 0)) {
			aux_ints_0[i] = v_id_base;
		}

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
	uint const slc_id = get_global_id(0);
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

		uint syn4_idx = (((cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id) + cel_idz) 
			<< syns_per_tuft_l2) >> 2) + cur_syn4_ofs; // VEC4'D IDX

		char4 v_ofs = syn_src_col_v_offs[syn4_idx];
		char4 u_ofs = syn_src_col_u_offs[syn4_idx];
		uchar4 src_slc_id = syn_src_slc_ids[syn4_idx];

		// uchar axn_state = axn_state_3d_safe(src_slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs, axn_states);
		// syn_states[syn4_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);

		uchar4 axn_state = axn_state_3d_safe_vec4(src_slc_id, v_size, (int4)((int)v_id), 
			v_ofs, u_size, (int4)((int)u_id), u_ofs, axn_states);

		syn_states[syn4_idx] = (convert_uchar4(axn_state != (uchar)0) & (uchar4)0x80) | (axn_state >> (uchar4)1);


		if ((slc_id == 1) && (get_global_id(1) == 6) && (get_global_id(2) == 6) && (cel_idz == 0)) {
			aux_ints_0[i] = cur_u_wg;
		}

		cur_syn4_ofs += syn4s_per_iter;
		cur_u_wg += u_per_iter;
		cur_v_wg += v_per_iter;

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
__kernel void syns_cycle_2d_wow(
				__global uchar const* const axn_states,
				__global char const* const syn_src_col_v_offs,
				__global uchar const* const syn_src_slc_ids,
				__private uint const syn_tuft_i,
				__private uchar const syns_per_tuft_l2,
				__global int* const aux_ints_0,
				__global uchar* const syn_states) 
{
	uint const slc_columns = mul24(get_global_size(1), get_global_size(2)); // PRECOMPUTE or depricate
	uint const layer_total_per_tuft = mul24(slc_columns, get_global_size(0)); // PRECOMPUTE
	uint const base_cel_tuft_ofs = mul24(syn_tuft_i, layer_total_per_tuft); // PRECOMPUTE
	uint const wg_size = mul24(get_local_size(1), get_local_size(2)); // PRECOMPUTE or depricate

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
	uint const base_syn_idx = (base_cel_idx << syns_per_tuft_l2);
	uint const init_syn_idx = base_syn_idx + l_id;

	uint const syns_per_slc = slc_columns << syns_per_tuft_l2;
	uint const syn4s_per_wg = wg_size << syns_per_tuft_l2;


	int syn_col_i = (base_col_id << syns_per_tuft_l2) + l_id;
	uint syn_idx = init_syn_idx;
	uint const syn_n = base_syn_idx + syn4s_per_wg;

	for (; syn_idx < syn_n; syn_idx += wg_size) {
		syn_col_i -= mul24((int)syns_per_slc, (syn_col_i >= syns_per_slc));
		int col_pos = syn_col_i >> syns_per_tuft_l2;
		uint axn_idx = axn_idx_2d(syn_src_slc_ids[syn_idx], slc_columns, col_pos, syn_src_col_v_offs[syn_idx]);
		uchar axn_state = axn_states[axn_idx]; 

		syn_states[syn_idx] = ((axn_state != 0) << 7) + (axn_state >> 1);	
		syn_col_i += wg_size;
	}
}


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
				__global uchar* const den_states) 
{
	uint const den_idx = get_global_id(0);
	uint const syn_idz = den_idx << syns_per_den_l2;

	uchar den_energy = den_energies[den_idx];

	int syn_sum = 0;
	int syn_sum_raw = 0;

	int const n = (1 << syns_per_den_l2);

	for (int i = 0; i < n; i += 1) {
		char syn_strength = syn_strengths[syn_idz + i];
		uchar syn_state = syn_states[syn_idz + i]; 
		syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum); 
		syn_sum_raw += syn_state;
	}
	
	syn_sum = mul24((syn_sum > den_threshold), syn_sum);

	if (syn_sum != 0) {
		if (den_energy >= ENERGY_LEVEL_MIN) {
			den_energy -= ENERGY_LEVEL_MIN;
		} else {
			den_energy += ENERGY_REGEN_AMOUNT;
			syn_sum = 0;
		}
	} else {
		if (den_energy < ENERGY_LEVEL_MAX) {
			den_energy += ENERGY_REGEN_AMOUNT;
		}
	}

	int den_reduction = syns_per_den_l2 - 1;

	den_states_raw[den_idx] = clamp((syn_sum_raw >> den_reduction), 0, 255); 
	den_states[den_idx] = clamp((syn_sum >> den_reduction), 0, 255); 
}


// 	INHIB_SIMPLE(): Cell Inhibition - reads from soma, writes to axon
//		- If any nearby cells are more active (have a higher soma 'state')
//			- cell will not 'fire'
//			- otherwise, write soma (cel_states[cel_idx]) to axon (axn_states[axn_idx])
//
//		- Overly simplistic algorithm 
// 			- Distance should be taken into account when state is considered
//			- Search area broadened
// 		- Horribly unoptimized, Should:
//			- cache values for an area in local (workgroup) memory
//				- or just prefetch global cache? (comparison needed)
//			- be vectorized
__kernel void inhib_simple(
				__global uchar const* const cel_states,
				__private uchar const cel_base_axn_slc,		// <<<<< DEPRICATE: USE A GLOBAL OFFSET
				__global int* const aux_ints_1,
				__global uchar* const axn_states) 
{
	uint const slc_id = get_global_id(0);	// <<<<< TODO: USE A GLOBAL OFFSET
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);
	uint const cel_idx = cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id);
	uint const axn_idx = axn_idx_3d_safe(slc_id + cel_base_axn_slc, v_size, v_id, 0, u_size, u_id, 0);

	uchar const cel_state = cel_states[cel_idx];

	int const radius_pos = INHIB_RADIUS;
	int const radius_neg = 0 - radius_pos;

	int uninhibited = 1;

	//uint dumb_iter = 0;

	for (int v_ofs = radius_neg; v_ofs <= radius_pos; v_ofs++) {
		int v_neg = 0 - v_ofs;
		int u_z = max(radius_neg, v_neg - radius_pos);
		int u_m = min(radius_pos, v_neg + radius_pos);

		for (int u_ofs = u_z; u_ofs <= u_m; u_ofs++) {

			uchar neighbor_state 
				= cel_state_3d_safe(slc_id, v_size, v_id, v_ofs, u_size, u_id, u_ofs, cel_states);	// ORIGINAL		
			//uchar neighbor_state = cel_states[
			//cel_idx_3d_unsafe(slc_id, v_size, v_id + v_ofs, u_size, u_id + u_ofs)]; // DEBUG


			int distance = (abs(v_ofs) + abs(u_ofs) + abs(w_ofs(v_ofs, u_ofs)))	>> 1;


			//int cel_influence = mul24((int)cel_state, (distance + 1) << 1); // CRAP
			//int neighbor_influence = mul24((int)neighbor_state, radius_pos - distance); // CRAP


			// 	NEW ALGORITHM 16-JUL:
			// 		- FOCAL CELL IS AT LEAST AS INFLUENTIAL AS NEIGHBOR AT THE FOCAL 
			// 		CELL'S LOCATION (A.K.A. THE CELL CELL IS UNINHIBITED)
			// 			- IF CEL_FOCAL_INFLUENCE__AT_CEL_FOCAL >= NEIGHBOR_INFLUENCE__AT_CEL_FOCAL
			//

			 // MOVES CENTER OF INHIBITION CURVE NEARER(-) OR FURTHER(+) FROM FOCAL CELL
			int influence_center_offset = INHIB_INFL_CENTER_OFFSET;
			// STRETCHES EDGE OF INHIBITION CURVE NEARER(-) OR FURTHER(+) FROM FOCAL CELL
			int influence_horizon_offset = INHIB_INFL_HORIZ_OFFSET; 

			int influence_horizon = radius_pos + influence_horizon_offset;
			int influence_max = square(influence_horizon);

			int cel_influence_factor = influence_max;
			int nei_influence_factor = influence_max - square(distance - influence_center_offset);

			int cel_influence = mul24((int)cel_state, cel_influence_factor);
			int nei_influence = mul24((int)neighbor_state, nei_influence_factor);

			//int cel_win = (cel_influence - nei_influence) > 0;
			//int cel_win = cel_influence >= nei_influence;
			//int cel_win = cel_state >= neighbor_state;

			uninhibited &= cel_influence >= nei_influence;


			// STREAMLINE ME
			/*if (cel_influence < neighbor_influence) {
				inhibited = 0;
			}*/


			//int distance = abs(v_ofs) + abs(u);
			//int distance = abs_diff(v_id, v_id + v_ofs) + abs_diff(u_id, u_id + u_ofs);
			//int distance = cel_dist(v_id, u_id, v_id + v_ofs, u_id + u_ofs);

			//int distance = (v_id + v_ofs) - (u_id + u_ofs);
			//int distance = v_ofs - u_ofs;

			//int distance = w_ofs(v_ofs, u_ofs);


			// [DEBUG]: PICK ONLY A FEW POINTS
			/*
			if (((v_id == 10) && (u_id == 10)) 
				|| ((v_id == 20) && (u_id == 20)) 
				|| ((v_id == 30) && (u_id == 30))
				|| ((v_id == 40) && (u_id == 40))) {
				uint unsafe_target_axn_idx = axn_idx_3d_safe(slc_id + cel_base_axn_slc, v_size, v_id, v_ofs, u_size, u_id, u_ofs);

				//aux_ints_1[dumb_iter] = cel_state_3d_safe(slc_id, 
				//	v_size, v_id, v_ofs, u_size, u_id, u_ofs, cel_states);
				//aux_ints_1[unsafe_target_axn_idx] = 1;
				//axn_states[unsafe_target_axn_idx] = neighbor_state;
				axn_states[unsafe_target_axn_idx] = 1 + inhibited;
			}
			*/
			
			//dumb_iter += 1;
			

			// int debug_idx_ofs = 257;	 // SET TO WHATEVER
			// for (int i = 0; i < mul24(get_global_size(0), mul24(v_size, u_size)); i += 1024) {

			// 	if (((int)cel_idx & 0xFFFFFFFF) == debug_idx_ofs) {
			// 		aux_ints_1[mul24(i, 1024) + dumb_iter] 
			// 				//= cel_influence;
			// 				//= distance + 100;
			// 				//= cel_idx;
			// 				= neighbor_state - cel_state;

			// 	}

			// 	// if (cel_idx == 384) {
			// 	// 	//aux_ints_1[axn_idx_3d_safe(slc_id + cel_base_axn_slc, v_size, v_id, v_ofs, u_size, u_id, u_ofs)] = distance;
			// 	// 	aux_ints_1[520 + dumb_iter] 
			// 	// 		//= cel_influence;
			// 	// 		= distance + 100;
			// 	// }
			// }
		}
	}

	axn_states[axn_idx] = mul24((uint)uninhibited, (uint)cel_state);

}

__kernel void inhib_passthrough(
				__global uchar const* const cel_states,
				__private uchar const cel_base_axn_slc,		// <<<<< DEPRICATE: USE A GLOBAL OFFSET
				__global uchar* const axn_states) 
{
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



// SST_LTP(): Long term potentiation for Spiny Stellate Cells - Completely unoptimized
__kernel void sst_ltp_simple(
				__global uchar const* const axn_states,
				__global uchar const* const syn_states,
				__private uint const cel_axn_idz,
				//__private uint const tufts_per_cel,
				__private uchar const syns_per_tuft_l2,
				__private uint const rnd,
				__global int* const aux_ints_0,
				__global char* const syn_strengths) 
{
	uint const tuft_id = get_global_id(0);
	uint const cel_id = get_global_id(1);
	uint const cel_count = get_global_size(1);

	uint const axn_idx = cel_axn_idz + cel_id;
	uint const axn_state = axn_states[axn_idx];

	// TESTING
	// uint const cel_tuft_id = cel_id + mul24(tuft_id, cel_count);
	// aux_ints_0[cel_tuft_id] = axn_state;
	// END TESTING	

	if (axn_state) {
		uint const syn_idz = calc_syn_idz(tuft_id, cel_count, cel_id, syns_per_tuft_l2);
		prx_syns__active__ltp_ltd(syn_states, syn_idz, syns_per_tuft_l2, rnd, syn_strengths);
	}
}




// PYR_ACTIVATE(): CONVERT TO 1 WORK_DIM
__kernel void pyr_activate(
				__global uchar const* const mcol_pred_totals, // COL
				__global uchar const* const mcol_best_pyr_den_states,
				__global uchar const* const pyr_best_den_ids,
				// ADD PYR BEST DEN STATE NOW THAT WE'VE ADDED IT (and to another kernel somewhere also)
				__global uchar const* const den_states,
				__private uint const ssts_axn_idz,
				__private uchar const pyr_axn_slc_base,
				__private uchar const dens_per_tuft_l2,
				__global uchar* const pyr_flag_sets,
				__global uchar* const pyr_preds,
				//__global int* const aux_ints_0,
				__global uchar* const axn_states) 
{
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
	uchar const mcol_pred_total = mcol_pred_totals[col_id];
	uchar const pyr_pred = pyr_preds[pyr_idx];
	uchar pyr_flag_set = pyr_flag_sets[pyr_idx];

	//aux_ints_0[pyr_idx] = pyr_flag_set;

	int const mcol_active = sst_axn_state != 0;
	//int const mcol_active = mcol_state != 0;
	int const mcol_any_pred = mcol_pred_total != 0;
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
				__global uchar const* const den_states,
				__global uchar const* const syn_states,
				__private uint const pyr_axn_idx_base, 
				__private uint const syns_per_den_l2,
				__private uint const dens_per_tuft_l2,
				__private uint const pyrs_per_wi,
				__private uint const rnd,
				__global uchar* const syn_flag_sets,
				__global uchar* const pyr_flag_sets,
				//__global int* const aux_ints_0,
				//__global int* const aux_ints_1,
				__global char* const syn_strengths) 
{
	uint const slc_id = get_global_id(0);
	uint const col_tuft_id = get_global_id(1);
	uint const tufts_per_slc = get_global_size(1);
	uint const pyr_tuft_id = mad24(slc_id, tufts_per_slc, col_tuft_id);
	uint const pyr_idz = mul24(pyr_tuft_id, pyrs_per_wi);
	uint const pyr_idn = pyr_idz + pyrs_per_wi;
 
	for (uint i = pyr_idz; i < pyr_idn; i++) {
		uchar pyr_best_den_id = pyr_best_den_ids[i];
		uchar pyr_flag_set = pyr_flag_sets[i];

		int pyr_concrete = axn_states[i + pyr_axn_idx_base] != 0;
		int pyr_fuzzy = pyr_preds[i] != 0;

		int pyr_prev_concrete = (pyr_flag_set & PYR_PREV_CONCRETE_FLAG) == PYR_PREV_CONCRETE_FLAG;
		int pyr_prev_fuzzy = (pyr_flag_set & PYR_PREV_FUZZY_FLAG) == PYR_PREV_FUZZY_FLAG;
		int pyr_best_in_col = (pyr_flag_set & PYR_BEST_IN_COL_FLAG) == PYR_BEST_IN_COL_FLAG;

		uint den_idx_base = i << dens_per_tuft_l2;

		uint pyr_syn_idz = ((den_idx_base) << syns_per_den_l2);	 // WHOLE CELL
		uint best_den_syn_idz = (den_idx_base + pyr_best_den_id) << syns_per_den_l2;

		if (pyr_concrete) {
			if (pyr_prev_fuzzy) { 
				// PREVIOUS (CORRECT) PREDICTION (EVERY PYR IN COL): REINFORCE DEN + TRAIN NEW DEN
				// SAME AS ANO + TRAIN A SECOND REDUNDANT DENDRITE AS WELL (OR DON'T)
				dst_syns__active__stp_ltd(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);

			} else if (pyr_best_in_col) { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN 
			//} else { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
				dst_syns__active__stp_ltd(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
			}

			pyr_flag_set |= PYR_PREV_CONCRETE_FLAG;

		} else if (pyr_prev_concrete) {

			cel_syns_trm(syn_states, pyr_syn_idz, syns_per_den_l2 + dens_per_tuft_l2, rnd, syn_flag_sets, syn_strengths);

			pyr_flag_set &= ~PYR_PREV_CONCRETE_FLAG;
		}		

		pyr_flag_set &= ~PYR_PREV_FUZZY_FLAG;
		pyr_flag_set |= mul24(pyr_fuzzy, PYR_PREV_FUZZY_FLAG);

		pyr_flag_sets[i] = pyr_flag_set;
	}
}




__kernel void pyr_cycle(
				__global uchar const* const den_states,
				__global uchar const* const den_states_raw,
				__private uint const den_tufts_per_cel,
				__private uchar const dens_per_tuft_l2,
				__global uchar* const pyr_best_den_ids,
				__global uchar* const pyr_best_den_states,
				__global uchar* const pyr_preds) 
{
	uint const pyr_idx = get_global_id(0);
	uchar best_den_state = 0;
	uchar best_den_id = 0;
	uchar pyr_state = 0;

	for (uint den_tuft = 0; den_tuft < den_tufts_per_cel; den_tuft++) {
		uint const den_idz = mad24(den_tuft, get_global_size(0), pyr_idx) << dens_per_tuft_l2;
 
		for (uint den_idx = 0; den_idx < (1 << dens_per_tuft_l2); den_idx++) {
			uchar den_state = den_states[den_idz + den_idx];
			int den_state_bigger = (den_state > best_den_state);

			best_den_id = mad24(den_state_bigger, (int)den_idx, mul24(!den_state_bigger, best_den_id));
			best_den_state = mad24(den_state_bigger, den_state, mul24(!den_state_bigger, best_den_state));
		}
	}

	pyr_state = best_den_state; // YEAH... I KNOW...

	pyr_best_den_ids[pyr_idx] = best_den_id;
	pyr_best_den_states[pyr_idx] = best_den_state;
	pyr_preds[pyr_idx] = pyr_state;
}


//	COL_OUTPUT()
//		- rename coming
//
// 
__kernel void col_output(
				__global uchar const* const pyr_preds,
				__global uchar const* const pyr_best_den_states,
				__private uint const sst_axn_idz,
				__private uchar const pyr_depth,
				__private uchar const output_axn_slc,
				__global uchar* const mcol_pred_totals,
				__global uchar* const mcol_best_pyr_den_states,
				__global uchar* const axn_states)
{
	uint const slc_id = get_global_id(0);
	uint const col_id = get_global_id(1);
	uint const slc_columns = get_global_size(1);
	uint const output_axn_idx = axn_idx_2d(output_axn_slc + slc_id, slc_columns, col_id, 0);
	uint const col_idx = mad24(slc_id, slc_columns, col_id);

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
	}


	mcol_pred_totals[col_idx] = clamp(col_pyr_pred_total, 0, 255); // <<<<< FIX ME TO BE A FLAGSET
	mcol_best_pyr_den_states[col_idx] = max_den_state;
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





// __kernel void sst_ltp(
// 				__global uchar const* const axn_states,
// 				__global uchar const* const syn_states,
// 				__private uint const cel_axn_idz,
// 				__private uchar const syns_per_tuft_l2,
// 				__private uint const rnd,
// 				__global int* const aux_ints_0,
// 				__global char* const syn_strengths) 
// {
// 	uint const slc_id = get_global_id(0);
// 	uint const col_tuft_id = get_global_id(1);
// 	uint const tuft_size = get_global_size(1);
// 	uint const cel_tuft_id = mad24(slc_id, tuft_size, col_tuft_id);

// 	uint cels_per_tuft = get_local_size(1);

// 	uint const cel_idz = mul24(cel_tuft_id, cels_per_tuft);
// 	uint const cel_idn = cel_idz + cels_per_tuft;

// 	for (uint cel_idx = cel_idz; cel_idx < cel_idn; cel_idx++) {
// 		uchar axn_state = axn_states[cel_axn_idz + cel_idx];

// 		aux_ints_0[cel_idx] = axn_state;
		
// 		if (axn_state) {
// 			uint syn_idz = cel_idx << syns_per_tuft_l2;	
// 			prx_syns__active__ltp_ltd(syn_states, syn_idz, syns_per_tuft_l2, rnd, syn_strengths);
// 		}
// 	}
// }







// DEPRICATE
static inline uint asp_to_sst_ofs(uint asp_idx) {
	return (asp_idx - ASPINY_REACH) << ASPINY_SPAN_LOG2;
}

// DEPRICATE
static inline uint asp_sst_id_to_sst_idx(uint const asp_idx, uint const asp_sst_id) {
	return (asp_to_sst_ofs(asp_idx) + (asp_sst_id & (ASPINY_SPAN - 1)));
}



// <<<<< FOLLOWING SECTION SLATED FOR REMOVAL/DEPRICATION >>>>>

__kernel void peak_sst_cycle_pre(
				__global uchar const* const sst_states,
				__global uchar* const asp_states,
				__global uchar* const asp_sst_ids) 
{
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
				__global uchar* const asp_wins) 
{
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
				__global uchar* const asp_states)
				//__global uchar* const sst_states
{
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
				__global uchar* const axn_states) 
{
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

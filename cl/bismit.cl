
#define LTD_BIAS_LOG2					0
#define LTP_BIAS_LOG2					0
#define ENERGY_SETTING 					4
#define ENERGY_RECHARGE					1

#define FLAG_ON(flag_set, mask)			((flag_set) |= (mask))
#define FLAG_OFF(flag_set, mask)		((flag_set) &= ~(mask))
#define FLAG_EVAL(flag_set, mask)		(((flag_set) & (mask)) == (mask))

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
// 		idz:= index[0], first element, starting element
//
// 		idn:= index[len]: element after final element, termination point
//			- ex.: for(int i = 0, i < idn, i++)
//
// 		idm:= index[max]:= index[len - 1]: final (valid) element just before idn. idn - 1 = idm.
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
// 		fuz [Tenative]: fuzzyness, level of predictiveness
//
// 		***** High Priority Comment, Temporary Code Change
// 		<<<<< Medium Priority Comment, To Do
// 		##### Debug / Informational Message
//
//
// 		ASSUMPTIONS BEING MADE: (add assert!s)
//			syns_per_tuft > 4
// 			u_size and v_size (global) are multiples of 8
//
//		vloadn(size_t offset, const __constant gentype *p )





/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
================================= __CONSTANT ==================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/

__constant uint axn_slc_idzs[AXN_SLC_COUNT] = { AXN_SLC_IDZS };
__constant uint axn_slc_v_sizes[AXN_SLC_COUNT] = { AXN_SLC_V_SIZES };
__constant uint axn_slc_u_sizes[AXN_SLC_COUNT] = { AXN_SLC_U_SIZES };
__constant uchar axn_slc_v_scales[AXN_SLC_COUNT] = { AXN_SLC_V_SCALES };
__constant uchar axn_slc_u_scales[AXN_SLC_COUNT] = { AXN_SLC_U_SCALES };


static inline uint get_axn_slc_idz(uchar slc_id) {
	return axn_slc_idzs[slc_id];
}

static inline int4 get_axn_slc_idz_vec4(uchar4 slc_id) {
	return (int4)(	axn_slc_idzs[slc_id.s0], 
					axn_slc_idzs[slc_id.s1], 
					axn_slc_idzs[slc_id.s2], 
					axn_slc_idzs[slc_id.s3]);
}


static inline uchar get_axn_v_size(uchar slc_id) {
	return axn_slc_v_sizes[slc_id];
}

static inline int4 get_axn_v_size_vec4(uchar4 slc_id) {
	return (int4)(	axn_slc_v_sizes[slc_id.s0], 
					axn_slc_v_sizes[slc_id.s1], 
					axn_slc_v_sizes[slc_id.s2], 
					axn_slc_v_sizes[slc_id.s3]);
}


static inline uint get_axn_u_size(uchar slc_id) {
	return axn_slc_u_sizes[slc_id];
}

static inline int4 get_axn_u_size_vec4(uchar4 slc_id) {
	return (int4)(	axn_slc_u_sizes[slc_id.s0], 
					axn_slc_u_sizes[slc_id.s1], 
					axn_slc_u_sizes[slc_id.s2], 
					axn_slc_u_sizes[slc_id.s3]);
}


static inline uint get_axn_v_scale(uchar slc_id) {
	return axn_slc_v_scales[slc_id];
}

static inline int4 get_axn_v_scale_vec4(uchar4 slc_id) {
	return (int4)(	axn_slc_v_scales[slc_id.s0], 
					axn_slc_v_scales[slc_id.s1], 
					axn_slc_v_scales[slc_id.s2], 
					axn_slc_v_scales[slc_id.s3]);
}


static inline uint get_axn_u_scale(uchar slc_id) {
	return axn_slc_u_scales[slc_id];
}

static inline int4 get_axn_u_scale_vec4(uchar4 slc_id) {
	return (int4)(	axn_slc_u_scales[slc_id.s0], 
					axn_slc_u_scales[slc_id.s1], 
					axn_slc_u_scales[slc_id.s2], 
					axn_slc_u_scales[slc_id.s3]);
}


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


static inline int square(int x) {
	return mul24(x, x);
}


static inline int rnd_inc(uint const rnd_a,	uint const rnd_b, char const syn_strength) {
		return ((rnd_a ^ rnd_b) & 0x7F) > abs(syn_strength);
}


static inline uint calc_syn_idz(uint const tuft_id, uint const cel_count, uint const cel_id, 
				uchar const syns_per_tuft_l2) 
{
	uint const syn_tuft_ofs = mul24(tuft_id, cel_count) << syns_per_tuft_l2;
	return syn_tuft_ofs + (cel_id << syns_per_tuft_l2);
}


// DIM_IS_SAFE(): BOUNDS CHECK FOR A SINGLE DIMENSION OF A CELLULAR COORDINATE
static inline int dim_is_safe(int dim_size, int dim_id, int dim_ofs) {
	int dim_ttl = dim_id + dim_ofs;
	return (dim_ttl >= 0) & (dim_ttl < dim_size);
}


// DIM_IS_SAFE_VEC4(): BOUNDS CHECK FOR A SINGLE DIMENSION OF A CELLULAR COORDINATE
static inline int4 dim_is_safe_vec4(int4 dim_size, int4 dim_id, int4 dim_ofs) {
	int4 dim_ttl = dim_id + dim_ofs;
	return (dim_ttl >= 0) & (dim_ttl < dim_size);
}


/*=============================================================================
================================ CELL INDEXING ================================
=============================================================================*/

// CEL_IDX_3D_UNSAFE(): LINEAR INDEX OF A CELL - NOT ACCURATE FOR AXONS
static inline uint cel_idx_3d_unsafe(uint slc_id, uint v_size, uint v_id, uint u_size, uint u_id) {
	return mad24(slc_id, mul24(v_size, u_size), mad24(v_id, u_size, u_id));	
}

// CEL_IDX_3D_UNSAFE_VEC4(): LINEAR INDEX OF A CELL - NOT FOR ACCURATE AXONS
static inline int4 cel_idx_3d_unsafe_vec4(uchar4 slc_id_uchar4, int4 v_size, int4 v_id, int4 u_size, int4 u_id) {
	int4 slc_id = convert_int4(slc_id_uchar4);
	return mad24(slc_id, mul24(v_size, u_size), mad24(v_id, u_size, u_id));	
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

	uint cel_idx = cel_idx_3d_unsafe(slc_id, v_size, (int)v_id + v_ofs, u_size, (int)u_id + u_ofs);

	return mul24(cel_idx_is_safe, cel_states[cel_idx]);
}


/*=============================================================================
================================ AXON INDEXING ================================
=============================================================================*/

// AXN_IDX_3D_UNSAFE(): Linear index of an axon
// 		- Using ints as intermediate variables to be consistent with vectorized version even
// 		though it will only affect invalid indexes
static inline uint axn_idx_3d_unsafe(uchar const slc_id, uint const v_id, char const v_ofs, 
				uint const u_id, char const u_ofs, int* const idx_is_safe) 
	{
	// GET THE DIM SIZES:
	// 		- 'u' is (the 'least significant' dimension as far as opencl is concerned)
	int const v_size = get_axn_v_size(slc_id);
	int const u_size = get_axn_u_size(slc_id);

	// 	CALCULATE SCALED INDEX:
	// 		- Multiply by the pre-defined scale for specified slice then divide by 16.
	// 		- A scale of 16 = 100%, 8 = 50%, 32 = 200%, etc.
	int const scaled_v_id = (mul24(v_id, get_axn_v_scale(slc_id)) >> 4);
	int const scaled_u_id = (mul24(u_id, get_axn_u_scale(slc_id)) >> 4);

	// CHECK SAFETY:
	*idx_is_safe = dim_is_safe(v_size, scaled_v_id, v_ofs) & dim_is_safe(u_size, scaled_u_id, u_ofs);

	// RETURN the sum of the pre-defined axon offset for the slice and the linear offset within that slice
	return get_axn_slc_idz(slc_id) + (uint)mad24(scaled_v_id + v_ofs, u_size, scaled_u_id + u_ofs);
}

// AXN_IDX_3D_UNSAFE_VEC4(): Linear index of an axon, vec4
static inline int4 axn_idx_3d_unsafe_vec4(uchar4 const slc_id, int4 const v_id, char4 const v_ofs_char4, 
			int4 const u_id, char4 const u_ofs_char4, int4* const idx_is_safe) 
{
	int4 v_ofs = convert_int4(v_ofs_char4);
	int4 u_ofs = convert_int4(u_ofs_char4);

	int4 const v_size = get_axn_v_size_vec4(slc_id);
	int4 const u_size = get_axn_u_size_vec4(slc_id);

	int4 const scaled_v_id = (mul24(v_id, get_axn_v_scale_vec4(slc_id)) >> 4) + v_ofs;
	int4 const scaled_u_id = (mul24(u_id, get_axn_u_scale_vec4(slc_id)) >> 4) + u_ofs;

	*idx_is_safe = dim_is_safe_vec4(v_size, scaled_v_id, v_ofs) & dim_is_safe_vec4(u_size, scaled_u_id, u_ofs);

	return get_axn_slc_idz_vec4(slc_id) + mad24(scaled_v_id, u_size, scaled_u_id);
}


/*===========================================================================*/

// AXN_STATE_3D_SAFE():
static inline uchar axn_state_3d_safe(uchar slc_id, uint v_id, char v_ofs, uint u_id, 
			char u_ofs, __global uchar const* const axn_states) 
{
	int idx_is_safe = 0;

	//uint idx_hrz = axn_idx_hrz(slc_id, v_size, v_ofs, u_size, u_ofs);
	uint idx_spt = axn_idx_3d_unsafe(slc_id, v_id, v_ofs, u_id, u_ofs, &idx_is_safe);
	//int idx_is_hrz = idx_hrz != 0;
	//uint axn_idx = mad24((uint)idx_is_hrz, idx_hrz, mul24((uint)!idx_is_hrz, idx_spt));
	uint axn_idx = idx_spt; // TEMP

	return mul24(idx_is_safe, axn_states[axn_idx]);
	//return axn_states[axn_idx];
}

// AXN_STATE_3D_SAFE_VEC4():
static inline uchar4 axn_state_3d_safe_vec4(uchar4 slc_id, int4 v_id, char4 v_ofs, 
	int4 u_id, char4 u_ofs, __global uchar const* const axn_states) 
{
	//int4 v_size = (int4)((int)v_size_scl);
	//int4 u_size = (int4)((int)u_size_scl);
	//int4 slc_id = convert_int4(slc_id_uchar4);
	//int4 v_ofs = convert_int4(v_ofs_char4);
	//int4 u_ofs = convert_int4(u_ofs_char4);

	int4 idx_is_safe = (int4)0;

	//int4 idx_hrz = axn_idx_hrz_vec4(slc_id, v_size, v_ofs, u_size, u_ofs);
	int4 idx_spt = axn_idx_3d_unsafe_vec4(slc_id, v_id, v_ofs, u_id, u_ofs, &idx_is_safe);
	//int4 idx_is_hrz = idx_hrz != 0;

	//int4 axn_idx = (idx_is_hrz & idx_hrz) | (~idx_is_hrz & idx_spt);
	int4 axn_idx = idx_spt;

	uchar4 axn_state = (uchar4)(
		((uchar)idx_is_safe.s0 & axn_states[axn_idx.s0]),
		((uchar)idx_is_safe.s1 & axn_states[axn_idx.s1]),
		((uchar)idx_is_safe.s2 & axn_states[axn_idx.s2]),
		((uchar)idx_is_safe.s3 & axn_states[axn_idx.s3]));
	 
	// uchar4 axn_state = (uchar4)(
	// 	(axn_states[axn_idx.s0]),
	// 	(axn_states[axn_idx.s1]),
	// 	(axn_states[axn_idx.s2]),
	// 	(axn_states[axn_idx.s3]));

	return axn_state;
}

/*===========================================================================*/

// AXN_IDX_HRZ(): Axon index for a horizontal axon
// <<<<< TODO: DEPRICATE >>>>>
//		- If axon is not horizontal, return 0
static inline uint axn_idx_hrz(uchar slc_id, uint v_size, char v_ofs, uint u_size, char u_ofs) {
		// HRZ_SCT_ID: Id of horizontal section (basically a sub-slice_id)
		int hrz_sct_id = slc_id - HORIZONTAL_AXON_ROW_DEMARCATION;

		// 	IDX_HRZ_SCT: Axon position within horizontal section
		// 		- AXON_MARGIN_SIZE := Dead center of section
		// 		- SYNAPSE_SPAN_RHOMBAL_AREA used in lieu of u_size because indexes are bounded 
		//			by the horizontal section rather than the entire slice.
		uint idx_hrz_sct = AXON_MARGIN_SIZE + mad24((int)v_ofs, (int)SYNAPSE_SPAN_RHOMBAL_AREA, (int)u_ofs);

		// HRZ_AXN_ID: Position within slice
		uint hrz_axn_id = mad24(hrz_sct_id,  SYNAPSE_SPAN_RHOMBAL_AREA, (int)idx_hrz_sct);

		// AXN_IDX: Physical index within axon space (array)
		int axn_idx = mad24((uint)HORIZONTAL_AXON_ROW_DEMARCATION, mul24(u_size, v_size), hrz_axn_id);

		// Let's see if our address is even a horizontal one...
		int slc_id_is_hrz = hrz_sct_id >= 0;

		// If this isn't a horizontal address, return 0, which cannot be a horizontal address
		//		- unless, of course, there are only horizontal axons in this space...
		return mul24(slc_id_is_hrz, axn_idx);
}

// AXN_IDX_HRZ_VEC4(): Axon index for a horizontal axon
// <<<<< TODO: DEPRICATE >>>>>
static inline int4 axn_idx_hrz_vec4(int4 slc_id, int4 v_size, int4 v_ofs, int4 u_size, int4 u_ofs) {
		int4 hrz_sct_id = slc_id - (int4)HORIZONTAL_AXON_ROW_DEMARCATION;
		int4 idx_hrz_sct = (int4)AXON_MARGIN_SIZE + mad24(v_ofs, (int4)SYNAPSE_SPAN_RHOMBAL_AREA, u_ofs);
		int4 hrz_axn_id = mad24(hrz_sct_id,  (int4)SYNAPSE_SPAN_RHOMBAL_AREA, idx_hrz_sct);
		int4 axn_idx = mad24((int4)HORIZONTAL_AXON_ROW_DEMARCATION, mul24(u_size, v_size), hrz_axn_id);
		int4 slc_id_is_hrz = hrz_sct_id >= 0;
		return (slc_id_is_hrz & axn_idx);
}


/*=============================================================================
================================== LEARNING ===================================
=============================================================================*/

// TODO: VECTORIZE
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

// TODO: VECTORIZE 
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

// TODO: VECTORIZE 
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




__kernel void reference_all_the_things(__private int const for_sanitys_sake) {
	//get_axn_u_size_vec4((uchar4)0);
	cel_idx_3d_unsafe_vec4((uchar4)0, (int4)0, (int4)0, (int4)0, (int4)0);
	axn_idx_hrz(0, 0, 0, 0, 0);
	dim_is_safe_vec4((int4)0, (int4)0, (int4)0);
	axn_idx_hrz_vec4((int4)0, (int4)0, (int4)0, (int4)0, (int4)0);
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
// 	- TODO: Split this beast up.




// DEN_CYCLE():
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


// 	INHIB_SIMPLE(): [DESCRIPTION OUT OF DATE] Cell Inhibition - reads from soma, writes to axon
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
				__private uchar const cel_base_axn_slc,
				__global int* const aux_ints_1,
				__global uchar* const axn_states) 
{
	uint const slc_id = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);
	uint const cel_idx = cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id);
	//uint const axn_idx = axn_idx_3d_safe(slc_id + cel_base_axn_slc, v_size, v_id, 0, u_size, u_id, 0);
	int idx_is_safe = 0;
	uint const cel_axn_idx = axn_idx_3d_unsafe(slc_id + cel_base_axn_slc, v_id, 0, u_id, 0, &idx_is_safe);

	uchar const cel_state = cel_states[cel_idx];

	int const radius_pos = INHIB_RADIUS;
	int const radius_neg = 0 - radius_pos;

	int uninhibited = 1;


	// ***** DEBUG-TESTING *****
	// if (cel_idx < AXN_SLC_COUNT) {
	// 	aux_ints_1[cel_idx] = get_axn_v_scale(cel_idx);
	// }

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

	axn_states[cel_axn_idx] = mul24((uint)uninhibited, (uint)cel_state);

}

__kernel void inhib_passthrough(
				__global uchar const* const cel_states,
				__private uchar const cel_base_axn_slc,
				__global uchar* const axn_states) 
{
	uint const slc_id = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const cel_idx = cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id);
	//uint const axn_idx = axn_idx_3d_safe(slc_id + cel_base_axn_slc, v_size, v_id, 0, u_size, u_id, 0);
	int idx_is_safe = 0;
	uint const cel_axn_idx = axn_idx_3d_unsafe(slc_id + cel_base_axn_slc, v_id, 0, u_id, 0, &idx_is_safe);

	uchar const cel_state = cel_states[cel_idx];

	axn_states[cel_axn_idx] = cel_state;
}


// SST_LTP_SIMPLE(): Long term potentiation for Spiny Stellate Cells - Completely unoptimized
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

	uint const cel_axn_idx = cel_axn_idz + cel_id;
	uint const axn_state = axn_states[cel_axn_idx];

	// TESTING
	// uint const cel_tuft_id = cel_id + mul24(tuft_id, cel_count);
	// aux_ints_0[cel_tuft_id] = axn_state;
	// END TESTING	

	if (axn_state) {
		uint const syn_idz = calc_syn_idz(tuft_id, cel_count, cel_id, syns_per_tuft_l2);
		prx_syns__active__ltp_ltd(syn_states, syn_idz, syns_per_tuft_l2, rnd, syn_strengths);
	}
}


// SST_LTP(): Long term potentiation for Spiny Stellate Cells
// <<<<< TODO: ADD AN ADDITIONAL DIMENSION [0] FOR SLC_ID TO SUPPORT MULTIPLE SLICE SST LAYERS >>>>>
__kernel void sst_ltp(
				__global uchar const* const axn_states,
				__global uchar const* const syn_states,
				__private uint const cel_lyr_axn_idz,
				__private uint const cols_per_grp,
				__private uchar const syns_per_tuft_l2,
				__private uint const rnd,
				__global int* const aux_ints_0,
				__global char* const syn_strengths) 
{
	uint const tuft_id = get_global_id(0);
	uint const cel_grp_id = get_global_id(1);
	uint const cel_count = get_global_size(1);

	uint const cel_idz = mul24(cel_grp_id, cols_per_grp);
	uint const cel_axn_idz = cel_lyr_axn_idz + cel_idz;	

	// TESTING
	// uint const cel_tuft_id = cel_id + mul24(tuft_id, cel_count);
	// aux_ints_0[cel_tuft_id] = axn_state;
	// END TESTING

	for (uint i = 0; i < cols_per_grp; i += 1) {
		uint const cel_idx = cel_idz + i;
		uint const cel_axn_idx = cel_axn_idz + i;
		uint const axn_state = axn_states[cel_axn_idx];

		if (axn_state) {			
			uint const syn_idz = calc_syn_idz(tuft_id, cel_count, cel_idx, syns_per_tuft_l2);
			prx_syns__active__ltp_ltd(syn_states, syn_idz, syns_per_tuft_l2, rnd, syn_strengths);
		}
	}
}


// MCOL_ACTIVATE(): CONVERT TO 3 WORK DIMS
// 		- ASSUMES SSTS IS ONLY 1 SLICE DEEP
__kernel void mcol_activate_pyrs(
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
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);
	//uint const col_id = get_global_id(1);

	uint const slc_columns = get_global_size(1);
	//uint const pyr_idx = mad24(slc_id, slc_columns, col_id);
	//uint const axn_idx = axn_idx_2d(pyr_axn_slc_base + slc_id, slc_columns, col_id, 0);
	uint const pyr_idx = cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id);
	int idx_is_safe = 0;
	uint const cel_axn_idx = axn_idx_3d_unsafe(pyr_axn_slc_base + slc_id, v_id, 0, u_id, 0, &idx_is_safe);
	uint const col_id = cel_idx_3d_unsafe(0, v_size, v_id, u_size, u_id);

	// ******************

	uint const den_ofs = pyr_idx << dens_per_tuft_l2;			// REPLACE
	uint const best_den_idx = den_ofs + pyr_best_den_ids[pyr_idx];		// REPLACE

	uchar const best_den_state = den_states[best_den_idx];				// CHANGE

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

	axn_states[cel_axn_idx] = (uchar)mad24(anomaly, (int)sst_axn_state, mul24(crystal, (int)pyr_pred));
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
//					- NOTE: The reasoning here is that any synapses which are active just after 
//					(but not before) the cell was active are likely to be unrelated to it's prior 
//					activity. In other words, a rough implementation of LTD (simplified and 
//					optimized and theorized and ... oh who knows). 
//
//	- TODO:
//		- Vectorize (should be highly vectorizable)
//		- reducing branching will be tough with this one
//		- Tests (check that flag_set and prev_best_den_id are robustly maintained)
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
__kernel void pyrs_ltp_unoptd(
				__global uchar const* const axn_states,
				__global uchar const* const pyr_preds,
				__global uchar const* const pyr_best_den_ids,
				__global uchar const* const den_states,
				__global uchar const* const syn_states,
				__private uint const pyr_lyr_axn_idz, 
				__private uint const syns_per_den_l2,
				__private uint const dens_per_tuft_l2,
				__private uint const cols_per_grp,
				__private uint const rnd,
				__global uchar* const syn_flag_sets,
				__global uchar* const pyr_flag_sets,
				//__global int* const aux_ints_0,
				//__global int* const aux_ints_1,
				__global char* const syn_strengths) 
{
	uint const slc_id = get_global_id(0);
	uint const tuft_id = get_global_id(1);
	uint const grp_id = get_global_id(2);
	uint const tuft_count = get_global_size(1);	
	uint const grp_count = get_global_size(2); // GRP_COUNT: COLUMNS / COLS_PER_GRP

	uint const pyr_grp_id = cel_idx_3d_unsafe(slc_id, tuft_count, tuft_id, grp_count, grp_id);
	uint const pyr_idz = mul24(grp_id, cols_per_grp);

	uint const axn_grp_id = get_axn_slc_idz(slc_id) + cel_idx_3d_unsafe(0, tuft_count, tuft_id, grp_count, grp_id);
	uint const pyr_axn_idz = mul24(grp_id, cols_per_grp);
 
	//for (uint pyr_idx = pyr_idz; pyr_idx < pyr_idn; pyr_idx++) {
	for (uint i = 0; i < cols_per_grp; i++) {
		uint const pyr_idx = pyr_idz + i;
		uint const pyr_axn_idx = pyr_axn_idz + i;

		uchar pyr_best_den_id = pyr_best_den_ids[pyr_idx];
		uchar pyr_flag_set = pyr_flag_sets[pyr_idx];

		int pyr_concrete = axn_states[pyr_axn_idx] != 0;
		int pyr_fuzzy = pyr_preds[pyr_idx] != 0;

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

		pyr_flag_sets[pyr_idx] = pyr_flag_set;
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
// TODO: CONVERT TO 3 WORK DIMS
__kernel void mcol_output(
				__global uchar const* const pyr_preds,
				__global uchar const* const pyr_best_den_states,
				__private uint const sst_axn_idz,
				__private uchar const pyr_depth,
				__private uchar const aff_out_axn_slc,
				__global uchar* const mcol_pred_totals,
				__global uchar* const mcol_best_pyr_den_states,
				__global uchar* const axn_states)
{
	uint const slc_id = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	int idx_is_safe = 0;
	uint const aff_out_axn_idx = axn_idx_3d_unsafe(aff_out_axn_slc + slc_id, v_id, 0, u_id, 0, &idx_is_safe);
	uint const col_id = cel_idx_3d_unsafe(0, v_size, v_id, u_size, u_id);
	//uint const pyr_axn_idx = axn_idx_2d( + slc_id, slc_columns, col_id, 0);
	//uint const col_id = mad24(slc_id, slc_columns, col_id);

	int sst_axn_state = axn_states[sst_axn_idz + col_id];
	uchar max_den_state = 0;
	int col_pyr_pred_total = 0;

	for (uint i = 0; i < pyr_depth; i++) {
		// POTENTIALLY FALSE ASSUMPTION HERE ABOUT PYR CELLS ALL BEING INVOLVED IN OUTPUT
		//uint pyr_idx = mad24(i, slc_columns, col_id);
		uint pyr_idx = cel_idx_3d_unsafe(i, v_size, v_id, u_size, u_id);

		uchar pyr_best_den_state = pyr_best_den_states[pyr_idx];
		uchar pyr_pred = pyr_preds[pyr_idx];

		max_den_state = max(max_den_state, pyr_best_den_state);
		
		col_pyr_pred_total = max(col_pyr_pred_total, (int)pyr_pred);
	}


	mcol_pred_totals[col_id] = clamp(col_pyr_pred_total, 0, 255); // <<<<< FIX ME TO BE A FLAGSET
	mcol_best_pyr_den_states[col_id] = max_den_state;
	axn_states[aff_out_axn_idx] = clamp(col_pyr_pred_total + sst_axn_state, 0, 255);
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



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

// SYNAPSE_AXON_BIAS_LOG2: Reduces source axon influence on synaptic dendrite 
#define SYNAPSE_AXON_BIAS_LOG2			2

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
// 		idz := index[0], first element, starting element
//
//		idz_parent: first element within the subset of a parent group
//			- ex.: syn_idx := syn_idz_den + syn_id_den
//
// 		idn := index[len]: element after final element, termination point
//			- ex.: for(int i = 0, i < idn, i++)
//
// 		idm := index[max] := index[len - 1]: final (valid) element just before idn. idn - 1 = idm.
//			- ex.: for(int i = 0, i <= idm, i++)
//
//
// 		id: identifier, but not a physical array index
//
//		id_parent: identifier within the subset of a parent group
//			- ex.: syn_idx := syn_idz_den + syn_id_den
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


static inline uint get_axn_idz(uchar slc_id) {
	return axn_slc_idzs[slc_id];
}

static inline int4 get_axn_idz_vec4(uchar4 slc_id) {
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
static inline int w_ofs(int const v_ofs, int const u_ofs) {
	return (0 - v_ofs) - u_ofs;
}


static inline int square(int const x) {
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


// COORD_IS_SAFE(): BOUNDS CHECK FOR A SINGLE DIMENSION OF A CELLULAR COORDINATE
static inline int coord_is_safe(int const dim_size, int const coord_id, int const coord_ofs) {
	int coord_ttl = coord_id + coord_ofs;
	return (coord_ttl >= 0) & (coord_ttl < dim_size);
}


// COORD_IS_SAFE_VEC4(): BOUNDS CHECK FOR A SINGLE DIMENSION OF A CELLULAR COORDINATE
static inline int4 coord_is_safe_vec4(int4 const dim_size, int4 const coord_id, int4 const coord_ofs) {
	int4 coord_ttl = coord_id + coord_ofs;
	return (coord_ttl >= 0) & (coord_ttl < dim_size);
}


/*=============================================================================
================================ CELL INDEXING ================================
=============================================================================*/

// CEL_IDX_3D_UNSAFE(): LINEAR INDEX OF A CELL - NOT ACCURATE FOR AXONS
static inline uint cel_idx_3d_unsafe(uint slc_id_lyr, uint v_size, uint v_id, uint u_size, uint u_id) {
	return mad24(slc_id_lyr, mul24(v_size, u_size), mad24(v_id, u_size, u_id));	
}

// CEL_IDX_3D_UNSAFE_VEC4(): LINEAR INDEX OF A CELL - NOT FOR ACCURATE AXONS
static inline int4 cel_idx_3d_unsafe_vec4(uchar4 slc_id_lyr_uchar4, int4 v_size, int4 v_id, int4 u_size, int4 u_id) {
	int4 slc_id_lyr = convert_int4(slc_id_lyr_uchar4);
	return mad24(slc_id_lyr, mul24(v_size, u_size), mad24(v_id, u_size, u_id));	
}

// 	SAFE_CEL_STATE_3D(): 'Safe' Cell State Resolution
// 		- If id + ofs are out of cortical bounds, zero is returned
//			- otherwise resolved state is returned 
//		- Intended primarily for use by the inhibition-related kernel(s)
static inline uchar cel_state_3d_safe(uchar slc_id_lyr, 
				uint v_size, uint v_id, char v_ofs, 
				uint u_size, uint u_id, char u_ofs, 
				__global uchar const* const cel_states) 
{
	int v_ofs_is_safe = coord_is_safe(v_size, v_id, v_ofs);
	int u_ofs_is_safe = coord_is_safe(u_size, u_id, u_ofs);
	int cel_idx_is_safe = v_ofs_is_safe & u_ofs_is_safe;

	uint cel_idx = cel_idx_3d_unsafe(slc_id_lyr, v_size, (int)v_id + v_ofs, u_size, (int)u_id + u_ofs);

	return mul24(cel_idx_is_safe, cel_states[cel_idx]);
}


/*=============================================================================
================================ AXON INDEXING ================================
=============================================================================*/

// AXN_IDX_3D_UNSAFE(): Linear index of an axon
// 		- Using ints as intermediate variables to be consistent with vectorized version 
// 			(will only affect invalid indexes)
static inline uint axn_idx_3d_unsafe(uchar const slc_id, uint const v_id_unscaled, 
			char const v_ofs, uint const u_id_unscaled, char const u_ofs, int* const idx_is_safe) 
	{
	// GET THE DIM SIZES:
	int const v_size = get_axn_v_size(slc_id);
	int const u_size = get_axn_u_size(slc_id);

	// 	CALCULATE SCALED INDEX:
	// 		- Multiply by the pre-defined scale for specified slice then divide by 16.
	// 		- A scale of 16 = 100%, 8 = 50%, 32 = 200%, etc.
	int const v_id_scaled = (mul24(v_id_unscaled, get_axn_v_scale(slc_id)) >> 4);
	int const u_id_scaled = (mul24(u_id_unscaled, get_axn_u_scale(slc_id)) >> 4);

	// CALCULATE HORIZONTAL INDEX:
	int const v_id_hrz = v_size >> 1;
	int const u_id_hrz = u_size >> 1;	

	// DETERMINE IF THIS IS A HORIZONTAL SLICE:
	int const idx_is_hrz = slc_id >= HORIZONTAL_AXON_ROW_DEMARCATION;

	// IF SLICE IS HORIZONTAL ASSIGN CORRESPONDING ID AND VICE VERSA:
	int const v_id = mad24(idx_is_hrz, v_id_hrz, mul24(!idx_is_hrz, v_id_scaled));
	int const u_id = mad24(idx_is_hrz, u_id_hrz, mul24(!idx_is_hrz, u_id_scaled));
		
	// CHECK SAFETY:
	*idx_is_safe = coord_is_safe(v_size, v_id, v_ofs) & coord_is_safe(u_size, u_id, u_ofs);

	// RETURN the sum of the pre-defined axon offset for the slice and the linear offset within that slice:
	return get_axn_idz(slc_id) + (uint)(mad24(v_id + v_ofs, u_size, u_id + u_ofs));
}

// AXN_IDX_3D_UNSAFE_VEC4(): Linear index of an axon, vec4
static inline int4 axn_idx_3d_unsafe_vec4(uchar4 const slc_id, int4 const v_id_unscaled, 
		char4 const v_ofs_char4, int4 const u_id_unscaled, char4 const u_ofs_char4, int4* const idx_is_safe)
{
	int4 const v_ofs = convert_int4(v_ofs_char4);
	int4 const u_ofs = convert_int4(u_ofs_char4);

	int4 const v_size = get_axn_v_size_vec4(slc_id);
	int4 const u_size = get_axn_u_size_vec4(slc_id);

	int4 const v_id_scaled = (mul24(v_id_unscaled, get_axn_v_scale_vec4(slc_id)) >> 4);
	int4 const u_id_scaled = (mul24(u_id_unscaled, get_axn_u_scale_vec4(slc_id)) >> 4);

	int4 const v_id_hrz = v_size >> 1;
	int4 const u_id_hrz = u_size >> 1;

	int4 const idx_is_hrz = convert_int4(slc_id) >= (int4)HORIZONTAL_AXON_ROW_DEMARCATION;

	int4 const v_id = (idx_is_hrz & v_id_hrz) | (~idx_is_hrz & v_id_scaled);
	int4 const u_id = (idx_is_hrz & u_id_hrz) | (~idx_is_hrz & u_id_scaled);

	*idx_is_safe = coord_is_safe_vec4(v_size, v_id, v_ofs) & coord_is_safe_vec4(u_size, u_id, u_ofs);

	return get_axn_idz_vec4(slc_id) + mad24(v_id + v_ofs, u_size, u_id + u_ofs);
}


/*===========================================================================*/

// AXN_STATE_3D_SAFE():
static inline uchar axn_state_3d_safe(uchar slc_id, uint v_id, char v_ofs, uint u_id, 
			char u_ofs, __global uchar const* const axn_states) 
{
	int idx_is_safe = 0;
	uint axn_idx = axn_idx_3d_unsafe(slc_id, v_id, v_ofs, u_id, u_ofs, &idx_is_safe);
	return mul24(idx_is_safe, axn_states[axn_idx]);
}

// AXN_STATE_3D_SAFE_VEC4():
static inline uchar4 axn_state_3d_safe_vec4(uchar4 slc_id, int4 v_id, char4 v_ofs, 
	int4 u_id, char4 u_ofs, __global uchar const* const axn_states) 
{
	int4 idx_is_safe = (int4)0;
	int4 axn_idx = axn_idx_3d_unsafe_vec4(slc_id, v_id, v_ofs, u_id, u_ofs, &idx_is_safe);

	return (uchar4)(
		((uchar)idx_is_safe.s0 & axn_states[axn_idx.s0]),
		((uchar)idx_is_safe.s1 & axn_states[axn_idx.s1]),
		((uchar)idx_is_safe.s2 & axn_states[axn_idx.s2]),
		((uchar)idx_is_safe.s3 & axn_states[axn_idx.s3]));

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
	//axn_idx_hrz(0, 0, 0, 0, 0);
	//coord_is_safe_vec4((int4)0, (int4)0, (int4)0);
	//axn_idx_hrz_vec4((int4)0, (int4)0, (int4)0, (int4)0, (int4)0);
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

	// uchar den_energy = den_energies[den_idx];

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

	// if (syn_sum != 0) {
	// 	if (den_energy >= ENERGY_LEVEL_MIN) {
	// 		den_energy -= ENERGY_LEVEL_MIN;
	// 	} else {
	// 		den_energy += ENERGY_REGEN_AMOUNT;
	// 		syn_sum = 0;
	// 	}
	// } else {
	// 	if (den_energy < ENERGY_LEVEL_MAX) {
	// 		den_energy += ENERGY_REGEN_AMOUNT;
	// 	}
	// }

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
				// __global int* const aux_ints_1,
				__global uchar* const axn_states)
{
	uint const slc_id_lyr = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);
	uint const cel_idx = cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id);
	//uint const axn_idx = axn_idx_3d_safe(slc_id_lyr + cel_base_axn_slc, v_size, v_id, 0, u_size, u_id, 0);
	int idx_is_safe = 0;
	uint const cel_axn_idx = axn_idx_3d_unsafe(slc_id_lyr + cel_base_axn_slc, v_id, 0, u_id, 0, &idx_is_safe);

	uchar const cel_state = mul24(idx_is_safe, (int)cel_states[cel_idx]);

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
				= cel_state_3d_safe(slc_id_lyr, v_size, v_id, v_ofs, u_size, u_id, u_ofs, cel_states);	// ORIGINAL		
			//uchar neighbor_state = cel_states[
			//cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id + v_ofs, u_size, u_id + u_ofs)]; // DEBUG


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
				uint unsafe_target_axn_idx = axn_idx_3d_safe(slc_id_lyr + cel_base_axn_slc, v_size, v_id, v_ofs, u_size, u_id, u_ofs);

				//aux_ints_1[dumb_iter] = cel_state_3d_safe(slc_id_lyr, 
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
			// 	// 	//aux_ints_1[axn_idx_3d_safe(slc_id_lyr + cel_base_axn_slc, v_size, v_id, v_ofs, u_size, u_id, u_ofs)] = distance;
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
	uint const slc_id_lyr = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	uint const cel_idx = cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id);
	//uint const axn_idx = axn_idx_3d_safe(slc_id_lyr + cel_base_axn_slc, v_size, v_id, 0, u_size, u_id, 0);
	int idx_is_safe = 0;
	uint const cel_axn_idx = axn_idx_3d_unsafe(slc_id_lyr + cel_base_axn_slc, v_id, 0, u_id, 0, &idx_is_safe);

	//uchar const cel_state = mul24(idx_is_safe, (int)cel_states[cel_idx]);
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
				// __global int* const aux_ints_0,
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
// <<<<< NOTE: THIS KERNEL MAY BE FLAWED WHEN USED WITH MULTIPLE TUFTS - SEE PYR_LTP >>>>>
__kernel void sst_ltp(
				__global uchar const* const axn_states,
				__global uchar const* const syn_states,
				__private uint const cel_lyr_axn_idz,
				__private uint const cols_per_grp,
				__private uchar const syns_per_tuft_l2,
				__private uint const rnd,
				// __global int* const aux_ints_0,
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
				__global uchar const* const mcol_flag_sets, // COL
				__global uchar const* const mcol_best_pyr_den_states,
				__global uchar const* const pyr_best_den_ids,
				// ADD PYR BEST DEN STATE NOW THAT WE'VE ADDED IT (and to another kernel somewhere also)
				__global uchar const* const den_states,
				__private uint const ssts_axn_idz,
				__private uchar const pyr_axn_slc_base,
				__private uchar const dens_per_tuft_l2,
				__global uchar* const pyr_flag_sets,
				__global uchar* const pyr_states,
				// __global int* const aux_ints_0,
				__global uchar* const axn_states) 
{
	uint const slc_id_lyr = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);
	//uint const col_id = get_global_id(1);

	uint const slc_columns = get_global_size(1);
	//uint const pyr_idx = mad24(slc_id_lyr, slc_columns, col_id);
	//uint const axn_idx = axn_idx_2d(pyr_axn_slc_base + slc_id_lyr, slc_columns, col_id, 0);
	uint const pyr_idx = cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id);
	int idx_is_safe = 0;
	uint const cel_axn_idx = axn_idx_3d_unsafe(pyr_axn_slc_base + slc_id_lyr, v_id, 0, u_id, 0, &idx_is_safe);
	uint const col_id = cel_idx_3d_unsafe(0, v_size, v_id, u_size, u_id);

	// ******************

	uint const den_ofs = pyr_idx << dens_per_tuft_l2;			// REPLACE
	uint const best_den_idx = den_ofs + pyr_best_den_ids[pyr_idx];		// REPLACE

	uchar const best_den_state = den_states[best_den_idx];				// CHANGE

	uchar const mcol_best_col_den_state = mcol_best_pyr_den_states[col_id];
	uchar const sst_axn_state = axn_states[ssts_axn_idz + col_id];
	//uchar const mcol_state = mcol_states[col_id];
	uchar const mcol_flag_set = mcol_flag_sets[col_id];
	uchar const pyr_pred = pyr_states[pyr_idx];
	uchar pyr_flag_set = pyr_flag_sets[pyr_idx];

	//aux_ints_0[pyr_idx] = pyr_flag_set;

	int const mcol_active = sst_axn_state != 0;
	//int const mcol_active = mcol_state != 0;
	int const mcol_any_pred = mcol_flag_set & MCOL_IS_PREDICTIVE_FLAG == MCOL_IS_PREDICTIVE_FLAG;
	int const pyr_predictive = (pyr_pred != 0);

	int const crystal = pyr_predictive && mcol_active;
	int const anomaly = mcol_active && !mcol_any_pred;

	//int const activate_axon = crystal || anomaly;
	//pyr_pred = (crystal | anomaly) && (mcol_state);
	//pyr_pred = mul24(((crystal != 0) || (anomaly != 0)), mcol_state);
	pyr_flag_set &= ~PYR_BEST_IN_COL_FLAG;
	
	//pyr_flag_set |= mul24(best_den_state == mcol_best_col_den_state, PYR_BEST_IN_COL_FLAG);
	//pyr_flag_set |= mul24((mcol_best_col_den_state == best_den_state) && pyr_predictive, 
	//	PYR_BEST_IN_COL_FLAG);
	pyr_flag_set |= mul24(best_den_state != 0 && best_den_state == mcol_best_col_den_state, 
		PYR_BEST_IN_COL_FLAG);


	// SHOULDN'T BE ACTIVATING IF OTHER PYRS IN COLUMN ARE PREDICTIVE

	axn_states[cel_axn_idx] = (uchar)mad24(anomaly, (int)sst_axn_state, mul24(crystal, (int)pyr_pred));
	//axn_states[axn_idx] = (uchar)mad24(anomaly, (int)mcol_state, mul24(crystal, (int)pyr_pred));

	pyr_flag_sets[pyr_idx] = pyr_flag_set;

	//pyr_states[pyr_idx] = pyr_pred;

	//aux_ints_0[pyr_idx] = 5;
	//aux_ints_0[pyr_idx] = pyr_pred;
}







// PYRS_LTP(): Pyramidal long term potentiation and depression - adjusting synapse strengths
//
//	- For each pyramidal cell:
//		- if cell axon is currently active:
//			- cause learning to take place on its most active dendrite
//		- if cell axon is currently inactive:
//			- check to see if the cell's axon was previously active (by checking flag_set)
//				- if so, depress (reduce strengths of) any currently active synapses
//					- NOTE: The reasoning here is that any synapses which are active just after 
//					(but not before) the cell was active are likely to be unrelated to its prior 
//					activity. In other words, a rough implementation of 'real' LTD.
//
//	- TODO:
//		- [incomplete] Vectorize (should be highly vectorizable)
//		- reducing branching will be tough with this one
//		- [in progress] Tests (check that flag_set and prev_best_den_id are robustly maintained)
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

/* INDEXING EXAMPLE:

		Let's imagine we have an area in the cortex with the following properties:

			COL_COUNT (column count for area): 1024

		This area has a layer of pyramidal cells (such as layer iii) with:

			DEPTH / CELS_PER_COL (cells per column, aka. layer depth): 3
			TFTS_PER_CEL (tufts per cell): 2
			DENS_PER_TFT (dendrites per tuft): 4
			SYNS_PER_DEN (synapses per dendrite): 32

		So we have a layer 3 cells deep, each layer containing 1024 cells. We can also think of it as 1024 columns, each with three cells. Each of the cells in a column share the same spatial input axon. That is, cells are activated (in the previous kernel) based on the same spatial input axon.

		A few things to keep in mind when indexing axons, cells, dendrites, and synapses: 
			- The axons will correspond to the cell indexes 1:1, just with a different idz (starting index).
			- Synapses and dendrites is where things are trickier. Synapse (and dendrite) space (within the syn_states[] array) is primarily divided by tuft, unintuitively. For an explanation and more information see 'synapses.rs'.
				- First let's calculate our den_idx:
					den_idx := 

					{ INCOMPLETE - LOOK AT DENDRITE INDEXING }

				- Synapse indexes will first need to be calculated using the dendrite index, i.e.:
					syn_id_tuft := den_idx * syns_per_den (or equivalantly: den_idx << syns_per_den_l2).
				- Next they will be added to the tuft offset:
					- syns_per_tuft_space := syns_per_den * dens_per_tuft * cels_per_col * col_count
						- (note that tufts per cell is not factored in here)
					- syn_idz_tuft := tuft_id * syns_per_tuft_space
					- syn_idx := syn_idz_tuft + syn_id_tuft


		So, here's an example breakdown of how this all plays out:
			- Notation: OBJ[id_parent]:[idx_physical] 
				 - More plainly: 'object [ id within parent object ]:[ global (physical) index ]'
			
			-----------------------

			CEL[0]:[0] (COL[0])
				TFT[0]
					DEN[0]:[0]
						SYN[0]:[0], SYN[1]:[1], ..., SYN[31]:[31]
					DEN[1]:[1]
						SYN[0]:[32], SYN[1]:[33], ..., SYN[31]:[68]
					...	
					DEN[3]:[3]
						SYN[0]:[96], SYN[1]:[97], ..., SYN[31]:[127]
				TFT[1]
					DEN[0]:[4]
						SYN[0]:[393216], SYN[1]:[393217], ..., SYN[31]:[393247]
					...	
					DEN[3]:[7]
						SYN[0]:[393312], SYN[1]:[393313], ..., SYN[31]:[393343]
			CEL[1]:1 (COL[0])
				TFT[0]
					DEN[0]:[8]
						SYN[0]:[0], SYN[1]:[1], ..., SYN[31]:[31]
					...	
					DEN[3][11]
						SYN[0]:[96], SYN[1]:[97], ..., SYN[31]:[127]
				TFT[1]
					DEN[0][12]
						SYN[0]:[393216], SYN[1]:[393217], ..., SYN[31]:[393247]
					...	
			CEL[2]:[2] (COL[0])
				...
			CEL[3]:[3] (COL[1])
				...
			CEL[5]:[5] (COL[1])
				...
			CEL[6]:[6] (COL[2])
				...
			...
			CEL[3071]:[3071] (COL[1023])
				...

			-----------------------
		
		Given that indexing structure, the following kernel structure appears to be the best balance of performance and simplicity:

			Kernel: WorkSize: OneDim(cell groups**) - Iterate through cell groups
				- Loop: Cells - Iterate through cells within each group.
					- Loop: Tufts - Iterate through cell tufts.
						- Function call: For each tuft, if work needs to be done, a function is called which will ->
						  Loop: Synapses - Unroll tuft synapses.
							- STP, LTP, and LTD take place on synapses within this smallest loop.



		** Note: Cell groups are divisions of the cell space for the layer into groups of arbitrary size. Cell groups are used in lieu of individual cells as the primary work dimension because during any given cycle. Most cells will need no work done on its synapses, therefore most work items would be idle. By bundling a group of cells into each work item, all threads can keep busy.

*/
// <<<<< TODO: FIX: NOT TAKING IN TO ACCOUNT MULTIPLE TUFTS! MAJOR INDEXING PROBLEMS >>>>>
__kernel void pyrs_ltp(
				__global uchar const* const axn_states,
				__global uchar const* const cel_fuzzyness,
				__global uchar const* const cel_best_den_ids,
				__global uchar const* const den_states,
				__global uchar const* const syn_states,
				__private uint const dens_per_tuft_l2,
				__private uint const syns_per_den_l2,
				__private uint const cols_per_grp,
				__private uint const axn_idz_cel_lyr,
				__private uint const rnd,
				__global uchar* const syn_flag_sets,
				__global uchar* const cel_flag_sets,
				__global int* const aux_ints_0,
				// __global int* const aux_ints_1,
				__global char* const syn_strengths) 
{
	uint const tuft_id = get_global_id(0);
	uint const slc_id_lyr = get_global_id(1);	
	uint const grp_id = get_global_id(2);

	uint const cel_id = get_global_id(0);

	uint const tuft_count = get_global_size(1);
	uint const grp_count = get_global_size(2); // GRP_COUNT: COLUMNS / COLS_PER_GRP

	uint const cel_grp_id = cel_idx_3d_unsafe(slc_id_lyr, tuft_count, tuft_id, grp_count, grp_id);
	uint const cel_idz_grp = mul24(grp_id, cols_per_grp);

	uint const axn_grp_id = get_axn_idz(slc_id_lyr) + cel_idx_3d_unsafe(0, tuft_count, tuft_id, grp_count, grp_id);
	//uint const cel_axn_idz = mul24(grp_id, cols_per_grp);
	uint const axn_idz_cel_slc = axn_idz_cel_lyr + cel_idz_grp;
 
	//for (uint cel_idx = cel_idz_grp; cel_idx < cel_idn; cel_idx++) {
	for (uint cel_id_grp = 0; cel_id_grp < cols_per_grp; cel_id_grp++) {
		uint const cel_idx = cel_idz_grp + cel_id_grp;
		uint const cel_axn_idx = axn_idz_cel_slc + cel_id_grp;

		uchar cel_best_den_id = cel_best_den_ids[cel_idx];
		uchar cel_flag_set = cel_flag_sets[cel_idx];

		int cel_concrete = axn_states[cel_axn_idx] != 0;
		int cel_fuzzy = cel_fuzzyness[cel_idx] != 0;

		int cel_prev_concrete = (cel_flag_set & PYR_PREV_CONCRETE_FLAG) == PYR_PREV_CONCRETE_FLAG;
		int cel_prev_fuzzy = (cel_flag_set & PYR_PREV_FUZZY_FLAG) == PYR_PREV_FUZZY_FLAG;
		int cel_best_in_col = (cel_flag_set & PYR_BEST_IN_COL_FLAG) == PYR_BEST_IN_COL_FLAG;		

		// NOT TAKING IN TO ACCOUNT TUFTS!
		uint den_idx_base = cel_idx << dens_per_tuft_l2;
		uint cel_syn_idz = ((den_idx_base) << syns_per_den_l2);	 // WHOLE CELL -- BROKEN
		uint best_den_syn_idz = (den_idx_base + cel_best_den_id) << syns_per_den_l2; // BROKEN

		// TESTING 
		// dst_syns__active__stp_ltd(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);

		if (cel_concrete) {
			// aux_ints_0[cel_idx] = 10;
			aux_ints_0[cel_idx] = cel_syn_idz;

			if (cel_prev_fuzzy) { 
				// PREVIOUS (CORRECT) PREDICTION (EVERY PYR IN COL): REINFORCE DEN + TRAIN NEW DEN
				// SAME AS ANO + TRAIN A SECOND REDUNDANT DENDRITE AS WELL (OR DON'T)
				dst_syns__active__stp_ltd(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
				// aux_ints_0[cel_idx] = 11;

			} else if (cel_best_in_col) { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN 
			//} else { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
				dst_syns__active__stp_ltd(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
				// aux_ints_0[cel_idx] = 12;
			}

			cel_flag_set |= PYR_PREV_CONCRETE_FLAG;

		} else if (cel_prev_concrete) {
			cel_syns_trm(syn_states, cel_syn_idz, syns_per_den_l2 + dens_per_tuft_l2, rnd, syn_flag_sets, syn_strengths);
			// aux_ints_0[cel_idx] = 20;

			cel_flag_set &= ~PYR_PREV_CONCRETE_FLAG;
		}


		cel_flag_set &= ~PYR_PREV_FUZZY_FLAG;
		cel_flag_set |= mul24(cel_fuzzy, PYR_PREV_FUZZY_FLAG);

		cel_flag_sets[cel_idx] = cel_flag_set;
	}
}




__kernel void pyr_cycle(
				__global uchar const* const den_states,
				__global uchar const* const den_states_raw,
				__private uint const den_tufts_per_cel,
				__private uchar const dens_per_tuft_l2,
				__global uchar* const pyr_best_den_ids,
				__global uchar* const pyr_best_den_states,
				__global uchar* const pyr_states) 
{
	uint const pyr_idx = get_global_id(0);
	uchar best_den_state = 0;
	uchar best_den_id = 0;
	//uchar pyr_state = 0;
	int pyr_state = 0;

	for (uint den_tuft = 0; den_tuft < den_tufts_per_cel; den_tuft++) {
		uint const den_idz = mad24(den_tuft, get_global_size(0), pyr_idx) << dens_per_tuft_l2;
 
		for (uint den_idx = 0; den_idx < (1 << dens_per_tuft_l2); den_idx++) {
			uchar den_state = den_states[den_idz + den_idx];
			int den_state_bigger = (den_state > best_den_state);

			best_den_id = mad24(den_state_bigger, (int)den_idx, mul24(!den_state_bigger, best_den_id));
			best_den_state = mad24(den_state_bigger, den_state, mul24(!den_state_bigger, best_den_state));
			pyr_state += den_state;
		}

		// TESTING: 
		// if (den_tuft == 0) {
		// 	// pyr_state = 200;
		// 	pyr_state = 1;
		// }
	}

	//pyr_state = best_den_state;

	pyr_best_den_ids[pyr_idx] = best_den_id;
	pyr_best_den_states[pyr_idx] = best_den_state;
	pyr_states[pyr_idx] = clamp(pyr_state, 0, 255);
}


//	COL_OUTPUT()
//		- rename coming
//
// TODO: CONVERT TO 3 WORK DIMS
__kernel void mcol_output(
				__global uchar const* const pyr_states,
				__global uchar const* const pyr_best_den_states,
				__private uint const sst_axn_idz,
				__private uchar const pyr_depth,
				__private uchar const aff_out_axn_slc,
				__global uchar* const mcol_flag_sets,
				__global uchar* const mcol_best_pyr_den_states,
				// __global int* const aux_ints_0,
				__global uchar* const axn_states)
{
	uint const slc_id_lyr = get_global_id(0);
	uint const v_id = get_global_id(1);
	uint const u_id = get_global_id(2);
	uint const v_size = get_global_size(1);
	uint const u_size = get_global_size(2);

	int idx_is_safe = 0;
	uint const aff_out_axn_idx = axn_idx_3d_unsafe(aff_out_axn_slc + slc_id_lyr, v_id, 0, u_id, 0, &idx_is_safe);
	uint const col_id = cel_idx_3d_unsafe(0, v_size, v_id, u_size, u_id);
	//uint const pyr_axn_idx = axn_idx_2d( + slc_id_lyr, slc_columns, col_id, 0);
	//uint const col_id = mad24(slc_id_lyr, slc_columns, col_id);

	int sst_axn_state = axn_states[sst_axn_idz + col_id];
	uchar max_den_state = 0;
	int col_pyr_pred_total = 0;

	for (uint i = 0; i < pyr_depth; i++) {
		// POTENTIALLY FALSE ASSUMPTION HERE ABOUT PYR CELLS ALL BEING INVOLVED IN OUTPUT
		//uint pyr_idx = mad24(i, slc_columns, col_id);
		uint pyr_idx = cel_idx_3d_unsafe(i, v_size, v_id, u_size, u_id);

		uchar pyr_best_den_state = pyr_best_den_states[pyr_idx];
		uchar pyr_pred = pyr_states[pyr_idx];

		max_den_state = max(max_den_state, pyr_best_den_state);
		
		col_pyr_pred_total = max(col_pyr_pred_total, (int)pyr_pred);
	}


	mcol_flag_sets[col_id] = mul24((col_pyr_pred_total > 0), MCOL_IS_PREDICTIVE_FLAG);
	mcol_best_pyr_den_states[col_id] = max_den_state;
	//axn_states[aff_out_axn_idx] = mul24(idx_is_safe, clamp(col_pyr_pred_total + sst_axn_state, 0, 255)); // N1
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




// __kernel void pyrs_ltp_unoptd_broken(
// 				__global uchar const* const axn_states,
// 				__global uchar const* const pyr_states,
// 				__global uchar const* const pyr_best_den_ids,
// 				__global uchar const* const den_states,
// 				__global uchar const* const syn_states,
// 				__private uint const axn_idz_pyr_lyr, 
// 				__private uint const syns_per_den_l2,
// 				__private uint const dens_per_tuft_l2,
// 				__private uint const cels_per_grp,
// 				__private uint const rnd,
// 				__global uchar* const syn_flag_sets,
// 				__global uchar* const pyr_flag_sets,
// 				__global int* const aux_ints_0,
// 				//__global int* const aux_ints_1,
// 				__global char* const syn_strengths) 
// {
// 	uint const slc_id_lyr = get_global_id(0);
// 	uint const tuft_id = get_global_id(1);
// 	uint const grp_id = get_global_id(2);
// 	uint const tuft_count = get_global_size(1);	
// 	uint const grp_count = get_global_size(2); // GRP_COUNT: COLUMNS / COLS_PER_GRP

// 	uint const pyr_grp_id = cel_idx_3d_unsafe(slc_id_lyr, tuft_count, tuft_id, grp_count, grp_id);
// 	uint const pyr_idz = mul24(grp_id, cels_per_grp);

// 	uint const axn_grp_id = get_axn_idz(slc_id_lyr) + cel_idx_3d_unsafe(0, tuft_count, tuft_id, grp_count, grp_id);
// 	//uint const pyr_axn_idz = mul24(grp_id, cels_per_grp);
// 	uint const axn_idz_pyr_slc = axn_idz_pyr_lyr + pyr_idz;
 
// 	//for (uint pyr_idx = pyr_idz; pyr_idx < pyr_idn; pyr_idx++) {
// 	for (uint i = 0; i < cels_per_grp; i++) {
// 		uint const pyr_idx = pyr_idz + i;
// 		uint const pyr_axn_idx = axn_idz_pyr_slc + i;

// 		uchar pyr_best_den_id = pyr_best_den_ids[pyr_idx];
// 		uchar pyr_flag_set = pyr_flag_sets[pyr_idx];

// 		int pyr_concrete = axn_states[pyr_axn_idx] != 0;
// 		int pyr_fuzzy = pyr_states[pyr_idx] != 0;

// 		int pyr_prev_concrete = (pyr_flag_set & PYR_PREV_CONCRETE_FLAG) == PYR_PREV_CONCRETE_FLAG;
// 		int pyr_prev_fuzzy = (pyr_flag_set & PYR_PREV_FUZZY_FLAG) == PYR_PREV_FUZZY_FLAG;
// 		int pyr_best_in_col = (pyr_flag_set & PYR_BEST_IN_COL_FLAG) == PYR_BEST_IN_COL_FLAG;

// 		uint den_idx_base = i << dens_per_tuft_l2;

// 		uint pyr_syn_idz = ((den_idx_base) << syns_per_den_l2);	 // WHOLE CELL
// 		uint best_den_syn_idz = (den_idx_base + pyr_best_den_id) << syns_per_den_l2;

// 		//

// 		// TESTING 
// 		//dst_syns__active__stp_ltd(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);

// 		if (pyr_concrete) {
// 			aux_ints_0[pyr_idx] = 1;
// 			if (pyr_prev_fuzzy) { 
// 				// PREVIOUS (CORRECT) PREDICTION (EVERY PYR IN COL): REINFORCE DEN + TRAIN NEW DEN
// 				// SAME AS ANO + TRAIN A SECOND REDUNDANT DENDRITE AS WELL (OR DON'T)
// 				dst_syns__active__stp_ltd(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
// 				aux_ints_0[pyr_idx] = 2;

// 			} else if (pyr_best_in_col) { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN 
// 			//} else { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
// 				dst_syns__active__stp_ltd(syn_states, best_den_syn_idz, syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
// 				aux_ints_0[pyr_idx] = 3;
// 			}

// 			pyr_flag_set |= PYR_PREV_CONCRETE_FLAG;

// 		} else if (pyr_prev_concrete) {
// 			cel_syns_trm(syn_states, pyr_syn_idz, syns_per_den_l2 + dens_per_tuft_l2, rnd, syn_flag_sets, syn_strengths);
// 			aux_ints_0[pyr_idx] = 4;

// 			pyr_flag_set &= ~PYR_PREV_CONCRETE_FLAG;
// 		}


// 		pyr_flag_set &= ~PYR_PREV_FUZZY_FLAG;
// 		pyr_flag_set |= mul24(pyr_fuzzy, PYR_PREV_FUZZY_FLAG);

// 		pyr_flag_sets[pyr_idx] = pyr_flag_set;
// 	}
// }



/*===========================================================================*/

// AXN_IDX_HRZ(): Axon index for a horizontal axon
// <<<<< TODO: DEPRICATE >>>>>
//		- If axon is not horizontal, return 0
// static inline uint axn_idx_hrz(uchar slc_id, uint v_size, char v_ofs, uint u_size, char u_ofs) {
// 		// HRZ_SCT_ID: Id of horizontal section (basically a sub-slice_id)
// 		int hrz_sct_id = slc_id - HORIZONTAL_AXON_ROW_DEMARCATION;

// 		// 	IDX_HRZ_SCT: Axon position within horizontal section
// 		// 		- AXON_MARGIN_SIZE := Dead center of section
// 		// 		- SYNAPSE_SPAN_RHOMBAL_AREA used in lieu of u_size because indexes are bounded 
// 		//			by the horizontal section rather than the entire slice.
// 		uint idx_hrz_sct = AXON_MARGIN_SIZE + mad24((int)v_ofs, (int)SYNAPSE_SPAN_RHOMBAL_AREA, (int)u_ofs);

// 		// HRZ_AXN_ID: Position within slice
// 		uint hrz_axn_id = mad24(hrz_sct_id,  SYNAPSE_SPAN_RHOMBAL_AREA, (int)idx_hrz_sct);

// 		// AXN_IDX: Physical index within axon space (array)
// 		int axn_idx = mad24((uint)HORIZONTAL_AXON_ROW_DEMARCATION, mul24(u_size, v_size), hrz_axn_id);

// 		// Let's see if our address is even a horizontal one...
// 		int slc_id_is_hrz = hrz_sct_id >= 0;

// 		// If this isn't a horizontal address, return 0, which cannot be a horizontal address
// 		//		- unless, of course, there are only horizontal axons in this space...
// 		return mul24(slc_id_is_hrz, axn_idx);
// }

// AXN_IDX_HRZ_VEC4(): Axon index for a horizontal axon
// <<<<< TODO: DEPRICATE >>>>>
// static inline int4 axn_idx_hrz_vec4(int4 slc_id, int4 v_size, int4 v_ofs, int4 u_size, int4 u_ofs) {
// 		int4 hrz_sct_id = slc_id - (int4)HORIZONTAL_AXON_ROW_DEMARCATION;
// 		int4 idx_hrz_sct = (int4)AXON_MARGIN_SIZE + mad24(v_ofs, (int4)SYNAPSE_SPAN_RHOMBAL_AREA, u_ofs);
// 		int4 hrz_axn_id = mad24(hrz_sct_id,  (int4)SYNAPSE_SPAN_RHOMBAL_AREA, idx_hrz_sct);
// 		int4 axn_idx = mad24((int4)HORIZONTAL_AXON_ROW_DEMARCATION, mul24(u_size, v_size), hrz_axn_id);
// 		int4 slc_id_is_hrz = hrz_sct_id >= 0;
// 		return (slc_id_is_hrz & axn_idx);
// }

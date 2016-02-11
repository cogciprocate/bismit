
// #define LTD_BIAS_LOG2                    0
// #define LTP_BIAS_LOG2                    0
#define ENERGY_SETTING                     4
#define ENERGY_RECHARGE                    1

// #define FLAG_ON(flag_set, mask)            ((flag_set) |= (mask))
// #define FLAG_OFF(flag_set, mask)        ((flag_set) &= ~(mask))
// #define FLAG_EVAL(flag_set, mask)        (((flag_set) & (mask)) == (mask))

#define ENERGY_LEVEL_MIN                9        
#define ENERGY_LEVEL_MAX                255
#define ENERGY_REGEN_AMOUNT                1

// SYNAPSE_AXON_BIAS_LOG2: Reduces source axon influence on synaptic dendrite 
#define SYNAPSE_AXON_BIAS_LOG2            2

// INHIB_RADIUS: A CELL'S SPHERE OF INFLUENCE
#define INHIB_RADIUS                    4
// INHIB_INFL_CENTER_OFFSET: MOVES CENTER OF INHIBITION CURVE NEARER(-) OR FURTHER(+) FROM CELL
#define INHIB_INFL_CENTER_OFFSET        1 
// INHIB_INFL_HORIZ_OFFSET: STRETCHES EDGE OF INHIBITION CURVE NEARER(-) OR FURTHER(+) FROM CELL
#define INHIB_INFL_HORIZ_OFFSET            3

#define RETNAL_THRESHOLD                 48

 /* bismit.cl: CONVENTIONS

        idx: current index (of a loop, workgroup, queue, etc.)
        - almost always a physical in-memory address

        idz := index[0], first element, starting element

        idz_parent: first element within the subset of a parent group
        - ex.: syn_idx := syn_idz_den + syn_id_den

        idn := index[len]: element after final element, termination point
        - ex.: for(int i = 0, i < idn, i++)

        idm := index[max] := index[len - 1]: final (valid) element just before idn. idn - 1 = idm.
        - ex.: for(int i = 0, i <= idm, i++)

        id: identifier, but not a physical array index

        id_{parent}: identifier within the subset of a parent group
        - ex.: syn_idx := syn_idz_den + syn_id_den

        {var_name}_l2 : A scalar representing the log base 2 representation of a value (log2 val).
        - 'lg' is too vague and 'lb' is seldom used therefore l2 is the convention used for logs.

        {var_name}_l2i : A scalar representing the inverse log base 2 of a value (1 / log2 val).

        Coordinates: 
        - slc_id : The id of a slice in axon space (or subset therof such as a layer). This is the 'depth' coordinate corresponding to how far from the top of layer 0/1 we would be in a neocortex.
        - v_id : The 'v' coordinate of a tile (or in this case an axon, cell, etc.) within a slice in hexagonal tile space.
        - u_id : The 'u' coordinate of a tile in hexagonal tile space.
        - w_id : The 'w' coordinate of a tile in hexagonal tile space.

        - Coordinates are oriented (on the unit circle) with 'u' at 30deg, 'v' at 150deg, and 'w' at 270deg. Any references to 'v' are considered to be inverted (negative) when plotting coordinates in real space. In other words a 'v' value of 5 would equal -5 when plotting or mapping to real 2d space. This is simply a convenience ( / necessity?) for indexing in OpenCL.

        - 'w' is seldom used because coordinates are stored in 'axial coordinates' which just means that only two of the three coordinates are actually stored / used because the third can be reconstructed from the other two when needed.


        vat [tenative]: vatic, fuzzyness, level of predictiveness

        ***** High Priority Comment, Temporary Code Change
        <<<<< Medium Priority Comment, To Do
        ##### Debug / Informational Message

        
        Kernel variable order guideline: 
        - __global const* pointers (read-only arrays) first, 
        - __local, __private scalars, etc. in the middle,
        - __global non-const pointers (output arrays) last,
        

        ASSUMPTIONS BEING MADE: (TODO: add assert!s in host)
        - syns_per_tft > 4
        - u_size and v_size (global) are multiples of 8

        vloadn(size_t offset, const __constant gentype *p )
*/





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
__constant uchar axn_slc_v_mids[AXN_SLC_COUNT] = { AXN_SLC_V_MIDS };
__constant uchar axn_slc_u_mids[AXN_SLC_COUNT] = { AXN_SLC_U_MIDS };


static inline uint get_axn_idz(uchar const slc_id) {
    return axn_slc_idzs[slc_id];
}

static inline int4 get_axn_idz_vec4(uchar4 const slc_id) {
    return (int4)(    axn_slc_idzs[slc_id.s0], 
                    axn_slc_idzs[slc_id.s1], 
                    axn_slc_idzs[slc_id.s2], 
                    axn_slc_idzs[slc_id.s3]);
}


static inline uchar get_axn_v_size(uchar const slc_id) {
    return axn_slc_v_sizes[slc_id];
}

static inline int4 get_axn_v_size_vec4(uchar4 const slc_id) {
    return (int4)(    axn_slc_v_sizes[slc_id.s0], 
                    axn_slc_v_sizes[slc_id.s1], 
                    axn_slc_v_sizes[slc_id.s2], 
                    axn_slc_v_sizes[slc_id.s3]);
}

static inline uint get_axn_u_size(uchar const slc_id) {
    return axn_slc_u_sizes[slc_id];
}

static inline int4 get_axn_u_size_vec4(uchar4 const slc_id) {
    return (int4)(    axn_slc_u_sizes[slc_id.s0], 
                    axn_slc_u_sizes[slc_id.s1], 
                    axn_slc_u_sizes[slc_id.s2], 
                    axn_slc_u_sizes[slc_id.s3]);
}


static inline uint get_axn_v_scale(uchar const slc_id) {
    return axn_slc_v_scales[slc_id];
}

static inline int4 get_axn_v_scale_vec4(uchar4 const slc_id) {
    return (int4)(    axn_slc_v_scales[slc_id.s0], 
                    axn_slc_v_scales[slc_id.s1], 
                    axn_slc_v_scales[slc_id.s2], 
                    axn_slc_v_scales[slc_id.s3]);
}

static inline uint get_axn_u_scale(uchar const slc_id) {
    return axn_slc_u_scales[slc_id];
}

static inline int4 get_axn_u_scale_vec4(uchar4 const slc_id) {
    return (int4)(    axn_slc_u_scales[slc_id.s0], 
                    axn_slc_u_scales[slc_id.s1], 
                    axn_slc_u_scales[slc_id.s2], 
                    axn_slc_u_scales[slc_id.s3]);
}


static inline uint get_axn_v_mid(uchar const slc_id) {
    return axn_slc_v_mids[slc_id];
}

static inline int4 get_axn_v_mid_vec4(uchar4 const slc_id) {
    return (int4)(    axn_slc_v_mids[slc_id.s0], 
                    axn_slc_v_mids[slc_id.s1], 
                    axn_slc_v_mids[slc_id.s2], 
                    axn_slc_v_mids[slc_id.s3]);
}

static inline uint get_axn_u_mid(uchar const slc_id) {
    return axn_slc_u_mids[slc_id];
}

static inline int4 get_axn_u_mid_vec4(uchar4 const slc_id) {
    return (int4)(    axn_slc_u_mids[slc_id.s0], 
                    axn_slc_u_mids[slc_id.s1], 
                    axn_slc_u_mids[slc_id.s2], 
                    axn_slc_u_mids[slc_id.s3]);
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

//     W_COORD():
static inline int w_ofs(int const v_ofs, int const u_ofs) {
    return (0 - v_ofs) - u_ofs;
}


static inline int square(int const x) {
    return mul24(x, x);
}


static inline int rnd_mix(int const rnd_a, int seed) {
    seed ^= (seed ^ rnd_a) << 13;
    seed ^= seed >> 17;
    seed ^= seed << 5;
    return seed;
}

static inline uint calc_syn_idz(uint const tuft_id, uint const cel_count, uint const cel_id, 
            uchar const syns_per_tft_l2) 
{
    uint const syn_tuft_ofs = mul24(tuft_id, cel_count) << syns_per_tft_l2;
    return syn_tuft_ofs + (cel_id << syns_per_tft_l2);
}

// static inline uint calc_cel_tft_idx(uint const cels_per_lyr, uint const cel_idx, 
//             uint const tfts_per_cel, uint const tft_id)
// {
//     return mad24(cel_idx, tfts_per_cel, tft_id);
// }

// GET_CEL_TFT_IDX(): Uses same indexing principle as dendrites and synapses (see synapses.rs)
static inline uint calc_cel_tft_idx(uint const cel_count, uint const cel_idx, 
            uint const tfts_per_cel, uint const tft_id)
{
    return  mad24(tft_id, cel_count, cel_idx);
}

// COORD_IS_SAFE(): Bounds check for a single dimension of a cellular coordinate
static inline int coord_is_safe(int const dim_size, int const coord_id, int const coord_ofs) {
    int coord_ttl = coord_id + coord_ofs;
    return (coord_ttl >= 0) & (coord_ttl < dim_size);
}


// COORD_IS_SAFE_VEC4(): Bounds check for a single dimension of a cellular coordinate
static inline int4 coord_is_safe_vec4(int4 const dim_size, int4 const coord_id, int4 const coord_ofs) {
    int4 coord_ttl = coord_id + coord_ofs;
    return (coord_ttl >= 0) & (coord_ttl < dim_size);
}


/*=============================================================================
================================ CELL INDEXING ================================
=============================================================================*/

// CEL_IDX_3D_UNSAFE(): LINEAR INDEX OF A CELL - NOT ACCURATE FOR AXONS
static inline uint cel_idx_3d_unsafe(uint const slc_id_lyr, uint const v_size, uint const v_id, 
            uint const u_size, uint const u_id) 
{
    return mad24(slc_id_lyr, mul24(v_size, u_size), mad24(v_id, u_size, u_id));    
}

// CEL_IDX_3D_UNSAFE_VEC4(): LINEAR INDEX OF A CELL - NOT FOR ACCURATE AXONS
static inline int4 cel_idx_3d_unsafe_vec4(uchar4 const slc_id_lyr_uchar4, int4 const v_size, 
            int4 const v_id, int4 const u_size, int4 const u_id) 
{
    int4 slc_id_lyr = convert_int4(slc_id_lyr_uchar4);
    return mad24(slc_id_lyr, mul24(v_size, u_size), mad24(v_id, u_size, u_id));    
}

//     SAFE_CEL_STATE_3D(): 'Safe' Cell State Resolution
//     - If id + ofs are out of cortical bounds, zero is returned
//        - otherwise resolved state is returned 
//    - Intended primarily for use by the inhibition-related kernel(s)
static inline uchar cel_state_3d_safe(uchar const slc_id_lyr, 
            uint const v_size, uint const v_id, char const v_ofs, 
            uint const u_size, uint const u_id, char const u_ofs, 
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

// <<<<< TODO: 
// - Define a version of axn_idx_3d_safe which does not require '*_ofs' or 
//   'idx_is_safe' pointer.
// - Update axn_idx_3d_unsafe(_*) to re-use function parameter variables rather than
//   create new ones for _id and _ofs.
//    - Find out if the compiler may already be doing this.
// >>>>>
// AXN_IDX_3D_UNSAFE(): Linear index of an axon
// - Using ints as intermediate variables to be consistent with vectorized version 
//   (will only affect invalid indexes)
static inline uint axn_idx_3d_unsafe(uchar const slc_id, uint const v_id_unscaled, 
            char v_ofs, uint const u_id_unscaled, char u_ofs, int* idx_is_safe) 
    {
    // GET THE DIM SIZES:
    int const v_size = get_axn_v_size(slc_id);
    int const u_size = get_axn_u_size(slc_id);

    // ADD MIDDLE OFFSETS (used for horizontal axons) TO SYNAPSE OFFSET:
    v_ofs += get_axn_v_mid(slc_id);
    u_ofs += get_axn_u_mid(slc_id);

    // CALCULATE SCALED INDEX:
    // - Multiply by the pre-defined scale for specified slice then divide by 16.
    // - A scale of 16 = 100%, 8 = 50%, 32 = 200%, etc. A scale of 0 is a horizontal slice.
    int const v_id_scaled = (mul24(v_id_unscaled, get_axn_v_scale(slc_id)) >> 4);
    int const u_id_scaled = (mul24(u_id_unscaled, get_axn_u_scale(slc_id)) >> 4);

    // // CALCULATE HORIZONTAL INDEX:
    // int const v_id_hrz = v_size >> 1;
    // int const u_id_hrz = u_size >> 1;    

    // // DETERMINE IF THIS IS A HORIZONTAL SLICE:
    // int const idx_is_hrz = slc_id >= HORIZONTAL_AXON_ROW_DEMARCATION;

    // // IF SLICE IS HORIZONTAL ASSIGN CORRESPONDING ID AND VICE VERSA:
    // int const v_id = mad24(idx_is_hrz, v_id_hrz, mul24(!idx_is_hrz, v_id_scaled));
    // int const u_id = mad24(idx_is_hrz, u_id_hrz, mul24(!idx_is_hrz, u_id_scaled));
        
    // CHECK SAFETY (bounds):
    *idx_is_safe = coord_is_safe(v_size, v_id_scaled, v_ofs) 
        & coord_is_safe(u_size, u_id_scaled, u_ofs);

    // RETURN the sum of the following:
    // - the pre-defined axon offset (the idz) for the slice and
    // - the linear offset within that slice including, for each of the two axial 
    //   (hex tile grid) dimensions:
    //    - the scaled position
    //    - the synapse offset
    //    - the previously added middle offset
    return get_axn_idz(slc_id) + (uint)(mad24(v_id_scaled + v_ofs, u_size, u_id_scaled + u_ofs));
}

// AXN_IDX_3D_UNSAFE_VEC4(): Linear index of an axon, vec4
static inline int4 axn_idx_3d_unsafe_vec4(uchar4 const slc_id, int4 const v_id_unscaled, 
            char4 const v_ofs_char4, int4 const u_id_unscaled, char4 const u_ofs_char4, 
            int4* idx_is_safe)
{
    int4 const v_size = get_axn_v_size_vec4(slc_id);
    int4 const u_size = get_axn_u_size_vec4(slc_id);

    int4 const v_ofs = convert_int4(v_ofs_char4) + get_axn_v_mid_vec4(slc_id);
    int4 const u_ofs = convert_int4(u_ofs_char4) + get_axn_u_mid_vec4(slc_id);

    int4 const v_id_scaled = (mul24(v_id_unscaled, get_axn_v_scale_vec4(slc_id)) >> 4);
    int4 const u_id_scaled = (mul24(u_id_unscaled, get_axn_u_scale_vec4(slc_id)) >> 4);

    // int4 const v_id_hrz = v_size >> 1;
    // int4 const u_id_hrz = u_size >> 1;

    // int4 const idx_is_hrz = convert_int4(slc_id) >= (int4)HORIZONTAL_AXON_ROW_DEMARCATION;

    // int4 const v_id = (idx_is_hrz & v_id_hrz) | (~idx_is_hrz & v_id_scaled);
    // int4 const u_id = (idx_is_hrz & u_id_hrz) | (~idx_is_hrz & u_id_scaled);

    *idx_is_safe = coord_is_safe_vec4(v_size, v_id_scaled, v_ofs) 
        & coord_is_safe_vec4(u_size, u_id_scaled, u_ofs);

    return get_axn_idz_vec4(slc_id) + mad24(v_id_scaled + v_ofs, u_size, u_id_scaled + u_ofs);
}


/*===========================================================================*/

// AXN_STATE_3D_SAFE():
static inline uchar axn_state_3d_safe(uchar const slc_id, uint const v_id, char const v_ofs, 
            uint const u_id, char const u_ofs, __global uchar const* const axn_states) 
{
    int idx_is_safe = 0;
    uint const axn_idx = axn_idx_3d_unsafe(slc_id, v_id, v_ofs, u_id, u_ofs, &idx_is_safe);
    return mul24(idx_is_safe, axn_states[axn_idx]);
}

// AXN_STATE_3D_SAFE_VEC4():
static inline uchar4 axn_state_3d_safe_vec4(uchar4 slc_id, int4 v_id, char4 v_ofs, 
            int4 u_id, char4 u_ofs, __global uchar const* const axn_states) 
{
    int4 idx_is_safe = (int4)0;
    int4 const axn_idx = axn_idx_3d_unsafe_vec4(slc_id, v_id, v_ofs, u_id, u_ofs, &idx_is_safe);

    return (uchar4)(
        ((uchar)idx_is_safe.s0 & axn_states[axn_idx.s0]),
        ((uchar)idx_is_safe.s1 & axn_states[axn_idx.s1]),
        ((uchar)idx_is_safe.s2 & axn_states[axn_idx.s2]),
        ((uchar)idx_is_safe.s3 & axn_states[axn_idx.s3]));

}


/*=============================================================================
================================== LEARNING ===================================
=============================================================================*/

// ISSUE: LOTS OF PROBLEMS WITH GLITCHES AND COMPILER BUGS ON THESE FUNCTIONS!
//         UPDATE: CATALYST 15.9 SEEMS TO HAVE FIXED SEVERAL (ALL?). NOT SURE ABOUT 

// DST_DEN_SYNS_LEARN_INIT(): 
//         - Occurs when a cell is active.
//         - Applies to a single dendrite on that cell.
//             - Must only be called with a syn_idz of the best (most active) dendrite on an active tuft on an active cell. 
//         - Is intended to handle both crystalization (predictions becoming or staying true) or anomalies (situations where no cell in the column had predicted the column's spatial input).
//
//         STDEP set when depression has already been applied (needs to be cleared by trmn)
//         STPOT set when potentiation is due to be applied (by trmn)
static inline void dst_syns__active__stpot_stdep( // RENAME TO ABOVE
            __global uchar const* const syn_states,
            uint const syn_idz,    
            uint const syns_per_den_l2,
            int const rnd,
            __global uchar* const syn_flag_sets,
            __global char* const syn_strengths) 
{
    uint const n = syn_idz + (1 << syns_per_den_l2);

    for (uint i = syn_idz; i < n; i++) {
        char syn_strength = syn_strengths[i];
        uchar syn_flag_set = syn_flag_sets[i];
        uchar const syn_state = syn_states[i];
        // int const inc = rnd_inc(rnd, syn_idz + i, syn_strength);
        int const syn_is_active = syn_state != 0;
        int const syn_has_stpot = (syn_flag_set & SYN_STPOT_FLAG) == (SYN_STPOT_FLAG);
        int const syn_has_stdep = (syn_flag_set & SYN_STDEP_FLAG) == (SYN_STDEP_FLAG);

        // Synapse has either a short term potentiation or short term depression flag:
        int const syn_has_stX_flag = (syn_has_stpot | syn_has_stdep);

        // Synapse is active and does not have a short term depression or potentiation flag:
        int const syn_needs_stpot = syn_is_active && (!syn_has_stX_flag);
        // Synapse is inactive and does not have a short term depression or potentiation flag:
        int const syn_needs_stdep = (!syn_is_active) && (!syn_has_stX_flag);

        // If syn_needs_stdep, depress the synapse's strength by 'inc' (generally a 1 or 0) ...
        // syn_strength -= mul24(syn_needs_stdep, inc);            

        // Deactivate synapse short term potentiation and depression flags regardless of their states:
        // syn_flag_set &= ~(SYN_STPOT_FLAG | SYN_STDEP_FLAG);


        // If syn_needs_stpot activate STPOT flag:
        syn_flag_set = syn_flag_set | mul24(syn_needs_stpot, (SYN_STPOT_FLAG)) 
            | mul24(syn_needs_stdep, (SYN_STDEP_FLAG));
        // If syn_needs_stdep activate STDEP flag:
        // syn_flag_set |= mul24(syn_needs_stdep, (SYN_STDEP_FLAG));

        syn_flag_sets[i] = syn_flag_set;
        // syn_flag_sets[i] = ;
        // syn_flag_sets[i] = 2 | 1;

        syn_strengths[i] = syn_strength;
    }
}



/*=============================================================================
===================================== WIP =====================================
=============================================================================*/

/* RND INC/DEC NOTES:
        - Must cap at the min and max limits (-127, 127).
        - Must not get stuck at max limit. If at max, must be decrementable. At min, who cares.
        - Must be easy to move near zero and more difficult the larger the pos or neg value.

        inc -> abs(val) < rnd 
        dec -> (abs(val) + is_max) < rnd
            - account for pos-neg val
            - handle deadlock

        - Learning rate (lr_l2i) is 100% at 0, 50% at 1, 25% at 2, etc.
*/

// RND_INC(): Returns a 1 or 0 representing whether or not to increment a value:
static inline int rnd_inc(int const rnd_a, int const seed, char const val, 
            int const lr_l2i, int const lr_mask) 
{
    // return (rnd_mix(rnd_a, seed) & 0x7F) > abs(val);    // FAST
    // return ((char)rnd_mix(rnd_a, seed)) > abs(val);     // SLOWER VARIANT
    return (rnd_mix(rnd_a, seed) & lr_mask) 
        > (abs(val) + (lr_mask - 0x7f));                // ADJUSTABLE VARIANT
}

// RND_DEC(): Returns a 1 or 0 representing whether or not to decrement a value:
static inline int rnd_dec(int const rnd_a, int const seed, char const val, 
            int const lr_l2i, int const lr_mask) 
{
    int const str_is_max = val == 127;

    // return ((rnd_mix(rnd_a, seed) & 0x7F)) > (abs(val) - str_is_max);       // FAST
    // return ((char)rnd_mix(rnd_a, seed)) > (char)(abs(val) - str_is_max);    // SLOWER VARIANT
    return ((rnd_mix(rnd_a, seed) & lr_mask)) 
        > ((abs(val) - str_is_max) + (lr_mask - 0x7f));                        // ADJUSTABLE VARIANT
}

static inline void lshft_mask(int* mask, int const shft_l2) {
    for (int i = 0; i < shft_l2; i++) { *mask |= (1 << i); }
}

/*=============================================================================
==================================== /WIP =====================================
=============================================================================*/


// DST_TFT_SYNS_LEARN_TRMN(): Learning termination for a tuft:
//         - Occurs when a cell which had been active becomes inactive.
// TODO: VECTORIZE
static inline void tft_syns_trm( 
            __global uchar const* const syn_states,
            uint const syn_idz,
            uint const syns_per_tft_l2,
            int const rnd,
            int const lr_l2i, // LEARNING RATE INVERSE LOG BASE 2 (1/L2)
            __global uchar* const syn_flag_sets,
            __global int* const aux_ints_0,
            __global char* const syn_strengths) 
{
    uint const n = syn_idz + (1 << syns_per_tft_l2);

    int lr_mask = 0x7F << lr_l2i;
    lshft_mask(&lr_mask, lr_l2i);

    // aux_ints_0[syn_idz] = lr_mask;
    // aux_ints_0[syn_idz] = rnd_mix(rnd, syn_idz) & lr_mask;

    for (uint i = syn_idz; i < n; i++) {
        uchar const syn_state = syn_states[i];
        char syn_strength = syn_strengths[i];
        uchar syn_flag_set = syn_flag_sets[i];
        int const should_inc = rnd_inc(rnd, (syn_idz + i), syn_strength, lr_l2i, lr_mask);
        int const should_dec = rnd_dec(rnd, (syn_idz + i), syn_strength, lr_l2i, lr_mask);
        int const syn_is_active = syn_state != 0;
        int const syn_has_stpot = (syn_flag_set & SYN_STPOT_FLAG) == SYN_STPOT_FLAG;
        int const syn_has_stdep = (syn_flag_set & SYN_STDEP_FLAG) == SYN_STDEP_FLAG;

        // If synapse had STPOT flag and is now inactive (synapse correlated with cell activity):
        syn_strength += mul24(syn_has_stpot && !syn_is_active, should_inc);

        // If synapse had STPOT flag and is still active (synapse did not correllate with cell activity):
        syn_strength -= mul24(syn_has_stpot && syn_is_active, should_dec);

        // If synapse had STDEP flag:
        syn_strength -= mul24(syn_has_stdep, should_dec);

        // Deactivate synapse short term potentiation and depression flags regardless of their states:
        syn_flag_set &= ~(SYN_STPOT_FLAG | SYN_STDEP_FLAG);

        syn_flag_sets[i] = syn_flag_set;
        syn_strengths[i] = syn_strength;
    }
}

// TODO: VECTORIZE 
static inline void prx_syns__active__ltp_ltd( 
            __global uchar const* const syn_states,
            uint const syn_idz,
            uint const syns_per_den_l2,
            int const rnd,
            __global char* const syn_strengths) 
{
    uint const n = syn_idz + (1 << syns_per_den_l2);

    // TODO: These should be calculated host side and passed in:
    int const lr_l2i = 0; 
    int lr_mask = 0x7F << lr_l2i;
    lshft_mask(&lr_mask, lr_l2i);

    for (uint i = syn_idz; i < n; i++) {
        uchar const syn_state = syn_states[i];
        char syn_strength = syn_strengths[i];
        // int const inc = rnd_inc(rnd, syn_idz + i, syn_strength);
        int const should_inc = rnd_inc(rnd, (syn_idz + i), syn_strength, lr_l2i, lr_mask);
        int const should_dec = rnd_dec(rnd, (syn_idz + i), syn_strength, lr_l2i, lr_mask);
        int const syn_is_active = syn_state != 0;

        syn_strength += mul24(syn_is_active, should_inc);
        syn_strength -= mul24(!syn_is_active, should_dec);

        syn_strengths[i] = syn_strength;
    }

}



// Just to squelch 'unused' warnings:
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
//     - Vectorize (pretty much everywhere)
//     - Fit data into workgroups better for several kernels
//         - Keep data loading contiguous for the workgroup
//     - Use Async copy
//         event_t async_work_group_copy(__local T *dst, const __global T *src, size_t num_elements, event_t event)
//         event_t async_work_group_copy(__global T *dst, const __local T *src, size_t num_elements, event_t event)
//         void wait_group_events (int num_events, event_t *event_list)
//     - Globalize wherever possible:
//         - slc_columns
//         - 
//
// CLEAN UP:
//     - TODO: Split this beast up.




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

    int const syn_idn = (1 << syns_per_den_l2);

    for (int syn_id = 0; syn_id < syn_idn; syn_id += 1) {
        char syn_strength = syn_strengths[syn_idz + syn_id];
        uchar syn_state = syn_states[syn_idz + syn_id]; 
        syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum); 
        syn_sum_raw += syn_state;
    }
    
    syn_sum = mul24((syn_sum > den_threshold), syn_sum);

    // if (syn_sum != 0) {
    //     if (den_energy >= ENERGY_LEVEL_MIN) {
    //         den_energy -= ENERGY_LEVEL_MIN;
    //     } else {
    //         den_energy += ENERGY_REGEN_AMOUNT;
    //         syn_sum = 0;
    //     }
    // } else {
    //     if (den_energy < ENERGY_LEVEL_MAX) {
    //         den_energy += ENERGY_REGEN_AMOUNT;
    //     }
    // }

    int den_reduction = syns_per_den_l2 - 1;

    den_states_raw[den_idx] = clamp((syn_sum_raw >> den_reduction), 0, 255); 
    den_states[den_idx] = clamp((syn_sum >> den_reduction), 0, 255); 
}


//     INHIB_SIMPLE(): [DESCRIPTION OUT OF DATE] Cell Inhibition - reads from soma, writes to axon
//        - If any nearby cells are more active (have a higher soma 'state')
//            - cell will not 'fire'
//            - otherwise, write soma (cel_states[cel_idx]) to axon (axn_states[axn_idx])
//
//        - Overly simplistic algorithm 
//             - Distance should be taken into account when state is considered
//            - Search area broadened
//         - Horribly unoptimized, Should:
//            - cache values for an area in local (workgroup) memory
//                - or just prefetch global cache? (comparison needed)
//            - be vectorized
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
    //     aux_ints_1[cel_idx] = get_axn_v_scale(cel_idx);
    // }

    //uint dumb_iter = 0;

    for (int v_ofs = radius_neg; v_ofs <= radius_pos; v_ofs++) {
        int v_neg = 0 - v_ofs;
        int u_z = max(radius_neg, v_neg - radius_pos);
        int u_m = min(radius_pos, v_neg + radius_pos);

        for (int u_ofs = u_z; u_ofs <= u_m; u_ofs++) {

            uchar neighbor_state 
                = cel_state_3d_safe(slc_id_lyr, v_size, v_id, v_ofs, u_size, u_id, u_ofs, cel_states);    // ORIGINAL        
            //uchar neighbor_state = cel_states[
            //cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id + v_ofs, u_size, u_id + u_ofs)]; // DEBUG


            int distance = (abs(v_ofs) + abs(u_ofs) + abs(w_ofs(v_ofs, u_ofs)))    >> 1;


            //int cel_influence = mul24((int)cel_state, (distance + 1) << 1); // CRAP
            //int neighbor_influence = mul24((int)neighbor_state, radius_pos - distance); // CRAP


            //     NEW ALGORITHM 16-JUL:
            //         - FOCAL CELL IS AT LEAST AS INFLUENTIAL AS NEIGHBOR AT THE FOCAL 
            //         CELL'S LOCATION (A.K.A. THE CELL CELL IS UNINHIBITED)
            //             - IF CEL_FOCAL_INFLUENCE__AT_CEL_FOCAL >= NEIGHBOR_INFLUENCE__AT_CEL_FOCAL
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
                //    v_size, v_id, v_ofs, u_size, u_id, u_ofs, cel_states);
                //aux_ints_1[unsafe_target_axn_idx] = 1;
                //axn_states[unsafe_target_axn_idx] = neighbor_state;
                axn_states[unsafe_target_axn_idx] = 1 + inhibited;
            }
            */
            
            //dumb_iter += 1;
            

            // int debug_idx_ofs = 257;     // SET TO WHATEVER
            // for (int i = 0; i < mul24(get_global_size(0), mul24(v_size, u_size)); i += 1024) {

            //     if (((int)cel_idx & 0xFFFFFFFF) == debug_idx_ofs) {
            //         aux_ints_1[mul24(i, 1024) + dumb_iter] 
            //                 //= cel_influence;
            //                 //= distance + 100;
            //                 //= cel_idx;
            //                 = neighbor_state - cel_state;

            //     }

            //     // if (cel_idx == 384) {
            //     //     //aux_ints_1[axn_idx_3d_safe(slc_id_lyr + cel_base_axn_slc, v_size, v_id, v_ofs, u_size, u_id, u_ofs)] = distance;
            //     //     aux_ints_1[520 + dumb_iter] 
            //     //         //= cel_influence;
            //     //         = distance + 100;
            //     // }
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
            __private uchar const syns_per_tft_l2,
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
        uint const syn_idz = calc_syn_idz(tuft_id, cel_count, cel_id, syns_per_tft_l2);
        prx_syns__active__ltp_ltd(syn_states, syn_idz, syns_per_tft_l2, rnd, syn_strengths);
    }
}


// SST_LTP(): Long term potentiation for Spiny Stellate Cells
// <<<<< TODO: ADD AN ADDITIONAL DIMENSION [0] FOR SLC_ID TO SUPPORT MULTIPLE SLICE SST LAYERS >>>>>
// <<<<< NOTE: THIS KERNEL MAY BE FLAWED WHEN USED WITH MULTIPLE TUFTS - SEE PYR_LTP >>>>>
__kernel void sst_ltp(
            __global uchar const* const axn_states,
            __global uchar const* const syn_states,
            __private uint const cel_lyr_axn_idz,
            __private uint const cels_per_grp,
            __private uchar const syns_per_tft_l2,
            __private uint const rnd,
            // __global int* const aux_ints_0,
            __global char* const syn_strengths) 
{
    uint const tuft_id = get_global_id(0);
    uint const cel_grp_id = get_global_id(1);
    
    uint const cel_count = get_global_size(1);

    uint const cel_idz = mul24(cel_grp_id, cels_per_grp);
    uint const cel_axn_idz = cel_lyr_axn_idz + cel_idz;    

    // TESTING
    // uint const cel_tuft_id = cel_id + mul24(tuft_id, cel_count);
    // aux_ints_0[cel_tuft_id] = axn_state;
    // END TESTING

    for (uint i = 0; i < cels_per_grp; i += 1) {
        uint const cel_idx = cel_idz + i;
        uint const cel_axn_idx = cel_axn_idz + i;
        uint const axn_state = axn_states[cel_axn_idx];

        if (axn_state) {            
            uint const syn_idz = calc_syn_idz(tuft_id, cel_count, cel_idx, syns_per_tft_l2);
            prx_syns__active__ltp_ltd(syn_states, syn_idz, syns_per_tft_l2, rnd, syn_strengths);
        }
    }
}



// MCOL_ACTIVATE_PYRS(): Activate the axon of the pyramidal cell with the most active dendrite (on any tuft).
//      - If every dendrite on every tuft of every pyramidal cell in the entire column is inactive (below threshold):
//            - Activate the axon of every pyramidal cell in the column.
//
//        In addition (for learning purposes):
//            - Keep track of whether or not predictions (pyramidal states) for any pyramidal cell in the column have come true (crystallized).
//             - Determine whether or not an unpredicted (anomalous) activity has occurred.
//
// TODO: TUFTIFY
// TODO: REMOVE BEST_DEN_IDS AND DEN_STATES AND REPLACE WITH BEST_DEN_STATES (KEEP INDEXING IN MIND)
__kernel void mcol_activate_pyrs(
            __global uchar const* const mcol_flag_sets, // COL
            __global uchar const* const mcol_best_den_states,
            // __global uchar const* const cel_tft_best_den_ids,
            __global uchar const* const pyr_best_den_states,
            // __global uchar const* const den_states,
            // __global uchar const* const cel_tft_best_den_ids, // ADD ME?
            __private uint const ssts_axn_idz,         // Primary spatial associative cell layer (ssts)
            __private uint const pyrs_axn_idz,          // Primary temporal associative cell layer (pyrs)
            // __private uchar const pyr_axn_slc_base,
            __private uchar const dens_per_tft_l2,
            __global uchar* const pyr_flag_sets,
            __global uchar* const pyr_states,
            __global int* const aux_ints_0,
            __global uchar* const axn_states) 
{
    uint const slc_id_lyr = get_global_id(0);
    uint const v_id = get_global_id(1);
    uint const u_id = get_global_id(2);
    uint const v_size = get_global_size(1);
    uint const u_size = get_global_size(2);

    // uint const cel_idx = get_global_id(0);

    uint const cel_idx = cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id);
    // int idx_is_safe = 0;
    // uint const pyr_axn_idx = axn_idx_3d_unsafe(pyr_axn_slc_base + slc_id_lyr, v_id, 0, u_id, 0, &idx_is_safe);
    uint const pyr_axn_idx = pyrs_axn_idz + cel_idx;
    uint const col_id = cel_idx_3d_unsafe(0, v_size, v_id, u_size, u_id);

    // ******************


    // uint const cel_tft_idx = mad24(cel_idx, tfts_per_cel, tft_id);
    // uint const den_ofs = cel_idx << dens_per_tft_l2;                    // OLD
    // uint const best_den_idx = den_ofs + pyr_best_den_ids[cel_idx];     // OLD
    uchar const best_den_state = pyr_best_den_states[cel_idx];

    uchar const mcol_best_col_den_state = mcol_best_den_states[col_id];
    uchar const psa_cel_axn_state = axn_states[ssts_axn_idz + col_id];
    //uchar const mcol_state = mcol_states[col_id];
    uchar const mcol_flag_set = mcol_flag_sets[col_id];
    uchar const pyr_state = pyr_states[cel_idx];
    uchar pyr_flag_set = pyr_flag_sets[cel_idx];

    int const mcol_is_active = psa_cel_axn_state != 0;
    //int const mcol_active = mcol_state != 0;
    int const mcol_any_pred = (mcol_flag_set & MCOL_IS_VATIC_FLAG) == MCOL_IS_VATIC_FLAG;
    int const pyr_is_vatic = (pyr_state != 0);

    // DEBUG
    // if (pyr_is_vatic) { 
    //     aux_ints_0[cel_idx] = pyr_axn_idx;
    // }

    int const crystalized = pyr_is_vatic && mcol_is_active;
    int const anomalous = mcol_is_active && !mcol_any_pred;

    //int const activate_axon = crystal || anomaly;
    //pyr_state = (crystal | anomaly) && (mcol_state);
    //pyr_state = mul24(((crystal != 0) || (anomaly != 0)), mcol_state);
    pyr_flag_set &= ~CEL_BEST_IN_COL_FLAG;
    
    //pyr_flag_set |= mul24(best_den_state == mcol_best_col_den_state, CEL_BEST_IN_COL_FLAG);
    //pyr_flag_set |= mul24((mcol_best_col_den_state == best_den_state) && pyr_is_vatic, 
    //    CEL_BEST_IN_COL_FLAG);
    pyr_flag_set |= mul24((best_den_state != 0) && (best_den_state == mcol_best_col_den_state), 
        CEL_BEST_IN_COL_FLAG);


    axn_states[pyr_axn_idx] = (uchar)mad24(anomalous, (int)psa_cel_axn_state, mul24(crystalized, (int)pyr_state));
    //axn_states[axn_idx] = (uchar)mad24(anomaly, (int)mcol_state, mul24(crystal, (int)pyr_state));

    pyr_flag_sets[cel_idx] = pyr_flag_set;

    //pyr_states[cel_idx] = pyr_state;

    //aux_ints_0[cel_idx] = 5;
    //aux_ints_0[cel_idx] = pyr_state;
}







// PYRS_LTP(): Pyramidal long term potentiation and depression - adjusting synapse strengths
/*

    First, to clarify:
        - The term 'vatic' is meant to mean predictive or in a state of expectance. May be referred to (incorrectly) as fuzzy or 'fuz'. A few other (depricated) terms might be thrown around due to much renaming but basically if a pyramidal soma is active, that cell is vatic.
        - the term 'concrete' is meant to mean that the cell has an active axon and therefore has not been inhibited by anything else (e.g. the most likely culprits, pyramidals within the same layer and column, our cells colleagues).

    The learning process is as follows:
        - For each pyramidal cell:
            - If the cell is concrete AND {the cell *was* previously vatic OR it *is* the best in the column}:
                - For each tuft on that cell:
                    - If tuft is active (i.e. has a best dendrite state != 0):
                        - Cause 'Learning Initiation' to take place on that most active dendrite (see below).
            - If the cell's axon is not concrete but was previously (flag_set & CEL_PREV_CONCRETE_FLAG):
                - 'Terminate' each tuft.                

    Learning Initiation:
        - 




    - TODO:
        - [incomplete] Vectorize (should be highly vectorizable)
        - reducing branching will be tough with this one
        - [in progress] Tests (check that flag_set and prev_best_den_id are robustly maintained)


        - if pyr_prev_concrete 
            - if pyr_concrete
            - if pyr_state

        - if pyr_prev_pred
            - if pyr_concrete
            - if pyr_state

    - Misc Notes:

        - SYN(    -> STPOT) WHEN: (SYN_STATE > 0) AND (CEL_TANGIBLE) AND (CEL_BEST_IN_COLUMN)
                            OR: (SYN_STATE > 0) AND (CEL_TANGIBLE) AND (CEL_PREV_PRED)

        - MAINTAIN STPOT STATE AS LONG AS: (SYN_STATE > 0) AND (CEL_ACTIVE)

        - SYN(STPOT -> LTP) ONLY WHEN: ((CEL_ACTIVE -> 0)) SAME TIME AS (SYN_STATE -> 0)


    INDEXING EXAMPLE: <<<<< TODO: UPDATE TO CORRECT DENDRITE INDEXING >>>>>

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

                    { THE FOLLOWING IS OUT OF DATE }

                - Synapse indexes will first need to be calculated using the dendrite index, i.e.:
                    syn_id_tuft := den_idx * syns_per_den (or equivalantly: den_idx << syns_per_den_l2).
                - Next they will be added to the tuft offset:
                    - syns_per_tft_space := syns_per_den * dens_per_tft * cels_per_col * col_count
                        - (note that tufts per cell is not factored in here)
                    - syn_idz_tuft := tuft_id * syns_per_tft_space
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
                    DEN[0]:[XXX] <<<<< UPDATE ME! >>>>>
                        SYN[0]:[393216], SYN[1]:[393217], ..., SYN[31]:[393247]
                    ...    
                    DEN[3]:[XXX] <<<<< UPDATE ME! >>>>>
                        SYN[0]:[393312], SYN[1]:[393313], ..., SYN[31]:[393343]
            CEL[1]:1 (COL[0])
                TFT[0]
                    DEN[0]:[8]
                        SYN[0]:[0], SYN[1]:[1], ..., SYN[31]:[31]
                    ...    
                    DEN[3][11]
                        SYN[0]:[96], SYN[1]:[97], ..., SYN[31]:[127]
                TFT[1]
                    DEN[0][XXX] <<<<< UPDATE ME! >>>>>
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

            Kernel: WorkSize: OneDim(cell groups **) - Iterate through cell groups
                - Loop: cells - Iterate through cells within each group.
                    - Loop: tufts - Iterate through cell tufts.
                        - Function call: dendrites - For each tuft, if work needs to be done, pick the most active or previously active dendrite(s) then call a function is called which will ->
                          Loop: synapses - Iterate through dendrite synapses.
                            - STPOT, LTP, and LTD take place on synapses within this the smallest loop.



        ** Note: Cell groups are divisions of the cell space for the layer into groups of arbitrary size. Cell groups are used in lieu of individual cells as the primary work dimension because during any given cycle. Most cells will need no work done on its synapses, therefore most work items would be idle. By bundling a group of cells into each work item, all threads can keep busy.

*/
// <<<<< TODO: FIX: NOT TAKING IN TO ACCOUNT MULTIPLE TUFTS! MAJOR INDEXING PROBLEMS >>>>>
__kernel void pyrs_ltp(
            __global uchar const* const axn_states,
            __global uchar const* const cel_states,
            __global uchar const* const cel_tft_best_den_ids,
            __global uchar const* const cel_tft_best_den_states,
            __global uchar const* const den_states,
            __global uchar const* const syn_states,
            __private uint const tfts_per_cel,
            __private uint const dens_per_tft_l2,
            __private uint const syns_per_den_l2,
            __private uint const cels_per_cel_grp,
            __private uint const axn_idz_cel_lyr,
            __private int const learning_rate_l2i,
            __private int const rnd,
            __global uchar* const syn_flag_sets,
            __global uchar* const cel_flag_sets,
            __global int* const aux_ints_0,
            __global int* const aux_ints_1,
            __global char* const syn_strengths) 
{
    uint const cel_grp_id = get_global_id(0);
    uint const cel_grp_count = get_global_size(0);
    uint const cel_count = mul24(cel_grp_count, cels_per_cel_grp);
    uint const cel_idz_cel_grp = mul24(cel_grp_id, cels_per_cel_grp);

    // TODO: MOVE THIS INVERSE LEARNING RATE TO HOST:
    // int const learning_rate_l2i = 0;

    // aux_ints_1[cel_grp_id] = -1 - (rnd_mix(rnd, cel_grp_id) & 0x7F);
    // aux_ints_1[cel_grp_id] = rnd_mix(rnd, cel_grp_id);
 
     // TODO: (EVALUATE) Make 'cels_per_cel_grp' and 'tfts_per_cel' a constant and unroll loops.
     //    - Will mean making a separate program for each layer of pyramidals.
     //    - Could do more harm than good due to program size bloat.
     //    - Possibly do this for tfts only.
    for (uint cel_id_cel_grp = 0; cel_id_cel_grp < cels_per_cel_grp; cel_id_cel_grp++) {
        uint const cel_idx = cel_idz_cel_grp + cel_id_cel_grp;
        uint const cel_axn_idx = axn_idz_cel_lyr + cel_idx;

        uchar cel_flag_set = cel_flag_sets[cel_idx];

        int const cel_is_concrete = axn_states[cel_axn_idx] != 0;
        int const cel_is_vatic = cel_states[cel_idx] != 0;
        int const cel_prev_concrete = (cel_flag_set & (CEL_PREV_CONCRETE_FLAG)) == (CEL_PREV_CONCRETE_FLAG);
        int const cel_prev_vatic = (cel_flag_set & (CEL_PREV_VATIC_FLAG)) == (CEL_PREV_VATIC_FLAG);
        int const cel_best_in_col = (cel_flag_set & (CEL_BEST_IN_COL_FLAG)) == (CEL_BEST_IN_COL_FLAG);

        for (uint tft_id = 0; tft_id < tfts_per_cel; tft_id++) {
            // uint const cel_tft_idx = mad24(cel_idx, tfts_per_cel, tft_id);
            uint const cel_tft_idx = calc_cel_tft_idx(cel_count, cel_idx, tfts_per_cel, tft_id);
            uint const den_idz_tft = cel_tft_idx << dens_per_tft_l2;

            uchar const den_id_tft_best = cel_tft_best_den_ids[cel_tft_idx];            

            uint const syn_idz_tft = den_idz_tft << syns_per_den_l2;
            uint const syn_idz_best_den_tft = (den_idz_tft + den_id_tft_best) << syns_per_den_l2;

            int const tuft_is_active = cel_tft_best_den_states[cel_tft_idx] != 0;

            if (cel_is_concrete) {                

                if (tuft_is_active) {
                    // aux_ints_0[cel_tft_idx] = cel_prev_vatic;
                    // aux_ints_1[cel_tft_idx] = cel_best_in_col;


                    // PREVIOUS (CORRECT) PREDICTION (EVERY PYR IN COL): REINFORCE DEN
                    // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
                    if (cel_prev_vatic | cel_best_in_col) { 
                        // aux_ints_1[cel_tft_idx] = 10;                    
                        dst_syns__active__stpot_stdep(syn_states, syn_idz_best_den_tft, syns_per_den_l2, rnd, 
                            syn_flag_sets, syn_strengths);

                        // aux_ints_1[cel_tft_idx] = 11;
                    }

                    // } else if (cel_best_in_col) { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN 
                    // //} else { // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
                    //     dst_syns__active__stpot_ltd(syn_states, syn_idz_best_den_tft, syns_per_den_l2, rnd, 
                    //         syn_flag_sets, syn_strengths);

                    //     // aux_ints_1[cel_tft_idx] = 12;
                    // }
                }

                // TODO: Could be moved into above if block
                cel_flag_set |= CEL_PREV_CONCRETE_FLAG;

            } else if (cel_prev_concrete) {
                tft_syns_trm(syn_states, syn_idz_tft, syns_per_den_l2 + dens_per_tft_l2, rnd, 
                    learning_rate_l2i, syn_flag_sets, aux_ints_0, syn_strengths);

                // aux_ints_1[cel_tft_idx] = 20;

                cel_flag_set &= ~CEL_PREV_CONCRETE_FLAG;
            }
        }

        cel_flag_set &= ~CEL_PREV_VATIC_FLAG;
        cel_flag_set |= mul24(cel_is_vatic, CEL_PREV_VATIC_FLAG);

        cel_flag_sets[cel_idx] = cel_flag_set;
    }
}



// TODO: Investigate the bizarre glitch with this kernel with the phantom best_dens
__kernel void pyr_cycle(
            __global uchar const* const den_states_raw, // USE THIS TO DETERMINE BEST DEN
            __global uchar const* const den_states,                
            __private uint const tfts_per_cel,
            __private uchar const dens_per_tft_l2,                
            __global uchar* const cel_tft_best_den_ids,
            __global uchar* const cel_tft_best_den_states,
            __global uchar* const pyr_best_den_states,
            __global int* const aux_ints_0,
            __global int* const aux_ints_1,
            __global uchar* const pyr_states) 
{
    uint const cel_idx = get_global_id(0);
    uint const cel_count = get_global_size(0);

    uint pyr_state = 0;
    uint pyr_best_den_state = 0;

    for (uint tft_id = 0; tft_id < tfts_per_cel; tft_id++) {
        uint const cel_tft_idx = calc_cel_tft_idx(cel_count, cel_idx, tfts_per_cel, tft_id);
        uint const den_idz_tft = cel_tft_idx << dens_per_tft_l2;
        // uint const cel_tft_idx = mad24(cel_idx, tfts_per_cel, tft_id);

        // Used for activation and ultimately learning:
        uint best_den_id = 0;
        // Used for activation and ultimately learning:
        uint best_den_state_raw = 0;
        // Used for pyr_state and pyr_best_den_state:
        uint best_den_state = 0;
 
        for (uint den_id_tft = 0; den_id_tft < (1 << dens_per_tft_l2); den_id_tft++) {
            uint const den_idx = den_idz_tft + den_id_tft;

            uint const den_state_raw = den_states_raw[den_idx];
            uint const den_state = den_states[den_idx];

            int const den_state_bigger_raw = (den_state_raw > best_den_state_raw);
            int const den_state_bigger = (den_state > best_den_state);

            best_den_id = mad24((uint)den_state_bigger_raw, den_id_tft,
                mul24((uint)!den_state_bigger_raw, best_den_id));

            best_den_state_raw = mad24((uint)den_state_bigger_raw, den_state_raw, 
                mul24((uint)!den_state_bigger_raw, best_den_state_raw));            

            best_den_state = mad24((uint)den_state_bigger, den_state, 
                mul24((uint)!den_state_bigger, best_den_state));

            // if (den_state > 0) {
            //     aux_ints_0[den_idx] = cel_idx;
            // }
        }

        cel_tft_best_den_ids[cel_tft_idx] = best_den_id;
        cel_tft_best_den_states[cel_tft_idx] = best_den_state_raw;

        pyr_best_den_state = max(pyr_best_den_state, best_den_state_raw);

        // TODO: Might need a more sophisticated algorithm with a non-linear rate to determine pyr_state:
        pyr_state = max(pyr_state, best_den_state);
        // pyr_state += best_den_state;

        // if (best_den_state > 0) {
        //     aux_ints_0[cel_tft_idx] = pyr_state;
        // }
    }

    pyr_best_den_states[cel_idx] = pyr_best_den_state;
    
    // TODO: pyr_state: (see above)
    pyr_states[cel_idx] = clamp(pyr_state, (uint)0, (uint)255);

    // WTF?
    // uchar pyr_state_bizarre = pyr_states[cel_idx];

    // // aux_ints_1[cel_idx] = pyr_states[cel_idx];

    // if (pyr_state != 0) {
    //     aux_ints_0[cel_idx] = pyr_states[cel_idx];
    //     // aux_ints_1[cel_idx] = pyr_states[cel_idx];
    // }

    // if (pyr_state_crazy != 0) {
    //     aux_ints_1[cel_idx] = pyr_states[cel_idx];
    // }

}


//    COL_OUTPUT()
//        - rename coming
//
__kernel void mcol_output(
            __global uchar const* const pyr_states,                
            // __global uchar const* const cel_tft_best_den_states,
            __private uint const tfts_per_cel,
            __private uint const sst_axn_idz,
            __private uchar const pyr_depth,
            __private uchar const aff_out_axn_slc,
            __global uchar* const pyr_best_den_states,
            __global uchar* const mcol_flag_sets,
            __global uchar* const mcol_best_den_states,
            // __global int* const aux_ints_0,
            __global uchar* const axn_states)
{
    // uint const slc_id_lyr = get_global_id(0); // FIXED TO JUST ONE LAYER RIGHT NOW
    uint const v_id = get_global_id(0);
    uint const u_id = get_global_id(1);
    uint const v_size = get_global_size(0);
    uint const u_size = get_global_size(1);

    uint const cel_count = mul24(pyr_depth, mul24(v_size, u_size));

    int idx_is_safe = 0;
    uint const aff_out_axn_idx = axn_idx_3d_unsafe(aff_out_axn_slc, v_id, 0, u_id, 0, &idx_is_safe);
    uint const col_id = cel_idx_3d_unsafe(0, v_size, v_id, u_size, u_id);
    //uint const pyr_axn_idx = axn_idx_2d( + slc_id_lyr, slc_columns, col_id, 0);
    //uint const col_id = mad24(slc_id_lyr, slc_columns, col_id);

    // Primary spatial associative cell axon index (column spatial input, i.e. layer 4 spiny stellates)
    uint const psa_cel_axn_idx = sst_axn_idz + col_id;

    int const psa_cel_axn_state = axn_states[psa_cel_axn_idx];
    uchar mcol_den_state_max = 0;
    int mcol_pyr_state_max = 0;

    // Amalgamate the best dendrite out of every cell and cell tuft in the column:
    for (uint i = 0; i < pyr_depth; i++) {
        uint const cel_idx = cel_idx_3d_unsafe(i, v_size, v_id, u_size, u_id);

        uchar pyr_state = pyr_states[cel_idx];
        uchar pyr_best_den_state = pyr_best_den_states[cel_idx];

        // for (uint tft_id = 0; tft_id < tfts_per_cel; tft_id++) {
        //     // uint const cel_tft_idx = mad24(cel_idx, tfts_per_cel, tft_id);
        //     uint const cel_tft_idx = calc_cel_tft_idx(cel_count, cel_idx, tfts_per_cel, tft_id);

        //     pyr_best_den_state = max(pyr_best_den_state, 
        //         cel_tft_best_den_states[cel_tft_idx]);
        // }

        // pyr_best_den_states[cel_idx] = pyr_best_den_state;

        mcol_den_state_max = max(mcol_den_state_max, pyr_best_den_state);        
        mcol_pyr_state_max = max(mcol_pyr_state_max, (int)pyr_state);
    }

    //##### NOTE: Currently overwriting all flags:
    mcol_flag_sets[col_id] = mul24((mcol_pyr_state_max != 0), MCOL_IS_VATIC_FLAG);
    mcol_best_den_states[col_id] = mcol_den_state_max;
    //axn_states[aff_out_axn_idx] = mul24(idx_is_safe, clamp(mcol_pyr_state_max + psa_cel_axn_state, 0, 255)); // N1
    axn_states[aff_out_axn_idx] = clamp(mcol_pyr_state_max + psa_cel_axn_state, 0, 255);
    // axn_states[aff_out_axn_idx] = clamp(mcol_den_state_max, 0, 255);
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




// // AXN_IDX_3D_UNSAFE(): Linear index of an axon
// //         - Using ints as intermediate variables to be consistent with vectorized version 
// //             (will only affect invalid indexes)
// static inline uint axn_idx_3d_unsafe(uchar const slc_id, uint const v_id_unscaled, 
//             char const v_ofs, uint const u_id_unscaled, char const u_ofs, int* const idx_is_safe) 
//     {
//     // GET THE DIM SIZES:
//     int const v_size = get_axn_v_size(slc_id);
//     int const u_size = get_axn_u_size(slc_id);

//     //     CALCULATE SCALED INDEX:
//     //         - Multiply by the pre-defined scale for specified slice then divide by 16.
//     //         - A scale of 16 = 100%, 8 = 50%, 32 = 200%, etc.
//     int const v_id_scaled = (mul24(v_id_unscaled, get_axn_v_scale(slc_id)) >> 4);
//     int const u_id_scaled = (mul24(u_id_unscaled, get_axn_u_scale(slc_id)) >> 4);

//     // CALCULATE HORIZONTAL INDEX:
//     int const v_id_hrz = v_size >> 1;
//     int const u_id_hrz = u_size >> 1;    

//     // DETERMINE IF THIS IS A HORIZONTAL SLICE:
//     int const idx_is_hrz = slc_id >= HORIZONTAL_AXON_ROW_DEMARCATION;

//     // IF SLICE IS HORIZONTAL ASSIGN CORRESPONDING ID AND VICE VERSA:
//     int const v_id = mad24(idx_is_hrz, v_id_hrz, mul24(!idx_is_hrz, v_id_scaled));
//     int const u_id = mad24(idx_is_hrz, u_id_hrz, mul24(!idx_is_hrz, u_id_scaled));
        
//     // CHECK SAFETY:
//     *idx_is_safe = coord_is_safe(v_size, v_id, v_ofs) & coord_is_safe(u_size, u_id, u_ofs);

//     // RETURN the sum of the pre-defined axon offset for the slice and the linear offset within that slice:
//     return get_axn_idz(slc_id) + (uint)(mad24(v_id + v_ofs, u_size, u_id + u_ofs));
// }

// // AXN_IDX_3D_UNSAFE_VEC4(): Linear index of an axon, vec4
// static inline int4 axn_idx_3d_unsafe_vec4(uchar4 const slc_id, int4 const v_id_unscaled, 
//         char4 const v_ofs_char4, int4 const u_id_unscaled, char4 const u_ofs_char4, int4* const idx_is_safe)
// {
//     int4 const v_ofs = convert_int4(v_ofs_char4);
//     int4 const u_ofs = convert_int4(u_ofs_char4);

//     int4 const v_size = get_axn_v_size_vec4(slc_id);
//     int4 const u_size = get_axn_u_size_vec4(slc_id);

//     int4 const v_id_scaled = (mul24(v_id_unscaled, get_axn_v_scale_vec4(slc_id)) >> 4);
//     int4 const u_id_scaled = (mul24(u_id_unscaled, get_axn_u_scale_vec4(slc_id)) >> 4);

//     int4 const v_id_hrz = v_size >> 1;
//     int4 const u_id_hrz = u_size >> 1;

//     int4 const idx_is_hrz = convert_int4(slc_id) >= (int4)HORIZONTAL_AXON_ROW_DEMARCATION;

//     int4 const v_id = (idx_is_hrz & v_id_hrz) | (~idx_is_hrz & v_id_scaled);
//     int4 const u_id = (idx_is_hrz & u_id_hrz) | (~idx_is_hrz & u_id_scaled);

//     *idx_is_safe = coord_is_safe_vec4(v_size, v_id, v_ofs) & coord_is_safe_vec4(u_size, u_id, u_ofs);

//     return get_axn_idz_vec4(slc_id) + mad24(v_id + v_ofs, u_size, u_id + u_ofs);
// }

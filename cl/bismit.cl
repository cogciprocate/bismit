
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
#define SYNAPSE_AXON_BIAS_LOG2            1


#define RETNAL_THRESHOLD       48

// Passed to `rnd_0xFFFF()`. 65536 (max) ~> 1:1
#define DENDRITE_ACTIVITY_DECAY_FACTOR      1536

 /* bismit.cl: CONVENTIONS

        idx: current index (of a loop, workgroup, queue, etc.)
        - almost always a physical in-memory address

        idz := index[0], first element, starting element

        idz_parent: first element within the subset of a parent group
        - ex.: syn_idx := syn_idz_den + syn_id_den

        idn := index[len]: element after final element, termination point
        - ex.: for(int i = 0; i < idn; i++)

        idm := index[max] := index[len - 1]: final (valid) element just before idn. idn - 1 = idm.
        - ex.: for(int i = 0; i <= idm; i++)

        id: identifier, not necessarily a physical array index

        id_{parent}: identifier within the subset of a parent group
        - ex.: syn_idx := syn_idz_den + syn_id_den

        {var_name}_l2 : A scalar representing the log base 2 representation of a value (log2 val).
        - 'lg' is too vague and 'lb' is seldom used therefore l2 is the convention used for logs.

        {var_name}_l2i : A scalar representing the inverse log base 2 of a value (1 / log2 val).

        Coordinates:
        - slc_id : The id of a slice in axon space (or subset thereof such as a
          layer). This is the 'depth' coordinate corresponding to how far from
          the top of layer 0/1 we would be in a neocortex.
        - v_id : The 'v' coordinate of a tile (or in this case an axon, cell,
          etc.) within a slice in hexagonal tile space.
        - u_id : The 'u' coordinate of a tile in hexagonal tile space.
        - w_id : The 'w' coordinate of a tile in hexagonal tile space.

        - Coordinates are oriented (on the unit circle) with 'u' at 30deg, 'v'
          (technically v inverse or 'vi') at 150deg, and 'w' at 270deg. Any
          references to 'v' are considered to be inverted (negative) when
          plotting coordinates in real space. In other words a 'v' value of 5
          would equal -5 when plotting or mapping to real 2d space. This is
          simply a convenience ( / necessity?) for indexing in OpenCL.

        - 'w' is seldom used because coordinates are stored in 'axial
          coordinates' which just means that only two of the three coordinates
          are actually stored / used because the third can be reconstructed
          from the other two when needed.


        vat [tentative]: vatic, fuzziness, level of predictiveness

        ***** High Priority Comment, Temporary Code Change
        <<<<< Medium Priority Comment, To Do
        ##### Debug / Informational Message


        Kernel variable order guideline:
        - __global const* pointers (read-only arrays) first,
        - __local, __private scalars, etc. in the middle,
        - __global non-const pointers (output arrays) last,


        ASSUMPTIONS BEING MADE: (TODO: add assert!s in host)
        - syns_per_tft > 4
        - u_size and v_size (global) are multiples of 8 [outdated?]

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


static inline uint get_axn_v_size(uchar const slc_id) {
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

// Performs the equivalent of a ternary operation. 24-bit values max.
static int tern24(int condition, int val_if_true, int val_if_false) {
    return mul24(condition, val_if_true) | mul24(!condition, val_if_false);
}

//     W_COORD():
static inline int w_ofs(int const v_ofs, int const u_ofs) {
    return (0 - v_ofs) - u_ofs;
}


static inline int square(int const x) {
    return mul24(x, x);
}


// [NOTE]: Pre-variable-tuft size (2016-Dec-31).
static inline uint calc_syn_idz_OLD(uint const tuft_id, uint const cel_count, uint const cel_id,
            uint const syns_per_tft)
{
    uint const syn_tuft_ofs = mul24(tuft_id, cel_count) * syns_per_tft;
    return syn_tuft_ofs + (cel_id * syns_per_tft);
}

// static inline uint calc_cel_tft_idx(uint const cels_per_lyr, uint const cel_idx,
//             uint const tfts_per_cel, uint const tft_id)
// {
//     return mad24(cel_idx, tfts_per_cel, tft_id);
// }

// // GET_CEL_TFT_IDX(): Uses same indexing principle as dendrites and synapses (see synapses.rs)
// static inline uint calc_cel_tft_idx(uint const cel_count, uint const cel_idx,
//             uint const tfts_per_cel, uint const tft_id)
// {
//     return  mad24(tft_id, cel_count, cel_idx);
// }

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
static inline uint cel_idx_3d_unsafe(uint const slc_id_lyr, uint const v_size, int const v_id,
            uint const u_size, int const u_id)
{
    return (uint)mad24((int)slc_id_lyr, mul24((int)v_size, (int)u_size), mad24(v_id, (int)u_size, u_id));
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
// +8
static inline uchar cel_state_3d_safe(uchar const slc_id_lyr,
            uint const v_size, int const v_id, char const v_ofs,
            uint const u_size, int const u_id, char const u_ofs,
            __global const uchar* const cel_states)
{
    int v_ofs_is_safe = coord_is_safe(v_size, v_id, v_ofs);
    int u_ofs_is_safe = coord_is_safe(u_size, u_id, u_ofs);
    int cel_idx_is_safe = v_ofs_is_safe & u_ofs_is_safe;

    uint cel_idx = cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id + v_ofs, u_size, u_id + u_ofs);

    return mul24(cel_idx_is_safe, cel_states[mul24((uint)cel_idx_is_safe, cel_idx)]);
}

// CEL_IDX_3D_CHECKED(): LINEAR INDEX OF A CELL - NOT ACCURATE FOR AXONS
static inline uint cel_idx_3d_checked(uint const slc_id_lyr, uint const v_size, int const v_id,
            uint const u_size, int const u_id, int* idx_is_safe)
{
    int v_ofs_is_safe = coord_is_safe(v_size, v_id, 0);
    int u_ofs_is_safe = coord_is_safe(u_size, u_id, 0);
    *idx_is_safe = v_ofs_is_safe & u_ofs_is_safe;
    return (uint)mad24((int)slc_id_lyr, mul24((int)v_size, (int)u_size), mad24(v_id, (int)u_size, u_id));
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
    int const v_id_scaled = (mul24(v_id_unscaled, get_axn_v_scale(slc_id)) >> SLC_SCL_COEFF_L2);
    int const u_id_scaled = (mul24(u_id_unscaled, get_axn_u_scale(slc_id)) >> SLC_SCL_COEFF_L2);

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

    int4 const v_id_scaled = (mul24(v_id_unscaled, get_axn_v_scale_vec4(slc_id)) >> SLC_SCL_COEFF_L2);
    int4 const u_id_scaled = (mul24(u_id_unscaled, get_axn_u_scale_vec4(slc_id)) >> SLC_SCL_COEFF_L2);

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
            uint const u_id, char const u_ofs, __global const uchar* const axn_states)
{
    int idx_is_safe = 0;
    uint const axn_idx = axn_idx_3d_unsafe(slc_id, v_id, v_ofs, u_id, u_ofs, &idx_is_safe);
    return mul24(idx_is_safe, axn_states[mul24((uint)idx_is_safe, axn_idx)]);
}

// AXN_STATE_3D_SAFE_VEC4():
static inline uchar4 axn_state_3d_safe_vec4(uchar4 slc_id, int4 v_id, char4 v_ofs,
            int4 u_id, char4 u_ofs, __global const uchar* const axn_states)
{
    int4 idx_is_safe = (int4)0;
    int4 const axn_idx = axn_idx_3d_unsafe_vec4(slc_id, v_id, v_ofs, u_id, u_ofs, &idx_is_safe);

    return (uchar4)(
        ((uchar)idx_is_safe.s0 & axn_states[idx_is_safe.s0 & axn_idx.s0]),
        ((uchar)idx_is_safe.s1 & axn_states[idx_is_safe.s1 & axn_idx.s1]),
        ((uchar)idx_is_safe.s2 & axn_states[idx_is_safe.s2 & axn_idx.s2]),
        ((uchar)idx_is_safe.s3 & axn_states[idx_is_safe.s3 & axn_idx.s3]));

}


/*=============================================================================
===================================== RND =====================================
=============================================================================*/

// Cheap xorshift random number.
static inline int rnd_mix(int const rnd_a, int seed) {
    seed ^= (seed ^ rnd_a) << 13;
    seed ^= seed >> 17;
    seed ^= seed << 5;
    return seed;
}

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
    int const val_is_max = val == 127;

    // return ((rnd_mix(rnd_a, seed) & 0x7F)) > (abs(val) - str_is_max);       // FAST
    // return ((char)rnd_mix(rnd_a, seed)) > (char)(abs(val) - str_is_max);    // SLOWER VARIANT

    // Decrements by one if the value is maxed (to give it a chance) then
    // returns true if the random number is larger than the value.
    return ((rnd_mix(rnd_a, seed) & lr_mask))
        > ((abs(val) - val_is_max) + (lr_mask - 0x7f));                        // ADJUSTABLE VARIANT
}

// Creates a union of a provided mask and a new mask which marks every bit
// below `shft_l2` active. This is slow.
static inline void lshft_mask(int* mask, int const shft_l2) {
    for (int i = 0; i < shft_l2; i++) { *mask |= (1 << i); }
}


// Returns true if the value should be incremented (unsigned version).
//
// As values approach the max value they will be increasingly less likely to
// be incrementable.
static inline int rnd_inc_u(int const rnd_a, int const seed, uchar const val) {
    return (rnd_mix(rnd_a, seed) & 0xFF) > val;
}

// Returns true if the value should be decremented (unsigned version).
//
// As values approach the max value they will be increasingly less likely to
// be decrementable.
static inline int rnd_dec_u(int const rnd_a, int const seed, uchar const val) {
    int const val_is_max = val == 255;

    // Decrements by one if the value is maxed (to give it a chance) then
    // returns true if the random number is larger than the value.
    return (rnd_mix(rnd_a, seed) & 0xFF) > val - val_is_max;

    // Decay by exponential amount
}


// Returns true (1) [approx.] every `cutoff / 256` calls.
//
// `cutoff` of:
// 8 ~> 1/32
// 64 ~> 1/4
// 128 ~> 1/2
// etc.
//
// TODO: Add unit test.
//
static int rnd_0xFF(int rnd, int seed, uchar cutoff) {
    return (rnd_mix(rnd, seed) & 0xFF) < cutoff;
}

// Returns true (1) [approx.] every `cutoff / 65536` calls
//
// `cutoff` of:
// 256 ~> 1/256
// 1024 ~> 1/64
// 32768 ~> 1/2
// etc.
///
// TODO: Add unit test.
//
static int rnd_0xFFFF(int rnd, int seed, ushort cutoff) {
    return (rnd_mix(rnd, seed) & 0xFFFF) < cutoff;
}


// Updates activity rating.
static uchar update_activity_rating(uchar activity_rating, int is_active, int rnd,
        int rnd_seed, ushort decay_factor)
{
    // Increment activity rating if active:
    activity_rating += rnd_inc_u(rnd, rnd_seed, activity_rating) & is_active;
    // Decrement activities count at random (may need tuning):
    activity_rating -= rnd_0xFFFF(rnd, rnd_seed << 1, decay_factor) &
       (activity_rating > 0);
   return activity_rating;
}


/*=============================================================================
================================== LEARNING ===================================
=============================================================================*/


// Distal synapse medium-term potentiation/depression.
//
// - Occurs when a cell first becomes active.
// - Applies to a single dendrite on that cell's tuft (the most active).
// -
//
static inline void dst_syns__active__mtpot_mtdep(
            // Unused:
            __global const uchar* const syn_states,
            uint const syn_idz,
            uint const syns_per_den,
            // Potentiation rate inverse log2 (1/log2):
            int const pr_l2i,
            // Depression rate inverse log2 (1/log2):
            int const dr_l2i,
            int const rnd,
            __global uchar* const syn_flag_sets,
            // TODO: Switch to `u8` (`uchar`):
            __global char* const syn_strengths)
{
    uint const n = syn_idz + syns_per_den;

    // TODO: Pre-calculate host side:
    // Potentiation rate:
    int pr_mask = 0x7F << pr_l2i;
    lshft_mask(&pr_mask, pr_l2i);

    // Depression rate:
    int dr_mask = 0x7F << dr_l2i;
    lshft_mask(&dr_mask, dr_l2i);

    for (uint i = syn_idz; i < n; i++) {
        char syn_strength = syn_strengths[i];
        uchar syn_flag_set = syn_flag_sets[i];
        uchar const syn_state = syn_states[i];
        int const syn_is_active = syn_state != 0;
        int const syn_prev_active = (syn_flag_set & (SYN_PREV_ACTIVE_FLAG)) == (SYN_PREV_ACTIVE_FLAG);
        // int const syn_is_active = syn_state != 0;

        // TODO: De-branch
        if (syn_prev_active) {
            int const should_inc = rnd_inc(rnd, (syn_idz + i), syn_strength, pr_l2i, pr_mask);
            syn_strength += should_inc;
        } else {
            int const should_dec = rnd_dec(rnd, (syn_idz + i), syn_strength, dr_l2i, dr_mask);
            syn_strength -= should_dec;
        }

        // syn_flag_sets[i] = syn_flag_set;
        syn_strengths[i] = syn_strength;
    }

}


// TODO: VECTORIZE
static inline void prx_syns__active__mtp_ltd(
            __global const uchar* const syn_states,
            uint const syn_idz,
            uint const syns_per_den,
            int const rnd,
            // TODO: Switch to `u8` (`uchar`):
            __global char* const syn_strengths)
{
    uint const n = syn_idz + syns_per_den;

    // TODO: Pre-calculate host side:
    // Potentiation rate:
    int const pr_l2i = 0;
    int pr_mask = 0x7F << pr_l2i;
    lshft_mask(&pr_mask, pr_l2i);

    // Depression rate:
    int const dr_l2i = 2;
    int dr_mask = 0x7F << dr_l2i;
    lshft_mask(&dr_mask, dr_l2i);

    for (uint i = syn_idz; i < n; i++) {
        uchar const syn_state = syn_states[i];
        char syn_strength = syn_strengths[i];
        // int const inc = rnd_inc(rnd, syn_idz + i, syn_strength);
        int const should_inc = rnd_inc(rnd, (syn_idz + i), syn_strength, pr_l2i, pr_mask);
        int const should_dec = rnd_dec(rnd, (syn_idz + i), syn_strength, dr_l2i, dr_mask);
        int const syn_is_active = syn_state != 0;

        syn_strength += mul24(syn_is_active, should_inc);
        syn_strength -= mul24(!syn_is_active, should_dec);

        syn_strengths[i] = syn_strength;
    }

}


/*=============================================================================
=================================== OTHER =====================================
=============================================================================*/



// Just to squelch 'unused' warnings:
__kernel void reference_all_the_things(__private int const for_sanitys_sake) {
    //get_axn_u_size_vec4((uchar4)0);
    cel_idx_3d_unsafe_vec4((uchar4)0, (int4)0, (int4)0, (int4)0, (int4)0);
    //axn_idx_hrz(0, 0, 0, 0, 0);
    //coord_is_safe_vec4((int4)0, (int4)0, (int4)0);
    //axn_idx_hrz_vec4((int4)0, (int4)0, (int4)0, (int4)0, (int4)0);
    rnd_dec_u(0, 0, 0);
    rnd_0xFF(0, 0, 0);
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


// Cycles dendrites.
//
// The 'raw' tract disregards whether or not the synapse states have strengths
// `>= 0` AND whether or not the synapse sum is greater than the threshold.
// The 'non-raw' tract totals synapses which have non-negative strengths and
// sum to supra-threshold totals.
//
__kernel void den_cycle_tft(
            __global const uchar* const syn_states,
            // TODO: Switch to `u8` (`uchar`):
            __global const char* const syn_strengths,
            __private uint const tft_den_idz,
            __private uint const tft_syn_idz,
            __private uint const syns_per_den,
            __private uint const den_threshold,
            __private int const rnd,
            __global uchar* const den_energies,
            __global uchar* const den_activities,
            __global uchar* const den_states_raw,
            __global int* const aux_ints_0,
            __global int* const aux_ints_1,
            __global uchar* const den_states)
{
    uint const den_id_lyrtft = get_global_id(0);
    uint const syn_idz_den = (den_id_lyrtft * syns_per_den) + tft_syn_idz;
    uint const syn_idn_den = syn_idz_den + syns_per_den;

    int syn_sum = 0;
    int syn_sum_raw = 0;

    for (uint syn_idx = syn_idz_den; syn_idx < syn_idn_den; syn_idx++) {
        char syn_strength = syn_strengths[syn_idx];
        uchar syn_state = syn_states[syn_idx];
        syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum);
        syn_sum_raw += syn_state;
    }

    int den_is_active = (syn_sum > den_threshold);
    syn_sum = mul24(den_is_active, syn_sum);

    uint const den_idx = den_id_lyrtft + tft_den_idz;

    // Update activity rating:
    uint const rnd_seed = (syn_sum_raw + den_threshold) | den_idx;
    den_activities[den_idx] = update_activity_rating(den_activities[den_idx], den_is_active,
        rnd, rnd_seed, DENDRITE_ACTIVITY_DECAY_FACTOR);

    // int den_reduction = clamp(syns_per_den_l2 - 1, 0, 255);

    // At `den_reduction = 4`, Each syn state will amount to between 8 and 16:
    int den_reduction = 4;
    den_states_raw[den_idx] = clamp((syn_sum_raw >> den_reduction), 0, 255);
    den_states[den_idx] = clamp((syn_sum >> den_reduction), 0, 255);
    // int den_state_raw = (int)((float)syn_sum_raw / (float)syns_per_den);
    // int den_state = (int)((float)syn_sum / (float)syns_per_den);
    // int den_state_raw = (int)(syn_sum_raw);
    // int den_state = (int)(syn_sum);
    // den_states_raw[den_idx] = clamp(den_state_raw, 0, 255);
    // den_states[den_idx] = clamp(den_state, 0, 255);
}


// Determine the best dendrite state for the two tracts.
//
// The 'raw' tract disregards whether or not the synapse states have strengths
// >= 0. The 'non-raw' tract is for synapses which have developed strength.
//
__kernel void tft_cycle(
            __global const uchar* const den_states_raw, // USE THIS TO DETERMINE BEST DEN
            __global const uchar* const den_states,
            // __private uint const tfts_per_cel,
            // __private uint const tft_id,
            ////// Could be made GWO (and could be calculated from tuft_id as well):
            __private uint const lyrtft_cel_idz,
            __private uint const lyrtft_den_idz,
            __private uint const dens_per_tft,
            __private uchar const max_active_dens_l2,
            __global uchar* const celtft_prev_best_den_ids,
            __global uchar* const celtft_prev_best_den_states_raw,
            __global uchar* const celtft_prev_best_den_states,
            __global uchar* const celtft_prev_states,
            __global uchar* const celtft_best_den_ids,
            __global uchar* const celtft_best_den_states_raw,
            __global uchar* const celtft_best_den_states,
            // __global uchar* const pyr_best_den_states,
            __global int* const aux_ints_0,
            __global int* const aux_ints_1,
            __global uchar* const celtft_states)
{
    uint const cel_id_lyrtft = get_global_id(0);
    uint const lyrtft_cel_count = get_global_size(0);

    uint const den_id_lyrtft = cel_id_lyrtft * dens_per_tft;
    uint const cel_den_idz  = lyrtft_den_idz + den_id_lyrtft;

    // Cumulative best dendrite id (within the cell-tuft). Used for activation
    // and ultimately learning:
    uint best_den_id = 0;
    // Cumulative best raw dendrite state (within the cell-tuft). Used for
    // activation and ultimately learning:
    uint best_den_state_raw = 0;
    // Cumulative best dendrite state (within the cell-tuft). Used for
    // pyr_state and pyr_best_den_state:
    uint best_den_state = 0;

    // Total of all (active) dendrite states:
    uint active_den_state_sum = 0;

    for (uint den_id_celtft = 0; den_id_celtft < dens_per_tft; den_id_celtft++) {
        uint const den_idx = cel_den_idz + den_id_celtft;

        uint const den_state_raw = den_states_raw[den_idx];
        uint const den_state = den_states[den_idx];

        int const den_state_bigger_raw = (den_state_raw > best_den_state_raw);
        int const den_state_bigger = (den_state > best_den_state);

        // If `den_state_raw` is bigger than `best_den_state_raw`, will equal
        // `den_id_celtft`. If not, will equal `best_den_id`.
        best_den_id = mad24((uint)den_state_bigger_raw, den_id_celtft,
            mul24((uint)!den_state_bigger_raw, best_den_id));

        // if `den_state_raw` is bigger than `best_den_state_raw`, will equal
        // `den_state_raw`. If not, will equal `best_den_state_raw`.
        best_den_state_raw = mad24((uint)den_state_bigger_raw, den_state_raw,
            mul24((uint)!den_state_bigger_raw, best_den_state_raw));

        // if `den_state` is bigger than `best_den_state`, will equal
        // `den_state`. If not, will equal `best_den_state`.
        best_den_state = mad24((uint)den_state_bigger, den_state,
            mul24((uint)!den_state_bigger, best_den_state));

        // Accumulate:
        active_den_state_sum += den_state;
    }

    // Scale `active_den_state_sum` based on a max of `max_active_dens_l2` within 0-255:
    uint active_den_state_max_l2 = max_active_dens_l2 + 8;
    int celtft_is_max_active = active_den_state_sum >= (1 << active_den_state_max_l2);
    uchar celtft_state = (uchar)mul24((uint)celtft_is_max_active, (uint)255) +
        (uchar)mul24((uint)(!celtft_is_max_active), active_den_state_sum >> max_active_dens_l2);

    uint const celtft_idx = lyrtft_cel_idz + cel_id_lyrtft;


    celtft_prev_best_den_ids[celtft_idx] = celtft_best_den_ids[celtft_idx];
    celtft_prev_best_den_states_raw[celtft_idx] = celtft_best_den_states_raw[celtft_idx];
    celtft_prev_best_den_states[celtft_idx] = celtft_best_den_states[celtft_idx];
    celtft_prev_states[celtft_idx] = celtft_states[celtft_idx];

    celtft_best_den_ids[celtft_idx] = best_den_id;
    celtft_best_den_states_raw[celtft_idx] = best_den_state_raw;
    celtft_best_den_states[celtft_idx] = best_den_state;
    celtft_states[celtft_idx] = celtft_state;
}


// Sets the cell state (currently synonymous with the dendrite state).
//
// Adds cell energy to the dendrite state(s). This is important to do before
// inhibition.
__kernel void ssc_cycle(
        __global uchar* const energies,
        __global uchar* const cel_states)
{
    uint const slc_id_lyr = get_global_id(0);
    uint const v_id = get_global_id(1);
    uint const u_id = get_global_id(2);
    uint const v_size = get_global_size(1);
    uint const u_size = get_global_size(2);
    uint const cel_idx = cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id);

    uint const energy = energies[cel_idx];
    uint const state = cel_states[cel_idx];
    int const is_active = (state != 0);

    // If the cell is relatively high in energy and is active, fire:
    int const high_energy_cutoff = 191;
    int const is_restless = (energy > high_energy_cutoff) & is_active;
    uint restless_contrib = mul24((uint)is_restless, (uint)255);

    // If the cell has gone unused (has constantly been the least active of
    // its groups), fire:
    int const is_dark = energy == 255;
    uint dark_contrib = mul24((uint)is_dark, (uint)255);

    // If cell has fired, reduce energy:
    energies[cel_idx] = tern24(is_dark | is_restless, energy - 64, energy);
    // energies[cel_idx] = tern24(is_dark, energy - 32, energy);

    // State:
    // uint const state_contrib = state >> 1; // max 127.
    uint const state_contrib = state;
    cel_states[cel_idx] = clamp(state_contrib + restless_contrib + dark_contrib, (uint)0, (uint)255);
    // cel_states[cel_idx] = clamp(state_contrib + dark_contrib, (uint)0, (uint)255);
    // cel_states[cel_idx] = state_contrib;
}


// SST_LTP_SIMPLE(): Long term potentiation for Spiny Stellate Cells - Completely unoptimized
__kernel void ssc_mtp_simple(
        __global const uchar* const axn_states,
        __global const uchar* const syn_states,
        __private uint const cel_axn_idz,
        //__private uint const tufts_per_cel,
        __private uint const syns_per_tft,
        __private uint const rnd,
        // __global int* const aux_ints_0,
        // TODO: Switch to `u8` (`uchar`):
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
        uint const syn_idz = calc_syn_idz_OLD(tuft_id, cel_count, cel_id, syns_per_tft);
        prx_syns__active__mtp_ltd(syn_states, syn_idz, syns_per_tft, rnd, syn_strengths);
    }
}


// SST_LTP(): Long term potentiation for Spiny Stellate Cells
// <<<<< TODO: ADD AN ADDITIONAL DIMENSION [0] FOR SLC_ID TO SUPPORT MULTIPLE SLICE SST LAYERS >>>>>
// <<<<< NOTE: THIS KERNEL MAY BE FLAWED WHEN USED WITH MULTIPLE TUFTS - SEE PYR_LTP >>>>>
__kernel void ssc_mtp(
            __global const uchar* const axn_states,
            __global const uchar* const syn_states,
            __private uint const cel_lyr_axn_idz,
            __private uint const cels_per_grp,
            __private uchar const syns_per_tft,
            __private uint const rnd,
            // __global int* const aux_ints_0,
            // TODO: Switch to `u8` (`uchar`):
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
            uint const syn_idz = calc_syn_idz_OLD(tuft_id, cel_count, cel_idx, syns_per_tft);
            prx_syns__active__mtp_ltd(syn_states, syn_idz, syns_per_tft, rnd, syn_strengths);
        }
    }
}



// Cycles each pyramidal cell.
//
__kernel void pyr_cycle(
            // __global const uchar* const celtft_best_den_ids,
            __global const uchar* const tft_prev_states,
            // __global const uchar* const celtft_best_den_states,
            __global const uchar* const tft_states,
            __private uchar const tft_count,
            __private uchar const enabled_tft_flags,
            __private uchar const bsl_prx_tft_id,
            __private uchar const bsl_dst_tft_id,
            __private uchar const apc_dst_tft_id,
            // __global uchar* const pyr_best_den_states_raw,
            __global int* const aux_ints_0,
            __global int* const aux_ints_1,
            __global uchar* const pyr_states)
{
    uint const cel_idx = get_global_id(0);
    uint const cel_count = get_global_size(0);

    // uchar pyr_best_den_state_raw = 0;
    uchar pyr_state = 0;

    // // TODO: Remove this loop:
    // for (uint tft_id = 0; tft_id < tft_count; tft_id++) {
    //     uint const celtft_idx = mad24(tft_id, cel_count, cel_idx);

    //     uchar pyr_best_den_state_raw = celtft_best_den_states_raw[celtft_idx];
    //     // uchar pyr_best_den_state = celtft_best_den_states[celtft_idx];
    //     pyr_best_den_state_raw = max(pyr_state, pyr_best_den_state_raw);
    //     // pyr_state = max(pyr_state, pyr_best_den_state);
    // }

    int bsl_prx_is_enabled = (enabled_tft_flags & DEN_BASAL_PROXIMAL_FLAG) != 0;
    int bsl_dst_is_enabled = (enabled_tft_flags & DEN_BASAL_DISTAL_FLAG) != 0;
    int apc_dst_is_enabled = (enabled_tft_flags & DEN_APICAL_DISTAL_FLAG) != 0;

    uint bsl_prx_celtft_idx = mad24((uint)bsl_prx_tft_id, cel_count, cel_idx);
    uint bsl_dst_celtft_idx = mad24((uint)bsl_dst_tft_id, cel_count, cel_idx);
    uint apc_dst_celtft_idx = mad24((uint)apc_dst_tft_id, cel_count, cel_idx);

    uchar bsl_prx_state = mul24(bsl_prx_is_enabled, tft_states[bsl_prx_celtft_idx]);
    uchar bsl_dst_state = mul24(bsl_dst_is_enabled, tft_prev_states[bsl_dst_celtft_idx]);
    uchar apc_dst_state = mul24(apc_dst_is_enabled, tft_prev_states[apc_dst_celtft_idx]);

    int cel_is_active = bsl_prx_state != 0;

    //
    // TODO: Fix below to transmit bursting on the 7th bit.
    //

    // Divide by 4 but don't let small values get rounded to 0:
    int bsl_prx_is_min = (bsl_prx_state <= 3) && cel_is_active;
    uchar bsl_prx_contrib = (bsl_prx_state >> 2) + bsl_prx_is_min;

    // Divide by 2, ignore rounding:
    uchar bsl_dst_contrib = mul24(cel_is_active, bsl_dst_state >> 1);

    // Divide by 4, ignore rounding:
    uchar apc_dst_contrib = mul24(cel_is_active, apc_dst_state >> 2);

    // pyr_best_den_states_raw[cel_idx] = pyr_best_den_state_raw;
    pyr_states[cel_idx] = bsl_prx_contrib + bsl_dst_contrib, + apc_dst_contrib;
}





// Distal tuft medium-term potentiation and depression.
//
//
// First, to clarify:
//     - The term 'vatic' is meant to mean predictive or in a state of
//       expectance. May be referred to (incorrectly) as fuzzy or 'fuz'. A
//       few other (deprecated) terms might be thrown around due to much
//       renaming but basically if a pyramidal soma is active, that cell is
//       vatic.
//     - the term 'concrete' is meant to mean that the cell has an active axon
//       and therefore has not been inhibited by anything else (e.g. the most
//       likely culprits, pyramidals within the same layer and column -- our
//       cell's colleagues).
//
// The learning process is as follows:
//     - For each pyramidal cell:
//         - If the cell is concrete AND {the cell *was* previously vatic OR
//           it *is* the best in the column}:
//             - For each tuft on that cell:
//                 - If tuft is active (i.e. has a best dendrite state != 0):
//                     - Cause 'Learning Initiation' to take place on that
//                       most active dendrite (see below).
//         - If the cell's axon is not concrete but was previously (flag_set
//           & CEL_PREV_CONCRETE_FLAG):
//             - 'Terminate' each tuft.
//
// Learning Initiation:
//     -
//
//
// - TODO:
//     - [incomplete] Vectorize (should be highly vectorizable)
//     - reducing branching will be tough with this one
//     - [in progress] Tests (check that flag_set and prev_best_den_id are robustly maintained)
//
//
//     - if pyr_prev_concrete
//         - if pyr_concrete
//         - if pyr_state
//
//     - if pyr_prev_pred
//         - if pyr_concrete
//         - if pyr_state
//
// - Misc Notes:
//
//     - SYN(    -> STPOT) WHEN: (SYN_STATE > 0) AND (CEL_TANGIBLE) AND (CEL_BEST_IN_COLUMN)
//                         OR: (SYN_STATE > 0) AND (CEL_TANGIBLE) AND (CEL_PREV_PRED)
//
//     - MAINTAIN STPOT STATE AS LONG AS: (SYN_STATE > 0) AND (CEL_ACTIVE)
//
//     - SYN(STPOT -> LTP) ONLY WHEN: ((CEL_ACTIVE -> 0)) SAME TIME AS (SYN_STATE -> 0)
//
//
// INDEXING EXAMPLE: <<<<< TODO: UPDATE TO CORRECT DENDRITE INDEXING >>>>>
//
//     Let's imagine we have an area in the cortex with the following properties:
//
//         COL_COUNT (column count for area): 1024
//
//     This area has a layer of pyramidal cells (such as layer iii) with:
//
//         DEPTH / CELS_PER_COL (cells per column, aka. layer depth): 3
//         TFTS_PER_CEL (tufts per cell): 2
//         DENS_PER_TFT (dendrites per tuft): 4
//         SYNS_PER_DEN (synapses per dendrite): 32
//
//     So we have a layer 3 cells deep, each layer containing 1024 cells. We
//     can also think of it as 1024 columns, each with three cells. Each of
//     the cells in a column share the same spatial input axon. That is,
//     cells are activated (in the previous kernel) based on the same spatial
//     input axon.
//
//     A few things to keep in mind when indexing axons, cells, dendrites, and synapses:
//         - The axons will correspond to the cell indexes 1:1, just with a
//           different idz (starting index).
//         - Synapses and dendrites is where things are trickier. Synapse
//           (and dendrite) space (within the syn_states[] array) is
//           primarily divided by tuft, unintuitively. For an explanation and
//           more information see 'synapses.rs'.
//             - First let's calculate our den_idx:
//                 den_idx :=
//
//                 { INCOMPLETE - LOOK AT DENDRITE INDEXING }
//
//                 { THE FOLLOWING IS OUT OF DATE }
//
//             - Synapse indexes will first need to be calculated using the
//               dendrite index, i.e.: syn_id_tuft := den_idx * syns_per_den
//               (or equivalently: den_idx << syns_per_den_l2).
//             - Next they will be added to the tuft offset:
//                 - syns_per_tft_space := syns_per_den * dens_per_tft * cels_per_col * col_count
//                     - (note that tufts per cell is not factored in here)
//                 - syn_idz_tuft := tuft_id * syns_per_tft_space
//                 - syn_idx := syn_idz_tuft + syn_id_tuft
//
//
//     So, here's an example breakdown of how this all plays out:
//         - Notation: OBJ[id_parent]:[idx_physical]
//              - More plainly: 'object [ id within parent object ]:[ global (physical) index ]'
//
//         -----------------------
//
//         CEL[0]:[0] (COL[0])
//             TFT[0]
//                 DEN[0]:[0]
//                     SYN[0]:[0], SYN[1]:[1], ..., SYN[31]:[31]
//                 DEN[1]:[1]
//                     SYN[0]:[32], SYN[1]:[33], ..., SYN[31]:[68]
//                 ...
//                 DEN[3]:[3]
//                     SYN[0]:[96], SYN[1]:[97], ..., SYN[31]:[127]
//             TFT[1]
//                 DEN[0]:[XXX] <<<<< UPDATE ME! >>>>>
//                     SYN[0]:[393216], SYN[1]:[393217], ..., SYN[31]:[393247]
//                 ...
//                 DEN[3]:[XXX] <<<<< UPDATE ME! >>>>>
//                     SYN[0]:[393312], SYN[1]:[393313], ..., SYN[31]:[393343]
//         CEL[1]:1 (COL[0])
//             TFT[0]
//                 DEN[0]:[8]
//                     SYN[0]:[0], SYN[1]:[1], ..., SYN[31]:[31]
//                 ...
//                 DEN[3][11]
//                     SYN[0]:[96], SYN[1]:[97], ..., SYN[31]:[127]
//             TFT[1]
//                 DEN[0][XXX] <<<<< UPDATE ME! >>>>>
//                     SYN[0]:[393216], SYN[1]:[393217], ..., SYN[31]:[393247]
//                 ...
//         CEL[2]:[2] (COL[0])
//             ...
//         CEL[3]:[3] (COL[1])
//             ...
//         CEL[5]:[5] (COL[1])
//             ...
//         CEL[6]:[6] (COL[2])
//             ...
//         ...
//         CEL[3071]:[3071] (COL[1023])
//             ...

//         -----------------------

//     Given that indexing structure, the following kernel structure appears
//     to be the best balance of performance and simplicity:

//         Kernel: WorkSize: OneDim(cell groups **) - Iterate through cell groups
//             - Loop: cells - Iterate through cells within each group.
//                 - Loop: tufts - Iterate through cell tufts.
//                     - Function call: dendrites - For each tuft, if work
//                       needs to be done, pick the most active or previously
//                       active dendrite(s) then call a function is called
//                       which will -> Loop: synapses - Iterate through
//                       dendrite synapses.
//                         - STPOT, LTP, and LTD take place on synapses
//                           within this the smallest loop.
//
//
//
//     Note: Cell groups are divisions of the cell space for the layer
//        into groups of arbitrary size. Cell groups are used in lieu of
//        individual cells as the primary work dimension because during any
//        given cycle. Most cells will need no work done on its synapses,
//        therefore most work items would be idle. By bundling a group of
//        cells into each work item, all threads can keep busy.
//
//
__kernel void tft_dst_mtp(
        __global const uchar* const axn_states,
        __global const uchar* const cel_states,
        __global const uchar* const tft_cel_prev_best_den_ids,
        __global const uchar* const tft_cel_prev_best_den_states_raw,
        // UNUSED:
        __global const uchar* const den_states,
        // UNUSED:
        __global const uchar* const syn_states,

        __private uint const tft_cel_idz, // 0th tuft-cell index
        // UNUSED:
        __private uint const tft_den_idz, // 0th tuft-dendrite index
        __private uint const tft_syn_idz, // 0th tuft-synapse index

        // UNUSED:
        __private uint const dens_per_tft,
        __private uint const syns_per_den,
        __private uint const syns_per_tft,

        __private uint const cels_per_cel_grp,
        __private uint const axn_idz_cel_lyr,
        __private int const potentiation_rate_l2i,
        __private int const depression_rate_l2i,
        __private int const rnd,
        __global uchar* const syn_flag_sets,
        __global uchar* const cel_flag_sets,
        __global int* const aux_ints_0,
        __global int* const aux_ints_1,
        // TODO: Switch to `u8` (`uchar`):
        __global char* const syn_strengths)
{
    uint const cel_grp_id = get_global_id(0);
    uint const cel_grp_count = get_global_size(0);
    uint const cel_count = mul24(cel_grp_count, cels_per_cel_grp);
    // Index of the 0th cell in the cell group:
    uint const cel_idz_cel_grp = mul24(cel_grp_id, cels_per_cel_grp);

    // TODO: Add a tiny decay to all synapses on a previously active dendrite
    // whose cell is not active (will need to store prev den state).

    for (uint cel_id_cel_grp = 0; cel_id_cel_grp < cels_per_cel_grp; cel_id_cel_grp++) {
        // Current cell:
        uint const cel_idx = cel_idz_cel_grp + cel_id_cel_grp;
        // Current cell's axon:
        uint const cel_axn_idx = axn_idz_cel_lyr + cel_idx;
        // Current tuft-cell:
        uint const tft_cel_idx = cel_idx + tft_cel_idz;
        // Index of the 0th synapse within the current cell-tuft:
        uint const syn_idz_celtft = mad24(cel_idx, syns_per_tft, tft_syn_idz);

        uchar cel_flag_set = cel_flag_sets[cel_idx];

        int const cel_is_active = axn_states[cel_axn_idx] != 0;
        int const cel_prev_active = (cel_flag_set & (CEL_PREV_ACTIVE_FLAG)) == (CEL_PREV_ACTIVE_FLAG);
        int const cel_newly_active = !cel_prev_active & cel_is_active;
        int const tft_prev_active = tft_cel_prev_best_den_states_raw[tft_cel_idx] != 0;

        if (cel_newly_active & tft_prev_active) {
            // ID of the Best dendrite within the current tuft-cell:
            uchar const prev_best_den_id_celtft = tft_cel_prev_best_den_ids[tft_cel_idx];

            uint const syn_idz_prev_best_den_tft = (prev_best_den_id_celtft * syns_per_den) +
                syn_idz_celtft;

            dst_syns__active__mtpot_mtdep(syn_states, syn_idz_prev_best_den_tft, syns_per_den,
                potentiation_rate_l2i, depression_rate_l2i, rnd, syn_flag_sets, syn_strengths);
        }

        cel_flag_set &= ~CEL_PREV_ACTIVE_FLAG;
        cel_flag_set |= mul24(cel_is_active, CEL_PREV_ACTIVE_FLAG);
        cel_flag_sets[cel_idx] = cel_flag_set;
    }
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



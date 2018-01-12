// [LINE 1428]

static inline uchar syn_fire(uchar const axn_state) {
    return ((axn_state != 0) << 7) + (axn_state >> (SYNAPSE_AXON_BIAS_LOG2));
}

static inline uchar4 syn_fire_vec4(uchar4 const axn_state) {
    return (convert_uchar4(axn_state != (uchar)0) & (uchar4)0x80)
        | (axn_state >> (uchar4)(SYNAPSE_AXON_BIAS_LOG2));
}


// Update flags to indicate activity of the prior state.
//
// NOTE: This functionality should be integrated into synapse cycle kernels at
// some point (for performance reasons).
__kernel void tft_set_syn_flags(
        __global const uchar* const states,
        __global uchar* const flag_sets)
{
    uint const syn_idx = get_global_id(0);

    int syn_was_active = states > 0;
    uchar flag_set = flag_sets[syn_idx] & ~SYN_PREV_ACTIVE_FLAG;
    flag_set = flag_set | mul24((uchar)syn_was_active, SYN_PREV_ACTIVE_FLAG);
    flag_sets[syn_idx] = flag_set;
}


// Process synapses for a tuft assuming irregular tuft sizes.
__kernel void tft_cycle_syns(
        __global const uchar* const axn_states,
        __global const char* const syn_src_col_u_offs,
        __global const char* const syn_src_col_v_offs,
        __global const uchar* const syn_src_slc_ids,
        // __private uint const cel_idz_syntuft,
        __private uint const syn_idz_tft,
        __private uchar const syns_per_tft_l2,
        __private uchar const layer_depth,
        __global int* const aux_ints_0,
        __global int* const aux_ints_1,
        __global uchar* const syn_states)
{
    uint const v_id = get_global_id(0);
    uint const u_id = get_global_id(1);
    uint const v_size = get_global_size(0);
    uint const u_size = get_global_size(1);

    for (int slc_id_lyr = 0; slc_id_lyr < layer_depth; slc_id_lyr++) {
        // uint const syn_idz = (cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id)
        //     + cel_idz_syntuft) << syns_per_tuft_l2;
        uint const syn_idz = (cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id)
            << syns_per_tft_l2) + syn_idz_tft;

        uint const syn_idn = syn_idz + (1 << syns_per_tft_l2);

        ////// DEBUG:
            // if (v_id == 0 && u_id == 0) {
            //     uint syn_idz = 0 + (uint)slc_id_lyr;
            //     syn_states[100] = 200;
            // }

            // uint syn_idz = 0 + (uint)slc_id_lyr;
            // uint syn_idn = syn_idz + 1;
        //////

        for (uint syn_idx = syn_idz; syn_idx < syn_idn; syn_idx++) {
            uchar const src_slc_id = syn_src_slc_ids[syn_idx];
            char const v_ofs = syn_src_col_v_offs[syn_idx];
            char const u_ofs = syn_src_col_u_offs[syn_idx];

            uchar axn_state = axn_state_3d_safe(src_slc_id, v_id, v_ofs, u_id, u_ofs, axn_states);

            syn_states[syn_idx] = syn_fire(axn_state);

            ////// DEBUG:
                // if (axn_state != 0) {
                //     aux_ints_0[syn_idx] = axn_state;
                // }

                // if (v_id == 0 && u_id == 0) {
                //     // syn_states[slc_id_lyr] = slc_id_lyr;
                //     aux_ints_0[syn_idx] = axn_state;
                // }

                // if (v_id == (v_size - 1) && u_id == (u_size - 1)) {
                //     // syn_states[100 + slc_id_lyr] = 100 + slc_id_lyr;
                //     aux_ints_1[syn_idx] = axn_state;
                // }

                // if (src_slc_id != 0) {
                // // if (v_id == 5 && u_id == 5) {
                //     int idx_is_safe = 0;
                //     aux_ints_0[syn_idx] = axn_idx_3d_unsafe(src_slc_id, v_id, v_ofs, u_id, u_ofs, &idx_is_safe);
                // }

                // if (syn_idx == 99) {
                //     syn_states[syn_idx] = 199;
                // }

                // syn_states[syn_idx] = (uchar)clamp((uint)src_slc_id, (uint)0, (uint)255);
            //////
        }

        ////// DEBUG:
            // if (v_id == 0 && u_id == 0) {
            //     // syn_states[slc_id_lyr] = slc_id_lyr;
            //     aux_ints_0[slc_id_lyr] = slc_id_lyr;
            // }

            // if (v_id == (v_size - 1) && u_id == (u_size - 1)) {
            //     // syn_states[100 + slc_id_lyr] = 100 + slc_id_lyr;
            //     aux_ints_0[100 + slc_id_lyr] = 100 + slc_id_lyr;
            // }
        ////// DEBUG:
    }
}



// SYNS_CYCLE_SIMPLE_VEC4(): Simple synapse cycling with vectorization, layer-at-once
__kernel void tft_cycle_syns_vec4(
        __global const uchar* const axn_states,
        __global const char4* const syn_src_col_u_offs,
        __global const char4* const syn_src_col_v_offs,
        __global const uchar4* const syn_src_slc_ids,
        // __private uint const cel_idz_syntuft,
        __private uint const syn_idz_tft,
        __private uchar const syns_per_tft_l2,
        __private uchar const layer_depth,
        __global int* const aux_ints_0,
        __global int* const aux_ints_1,
        __global uchar4* const syn_states)
{
    uint const v_id = get_global_id(0);
    uint const u_id = get_global_id(1);
    uint const v_size = get_global_size(0);
    uint const u_size = get_global_size(1);

    for (int slc_id_lyr = 0; slc_id_lyr < layer_depth; slc_id_lyr++) {
        // // DIVIDED BY 4 BECAUSE VECTORS:
        // uint const syn4_idz = ((cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id)
        //     + cel_idz_syntuft) << (syns_per_tuft_l2 - 2));
        // DIVIDED BY 4 BECAUSE VECTORS:
        uint const syn4_idz = (cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id)
            << (syns_per_tft_l2 - 2)) + (syn_idz_tft >> 2);

        // DIVIDED BY 4 BECAUSE VECTORS:
        uint const syn4_idn = syn4_idz + (1 << (syns_per_tft_l2 - 2));


        for (uint syn4_idx = syn4_idz; syn4_idx < syn4_idn; syn4_idx++) {
            uchar4 const src_slc_id = syn_src_slc_ids[syn4_idx];
            char4 const v_ofs = syn_src_col_v_offs[syn4_idx];
            char4 const u_ofs = syn_src_col_u_offs[syn4_idx];

            uchar4 const axn_state = axn_state_3d_safe_vec4(
                src_slc_id,
                (int4)(int)v_id,
                v_ofs,
                (int4)(int)u_id,
                u_ofs,
                axn_states);

            syn_states[syn4_idx] = syn_fire_vec4(axn_state);
        }
    }
}



//SYNS_CYCLE_WG_OPT(): Cycle synapses with workgroup optimized writes, layer optimized
__kernel void layer_cycle_syns_wow(
        __global const uchar* const axn_states,
        __global const char* const syn_src_col_u_offs,
        __global const char* const syn_src_col_v_offs,
        __global const uchar* const syn_src_slc_ids,
        // __private uint const cel_idz_syntuft,
        __private uint const syn_idz_tft,
        __private uchar const syns_per_tft_l2,
        __private uchar const layer_depth,
        __global int* const aux_ints_0,
        __global int* const aux_ints_1,
        __global uchar* const syn_states)
{
    uint const v_size = get_global_size(0);
    uint const u_size = get_global_size(1);

    uint const v_work_size = get_local_size(0);
    uint const u_work_size = get_local_size(1);

    uint const v_id_local = get_local_id(0);
    uint const u_id_local = get_local_id(1);

    // BASE DIM_IDs FOR CURRENT WORKGROUP
    uint const v_idz_wg = get_global_id(0) - v_id_local;
    uint const u_idz_wg = get_global_id(1) - u_id_local;

    uint const syns_per_tft = 1 << syns_per_tft_l2;
    uint const syns_per_wg = mul24(v_work_size, u_work_size);


    int cur_syn_ofs = mad24(v_id_local, u_work_size, u_id_local);
    int u_id_wg_crnt = 0;
    int v_id_wg_crnt = 0;

    while (cur_syn_ofs >= syns_per_tft) {
        u_id_wg_crnt += 1;
        cur_syn_ofs -= syns_per_tft;
    }

    while (u_id_wg_crnt >= u_work_size) {
        v_id_wg_crnt += 1;
        u_id_wg_crnt -= u_work_size;
    }

    uint syns_per_iter = syns_per_wg;
    // PRECALCULATE THE FOLLOWING ON HOST
        uint u_per_iter = 0;
        uint v_per_iter = 0;

        while (syns_per_iter >= syns_per_tft) {
            u_per_iter += 1;
            syns_per_iter -= syns_per_tft;
        }

        while (u_per_iter >= u_work_size) {
            v_per_iter += 1;
            u_per_iter -= u_work_size;
        }
    // END PRECALCULATE


    // FOR EACH SYNAPSE ON CELL-TUFT
    for (uint i = 0; i < syns_per_tft; i += 1) {
        int const cur_syn_ofs_is_oob = (cur_syn_ofs >= syns_per_tft);
        u_id_wg_crnt += cur_syn_ofs_is_oob;
        cur_syn_ofs -= mul24(cur_syn_ofs_is_oob, (int)syns_per_tft);

        int const u_id_wg_crnt_is_oob = (u_id_wg_crnt >= u_work_size);
        v_id_wg_crnt += u_id_wg_crnt_is_oob;
        u_id_wg_crnt -= mul24(u_id_wg_crnt_is_oob, (int)u_work_size);

        uint const v_id = v_idz_wg + v_id_wg_crnt;
        uint const u_id = u_idz_wg + u_id_wg_crnt;

        // FOR EACH SLICE IN LAYER
        for (int slc_id_lyr = 0; slc_id_lyr < layer_depth; slc_id_lyr++) {
            // uint const syn_idx = ((cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id) + cel_idz_syntuft)
            //     << syns_per_tft_l2) + cur_syn_ofs;
            uint const syn_idx = (cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id)
                << syns_per_tft_l2) + syn_idz_tft + cur_syn_ofs;

            char const v_ofs = syn_src_col_v_offs[syn_idx];
            char const u_ofs = syn_src_col_u_offs[syn_idx];
            uchar const src_slc_id = syn_src_slc_ids[syn_idx];

            uchar const axn_state = axn_state_3d_safe(src_slc_id, v_id, v_ofs, u_id, u_ofs, axn_states);

            syn_states[syn_idx] = syn_fire(axn_state);

            // ### DO NOT REMOVE ###
            // if ((slc_id_lyr == 1) && (get_global_id(1) == 6) && (get_global_id(2) == 6) && (cel_idz_syntuft == 0)) {
            //     aux_ints_0[i] = v_idz_wg;
            // }
        }

        cur_syn_ofs += syns_per_iter;
        u_id_wg_crnt += u_per_iter;
        v_id_wg_crnt += v_per_iter;

    }
}



// SYNS_CYCLE_WG_OPT_VEC4(): Cycle synapses with workgroup optimized writes and vectorization, layer optimized
//         See above for annotated version.
__kernel void layer_cycle_syns_wow_vec4(
        __global const uchar* const axn_states,
        __global const char4* const syn_src_col_u_offs,
        __global const char4* const syn_src_col_v_offs,
        __global const uchar4* const syn_src_slc_ids,
        // __private uint const cel_idz_syntuft,
        __private uint const syn_idz_tft,
        __private uchar const syns_per_tft_l2,
        __private uchar const layer_depth,
        __global int* const aux_ints_0,
        __global int* const aux_ints_1,
        __global uchar4* const syn_states)
{
    uint const v_size = get_global_size(0);
    uint const u_size = get_global_size(1);

    uint const v_work_size = get_local_size(0);
    uint const u_work_size = get_local_size(1);

    uint const v_id_local = get_local_id(0);
    uint const u_id_local = get_local_id(1);

    uint const v_idz_wg = get_global_id(0) - v_id_local;
    uint const u_idz_wg = get_global_id(1) - u_id_local;

    uint const syn4s_per_tft = (1 << (syns_per_tft_l2)) >> 2; // VEC4'D
    uint const syn4s_per_wg = mul24(v_work_size, u_work_size); // DON'T DIVIDE ME (DOING SAME SYN4S AS SYNS)


    int cur_syn4_ofs = mad24(v_id_local, u_work_size, u_id_local);
    int u_id_wg_crnt = 0;
    int v_id_wg_crnt = 0;

    while (cur_syn4_ofs >= syn4s_per_tft) {
        u_id_wg_crnt += 1;
        cur_syn4_ofs -= syn4s_per_tft;
    }

    while (u_id_wg_crnt >= u_work_size) {
        v_id_wg_crnt += 1;
        u_id_wg_crnt -= u_work_size;
    }


    uint syn4s_per_iter = syn4s_per_wg;     // PRECALCULATE -- MAKE CONST
    uint u_per_iter = 0;    // PRECALCULATE -- MAKE CONST
    uint v_per_iter = 0;     // PRECALCULATE -- MAKE CONST

    while (syn4s_per_iter >= syn4s_per_tft) { // PRECALCULATE
        u_per_iter += 1;
        syn4s_per_iter -= syn4s_per_tft;
    }

    while (u_per_iter >= u_work_size) { // PRECALCULATE
        v_per_iter += 1;
        u_per_iter -= u_work_size;
    }


    for (uint i = 0; i < syn4s_per_tft; i++) {
        int const cur_syn4_ofs_is_oob = (cur_syn4_ofs >= syn4s_per_tft);
        u_id_wg_crnt += cur_syn4_ofs_is_oob;
        cur_syn4_ofs -= mul24(cur_syn4_ofs_is_oob, (int)syn4s_per_tft);

        int const u_id_wg_crnt_is_oob = (u_id_wg_crnt >= u_work_size);
        v_id_wg_crnt += u_id_wg_crnt_is_oob;
        u_id_wg_crnt -= mul24(u_id_wg_crnt_is_oob, (int)u_work_size);

        uint const v_id = v_idz_wg + v_id_wg_crnt;
        uint const u_id = u_idz_wg + u_id_wg_crnt;

        for (int slc_id_lyr = 0; slc_id_lyr < layer_depth; slc_id_lyr++) {
            // uint syn4_idx = (((cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id) + cel_idz_syntuft)
            //     << syns_per_tft_l2) >> 2) + cur_syn4_ofs; // VEC4'D IDX

            // VEC4'D:
            uint syn4_idx = (cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id) <<
                (syns_per_tft_l2 - 2)) + (syn_idz_tft >> 2) + cur_syn4_ofs;


            char4 const v_ofs = syn_src_col_v_offs[syn4_idx];
            char4 const u_ofs = syn_src_col_u_offs[syn4_idx];
            uchar4 const src_slc_id = syn_src_slc_ids[syn4_idx];

            uchar4 const axn_state = axn_state_3d_safe_vec4(
                src_slc_id,
                (int4)(int)v_id,
                v_ofs,
                (int4)(int)u_id,
                u_ofs,
                axn_states);

            syn_states[syn4_idx] = syn_fire_vec4(axn_state);

            // ### DO NOT REMOVE ###
            // if ((slc_id_lyr == 1) && (get_global_id(1) == 6) && (get_global_id(2) == 6) && (cel_idz_syntuft == 0)) {
            //     aux_ints_0[i] = u_id_wg_crnt;
            // }
        }

        cur_syn4_ofs += syn4s_per_iter;
        u_id_wg_crnt += u_per_iter;
        v_id_wg_crnt += v_per_iter;

    }
}


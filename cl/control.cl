// # CONTROL.CL
//
// Kernels for control (inhibitory, etc.) cells.

// 255 (max) ~> 1:1
#define CELL_ACTIVITY_DECAY_CUTOFF 8


//     INHIB_SIMPLE(): [DESCRIPTION OUT OF DATE] Cell Inhibition - reads from soma, writes to axon
//        - If any nearby cells are more active (have a higher soma 'state')
//            - cell will not 'fire'
//            - otherwise, write soma (cel_states[cel_idx]) to axon (axn_states[axn_idx])
//
//        - Overly simplistic algorithm
//            - Distance should be taken into account when state is considered
//            - Search area broadened
//         - Horribly unoptimized, Should:
//            - cache values for an area in local (workgroup) memory
//                - or just prefetch global cache? (comparison needed)
//            - be vectorized
__kernel void inhib_simple(
            __global uchar const* const cel_states,
            __private uchar const cel_base_axn_slc,
            __private int const rnd,
            __global uchar* const activities,
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
            uchar neighbor_state =
                cel_state_3d_safe(slc_id_lyr, v_size, v_id, v_ofs, u_size, u_id, u_ofs, cel_states);
            int distance = (abs(v_ofs) + abs(u_ofs) + abs(w_ofs(v_ofs, u_ofs)))    >> 1;

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
                uint unsafe_target_axn_idx = axn_idx_3d_safe(slc_id_lyr +
                cel_base_axn_slc, v_size, v_id, v_ofs, u_size, u_id, u_ofs);

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
            //     //     // aux_ints_1[axn_idx_3d_safe(slc_id_lyr + cel_base_axn_slc, v_size,
            //     //     // v_id, v_ofs, u_size, u_id, u_ofs)] = distance;
            //     //     aux_ints_1[520 + dumb_iter]
            //     //         //= cel_influence;
            //     //         = distance + 100;
            //     // }
            // }
        }
    }

    // Set axon state if cell is uninhibited:
    axn_states[cel_axn_idx] = mul24((uint)uninhibited, (uint)cel_state);

    int axon_is_active = uninhibited & (cel_state != 0);

    // Get activity rating:
    uchar activity_rating = activities[cel_idx];
    // Increment activity rating if active:
    activity_rating += rnd_inc_u(rnd, cel_state & cel_idx, activity_rating) & axon_is_active;
    // Decrement activities count at random (needs tuning [256 max]):
    activity_rating -= rnd_256(rnd, cel_state | cel_idx, CELL_ACTIVITY_DECAY_CUTOFF) &
       (activity_rating > 0);

    // /////// DEBUG:
    // uchar activity_rating = activities[cel_idx];
    // activity_rating += axon_is_active & (activity_rating < 254);
    // activity_rating -= rnd_256(rnd, cel_state | cel_idx, CELL_ACTIVITY_DECAY_CUTOFF)
    //     & (activity_rating > 0);
    // ///////

    activities[cel_idx] = activity_rating;
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


// __kernel void smooth(
//             __global uchar
//     )
// {

// }
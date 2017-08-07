// # CONTROL.CL
//
// Kernels for control (inhibitory, etc.) cells.

// Passed to `rnd_0xFFFF()`. 65536 (max) ~> 1:1
#define CELL_ACTIVITY_DECAY_FACTOR      768

// // INHIB_RADIUS: A CELL'S SPHERE OF INFLUENCE
// #define INHIB_RADIUS                    4
// INHIB_INFL_CENTER_OFFSET: MOVES CENTER OF INHIBITION CURVE NEARER(-) OR FURTHER(+) FROM CELL
#define INHIB_INFL_CENTER_OFFSET        1
// INHIB_INFL_HORIZ_OFFSET: STRETCHES EDGE OF INHIBITION CURVE NEARER(-) OR FURTHER(+) FROM CELL
#define INHIB_INFL_HORIZ_OFFSET          3


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
            // __global uchar const* const energies,
            __private uchar const cel_base_axn_slc,
            __private int const inhib_radius,
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
    uint const cel_axn_idx = axn_idx_3d_unsafe(slc_id_lyr + cel_base_axn_slc,
        v_id, 0, u_id, 0, &idx_is_safe);

    // // Add the energy (restlessness) to the feed-forward state:
    // uint energy = (uint)energies[cel_idx];
    // energy = mul24((uint)(energy > 127), energy);
    // uchar const cel_state_raw = clamp((uint)cel_states[cel_idx] + energy, (uint)0, (uint)255);

    // The cell state, if the index is not out of bounds (otherwise zero):
    uchar const cel_state = mul24(idx_is_safe, (int)cel_states[cel_idx]);

    // uchar const cel_state = cel_states[cel_idx];

    int const radius_pos = inhib_radius;
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

    // // Get activity rating:
    // uchar activity_rating = activities[cel_idx];
    // // Increment activity rating if active:
    // int axon_is_active = uninhibited & (cel_state != 0);
    // activity_rating += rnd_inc_u(rnd, cel_state ^ cel_idx, activity_rating) & axon_is_active;
    // // Decrement activities count at random (may need tuning):
    // activity_rating -= rnd_0xFFFF(rnd, cel_state | cel_idx, CELL_ACTIVITY_DECAY_FACTOR) &
    //    (activity_rating > 0);

    // /////// DEBUG:
    // uchar activity_rating = activities[cel_idx];
    // activity_rating += axon_is_active & (activity_rating < 254);
    // activity_rating -= rnd_0xFF(rnd, cel_state | cel_idx, CELL_ACTIVITY_DECAY_FACTOR)
    //     & (activity_rating > 0);
    // ///////

    // Increment activity rating if active:
    int axon_is_active = uninhibited & (cel_state != 0);

    // activities[cel_idx] = activity_rating;
    activities[cel_idx] = update_activity_rating(activities[cel_idx], axon_is_active,
        rnd, cel_state | cel_idx, CELL_ACTIVITY_DECAY_FACTOR);
}


__kernel void inhib_passthrough(
            __global uchar const* const cel_states,
            __private uchar const cel_base_axn_slc,
            __private int const rnd,
            __global uchar* const activities,
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
    int axon_is_active = cel_state != 0;
    activities[cel_idx] = update_activity_rating(activities[cel_idx], axon_is_active,
        rnd, cel_state | cel_idx, CELL_ACTIVITY_DECAY_FACTOR);
}


// Smooths cell activity by manipulating the cell's energy.
//
// * Iterate through data cells within the smoother cell's radius
// * Calculate a cell index for each iteration
// * Find most and least active cells within radius
// * Increment energy of least active cell if < 255
// * Decrement most active if > 0
__kernel void smooth_activity(
            __global int const* const centers_v,
            __global int const* const centers_u,
            __private uint const v_size,
            __private uint const u_size,
            __private int const radius,
            __private uchar const src_lyr_depth,
            __global uchar const* const cel_actvs,
            // __global int* const aux_ints_1,
            __global uchar* const cel_energies)
{
    uint const center_idx = get_global_id(0);
    int const center_v = centers_v[center_idx];
    int const center_u = centers_u[center_idx];

    uint least_active_cel_idx = 0;
    uchar least_active_cel_actv = 255;
    uint most_active_cel_idx = 0;
    uchar most_active_cel_actv = 0;

    int const radius_pos = radius;
    int const radius_neg = 0 - radius_pos;

    for (uchar slc_id_lyr = 0; slc_id_lyr < src_lyr_depth; slc_id_lyr++) {
        for (int v_ofs = radius_neg; v_ofs <= radius_pos; v_ofs++) {
            int v_neg = 0 - v_ofs;
            int u_z = max(radius_neg, v_neg - radius_pos);
            int u_m = min(radius_pos, v_neg + radius_pos);

            for (int u_ofs = u_z; u_ofs <= u_m; u_ofs++) {
                int idx_is_safe = 0;
                uint cel_idx = cel_idx_3d_checked(slc_id_lyr, v_size, center_v + v_ofs,
                    u_size, center_u + u_ofs, &idx_is_safe);
                uchar cel_actv = cel_actvs[mul24((uint)idx_is_safe, cel_idx)];

                int cel_is_least_active = (cel_actv <= least_active_cel_actv) & idx_is_safe;
                // int cel_is_least_active = (cel_actv <= least_active_cel_actv);
                least_active_cel_idx = tern24(cel_is_least_active, cel_idx, least_active_cel_idx);
                least_active_cel_actv = tern24(cel_is_least_active, cel_actv, least_active_cel_actv);

                int cel_is_most_active = (cel_actv >= most_active_cel_actv) & idx_is_safe;
                most_active_cel_idx = tern24(cel_is_most_active, cel_idx, most_active_cel_idx);
                most_active_cel_actv = tern24(cel_is_most_active, cel_actv, most_active_cel_actv);

                // DEBUG RADIUS/OFFSET CALCULATIONS:
                #ifdef DEBUG_SMOOTHER_OVERLAP
                    if (idx_is_safe) {
                        if ((cel_energies[cel_idx] < 255) ) {
                            cel_energies[cel_idx] += 1;
                        }
                    }
                #endif
            }
        }
    }

    #ifndef DEBUG_SMOOTHER_OVERLAP
        // Least Active (boost energy):
        uchar least_active_cel_energy = cel_energies[least_active_cel_idx];
        cel_energies[least_active_cel_idx] = least_active_cel_energy +
            ((least_active_cel_energy < 255) & (most_active_cel_actv < 255) &
                (least_active_cel_idx != most_active_cel_idx));

        // Most Active (sap energy):
        uchar most_active_cel_energy = cel_energies[most_active_cel_idx];
        cel_energies[most_active_cel_idx] = most_active_cel_energy -
            ((most_active_cel_energy > 0) & (most_active_cel_actv > 0) &
                (least_active_cel_idx != most_active_cel_idx));
    #endif
}

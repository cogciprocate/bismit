// FILTERS.CL: Experimental and very badly optimized.


#define INHIB_SMALL_CELL_RADIUS         3
#define INHIB_LARGE_CELL_RADIUS         5


static inline int get_neighbors_avg(uchar const slc_id_lyr, uint const v_size, uint const v_id, 
            uint const u_size, uint const u_id, __global uchar const* const cel_states, 
            uint const cel_idx, int const radius)
{
    int const radius_pos = radius; // (4:61), (7:XXX), (9:271)
    int const radius_neg = 0 - radius_pos;

    //int const center_state = cel_states[cel_idx];
    int neighbors_sum = 0; // includes cell itself
    int neighbor_count = 0;

    for (int v_ofs = radius_neg; v_ofs <= radius_pos; v_ofs++) {
        int v_neg = 0 - v_ofs;
        int u_z = max(radius_neg, v_neg - radius_pos);
        int u_m = min(radius_pos, v_neg + radius_pos);

        for (int u_ofs = u_z; u_ofs <= u_m; u_ofs++) {
            neighbors_sum += cel_state_3d_safe(slc_id_lyr, v_size, v_id, v_ofs, u_size, u_id, u_ofs, cel_states);
            neighbor_count += 1;
        }
    }    

    return neighbors_sum / neighbor_count;
}


__kernel void retina(
                __global uchar const* const cel_states,
                __private uchar const cel_base_axn_slc,        // <<<<< DEPRICATE: USE A GLOBAL OFFSET
                __global uchar* const axn_states
) {
    uint const slc_id_lyr = get_global_id(0);    // <<<<< TODO: USE A GLOBAL OFFSET
    uint const v_id = get_global_id(1);
    uint const u_id = get_global_id(2);
    uint const v_size = get_global_size(1);
    uint const u_size = get_global_size(2);

    uint const cel_idx = cel_idx_3d_unsafe(slc_id_lyr, v_size, v_id, u_size, u_id);
    //uint const axn_idx = axn_idx_3d_safe(slc_id_lyr + cel_base_axn_slc, v_size, v_id, 0, u_size, u_id, 0);
    uint const tar_axn_idz = get_axn_idz(slc_id_lyr + cel_base_axn_slc);
    uint const axn_idx = tar_axn_idz + cel_idx_3d_unsafe(0, v_size, v_id, u_size, u_id);
    //uint const axn_idx = axn_idx_3d_unsafe(tar_axn_slc, v_id, 0, u_id, 0);

    int const center_state = cel_states[cel_idx];

    // int const radius_pos = 4; // (4:61), (7:XXX), (9:271)
    // int const radius_neg = 0 - radius_pos;

    // int const center_state = cel_states[cel_idx];
    // int neighbors_sum = 0; // includes cell itself
    // int neighbor_count = 0;

    // for (int v_ofs = radius_neg; v_ofs <= radius_pos; v_ofs++) {
    //     int v_neg = 0 - v_ofs;
    //     int u_z = max(radius_neg, v_neg - radius_pos);
    //     int u_m = min(radius_pos, v_neg + radius_pos);

    //     for (int u_ofs = u_z; u_ofs <= u_m; u_ofs++) {
    //         neighbors_sum += cel_state_3d_safe(slc_id_lyr, v_size, v_id, v_ofs, u_size, u_id, u_ofs, cel_states);
    //         neighbor_count += 1;
    //     }
    // }    

    // int const neighbor_avg = neighbors_sum / neighbor_count;    

    int const one_of_two = cel_idx & 1;
    // int const one_of_two = (cel_idx + v_id) & 1; // UNNECESSARY OFFSET FLIP - testing purposes and what not

    int const two_of_four = ((cel_idx & 3) >> 1) ^ one_of_two;

    //int const is_fractal_thingy = mul24(cel_idx, cel_idx + 1) & 1;

    // INTRODUCE A SLIGHT FRACTAL TO DISRUPT THE NATURAL TENDENCY TO FORM LINES
    int const is_off_cen_cel = one_of_two; // ^ is_fractal_thingy;
    int const is_large_cel = two_of_four;

    int const radius = is_large_cel ? INHIB_LARGE_CELL_RADIUS : INHIB_SMALL_CELL_RADIUS;

    int const neighbors_avg = get_neighbors_avg(slc_id_lyr, v_size, v_id, u_size, u_id, cel_states,
        cel_idx, radius);

    uchar cel_state = 0;

    if (is_off_cen_cel) {
        int cst = center_state + RETNAL_THRESHOLD;
        cel_state = mul24(neighbors_avg - cst, neighbors_avg > cst);
    } else {
        int nat = neighbors_avg + RETNAL_THRESHOLD;
        cel_state = mul24(center_state - nat, center_state > nat);
    }

    axn_states[axn_idx] = cel_state;    
}

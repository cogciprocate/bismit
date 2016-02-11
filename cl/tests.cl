// __kernel void test_safe_dim_ofs(
//             __global uint const* const dim_ids,
//             __global char const* const dim_offs,
//             __private uint const dim_size,
//             __global char* const safe_dim_offs)
// {
//     uint id = get_global_id(0);

//     //char safe_do = safe_dim_ofs(dim_size, dim_ids[id], dim_offs[id]);

//     safe_dim_offs[id] = safe_do;
// }


__kernel void test_axn_idxs_scl(
            __global char const* const v_offs,
            __global char const* const u_offs,
            // __global char const* const dim_offs,
            // __private uint const dim_size,
            __global uint* const outs_n)
            // __global uint* const outs_v4)
{
    uint const slc_id = get_global_id(0);
    uint const v_id = get_global_id(1);
    uint const u_id = get_global_id(2);
    uint const v_size = get_global_size(1);
    uint const u_size = get_global_size(2);

    uint const cel_idx = cel_idx_3d_unsafe(slc_id, v_size, v_id, u_size, u_id);

    uint const v_ofs = v_offs[cel_idx];
    uint const u_ofs = u_offs[cel_idx];


    // uint const u_size_axn = get_axn_u_size(slc_id);
    // uint const scaled_v_id = (mul24(v_id, get_axn_v_scale(slc_id)) >> 4) + v_ofs;
    // uint const scaled_u_id = (mul24(u_id, get_axn_u_scale(slc_id)) >> 4) + u_ofs;
    // uint const axn_idx_n = get_axn_slc_idz(slc_id) + mad24(scaled_v_id, u_size_axn, scaled_u_id);
    
    int idx_is_safe = 0;
    outs_n[cel_idx] = axn_idx_3d_unsafe(slc_id, v_id, v_ofs, u_id, u_ofs, &idx_is_safe);
    //outs_n[cel_idx] = u_id; //scaled_u_id; //mad24(scaled_v_id, axn_u_size, scaled_u_id);
}

__kernel void test_axn_idxs_vec4(
            __global char4 const* const v_offs,
            __global char4 const* const u_offs,
            // __global char const* const dim_offs,
            // __private uint const dim_size,
            // __global uint* const outs_n,
            __global int4* const outs_v4)
{
    uchar const slc_id_scl = get_global_id(0);
    uint const v_id_scl = get_global_id(1);
    uint const u_id_scl = get_global_id(2); // <- diminished dimension
    uint const v_size_scl = get_global_size(1);
    uint const u_size_scl = get_global_size(2);

    uint const cel_idx_scl = cel_idx_3d_unsafe(slc_id_scl, v_size_scl, v_id_scl, u_size_scl, u_id_scl);

    uchar4 const slc_id = (uchar4)slc_id_scl;
    int4 const v_id = (int4)(int)v_id_scl;

    int const u_idz = mul24(u_id_scl, (uint)4);
    int4 const u_id = (int4)(u_idz, u_idz + 1, u_idz + 2, u_idz + 3);
    //int4 const v_size = (int4)(int)v_size_scl;
    //int4 const u_size = (int4)(int)u_size_scl;
    //int4 const cel_idx = cel_idx_3d_unsafe_vec4(slc_id, v_size, v_id, u_size, u_id);

    char4 const v_ofs_char4 = v_offs[cel_idx_scl];
    char4 const u_ofs_char4 = u_offs[cel_idx_scl];

    // int4 const v_ofs = convert_int4(v_ofs_char4);
    // int4 const u_ofs = convert_int4(u_ofs_char4);
    // int4 const u_size = get_axn_u_size_vec4(slc_id);
    // int4 const scaled_v_id = (mul24(v_id, get_axn_v_scale_vec4(slc_id)) >> 4) + v_ofs;
    // int4 const scaled_u_id = (mul24(u_id, get_axn_u_scale_vec4(slc_id)) >> 4) + u_ofs;
    
    int4 idx_is_safe = (int4)0;
    outs_v4[cel_idx_scl] = axn_idx_3d_unsafe_vec4(slc_id, v_id, v_ofs_char4, u_id, u_ofs_char4, &idx_is_safe);
    //outs_v4[cel_idx_scl] = u_id; //scaled_u_id; // mad24(scaled_v_id, u_size, scaled_u_id);
}

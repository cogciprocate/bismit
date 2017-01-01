diff --git a/cl/bismit.cl b/cl/bismit.cl
index f373966..016df92 100644
--- a/cl/bismit.cl
+++ b/cl/bismit.cl
@@ -51,7 +51,7 @@
         {var_name}_l2i : A scalar representing the inverse log base 2 of a value (1 / log2 val).
 
         Coordinates: 
-        - slc_id : The id of a slice in axon space (or subset therof such as a
+        - slc_id : The id of a slice in axon space (or subset thereof such as a
           layer). This is the 'depth' coordinate corresponding to how far from
           the top of layer 0/1 we would be in a neocortex.
         - v_id : The 'v' coordinate of a tile (or in this case an axon, cell,
@@ -72,7 +72,7 @@
           from the other two when needed.
 
 
-        vat [tenative]: vatic, fuzzyness, level of predictiveness
+        vat [tentative]: vatic, fuzziness, level of predictiveness
 
         ***** High Priority Comment, Temporary Code Change
         <<<<< Medium Priority Comment, To Do
@@ -241,12 +241,12 @@ static inline uint calc_syn_idz(uint const tuft_id, uint const cel_count, uint c
 //     return mad24(cel_idx, tfts_per_cel, tft_id);
 // }
 
-// GET_CEL_TFT_IDX(): Uses same indexing principle as dendrites and synapses (see synapses.rs)
-static inline uint calc_cel_tft_idx(uint const cel_count, uint const cel_idx, 
-            uint const tfts_per_cel, uint const tft_id)
-{
-    return  mad24(tft_id, cel_count, cel_idx);
-}
+// // GET_CEL_TFT_IDX(): Uses same indexing principle as dendrites and synapses (see synapses.rs)
+// static inline uint calc_cel_tft_idx(uint const cel_count, uint const cel_idx, 
+//             uint const tfts_per_cel, uint const tft_id)
+// {
+//     return  mad24(tft_id, cel_count, cel_idx);
+// }
 
 // COORD_IS_SAFE(): Bounds check for a single dimension of a cellular coordinate
 static inline int coord_is_safe(int const dim_size, int const coord_id, int const coord_ofs) {
@@ -425,7 +425,7 @@ static inline uchar4 axn_state_3d_safe_vec4(uchar4 slc_id, int4 v_id, char4 v_of
 //         - Applies to a single dendrite on that cell.
 //             - Must only be called with a syn_idz of the best (most active)
 //               dendrite on an active tuft on an active cell.
-//         - Is intended to handle both crystalization (predictions becoming
+//         - Is intended to handle both crystallization (predictions becoming
 //           or staying true) or anomalies (situations where no cell in the
 //           column had predicted the column's spatial input).
 //
@@ -564,7 +564,7 @@ static inline void tft_syns_trm(
         // If synapse had STPOT flag and is now inactive (synapse correlated with cell activity):
         syn_strength += mul24(syn_has_stpot && !syn_is_active, should_inc);
 
-        // If synapse had STPOT flag and is still active (synapse did not correllate with cell activity):
+        // If synapse had STPOT flag and is still active (synapse did not correlate with cell activity):
         syn_strength -= mul24(syn_has_stpot && syn_is_active, should_dec);
 
         // If synapse had STDEP flag:
@@ -652,7 +652,7 @@ __kernel void reference_all_the_things(__private int const for_sanitys_sake) {
 
 
 // DEN_CYCLE():
-__kernel void den_cycle(
+__kernel void den_cycle_DEPRICATE(
             __global uchar const* const syn_states,
             __global char const* const syn_strengths,
             __private uchar const syns_per_den_l2,
@@ -701,6 +701,59 @@ __kernel void den_cycle(
 }
 
 
+
+// Cycles dendrites.
+__kernel void den_cycle_tft(
+            __global uchar const* const syn_states,
+            __global char const* const syn_strengths,
+            __private uint const tft_den_idz,
+            __private uint const tft_syn_idz,
+            __private uchar const syns_per_den_l2,
+            __private uint const den_threshold,
+            __global uchar* const den_energies,
+            __global uchar* const den_states_raw,
+            //__global int* const aux_ints_1,
+            __global uchar* const den_states) 
+{
+    uint const den_idx = get_global_id(0) + tft_den_idz;
+    uint const syn_idz = den_idx << syns_per_den_l2;
+
+    // uchar den_energy = den_energies[den_idx];
+
+    int syn_sum = 0;
+    int syn_sum_raw = 0;
+
+    int const syn_idn = (1 << syns_per_den_l2);
+
+    for (int syn_id = 0; syn_id < syn_idn; syn_id += 1) {
+        char syn_strength = syn_strengths[syn_idz + syn_id];
+        uchar syn_state = syn_states[syn_idz + syn_id]; 
+        syn_sum = mad24((syn_strength >= 0), syn_state, syn_sum); 
+        syn_sum_raw += syn_state;
+    }
+    
+    syn_sum = mul24((syn_sum > den_threshold), syn_sum);
+
+    // if (syn_sum != 0) {
+    //     if (den_energy >= ENERGY_LEVEL_MIN) {
+    //         den_energy -= ENERGY_LEVEL_MIN;
+    //     } else {
+    //         den_energy += ENERGY_REGEN_AMOUNT;
+    //         syn_sum = 0;
+    //     }
+    // } else {
+    //     if (den_energy < ENERGY_LEVEL_MAX) {
+    //         den_energy += ENERGY_REGEN_AMOUNT;
+    //     }
+    // }
+
+    int den_reduction = syns_per_den_l2 - 1;
+
+    den_states_raw[den_idx] = clamp((syn_sum_raw >> den_reduction), 0, 255); 
+    den_states[den_idx] = clamp((syn_sum >> den_reduction), 0, 255); 
+}
+
+
 //     INHIB_SIMPLE(): [DESCRIPTION OUT OF DATE] Cell Inhibition - reads from soma, writes to axon
 //        - If any nearby cells are more active (have a higher soma 'state')
 //            - cell will not 'fire'
@@ -971,7 +1024,7 @@ __kernel void mcol_activate_pyrs(
             __private uint const ssts_axn_idz,         // Primary spatial associative cell layer (ssts)
             __private uint const pyrs_axn_idz,          // Primary temporal associative cell layer (pyrs)
             // __private uchar const pyr_axn_slc_base,
-            __private uchar const dens_per_tft_l2,
+            // __private uchar const dens_per_tft_l2,
             __global uchar* const pyr_flag_sets,
             __global uchar* const pyr_states,
             __global int* const aux_ints_0,
@@ -1054,7 +1107,7 @@ __kernel void mcol_activate_pyrs(
 // First, to clarify:
 //     - The term 'vatic' is meant to mean predictive or in a state of
 //       expectance. May be referred to (incorrectly) as fuzzy or 'fuz'. A
-//       few other (depricated) terms might be thrown around due to much
+//       few other (deprecated) terms might be thrown around due to much
 //       renaming but basically if a pyramidal soma is active, that cell is
 //       vatic.
 //     - the term 'concrete' is meant to mean that the cell has an active
@@ -1139,7 +1192,7 @@ __kernel void mcol_activate_pyrs(
 
 //             - Synapse indexes will first need to be calculated using the
 //               dendrite index, i.e.: syn_id_tuft := den_idx * syns_per_den
-//               (or equivalantly: den_idx << syns_per_den_l2).
+//               (or equivalently: den_idx << syns_per_den_l2).
 //             - Next they will be added to the tuft offset:
 //                 - syns_per_tft_space := syns_per_den * dens_per_tft * cels_per_col * col_count
 //                     - (note that tufts per cell is not factored in here)
@@ -1217,17 +1270,22 @@ __kernel void mcol_activate_pyrs(
 //        cells into each work item, all threads can keep busy.
 
 
-// <<<<< TODO: FIX: NOT TAKING IN TO ACCOUNT MULTIPLE TUFTS! MAJOR INDEXING PROBLEMS >>>>>
-__kernel void pyrs_ltp(
+__kernel void pyr_tft_ltp(
             __global uchar const* const axn_states,
             __global uchar const* const cel_states,
-            __global uchar const* const cel_tft_best_den_ids,
-            __global uchar const* const cel_tft_best_den_states,
+            __global uchar const* const tft_cel_best_den_ids,
+            __global uchar const* const tft_cel_best_den_states,
             __global uchar const* const den_states,
             __global uchar const* const syn_states,
-            __private uint const tfts_per_cel,
+
+            __private uint const tft_cel_idz, // 0th tuft-cell index
+            __private uint const tft_den_idz, // 0th tuft-dendrite index
+            __private uint const tft_syn_idz, // 0th tuft-synapse index
+
+            // __private uint const tfts_per_cel,
             __private uint const dens_per_tft_l2,
             __private uint const syns_per_den_l2,
+            __private uint const syns_per_tft_l2,
             __private uint const cels_per_cel_grp,
             __private uint const axn_idz_cel_lyr,
             __private int const learning_rate_l2i,
@@ -1241,21 +1299,18 @@ __kernel void pyrs_ltp(
     uint const cel_grp_id = get_global_id(0);
     uint const cel_grp_count = get_global_size(0);
     uint const cel_count = mul24(cel_grp_count, cels_per_cel_grp);
+    // Index of the 0th cell in the cell group:
     uint const cel_idz_cel_grp = mul24(cel_grp_id, cels_per_cel_grp);
-
-    // TODO: MOVE THIS INVERSE LEARNING RATE TO HOST:
-    // int const learning_rate_l2i = 0;
-
-    // aux_ints_1[cel_grp_id] = -1 - (rnd_mix(rnd, cel_grp_id) & 0x7F);
-    // aux_ints_1[cel_grp_id] = rnd_mix(rnd, cel_grp_id);
+    // uint const syns_per_tft_l2 = dens_per_tft_l2 + syns_per_den_l2;
+    // uint const cel_idz_cel_grp = mad24(cel_grp_id, cels_per_cel_grp, tft_cel_idz);
  
-     // TODO: (EVALUATE) Make 'cels_per_cel_grp' and 'tfts_per_cel' a constant and unroll loops.
-     //    - Will mean making a separate program for each layer of pyramidals.
-     //    - Could do more harm than good due to program size bloat.
-     //    - Possibly do this for tfts only.
     for (uint cel_id_cel_grp = 0; cel_id_cel_grp < cels_per_cel_grp; cel_id_cel_grp++) {
+        // Current cell:
         uint const cel_idx = cel_idz_cel_grp + cel_id_cel_grp;
+        // Current cell's axon:
         uint const cel_axn_idx = axn_idz_cel_lyr + cel_idx;
+        // Current tuft-cell:
+        uint const tft_cel_idx = cel_idx + tft_cel_idz;
 
         uchar cel_flag_set = cel_flag_sets[cel_idx];
 
@@ -1264,60 +1319,58 @@ __kernel void pyrs_ltp(
         int const cel_prev_concrete = (cel_flag_set & (CEL_PREV_CONCRETE_FLAG)) == (CEL_PREV_CONCRETE_FLAG);
         int const cel_prev_vatic = (cel_flag_set & (CEL_PREV_VATIC_FLAG)) == (CEL_PREV_VATIC_FLAG);
         int const cel_best_in_col = (cel_flag_set & (CEL_BEST_IN_COL_FLAG)) == (CEL_BEST_IN_COL_FLAG);
+        int const tft_is_active = tft_cel_best_den_states[tft_cel_idx] != 0;
 
-        for (uint tft_id = 0; tft_id < tfts_per_cel; tft_id++) {
-            // uint const cel_tft_idx = mad24(cel_idx, tfts_per_cel, tft_id);
-            uint const cel_tft_idx = calc_cel_tft_idx(cel_count, cel_idx, tfts_per_cel, tft_id);
-            uint const den_idz_tft = cel_tft_idx << dens_per_tft_l2;
-
-            uchar const den_id_tft_best = cel_tft_best_den_ids[cel_tft_idx];            
-
-            uint const syn_idz_tft = den_idz_tft << syns_per_den_l2;
-            uint const syn_idz_best_den_tft = (den_idz_tft + den_id_tft_best) << syns_per_den_l2;
-
-            int const tuft_is_active = cel_tft_best_den_states[cel_tft_idx] != 0;
-
-            if (cel_is_concrete) {                
+        // uint const cel_tft_idx = mad24(cel_idx, tfts_per_cel, tft_id);
+        // uint const cel_tft_idx = calc_cel_tft_idx(cel_count, cel_idx, tfts_per_cel, tft_id);
 
-                if (tuft_is_active) {
-                    // aux_ints_0[cel_tft_idx] = cel_prev_vatic;
-                    // aux_ints_1[cel_tft_idx] = cel_best_in_col;
+        // // Index of the 0th dendrite within the current tuft-cell:
+        // uint const den_idz_tft = (cel_idx << dens_per_tft_l2) + tft_den_idz;
 
+        // Index of the 0th synapse within the current tuft-cell:
+        uint const syn_idz_tft = (cel_idx << syns_per_tft_l2) + tft_syn_idz;        
 
-                    // PREVIOUS (CORRECT) PREDICTION (EVERY PYR IN COL): REINFORCE DEN
-                    // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
-                    if (cel_prev_vatic | cel_best_in_col) { 
-                        // aux_ints_1[cel_tft_idx] = 10;                    
-                        dst_syns__active__stpot_stdep(syn_states, syn_idz_best_den_tft, syns_per_den_l2, rnd, 
-                            syn_flag_sets, syn_strengths);
+        if (cel_is_concrete) {
+            if (tft_is_active) {
+                // ID of the Best dendrite within the current tuft-cell:
+                uchar const best_den_id_celtft = tft_cel_best_den_ids[tft_cel_idx];
 
-                        // aux_ints_1[cel_tft_idx] = 11;
-                    }
+                // uint const syn_idz_best_den_tft = (den_idz_tft + best_den_id_celtft) << syns_per_den_l2;
+                uint const syn_idz_best_den_tft = (best_den_id_celtft << syns_per_den_l2) + syn_idz_tft;
 
-                    // } else if (cel_best_in_col) { 
-                    // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN 
-                    // //} else { 
-                    // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
-                    //     dst_syns__active__stpot_ltd(syn_states, syn_idz_best_den_tft, syns_per_den_l2, rnd, 
-                    //         syn_flag_sets, syn_strengths);
+                // PREVIOUS (CORRECT) PREDICTION (EVERY PYR IN COL): REINFORCE DEN
+                // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
+                if (cel_prev_vatic | cel_best_in_col) { 
+                    // aux_ints_1[cel_tft_idx] = 10;                    
+                    dst_syns__active__stpot_stdep(syn_states, syn_idz_best_den_tft, 
+                        syns_per_den_l2, rnd, syn_flag_sets, syn_strengths);
 
-                    //     // aux_ints_1[cel_tft_idx] = 12;
-                    // }
+                    // aux_ints_1[cel_tft_idx] = 11;
                 }
 
-                // TODO: Could be moved into above if block
-                cel_flag_set |= CEL_PREV_CONCRETE_FLAG;
+                // } else if (cel_best_in_col) { 
+                // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN 
+                // //} else { 
+                // ANOMALY (NO PREVIOUS PREDICTION, BEST PYR IN COLUMN ONLY): TRAIN NEW DEN
+                //     dst_syns__active__stpot_ltd(syn_states, syn_idz_best_den_tft, syns_per_den_l2, rnd, 
+                //         syn_flag_sets, syn_strengths);
 
-            } else if (cel_prev_concrete) {
-                tft_syns_trm(syn_states, syn_idz_tft, syns_per_den_l2 + dens_per_tft_l2, rnd, 
-                    learning_rate_l2i, syn_flag_sets, aux_ints_0, syn_strengths);
+                //     // aux_ints_1[cel_tft_idx] = 12;
+                // }
+            }
 
-                // aux_ints_1[cel_tft_idx] = 20;
+            // TODO: Could be moved into above if block
+            cel_flag_set |= CEL_PREV_CONCRETE_FLAG;
+        } else if (cel_prev_concrete) {
+            tft_syns_trm(syn_states, syn_idz_tft, syns_per_tft_l2, rnd, 
+                learning_rate_l2i, syn_flag_sets, aux_ints_0, syn_strengths);
 
-                cel_flag_set &= ~CEL_PREV_CONCRETE_FLAG;
-            }
+            // aux_ints_1[cel_tft_idx] = 20;
+
+            cel_flag_set &= ~CEL_PREV_CONCRETE_FLAG;
         }
 
+
         cel_flag_set &= ~CEL_PREV_VATIC_FLAG;
         cel_flag_set |= mul24(cel_is_vatic, CEL_PREV_VATIC_FLAG);
 
@@ -1326,94 +1379,146 @@ __kernel void pyrs_ltp(
 }
 
 
-
-// TODO: Investigate the bizarre glitch with this kernel with the phantom best_dens
-__kernel void pyr_cycle(
+__kernel void pyr_tft_cycle(
             __global uchar const* const den_states_raw, // USE THIS TO DETERMINE BEST DEN
-            __global uchar const* const den_states,                
-            __private uint const tfts_per_cel,
-            __private uchar const dens_per_tft_l2,                
-            __global uchar* const cel_tft_best_den_ids,
-            __global uchar* const cel_tft_best_den_states,
-            __global uchar* const pyr_best_den_states,
+            __global uchar const* const den_states,
+            // __private uint const tfts_per_cel,
+            // __private uint const tft_id,
+            ////// Could be made GWO (and could be calculated from tuft_id as well):            
+            __private uint const lyrtft_cel_idz, 
+            __private uint const lyrtft_den_idz,
+            __private uchar const dens_per_tft_l2,
+            __global uchar* const celtft_best_den_ids,
+            __global uchar* const celtft_best_den_states_raw,
+            __global uchar* const celtft_best_den_states,
+            // __global uchar* const pyr_best_den_states,
             __global int* const aux_ints_0,
-            __global int* const aux_ints_1,
-            __global uchar* const pyr_states) 
+            __global int* const aux_ints_1)
+            // __global uchar* const pyr_states) 
 {
-    uint const cel_idx = get_global_id(0);
-    uint const cel_count = get_global_size(0);
+    uint const cel_id_lyrtft = get_global_id(0);
+    uint const lyrtft_cel_count = get_global_size(0);
 
-    uint pyr_state = 0;
-    uint pyr_best_den_state = 0;
+    // uint pyr_state = 0;
+    // uint pyr_best_den_state = 0;
 
-    for (uint tft_id = 0; tft_id < tfts_per_cel; tft_id++) {
-        uint const cel_tft_idx = calc_cel_tft_idx(cel_count, cel_idx, tfts_per_cel, tft_id);
-        uint const den_idz_tft = cel_tft_idx << dens_per_tft_l2;
-        // uint const cel_tft_idx = mad24(cel_idx, tfts_per_cel, tft_id);
+    uint const den_id_lyrtft = cel_id_lyrtft << dens_per_tft_l2;
 
-        // Used for activation and ultimately learning:
-        uint best_den_id = 0;
-        // Used for activation and ultimately learning:
-        uint best_den_state_raw = 0;
-        // Used for pyr_state and pyr_best_den_state:
-        uint best_den_state = 0;
- 
-        for (uint den_id_tft = 0; den_id_tft < (1 << dens_per_tft_l2); den_id_tft++) {
-            uint const den_idx = den_idz_tft + den_id_tft;
+    // for (uint tft_id = 0; tft_id < tfts_per_cel; tft_id++) {
 
-            uint const den_state_raw = den_states_raw[den_idx];
-            uint const den_state = den_states[den_idx];
+    // uint const cel_tft_idx = calc_cel_tft_idx(cel_count, cel_idx, tfts_per_cel, tft_id);
+    // uint const lyrtft_cel_idx = lyrtft_cel_idz + cel_id_lyrtft;
+    // uint const tft_den_idz = den;
+    // uint const cel_tft_idx = mad24(cel_idx, tfts_per_cel, tft_id);
+    uint const cel_den_idz  = lyrtft_den_idz + den_id_lyrtft;
 
-            int const den_state_bigger_raw = (den_state_raw > best_den_state_raw);
-            int const den_state_bigger = (den_state > best_den_state);
+    // Used for activation and ultimately learning:
+    uint best_den_id = 0;
+    // Used for activation and ultimately learning:
+    uint best_den_state_raw = 0;
+    // Used for pyr_state and pyr_best_den_state:
+    uint best_den_state = 0;
 
-            best_den_id = mad24((uint)den_state_bigger_raw, den_id_tft,
-                mul24((uint)!den_state_bigger_raw, best_den_id));
+    for (uint den_id_celtft = 0; den_id_celtft < (1 << dens_per_tft_l2); den_id_celtft++) {
+        uint const den_idx = cel_den_idz + den_id_celtft;
 
-            best_den_state_raw = mad24((uint)den_state_bigger_raw, den_state_raw, 
-                mul24((uint)!den_state_bigger_raw, best_den_state_raw));            
+        uint const den_state_raw = den_states_raw[den_idx];
+        uint const den_state = den_states[den_idx];
 
-            best_den_state = mad24((uint)den_state_bigger, den_state, 
-                mul24((uint)!den_state_bigger, best_den_state));
+        int const den_state_bigger_raw = (den_state_raw > best_den_state_raw);
+        int const den_state_bigger = (den_state > best_den_state);
 
-            // if (den_state > 0) {
-            //     aux_ints_0[den_idx] = cel_idx;
-            // }
-        }
+        best_den_id = mad24((uint)den_state_bigger_raw, den_id_celtft,
+            mul24((uint)!den_state_bigger_raw, best_den_id));
 
-        cel_tft_best_den_ids[cel_tft_idx] = best_den_id;
-        cel_tft_best_den_states[cel_tft_idx] = best_den_state_raw;
+        best_den_state_raw = mad24((uint)den_state_bigger_raw, den_state_raw, 
+            mul24((uint)!den_state_bigger_raw, best_den_state_raw));            
 
-        pyr_best_den_state = max(pyr_best_den_state, best_den_state_raw);
+        best_den_state = mad24((uint)den_state_bigger, den_state, 
+            mul24((uint)!den_state_bigger, best_den_state));
 
-        // TODO: Might need a more sophisticated algorithm with a non-linear rate to determine pyr_state:
-        pyr_state = max(pyr_state, best_den_state);
-        // pyr_state += best_den_state;
+        ////// DEBUG: 
+            // if (den_state > 0 || den_state_raw > 0) {
+            //     // aux_ints_0[den_idx] = tft_cel_idx;
+            //     aux_ints_0[den_idx] = 99;
+            // }
 
-        // if (best_den_state > 0) {
-        //     aux_ints_0[cel_tft_idx] = pyr_state;
-        // }
+            // if (den_id_celtft == 0) {
+            //     aux_ints_0[den_idx] = 99;
+            // }
+
+            // aux_ints_0[den_idx] = 99;
+
+        //////
     }
 
-    pyr_best_den_states[cel_idx] = pyr_best_den_state;
-    
-    // TODO: pyr_state: (see above)
-    pyr_states[cel_idx] = clamp(pyr_state, (uint)0, (uint)255);
+    ////// DEBUG:
+        // aux_ints_0[1000000] = 99;
+    //////
+
+    // uint const celtft_idx = mad24(tft_id, lyrtft_cel_count, cel_id_lyrtft);
+    uint const celtft_idx = lyrtft_cel_idz + cel_id_lyrtft;
+
+    celtft_best_den_ids[celtft_idx] = best_den_id;
+    celtft_best_den_states_raw[celtft_idx] = best_den_state_raw;
+    celtft_best_den_states[celtft_idx] = best_den_state;
 
-    // WTF?
-    // uchar pyr_state_bizarre = pyr_states[cel_idx];
+    // uint pyr_best_den_state = max(pyr_best_den_state, best_den_state_raw);
 
-    // // aux_ints_1[cel_idx] = pyr_states[cel_idx];
+    // [TODO][EDIT: Probably not]: Might need a more sophisticated algorithm
+    // with a non-linear rate to determine pyr_state:
+    // uint pyr_state = max(pyr_state, best_den_state);
+    // pyr_state += best_den_state;
 
-    // if (pyr_state != 0) {
-    //     aux_ints_0[cel_idx] = pyr_states[cel_idx];
-    //     // aux_ints_1[cel_idx] = pyr_states[cel_idx];
+    // if (best_den_state > 0) {
+    //     aux_ints_0[cel_tft_idx] = pyr_state;
     // }
 
-    // if (pyr_state_crazy != 0) {
-    //     aux_ints_1[cel_idx] = pyr_states[cel_idx];
     // }
 
+    // pyr_best_den_states[cel_idx] = pyr_best_den_state;
+    
+    // // TODO: pyr_state: (see above)
+    // pyr_states[cel_idx] = clamp(pyr_state, (uint)0, (uint)255);
+}
+
+
+// PYR_CYCLE(): Cycles every pyramidal cell in a layer.
+//
+__kernel void pyr_cycle(
+            __global uchar const* const celtft_best_den_ids,
+            __global uchar const* const celtft_best_den_states_raw,
+            __global uchar const* const celtft_best_den_states,
+            __private uint const tft_count,
+            __global uchar* const pyr_states,
+            __global int* const aux_ints_0,
+            __global int* const aux_ints_1)
+{
+    uint const cel_idx = get_global_id(0);
+    uint const cel_count = get_global_size(0);
+    // uint const celtft_idx = mul24(cel_idx, tft_count);
+
+    uchar pyr_state = 0;
+
+    for (uint tft_id = 0; tft_id < tft_count; tft_id++) {
+        uint const celtft_idx = mad24(tft_id, cel_count, cel_idx);
+
+        uchar pyr_best_den_state_raw = celtft_best_den_states_raw[celtft_idx];
+        uchar pyr_best_den_state = celtft_best_den_states[celtft_idx];
+        // pyr_states[cel_idx] = max(pyr_best_den_state, pyr_best_den_state_raw);
+        pyr_state = max(pyr_state, pyr_best_den_state_raw);
+        pyr_state = max(pyr_state, pyr_best_den_state);
+
+        // if (cel_idx == 199) {
+        //     aux_ints_0[celtft_idx] = 199;
+        // }
+
+        // aux_ints_0[celtft_idx] = pyr_best_den_state;
+        // aux_ints_1[celtft_idx] = pyr_state;
+    }
+
+    // aux_ints_1[cel_idx] = pyr_state;
+    pyr_states[cel_idx] = pyr_state;
 }
 
 
@@ -1423,7 +1528,7 @@ __kernel void pyr_cycle(
 __kernel void mcol_output(
             __global uchar const* const pyr_states,                
             // __global uchar const* const cel_tft_best_den_states,
-            __private uint const tfts_per_cel,
+            // __private uint const tfts_per_cel,
             __private uint const sst_axn_idz,
             __private uchar const pyr_depth,
             __private uchar const aff_out_axn_slc,

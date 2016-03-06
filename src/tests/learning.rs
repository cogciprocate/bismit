#![allow(non_snake_case, unused_imports)]
use std::ops::{Range};
// use std::iter;
// use std::io::{ Write };
// use std::mem;
// use rand;

// use ocl::{BufferTest, OclPrm};
use ocl::traits::OclPrm;
use proto::{layer};
use cortical_area::{CorticalArea, CorticalAreaTest};
use map;
use synapses::{SynapsesTest, SynCoords};
use dendrites::{DendritesTest};
use axon_space::{AxonSpaceTest};
use cortex::{Cortex};
use cmn::{self, CelCoords, DataCellLayer, DataCellLayerTest};
use super::{util, testbed};
use super::util::{NONE, ACTIVATE, LEARN, CYCLE, OUTPUT, ALL};

// const LEARNING_TEST_ITERATIONS: usize = 5; //50;
// const LEARNING_ITERS_PER_CELL: usize = 2;
const LEARNING_CONTINUATION_ITERS: usize = 3;

const PRINT_DEBUG_INFO: bool = false;
const PRINT_FINAL_ITER_ONLY: bool = true;

//=============================================================================
//=============================================================================
//================================== TESTS ====================================
//=============================================================================
//=============================================================================




// TEST_DST_DEN_LEARNING(): 
// Set up conditions in which a synapse should learn and its neighbors should not.
/*             
        Choose a pyramidal cell.
        Activate the column axon (mcol/sst) for that cell's column.

        Choose a synapse from that cell.        
        
            Check for safety/validity.    
            Confirm that the pyramidal cell has the correct flags (none):
                CEL_PREV_CONCRETE_FLAG: u8         = 0b10000000;    // 128    (0x80)
                CEL_BEST_IN_COL_FLAG: u8         = 0b01000000;    // 64    (0x40)
                CEL_PREV_STP_FLAG: u8             = 0b00100000;    // 32    (0x20)
                CEL_PREV_FUZZY_FLAG: u8            = 0b00010000;    // 16    (0x10)
            Determine whether or not the synapse has the correct flags (none):
                SYN_STP_FLAG: u8                = 0b00000001;
                SYN_STD_FLAG: u8                = 0b00000010;
                SYN_CONCRETE_FLAG: u8            = 0b00001000;
            Check flags on other synapses (should be none).

        Set synapse strengths to zero for entire cell.
        Find its source pyr axon index.
        Activate that pyr axon (to 196).

        Cycle.
            Check:
                Pyr should have no flags set.
                    - It should have CONCRETE flag if more than den thresh syns have been activated.
                Syn should have no flags set.
                Other syns on cell should have no flags or value
                Value should be non-zero (if src pyr axn was 196, syn should be 226).
                Values of other synapses should be zero.
        Learn.
            Check. 
                Value should be unchanged. 
                Pyr should have CONCRETE flag only.
                Syn should have STP & CONCRETE, others should have nothing.
        
        Verify that nearby synapses have undergone LTD.
        Deactivate column and nearby pyr axon.

        Cycle.
            Check.
        Learn.
            Check. 


    NOTES:
        - It is assumed that all axons, dendrites, and synapses for our cortical area are completely zeroed.
        - 'slc' and 'slc_id' are synonymous
        - unused_slc_id is actually unused! Remove?
*/

#[test]
fn dst_den_learning() {
    let mut ltb = LearningTestBed::new();
    // 180 -> +-64 (slow), +-96 (fast)
    // 360 -> +-96 (slow), +-119 (fast)
    let on_focus_iters = 360;
    let off_focus_iters = 360;
    ltb.test_on_off(on_focus_iters, off_focus_iters);

    // let on_focus_iters = 180;
    // let off_focus_iters = 180;
    // ltb.test_on_off(on_focus_iters, off_focus_iters);

    // print!("\n");
    // panic!(" -- DEBUGGING -- ");
}



pub struct LearningTestBed {
    unused_slc_id: u8,
    prx_src_slc: u8, 
    fake_neighbor_slc: u8,         
    aff_out_slc: u8, 

    prx_src_axn_idx: u32,
    fake_neighbor_axn_idx: u32, 
    aff_out_axn_idx: u32, 
    cel_axn_idx: u32,

    fake_v_ofs: i8,
    fake_u_ofs: i8,

    syn_coords: SynCoords,
    focus_syns: Range<usize>,
    off_focus_syns: Range<usize>,

    cortex: Cortex,
}

impl LearningTestBed {
    fn new() -> LearningTestBed {
        let mut cortex = testbed::cortex_with_lots_of_apical_tufts();

        let (unused_slc_id,
            prx_src_slc, 
            fake_neighbor_slc,         
            aff_out_slc,
            prx_src_axn_idx,
            fake_neighbor_axn_idx, 
            aff_out_axn_idx, 
            cel_axn_idx,
            fake_v_ofs,
            fake_u_ofs,
            syn_coords,
            focus_syns,
            off_focus_syns) = {

            let mut area = cortex.area_mut(testbed::PRIMARY_AREA_NAME);

            // Zero all dendrite and synapse buffers:
            area.ptal_mut().dens_mut().set_all_to_zero(true);
            area.axns.states.set_all_to(0).unwrap();

            // Set source slice to an unused slice for all synapses:
            let unused_slc_ids = area.area_map().axn_base_slc_ids_by_tags(map::UNUSED_TESTING);
            assert!(unused_slc_ids.len() >= 3, "Make sure at least three axon layers have the UNUSED_TESTING flag.");
            let unused_slc_id = unused_slc_ids[0];
            area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_all_to(unused_slc_id).unwrap();

            // Primary spatial layer slice idz (base axon slice):
            let prx_src_slc = area.psal().base_axn_slc();

            // Fake neighbor slice:
            let fake_neighbor_slc = unused_slc_ids[1];

            // DEBUG: Print slice map and synapse dims:
            println!("\nDEBUG INFO: \n{mt}{}, \n{mt}synapse dims: {:?}",
                area.area_map(), area.ptal().dens().syns().dims(), mt = cmn::MT);

            // Afferent output slice id:
            let aff_out_slcs = area.area_map().axn_base_slc_ids_by_tags(map::FF_OUT);
            assert!(aff_out_slcs.len() == 1);
            let aff_out_slc = aff_out_slcs[0];

            // Get a random cell and a random synapse on that cell:
            let cel_coords = area.ptal_mut().rand_cel_coords();
            let syn_coords = area.ptal_mut().dens_mut().syns_mut()
                .rand_syn_coords(&cel_coords);

            // Base slice for primary temporal pyramidals of our layer:
            let ptal_axn_slc_idz = area.ptal_mut().base_axn_slc();
            // assert!(ptal_axn_slc_idz == cel_coords.slc_id_lyr, "cel_coords axon slice mismatch");
            assert_eq!(ptal_axn_slc_idz, cel_coords.axn_slc_id - cel_coords.slc_id_lyr);

            // Our cell's proximal source axon (the column spatial axon):
            let prx_src_axn_idx = area.area_map().axn_idz(prx_src_slc) + cel_coords.col_id();

            // Our cell's axon:
            let cel_axn_idx = cel_coords.cel_axn_idx(area.area_map());

            // Our cell's COLUMN output axon:
            let aff_out_axn_idx = area.area_map().axn_idz(aff_out_slc) + cel_coords.col_id();

            // A random, nearby axon for our cell to use as a distal source axon:
            let (fake_v_ofs, fake_u_ofs, fn_col_id, fake_neighbor_axn_idx) = 
                area.rand_safe_src_axn(&cel_coords, fake_neighbor_slc);

            //================================ SYN RANGE ==================================
            // A random dendrite id on the cell tuft:
            let syn_idx_range_den = syn_coords.syn_idx_range_den();
            // The synapse range for the entire tuft in which our random synapse resides:
            let syn_idx_range_tft = syn_coords.syn_idx_range_tft();

            // The first half of the synapses on our tuft:
            // let syn_idx_range_tft_first_half = syn_idx_range_tft.start..(syn_idx_range_tft.start + syn_idx_range_tft.len() / 2);

            // The second half of the synapses on our tuft:
            // let syn_idx_range_tft_second_half = (syn_idx_range_tft.start + syn_idx_range_tft.len() / 2)
            //     ..(syn_idx_range_tft.start + syn_idx_range_tft.len());

            // The first half of the synapses on our tuft:
            let syn_idx_range_den_first_half = syn_idx_range_den.start..(syn_idx_range_den.start 
                + syn_idx_range_den.len() / 2);

            // The second half of the synapses on our tuft:
            let syn_idx_range_den_second_half = (syn_idx_range_den.start + syn_idx_range_den.len() / 2)
                ..(syn_idx_range_den.start + syn_idx_range_den.len());

            let focus_syns = syn_idx_range_den_second_half;
            let off_focus_syns = syn_idx_range_den_first_half;
                    

            // The synapse count for our cell's entire layer (all slices, cells, and tufts):
            let syn_range_all = 0..area.ptal_mut().dens_mut().syns_mut().states.len();

            // Set the sources for the synapses on the second half of our chosen tuft to our preselected nearby axon:
            // <<<<< [FIXME] TODO: IMPLEMENT THIS (for efficiency): >>>>>
            //         area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(unused_slc_id, den_syn_range).unwrap();

            for syn_idx in focus_syns.clone() {
                area.ptal_mut().dens_mut().syns_mut().set_src_offs(fake_v_ofs, fake_u_ofs, syn_idx as usize);
                area.ptal_mut().dens_mut().syns_mut().set_src_slc(fake_neighbor_slc, syn_idx as usize);
            }

            // PRINT ALL THE THINGS!:
            let syn_val = area.ptal_mut().dens_mut().syns_mut().syn_state(syn_coords.idx);
            let fake_neighbor_axn_val = area.axn_state(fake_neighbor_axn_idx as usize);
            println!("DEBUG INFO - PRINT ALL THE THINGS!: \n\
                {mt}[prx_src]: prx_src_axn_idx (prx_src_slc: {}, col_id: {}): {} \n\
                {mt}[dst_src]: fake_neighbor_axn_idx (''_slc: {}, col_id: {}): {}, \n\
                {mt}[cel_axn]: cel_axn_idx (cel_coords.axn_slc_id: {}, col_id: {}): {} \n\
                {mt}[col_out]: aff_out_axn_idx (aff_out_slc: {}, col_id: {}): {}, \n\n\
                \
                {mt}fake_v_ofs: {}, fake_u_ofs: {}, \n\
                {mt}fake_neighbor_axn_val: {}, syn_val: {}, syn_idx_range_den: {:?}, syn_idx_range_tft: {:?}, \n\
                {mt}syn_active_range (2nd half): {:?}, \n\
                {mt}syn_coords: {}",        

                prx_src_slc, cel_coords.col_id(), prx_src_axn_idx,
                fake_neighbor_slc, fn_col_id, fake_neighbor_axn_idx, 
                cel_coords.axn_slc_id, cel_coords.col_id(), cel_axn_idx,
                aff_out_slc, cel_coords.col_id(), aff_out_axn_idx, 

                fake_v_ofs, fake_u_ofs,
                fake_neighbor_axn_val, syn_val, syn_idx_range_den, syn_idx_range_tft, 
                focus_syns, 
                syn_coords,
                mt = cmn::MT);
            
            // This and every other util::print_all() is very expensive:
            if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm Init - "); }

            (unused_slc_id,
            prx_src_slc, 
            fake_neighbor_slc,         
            aff_out_slc,
            prx_src_axn_idx,
            fake_neighbor_axn_idx, 
            aff_out_axn_idx, 
            cel_axn_idx,
            fake_v_ofs,
            fake_u_ofs,
            syn_coords,
            focus_syns,
            off_focus_syns,)
        };

        LearningTestBed {
            unused_slc_id: unused_slc_id,
            prx_src_slc: prx_src_slc, 
            fake_neighbor_slc: fake_neighbor_slc,         
            aff_out_slc: aff_out_slc, 

            prx_src_axn_idx: prx_src_axn_idx,
            fake_neighbor_axn_idx: fake_neighbor_axn_idx, 
            aff_out_axn_idx: aff_out_axn_idx, 
            cel_axn_idx: cel_axn_idx,

            fake_v_ofs: fake_v_ofs,
            fake_u_ofs: fake_u_ofs,

            syn_coords: syn_coords,
            focus_syns: focus_syns,
            off_focus_syns: off_focus_syns,

            cortex: cortex,
        }
    }


    // Run tests:
    /*
        Possible options and things to test:
            - Activating synapses on more than one cell or tuft at a time.
            - Activate other dendrites on a tuft to make sure best den is working correctly.
            - Randomizing or having irregular groupings of active synapses on a dendrite (using a list of ranges).
    
    */
    fn on_off(&mut self, on_focus_iters: usize, off_focus_iters: usize) {
        for i in 0..on_focus_iters {
            let final_iter = i == (on_focus_iters - 1);
            let print_debug = ((PRINT_FINAL_ITER_ONLY && final_iter) || !PRINT_FINAL_ITER_ONLY)
                && PRINT_DEBUG_INFO;
            self.learning_iter(i, false, print_debug);            
        }

        self.flip_focus_syns();        

        for i in 0..off_focus_iters {
            let final_iter = i == (off_focus_iters - 1);
            let print_debug = ((PRINT_FINAL_ITER_ONLY && final_iter) || !PRINT_FINAL_ITER_ONLY)
                && PRINT_DEBUG_INFO;
            self.learning_iter(i, true, print_debug);            
        }

        self.clean_up(true);
    }

    

    // LEARNING_ITER(): The great-grandmother of all tests
    // [FIXME] TODO: Add awareness of the current averages of both the on and off-focus synapses from the previous run in order to make sure that the synapse strengths are moving in the correct direction.
    fn learning_iter(&mut self, i: usize, flipped: bool, print_debug: bool) {
        let mut area = self.cortex.area_mut(testbed::PRIMARY_AREA_NAME);
        let syn_idx = self.syn_coords.idx();
        let den_idx = self.syn_coords.den_idx();
        let tft_idx = self.syn_coords.tft_idx();
        let cel_idx = self.syn_coords.cel_coords.idx();
        let col_id = self.syn_coords.cel_coords.col_id();        

        let flpd_str = if flipped { "FLP" } else { "ORG" };

        //=============================================================================
        //===================================== 0 =====================================
        //=============================================================================
        if print_debug {
            println!("\n     ==============================================================     ");
            println!("   ==================================================================   ");
            println!(" ====================== {}[{}] 0: Initialization ====================== ", flpd_str, i);
            println!("   ==================================================================   ");
            println!("     ==============================================================   ");
        }

        // Activate distal source axon:
        if print_debug { printlny!("Activating distal source (neighbor) axon: [{}]...", self.fake_neighbor_axn_idx); }
        area.activate_axon(self.fake_neighbor_axn_idx);

        util::ptal_alco(area, CYCLE | OUTPUT, print_debug);

        if print_debug { util::print_all(area, "\n - Confirm 0 - "); print!("\n");}

        assert!(util::eval_range(&area.ptal().dens().syns().states, self.focus_syns.clone(), 
            | x | x != 0), "Synapses in range '{}..{}' are not active.", 
            self.focus_syns.start, self.focus_syns.end );

        assert!(area.ptal().dens().states_raw.read_idx_direct(den_idx as usize) != 0, 
            "Dendrite '{}' is not active.", den_idx);
        // println!();
        assert!(area.ptal().best_den_states.read_idx_direct(cel_idx as usize) != 0);
        // println!("Cell best dendrite state is correct.");
        assert!(area.ptal().tft_best_den_ids.read_idx_direct(tft_idx as usize) as u32 
            == self.syn_coords.den_id_tft);
        assert!(area.ptal().tft_best_den_states.read_idx_direct(tft_idx as usize) != 0);

        // Evaluate minicolumn activity:
        // [FIXME] TODO: SHOULD ONLY BE ACTIVE WHEN STRS >= 0:
        // assert!(area.mcols().flag_sets.read_idx_direct(col_id as usize) == cmn::MCOL_IS_VATIC_FLAG,
        //         "Minicolumn is not vatic (predictive).");

        // Ensure key axons are active:
        assert!(area.read_from_axon(self.fake_neighbor_axn_idx) > 0, 
            "Pyramidal cell fake neighbor axon is not active.");

        // Verify that afferent output for our column is active:
        // [FIXME] TODO: SHOULD ONLY BE ACTIVE WHEN STRS >= 0:
        // assert!(area.read_from_axon(self.aff_out_axn_idx as u32) > 0);

        // CHECK CELL AXON (should be zero here and active on next step):        
        assert!(area.read_from_axon(self.cel_axn_idx as u32) == 0);

        // FLAGS: [pyr: 0], [syns: 0's], [mcol: 0];

        //=============================================================================
        //==================================== 1A =====================================
        //=============================================================================
        if print_debug {
            println!("\n ====================== {}[{}] 1: Premonition ====================== ", flpd_str, i);
            println!("       ====================== {}[{}] 1A ====================== \n", flpd_str, i);
        }

        util::ptal_alco(area, ACTIVATE, print_debug);

        if print_debug { 
            util::print_all(area, "\n - Confirm 1A - ");
            print!("\n");
        }        

        // Ensure our cell is flagged best in (mini) column:
        // [FIXME] TODO: REENABLE BELOW!
        assert!(area.ptal().flag_sets.read_idx_direct(self.syn_coords.cel_coords.idx() as usize) 
            & cmn::CEL_BEST_IN_COL_FLAG == cmn::CEL_BEST_IN_COL_FLAG);
        // println!("Our cell is correctly flagged best in column.");

        // FLAGS: [pyr: 64], [syns: 0's], [mcol: 1];

        //=============================================================================
        //============================= 1B ===================================
        //=============================================================================
        if print_debug { println!(
            "\n ====================== {}[{}] 1B ====================== \n", flpd_str, i); }

        util::ptal_alco(area, LEARN, print_debug);

        if print_debug { 
            util::print_all(area, "\n - Confirm 1B - "); 
            print!("\n");
        }        

        // <<<<< TODO: Ensure our cells synapses have not learned anything: >>>>>

        // FLAGS: [pyr: 80], [syns: 0's], [mcol: 1]; (pyr changed)

        //=============================================================================
        //=============================== 1C ===================================
        //=============================================================================
        if print_debug { println!("
            \n ====================== {}[{}] 1C ====================== \n", flpd_str, i); }


        util::ptal_alco(area, CYCLE | OUTPUT, print_debug);

        if print_debug { 
            util::print_all(area, "\n - Confirm 1C - ");
            print!("\n");
        }

        // FLAGS: [pyr: 80], [syns: 0's], [mcol: 1]; (unchanged)

        //=============================================================================
        //=============================== 2A ===================================
        //=============================================================================
        if print_debug { 
            println!("\n ====================== {}[{}] 2: Vindication ====================== ", flpd_str, i);
            println!("       ====================== {}[{}] 2A ====================== \n", flpd_str, i);
        }

        // ACTIVATE COLUMN PSAL AXON
        if print_debug {  printlny!("Activating proximal source axon: [{}]...", self.prx_src_axn_idx); }
        area.activate_axon(self.prx_src_axn_idx);

        // ACTIVATE PTAL SYNAPSE SOURCE AXON
        // printlny!("Activating distal source (neighbor) axon: [{}]...", fake_neighbor_axn_idx);
        // area.activate_axon(fake_neighbor_axn_idx);

        util::ptal_alco(area, ACTIVATE, print_debug);

        if print_debug { 
            util::print_all(area, "\n - Confirm 2A - ");
            print!("\n");
        }

        // ##### ADD ME: assert!(THE PYRAMIDAL OUTPUT AXON (NOT SOMA) IS ACTIVE)
        assert!(area.read_from_axon(self.cel_axn_idx as u32) > 0);

        // MOVED THIS FROM 1B -- PROBABLY WAS IN WRONG SPOT
        // printlny!("\nConfirming flag sets...");
        // assert!(util::assert_neq_range(&area.ptal().dens().syns().flag_sets, focus_syns, 0));

        // FLAGS: [pyr: 80], [syns: 0's], [mcol: 1]; (unchanged)

        //=============================================================================
        //=============================== 2B ===================================
        //=============================================================================
        if print_debug { println!(
            "\n ====================== {}[{}] 2B ====================== \n", flpd_str, i); }

        util::ptal_alco(area, LEARN, print_debug);

        if print_debug { 
            util::print_all(area, "\n - Confirm 2B - ");
            print!("\n");
        }

        // <<<<< [FIXME] TODO: assert!(chosen-half of syns are STPOT, others are STDEP) >>>>>

        // FLAGS: [pyr: 208], [syns: 1's & 2's], [mcol: 1]; (pyr and syns changed)

        //=============================================================================
        //=============================== 2C ===================================
        //=============================================================================
        if print_debug { println!(
            "\n ====================== {}[{}] 2C ====================== \n", flpd_str, i); }

        // util::ptal_alco(area, CYCLE | OUTPUT, print_debug);

        if print_debug { 
            util::print_all(area, "\n - Confirm 2C - ");
            print!("\n");
        }

        // FLAGS: [pyr: 208], [syns: 1's & 2's], [mcol: 1]; (unchanged)

        //=============================================================================
        //=============================== 3A ===================================
        //=============================================================================
        if print_debug { println!(
            "\n ====================== {}[{}] 3: Continuation ====================== ", flpd_str, i); }

        // ACTIVATE, LEARN, CYCLE, & OUTPUT multiple times without touching inputs:

        if print_debug {  printlny!("Performing complete cycle(A|L|C|O) {} times...", LEARNING_CONTINUATION_ITERS); }

        for _ in 0..LEARNING_CONTINUATION_ITERS {
            util::ptal_alco(area, ALL, false);
        }

        if print_debug { 
            util::print_all(area, "\n - Confirm 3 - "); 
            print!("\n");
        }

        // FLAGS: [pyr: 208], [syns: 1's & 2's], [mcol: 1]; (unchanged)

        //=============================================================================
        //=============================== 4 ===================================
        //=============================================================================
        if print_debug {  println!(
            "\n ====================== {}[{}] 4: Deactivation ====================== ", flpd_str, i); }

        // ZERO PTAL SYNAPSE SOURCE AXON
        if print_debug { printlny!("Deactivating distal source (neighbor) axon..."); } 
        area.deactivate_axon(self.fake_neighbor_axn_idx);

        util::ptal_alco(area, CYCLE, print_debug);
        // util::ptal_alco(area, ACTIVATE, print_debug);

        if print_debug { util::print_all(area, "\n - Confirm 3 - "); print!("\n");}

        assert!(util::eval_range(&area.ptal().dens().syns().flag_sets, self.focus_syns.clone(), 
            | x | x == cmn::SYN_STPOT_FLAG ));

        // [FIXME] TODO: Check that off-focus synapses are STDEP
        // [FIXME] TODO: Check that synapses are correctly inactive
        // [FIXME] TODO: Check that EVERYTHING ELSE besides pyr flag_sets is 0
        

        // FLAGS: [pyr: 208], [syns: 1's & 2's], [mcol: 1]; (unchanged)

        //=============================================================================
        //=============================== 5 ===================================
        //=============================================================================
        if print_debug { println!(
            "\n ====================== {}[{}] 5: Termination ====================== ", flpd_str, i); }

        // ZERO COLUMN PSAL AXON
        if print_debug {  printlny!("Deactivating proximal source axon..."); }
        area.deactivate_axon(self.prx_src_axn_idx);
        // ZERO PTAL SYNAPSE SOURCE AXON
        // printlny!("Deactivating distal source (neighbor) axon...");
        // area.deactivate_axon(fake_neighbor_axn_idx);

        // util::ptal_alco(area, ALL, print_debug);
        util::ptal_alco(area, ACTIVATE | LEARN | OUTPUT, print_debug);

        if print_debug { 
            util::print_all(area, "\n - Confirm 3 - "); 
            print!("\n");
        }

        // [FIXME] TODO: Need a more sophisticated test that tracks the current syn strengths:
        // assert!(util::eval_range(&area.ptal().dens().syns().strengths, self.focus_syns.clone(), 
        //     | x | x > 0 ));
        // assert!(util::eval_range(&area.ptal().dens().syns().strengths, self.off_focus_syns.clone(), 
        //     | x | x < 0 ));

        // [FIXME] TODO: Check that synapses flags are all zero
        // [FIXME] TODO: Check that synapses strengths are + for focus and - for off-focus


        // FLAGS: [pyr: 0], [syns: 0's], [mcol: 0]; (all zeroed)
    }


    fn flip_focus_syns(&mut self) {
        // Zero the existing src slcs and offs for our dendrite:
        self.clean_up(false);

        // Make sure our ranges are valid:
        // [FIXME] TODO: Add a check which compares the total number of synapses in the two ranges with that of syn_coords.
        if self.focus_syns.end == self.off_focus_syns.start {
            assert!(self.off_focus_syns.end - self.focus_syns.start 
                == self.focus_syns.len() + self.off_focus_syns.len());
        } else if self.off_focus_syns.end == self.focus_syns.start {
            assert!(self.focus_syns.end - self.off_focus_syns.start 
                == self.focus_syns.len() + self.off_focus_syns.len());
        } else {
            panic!("Focus and off focus synapses ranges are overlapping or have a gap!");
        }

        // Swap the ranges:
        let old_focus = self.focus_syns.clone();
        self.focus_syns = self.off_focus_syns.clone();
        self.off_focus_syns = old_focus;

        // Set everything back up:
        let mut area = self.cortex.area_mut(testbed::PRIMARY_AREA_NAME);
        area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_all_to(self.unused_slc_id).unwrap();

        for syn_idx in self.focus_syns.clone() {
            area.ptal_mut().dens_mut().syns_mut().set_src_offs(self.fake_v_ofs, self.fake_u_ofs, syn_idx as usize);
            area.ptal_mut().dens_mut().syns_mut().set_src_slc(self.fake_neighbor_slc, syn_idx as usize);
        }
    }


    fn clean_up(&mut self, zero_strengths: bool) {
        //=============================================================================
        //=============================== CLEAN UP ===================================
        //=============================================================================
        println!("\n ====================== Clean-up ====================== \n");

        let mut area = self.cortex.area_mut(testbed::PRIMARY_AREA_NAME);

        printlny!("Cleaning up...");
        area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(self.unused_slc_id, 
            self.syn_coords.syn_idx_range_tft().clone()).unwrap();
        area.ptal_mut().dens_mut().syns_mut().src_col_v_offs.set_range_to(0, 
            self.syn_coords.syn_idx_range_tft().clone()).unwrap();
        area.ptal_mut().dens_mut().syns_mut().src_col_u_offs.set_range_to(0, 
            self.syn_coords.syn_idx_range_tft().clone()).unwrap();

        if zero_strengths {
            area.ptal_mut().dens_mut().syns_mut().strengths.set_range_to(0, 
                self.syn_coords.syn_idx_range_tft().clone()).unwrap();
        }
    }
}

// pub fn _test_pyr_learning(area: &mut CorticalArea, unused_slc_id: u8, prx_src_slc: u8,
//             fake_neighbor_slc: u8=ter: usize) 
// {
//     // Afferent output slice id:
//     let aff_out_slcs = area.area_map().axn_base_slc_ids_by_tags(map::FF_OUT);
//     assert!(aff_out_slcs.len() == 1);
//     let aff_out_slc = aff_out_slcs[0];

//     // Get a random cell and a random synapse on that cell:
//     let cel_coords = area.ptal_mut().rand_cel_coords();
//     let syn_coords = area.ptal_mut().dens_mut().syns_mut()
//         .rand_syn_coords(&cel_coords);

//     // Base slice for primary temporal pyramidals of our layer:
//     let ptal_axn_slc_idz = area.ptal_mut().base_axn_slc();
//     // assert!(ptal_axn_slc_idz == cel_coords.slc_id_lyr, "cel_coords axon slice mismatch");
//     assert_eq!(ptal_axn_slc_idz, cel_coords.axn_slc_id - cel_coords.slc_id_lyr);

//     // Our cell's proximal source axon (the column spatial axon):
//     let prx_src_axn_idx = area.area_map().axn_idz(prx_src_slc) + cel_coords.col_id();

//     // Our cell's axon:
//     let cel_axn_idx = cel_coords.cel_axn_idx(area.area_map());

//     // Our cell's COLUMN output axon:
//     let aff_out_axn_idx = area.area_map().axn_idz(aff_out_slc) + cel_coords.col_id();

//     // A random, nearby axon for our cell to use as a distal source axon:
//     let (fake_v_ofs, fake_u_ofs, fn_col_id, fake_neighbor_axn_idx) = area.rand_safe_src_axn(&cel_coords, fake_neighbor_slc);

//     //================================ SYN RANGE ==================================
//     // A random dendrite id on the cell tuft:
//     let syn_idx_range_den = syn_coords.syn_idx_range_den();
//     // The synapse range for the entire tuft in which our random synapse resides:
//     let syn_idx_range_tft = syn_coords.syn_idx_range_tft();

//     // The first half of the synapses on our tuft:
//     // let syn_idx_range_tft_first_half = syn_idx_range_tft.start..(syn_idx_range_tft.start + syn_idx_range_tft.len() / 2);

//     // The second half of the synapses on our tuft:
//     // let syn_idx_range_tft_second_half = (syn_idx_range_tft.start + syn_idx_range_tft.len() / 2)
//     //     ..(syn_idx_range_tft.start + syn_idx_range_tft.len());

//     // The first half of the synapses on our tuft:
//     // let syn_idx_range_den_first_half = syn_idx_range_den.start..(syn_idx_range_den.start + syn_idx_range_den.len() / 2);

//     // The second half of the synapses on our tuft:
//     let syn_idx_range_den_second_half = (syn_idx_range_den.start + syn_idx_range_den.len() / 2)
//         ..(syn_idx_range_den.start + syn_idx_range_den.len());

//     let focus_syns = syn_idx_range_den_second_half;

//     // The synapse count for our cell's entire layer (all slices, cells, and tufts):
//     let syn_range_all = 0..area.ptal_mut().dens_mut().syns_mut().states.len();

//     // Set the sources for the synapses on the second half of our chosen tuft to our preselected nearby axon:
//     // <<<<< TODO: IMPLEMENT THIS (for efficiency): >>>>>
//     //         area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(unused_slc_id, den_syn_range);

//     for syn_idx in focus_syns.clone() {
//         area.ptal_mut().dens_mut().syns_mut().set_src_offs(fake_v_ofs, fake_u_ofs, syn_idx as usize);
//         area.ptal_mut().dens_mut().syns_mut().set_src_slc(fake_neighbor_slc, syn_idx as usize);
//     }

//     // PRINT ALL THE THINGS!:
//     let syn_val = area.ptal_mut().dens_mut().syns_mut().syn_state(syn_coords.idx);
//     let fake_neighbor_axn_val = area.axn_state(fake_neighbor_axn_idx as usize);
    
//     // This and every other util::print_all() is very expensive:
//     if print_debug { util::print_all(area, "\n - Confirm Init - "); }

//     for i in 0..LEARNING_ITERS_PER_CELL {

//         //=============================================================================
//         //===================================== 0 =====================================
//         //=============================================================================
//         println!("\n ========================== {}.0: Initialization ========================== ", i);

//         println!("DEBUG INFO - PRINT ALL THE THINGS!: \n\
//             {mt}[prx_src]: prx_src_axn_idx (prx_src_slc: {}, col_id: {}): {} \n\
//             {mt}[dst_src]: fake_neighbor_axn_idx (''_slc: {}, col_id: {}): {}, \n\
//             {mt}[cel_axn]: cel_axn_idx (cel_coords.axn_slc_id: {}, col_id: {}): {} \n\
//             {mt}[col_out]: aff_out_axn_idx (aff_out_slc: {}, col_id: {}): {}, \n\n\
//             \
//             {mt}fake_v_ofs: {}, fake_u_ofs: {}, \n\
//             {mt}fake_neighbor_axn_val: {}, syn_val: {}, syn_idx_range_den: {:?}, syn_idx_range_tft: {:?}, \n\
//             {mt}syn_active_range (2nd half): {:?}, \n\
//             {mt}syn_coords: {}",        

//             prx_src_slc, cel_coords.col_id(), prx_src_axn_idx,
//             fake_neighbor_slc, fn_col_id, fake_neighbor_axn_idx, 
//             cel_coords.axn_slc_id, cel_coords.col_id(), cel_axn_idx,
//             aff_out_slc, cel_coords.col_id(), aff_out_axn_idx, 

//             fake_v_ofs, fake_u_ofs,
//             fake_neighbor_axn_val, syn_val, syn_idx_range_den, syn_idx_range_tft, 
//             focus_syns, 
//             syn_coords,
//             mt = cmn::MT);

//         println!(" ============================== {}.0.0 =============================== \n", i);

//         // Activate distal source axon:
//         printlny!("Activating distal source (neighbor) axon: [{}]...", fake_neighbor_axn_idx);
//         area.activate_axon(fake_neighbor_axn_idx);

//         util::ptal_alco(area, CYCLE | OUTPUT, PRINT_DEBUG_INFO);

//         if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm 0 - "); }
//         // util::confirm_syns(area, &focus_syns, 0, 0, 0);

//         print!("\n");
//         // <<<<< TODO: VERIFY THAT MCOLS.OUTPUT() AXON IS ACTIVE (and print it's idx) >>>>>
//         // <<<<< TODO: CHECK CELL AXON (should be zero here and active on next step) >>>>>

//         // Ensure key axons are active:
//         assert!(area.read_from_axon(fake_neighbor_axn_idx) > 0);
//         printlny!("Pyramidal cell fake neighbor axon is correctly active.");

//         // Ensure minicolumn is predictive as a result of pyramidal activity:
//         // [FIXME] TODO: REENABLE BELOW!
//         // assert!(area.mcols().flag_sets.read_idx_direct(cel_coords.col_id() as usize) == cmn::MCOL_IS_VATIC_FLAG);
//         // printlny!("Minicolumn is correctly vatic (predictive).");

//         //=============================================================================
//         //==================================== 1A =====================================
//         //=============================================================================
//         println!("\n ========================== {}.1: Premonition ========================== ", i);
//         println!(" ============================== {}.1.0 =============================== \n", i);

//         util::ptal_alco(area, ACTIVATE, PRINT_DEBUG_INFO);

//         if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm 1A - "); }
//         // util::confirm_syns(area, &focus_syns, 0, 0, 0);

//         print!("\n");

//         // Ensure our cell is flagged best in (mini) column:
//         // [FIXME] TODO: REENABLE BELOW!
//         // assert!(area.ptal().flag_sets.read_idx_direct(cel_coords.idx() as usize) == cmn::CEL_BEST_IN_COL_FLAG);
//         // printlny!("Our cell is correctly flagged best in column.");

//         //=============================================================================
//         //================================= 1B ===================================
//         //=============================================================================
//         println!("\n ========================== {}.1.1 ========================== \n", i);

//         util::ptal_alco(area, LEARN, PRINT_DEBUG_INFO);

//         if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm 1B - "); }
//         // util::confirm_syns(area, &focus_syns, 0, 0, 0);

//         print!("\n");

//         // <<<<< TODO: Ensure our cells synapses have not learned anything: >>>>>

//         //=============================================================================
//         //=================================== 1C ===================================
//         //=============================================================================
//         println!("\n ========================== {}.1.2 ========================== \n", i);

//         // ACTIVATE PTAL SYNAPSE SOURCE AXON
//         // printlny!("Activating distal source axon: [{}]...", fake_neighbor_axn_idx);
//         // area.activate_axon(fake_neighbor_axn_idx);

//         util::ptal_alco(area, CYCLE | OUTPUT, PRINT_DEBUG_INFO);

//         if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm 1C - "); }
//         // util::confirm_syns(area, &focus_syns, 0, 0, 0);

//         print!("\n");

//         //=============================================================================
//         //=================================== 2A ===================================
//         //=============================================================================
//         println!("\n ========================== {}.2: Vindication ========================== ", i);
//         println!(" ============================== {}.2.0 =============================== \n", i);

//         // ACTIVATE COLUMN PSAL AXON
//         printlny!("Activating proximal source axon: [{}]...", prx_src_axn_idx);
//         area.activate_axon(prx_src_axn_idx);
//         // ACTIVATE PTAL SYNAPSE SOURCE AXON
//         // printlny!("Activating distal source (neighbor) axon: [{}]...", fake_neighbor_axn_idx);
//         // area.activate_axon(fake_neighbor_axn_idx);

//         util::ptal_alco(area, ACTIVATE, PRINT_DEBUG_INFO);

//         if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm 2A - "); }
//         // util::confirm_syns(area, &focus_syns, 0, 0, 0);

//         print!("\n");

//         // ##### ADD ME: assert!(THE PYRAMIDAL OUTPUT AXON (NOT SOMA) IS ACTIVE)
//         // THIS IS CURRENTLY NOT ACTIVATING!!!

//         // MOVED THIS FROM 1B -- PROBABLY WAS IN WRONG SPOT
//         // printlny!("\nConfirming flag sets...");
//         // assert!(util::assert_neq_range(&area.ptal().dens().syns().flag_sets, focus_syns, 0));

//         //=============================================================================
//         //=================================== 2B ===================================
//         //=============================================================================
//         println!("\n ========================== {}.2.1 ========================== \n", i);

//         util::ptal_alco(area, LEARN, PRINT_DEBUG_INFO);

//         if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm 2B - "); }
//         // util::confirm_syns(area, &syn_idx_range_tft_first_half, 0, 0, 0);

//         print!("\n");

//         // <<<<< TODO: assert!(chosen-half of syns are +1, others are -1) >>>>>
//         // CURRENTLY: indexes are a mess

//         //=============================================================================
//         //=================================== 2C ===================================
//         //=============================================================================
//         println!("\n ========================== {}.2.2 ========================== \n", i);

//         util::ptal_alco(area, CYCLE | OUTPUT, PRINT_DEBUG_INFO);

//         if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm 2C - "); }

//         print!("\n");

//         //=============================================================================
//         //=================================== 3 ===================================
//         //=============================================================================
//         println!("\n ========================== {}.4: Continuation ========================== ", i);
//         println!(" =============================== {}.4.0 =============================== \n", i);

//         // ACTIVATE, LEARN, CYCLE, & OUTPUT multiple times without touching inputs:
//         let cont_iters = 3;
//         printlny!("Performing complete cycle(A|L|C|O) {} times...", cont_iters);

//         for _ in 0..cont_iters {
//             util::ptal_alco(area, ALL, false);
//         }

//         if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm 3 - "); }

//         print!("\n");

//         //=============================================================================
//         //=================================== 3 ===================================
//         //=============================================================================
//         println!("\n ========================== {}.5: Deactivation ========================== ", i);
//         println!(" =============================== {}.5.0 =============================== \n", i);

//         // ZERO PTAL SYNAPSE SOURCE AXON
//         printlny!("Deactivating distal source (neighbor) axon..."); 
//         area.deactivate_axon(fake_neighbor_axn_idx);

//         util::ptal_alco(area, CYCLE, PRINT_DEBUG_INFO);
//         // util::ptal_alco(area, ACTIVATE, PRINT_DEBUG_INFO);

//         if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm 3 - "); }

//         print!("\n");

//         //=============================================================================
//         //=================================== 4 ===================================
//         //=============================================================================
//         println!("\n ========================== {}.6: Termination ========================== ", i);
//         println!(" =============================== {}.6.0 =============================== \n", i);

//         // ZERO COLUMN PSAL AXON
//         printlny!("Deactivating proximal source axon...");
//         area.deactivate_axon(prx_src_axn_idx);
//         // ZERO PTAL SYNAPSE SOURCE AXON
//         // printlny!("Deactivating distal source (neighbor) axon...");
//         // area.deactivate_axon(fake_neighbor_axn_idx);

//         // util::ptal_alco(area, ALL, PRINT_DEBUG_INFO);
//         util::ptal_alco(area, ACTIVATE | LEARN | OUTPUT, PRINT_DEBUG_INFO);

//         if PRINT_DEBUG_INFO { util::print_all(area, "\n - Confirm 3 - "); }

//         print!("\n");        
//     }

//     //=============================================================================
//     //=================================== CLEAN UP ===================================
//     //=============================================================================
//     println!("\n ========================== Clean-up ========================== \n");

//     printlny!("Cleaning up...");
//     area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(unused_slc_id, syn_idx_range_tft.clone());
//     area.ptal_mut().dens_mut().syns_mut().src_col_v_offs.set_range_to(0, syn_idx_range_tft.clone());
//     area.ptal_mut().dens_mut().syns_mut().src_col_u_offs.set_range_to(0, syn_idx_range_tft.clone());

//     print!("\n");
//     panic!(" -- DEBUGGING -- ");
// }


// pub const CEL_PREV_CONCRETE_FLAG: u8         = 128    (0x80)
// pub const CEL_BEST_IN_COL_FLAG: u8             = 64    (0x40)
// pub const CEL_PREV_STP_FLAG: u8                 = 32    (0x20)
// pub const CEL_PREV_FUZZY_FLAG: u8            = 16    (0x10)

// pub const SYN_STP_FLAG: u8                    = 1;
// pub const SYN_STD_FLAG: u8                    = 2;
// pub const SYN_CONCRETE_FLAG: u8                = 8;


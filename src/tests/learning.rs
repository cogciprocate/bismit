#![allow(non_snake_case, dead_code)]

// use std::io;
use std::ops::Range;
use cortex::{Cortex, CorticalAreaTest, SynapsesTest, SynCoords, DendritesTest,
    DataCellLayer, DataCellLayerTest};
use map::LayerTags;
use cmn;
use super::testbed;
use super::util::{self, PtalAlco};

// const LEARNING_TEST_ITERATIONS: usize = 5; //50;
// const LEARNING_ITERS_PER_CELL: usize = 2;
const LEARNING_CONTINUATION_ITERS: usize = 3;

const PRNT_INFO: bool = false;
const PRINT_FINAL_ITER_ONLY: bool = false;

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

    LOG:
        - 2016-Dec: Some changes made to accommodate variable size tufts.
*/


////// #[test]
fn dst_den_learning_REIMPLEMENT_ME() {
    // assert!(!PRNT_INFO, "Printing debug info (or anything else) is currently disabled.");
    let mut ltb = LearningTestBed::new();
    // 180 -> +-64 (slow), +-96 (fast)
    // 360 -> +-96 (slow), +-119 (fast)

    // let on_focus_iters = 180;
    // let off_focus_iters = 180;
    let on_focus_iters = 80;
    let off_focus_iters = 80;

    ltb.test_on_off(on_focus_iters, off_focus_iters);
}


#[allow(dead_code)]
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

            let area = cortex.areas_mut().by_key_mut(testbed::PRIMARY_AREA_NAME).unwrap();

            // Zero all dendrite and synapse buffers:
            area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().set_all_to_zero(true);

            area.axns().states().default_queue().unwrap().finish().unwrap();
            area.axns().states().cmd().fill(0, None).enq().unwrap();
            area.axns().states().default_queue().unwrap().finish().unwrap();

            // Set source slice to an unused slice for all synapses:
            let unused_slc_ranges = area.area_map().layer_map().layers_containing_tags_slc_range(LayerTags::UNUSED);
            assert!(unused_slc_ranges.len() >= 3, "Make sure at least three axon layers have the UNUSED_TESTING flag.");
            let unused_slc_id = unused_slc_ranges[0].start as u8;

            area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().src_slc_ids().default_queue().unwrap().finish().unwrap();
            area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().src_slc_ids().cmd().fill(unused_slc_id, None).enq().unwrap();
            area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().src_slc_ids().default_queue().unwrap().finish().unwrap();

            // Finish queues:
            area.finish_queues();

            // Primary spatial layer slice idz (base axon slice):
            let prx_src_slc = area.ssc_layer(testbed::PRIMARY_SPATIAL_SSC_LAYER_NAME).unwrap().base_axn_slc();

            // Fake neighbor slice:
            let fake_neighbor_slc = unused_slc_ranges[1].start as u8;

            // DEBUG: Print slice map and synapse dims:
            println!("\nDEBUG INFO: \n{mt}{}, \n{mt}synapse dims: {:?}",
                area.area_map(), area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().syns().lyr_dims(), mt = cmn::MT);

            // Afferent output slice id:
            // [FIXME]: ASSIGN SPECIAL TAGS TO THIS LAYER:
            let aff_out_slc_ranges = area.area_map().layer_map().iter()
                .filter(|li| li.axn_domain().is_output() && li.slc_range().is_some())
                .map(|li| li.slc_range().unwrap().clone())
                .collect::<Vec<_>>();

            assert!(aff_out_slc_ranges.len() == 1);
            let aff_out_slc = aff_out_slc_ranges[0].start as u8;

            // Get a random cell and a random synapse on that cell:
            let cel_coords = area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().rand_cel_coords();
            let syn_coords = area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut()
                .rand_syn_coords(cel_coords.clone());

            // Base slice for primary temporal pyramidals of our layer:
            let ptal_axn_slc_idz = area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().base_axn_slc();
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
            let syn_idx_range_celtft = syn_coords.syn_idx_range_celtft();

            assert!(syn_idx_range_den.start >= syn_idx_range_celtft.start &&
                syn_idx_range_den.end <= syn_idx_range_celtft.end);

            // The first half of the synapses on our tuft:
            let syn_idx_range_den_first_half = syn_idx_range_den.start..(syn_idx_range_den.start
                + syn_idx_range_den.len() / 2);

            // The second half of the synapses on our tuft:
            let syn_idx_range_den_second_half = (syn_idx_range_den.start + syn_idx_range_den.len() / 2)
                ..(syn_idx_range_den.start + syn_idx_range_den.len());

            let focus_syns = syn_idx_range_den_second_half;
            let off_focus_syns = syn_idx_range_den_first_half;


            // The synapse count for our cell's entire layer (all slices, cells, and tufts):
            // let syn_range_all = 0..area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().states().len();

            // Set the sources for the synapses on the second half of our chosen tuft to our preselected nearby axon:
            // <<<<< [FIXME] TODO: IMPLEMENT THIS (for efficiency): >>>>>
            //         area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().src_slc_ids.set_range_to(unused_slc_id, den_syn_range).unwrap();

            // * TODO: Reimplement using fill:
            for syn_idx in focus_syns.clone() {
                area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().set_src_offs(fake_v_ofs, fake_u_ofs, syn_idx as usize);
                area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().set_src_slc(fake_neighbor_slc, syn_idx as usize);
            }

            // Finish queues:
            area.finish_queues();

            // PRINT ALL THE THINGS!:
            let syn_val = area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().syn_state(syn_coords.idx);
            let fake_neighbor_axn_val = area.axn_state(fake_neighbor_axn_idx as usize);
            println!("DEBUG INFO - PRINT ALL THE THINGS!: \n\
                {mt}[prx_src]: prx_src_axn_idx (prx_src_slc: {}, col_id: {}): {} \n\
                {mt}[dst_src]: fake_neighbor_axn_idx (''_slc: {}, col_id: {}): {}, \n\
                {mt}[cel_axn]: cel_axn_idx (cel_coords.axn_slc_id: {}, col_id: {}): {} \n\
                {mt}[col_out]: aff_out_axn_idx (aff_out_slc: {}, col_id: {}): {}, \n\n\
                \
                {mt}fake_v_ofs: {}, fake_u_ofs: {}, \n\
                {mt}fake_neighbor_axn_val: {}, syn_val: {}, syn_idx_range_den: {:?}, syn_idx_range_celtft: {:?}, \n\
                {mt}syn_active_range (2nd half): {:?}, \n\
                {mt}syn_coords: {}",

                prx_src_slc, cel_coords.col_id(), prx_src_axn_idx,
                fake_neighbor_slc, fn_col_id, fake_neighbor_axn_idx,
                cel_coords.axn_slc_id, cel_coords.col_id(), cel_axn_idx,
                aff_out_slc, cel_coords.col_id(), aff_out_axn_idx,

                fake_v_ofs, fake_u_ofs,
                fake_neighbor_axn_val, syn_val, syn_idx_range_den, syn_idx_range_celtft,
                focus_syns,
                syn_coords,
                mt = cmn::MT);

            // This and every other util::print_all() is very expensive:
            if PRNT_INFO {
                util::print_all(area, "\n - Confirm Init - ");
            }

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
    fn test_on_off(&mut self, on_focus_iters: usize, off_focus_iters: usize) {
        printlnc!(yellow: "\n###### on_focus_iters: ######");

        for i in 0..on_focus_iters {
            // // [DEBUG]: Prints a number for each iter
            // print!(" {}", i);
            // std::sio::stdout().flush().unwrap();

            let final_iter = i == (on_focus_iters - 1);

            let print_debug = ((PRINT_FINAL_ITER_ONLY && final_iter) || !PRINT_FINAL_ITER_ONLY)
                && PRNT_INFO;

            self.learning_iter(i, false, print_debug);
        }

        printlnc!(yellow: "\n\nFlipping focus syns...");
        self.flip_focus_syns();

        printlnc!(yellow: "\n###### off_focus_iters: ######");

        for i in 0..off_focus_iters {
            // // [DEBUG]: Prints a number for each iter
            // print!(" {}", i);
            // std::io::stdout().flush().unwrap();

            let final_iter = i == (off_focus_iters - 1);

            let print_debug = ((PRINT_FINAL_ITER_ONLY && final_iter) || !PRINT_FINAL_ITER_ONLY)
                && PRNT_INFO;

            self.learning_iter(i, true, print_debug);
        }

        printlnc!(yellow: "\n\nComplete.");
        self.clean_up(true, true);
        print!("\n");
    }


    /// Perform a comprehensive test of the default learning algorithm.
    ///
    /// The great-grandmother of all tests. Tests a little bit of everything.
    ///
    /// [FIXME] TODO: Add awareness of the current averages of both the on and
    /// off-focus synapses from the previous run in order to make sure that the
    /// synapse strengths are moving in the correct direction.
    ///
    /// [TODO 2016-Dec-24]: Ensure that this test continues to be thorough as
    /// our learning algorithm undergoes its many upcoming dramatic changes.
    ///
    fn learning_iter(&mut self, i: usize, flipped: bool, print_debug: bool) {
        let area = self.cortex.areas_mut().by_key_mut(testbed::PRIMARY_AREA_NAME).unwrap();
        let tft_id = self.syn_coords.tft_id;
        let tft_den_idz = area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().den_idzs_by_tft()[tft_id];
        let den_idx = self.syn_coords.den_idx(tft_den_idz);
        let celtft_idx = self.syn_coords.pyr_celtft_idx();
        assert!(celtft_idx == area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().celtft_idx(tft_id, &self.syn_coords.cel_coords));
        let cel_idx = self.syn_coords.cel_coords.idx();

        let flpd_str = if flipped { "FLP" } else { "ORG" };

        if print_debug {
            area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().syns().print_src_slc_ids(Some(self.syn_coords.syn_idx_range_den()));
            util::print_all(area, "\n - Pre-init - ");
            print!("\n");
        }

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
        if print_debug { printlnc!(yellow: "Activating distal source (neighbor) axon: [{}]...",
            self.fake_neighbor_axn_idx); }

        area.finish_queues();
        area.activate_axon(self.fake_neighbor_axn_idx);
        area.finish_queues();

        util::ptal_alco(area, PtalAlco::CYCLE | PtalAlco::OUTPUT, print_debug);

        if print_debug {
            util::print_all(area, "\n - Confirm 0 - ");
            print!("\n");
        }

        assert!(util::eval_range(self.focus_syns.clone(), &area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().syns().states(),
            |x| x != 0), "Synapses in range '{}..{}' are not active.",
            self.focus_syns.start, self.focus_syns.end );

        assert!(util::read_idx_direct(den_idx as usize, area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().states_raw()) != 0,
            "Dendrite '{}' is not active. This could be because learning is disabled.", den_idx);

        // // Ensure that we have a non-zero raw best dendrite state for the cell-tuft:
        // assert!(util::read_idx_direct(celtft_idx as usize, area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().tft_best_den_states_raw()) != 0);

        // Ensure that we have the correct best dendrite ID:
        assert!(util::read_idx_direct(celtft_idx as usize,
            area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME)
                .unwrap().tfts().best_den_ids()) as u32 ==
            self.syn_coords.den_id_celtft);

        // Ensure that we have a non-zero cell state:
        assert!(util::read_idx_direct(cel_idx as usize, area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().states()) != 0);
        // println!("Cell best dendrite state is correct.");

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

        util::ptal_alco(area, PtalAlco::ACTIVATE, print_debug);


        if print_debug {
            util::print_all(area, "\n - Confirm 1A - ");
            print!("\n");
        }

        // Ensure our cell is flagged best in (mini) column:
        // [FIXME] TODO: REENABLE BELOW!
        // assert!(area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().flag_sets.read_idx_direct(self.syn_coords.cel_coords.idx() as usize)
        //     & cmn::CEL_BEST_IN_COL_FLAG == cmn::CEL_BEST_IN_COL_FLAG);

        assert!(util::read_idx_direct(celtft_idx as usize,
            area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME)
                .unwrap().tfts().best_den_ids()) as u32 ==
            self.syn_coords.den_id_celtft);

        // FLAGS: [pyr: 64], [syns: 0's], [mcol: 1];

        //=============================================================================
        //============================= 1B ===================================
        //=============================================================================
        if print_debug { println!(
            "\n ====================== {}[{}] 1B ====================== \n", flpd_str, i); }

        if print_debug { println!("Finishing queue..."); }
        area.ocl_pq().queue().finish().unwrap();

        util::ptal_alco(area, PtalAlco::LEARN, print_debug);

        // if print_debug { println!("Finishing queues..."); }

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


        util::ptal_alco(area, PtalAlco::CYCLE | PtalAlco::OUTPUT, print_debug);

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
        if print_debug {  printlnc!(yellow: "Activating proximal source axon: [{}]...", self.prx_src_axn_idx); }
        area.finish_queues();
        area.activate_axon(self.prx_src_axn_idx);
        area.finish_queues();

        // // ACTIVATE PTAL SYNAPSE SOURCE AXON
        // // [NOTE 2017-Jan-01]: Unknown what this was used for or has been replaced with. Investigate.
        // printlnc!(yellow: "Activating distal source (neighbor) axon: [{}]...", self.fake_neighbor_axn_idx);
        // area.activate_axon(self.fake_neighbor_axn_idx);

        util::ptal_alco(area, PtalAlco::ACTIVATE, print_debug);

        if print_debug {
            util::print_all(area, "\n - Confirm 2A - ");
            print!("\n");
        }

        // ##### ADD ME: assert!(THE PYRAMIDAL OUTPUT AXON (NOT SOMA) IS ACTIVE)
        assert!(area.read_from_axon(self.cel_axn_idx as u32) > 0);

        // MOVED THIS FROM 1B -- PROBABLY WAS IN WRONG SPOT
        // printlnc!(yellow: "\nConfirming flag sets...");
        // assert!(util::assert_neq_range(&area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().syns().flag_sets, focus_syns, 0));

        // FLAGS: [pyr: 80], [syns: 0's], [mcol: 1]; (unchanged)

        //=============================================================================
        //=============================== 2B ===================================
        //=============================================================================
        if print_debug { println!(
            "\n ====================== {}[{}] 2B ====================== \n", flpd_str, i); }

        util::ptal_alco(area, PtalAlco::LEARN, print_debug);

        if print_debug {
            util::print_all(area, "\n - Confirm 2B - ");
            print!("\n");
            // unimplemented!();
        }

        // Check that all synapses within the `focus_syns` range have the
        // potentiation flag (`SYN_STPOT_FLAG`):
        assert!(util::eval_range(self.focus_syns.clone(), &area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().syns().flag_sets(),
            |x| x == cmn::SYN_STPOT_FLAG ), "tests::learning::learning_iter: \
            'On-focus' synapse flags are not properly set to potentiation.");

        // Check that all synapses within the `off_focus_syns` range have the
        // depression flag (`SYN_STDEP_FLAG`):
        assert!(util::eval_range(self.off_focus_syns.clone(), &area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().syns().flag_sets(),
            |x| x == cmn::SYN_STDEP_FLAG ), "tests::learning::learning_iter: \
            'Off-focus' synapse flags are not properly set to depression.");

        // [FIXME]: Check for flags: [pyr: 208], [syns: 1's & 2's], [mcol: 1]; (pyr and syns changed)


        //=============================================================================
        //=============================== 2C ===================================
        //=============================================================================
        if print_debug { println!(
            "\n ====================== {}[{}] 2C ====================== \n", flpd_str, i); }

        util::ptal_alco(area, PtalAlco::CYCLE | PtalAlco::OUTPUT, print_debug);

        if print_debug {
            util::print_all(area, "\n - Confirm 2C - ");
            print!("\n");
            // unimplemented!();
        }

        // FLAGS: [pyr: 208], [syns: 1's & 2's], [mcol: 1]; (unchanged)

        //=============================================================================
        //=============================== 3A ===================================
        //=============================================================================
        if print_debug { println!(
            "\n ====================== {}[{}] 3: Continuation ====================== ", flpd_str, i); }

        // ACTIVATE, LEARN, CYCLE, & OUTPUT multiple times without touching inputs:

        if print_debug { printlnc!(yellow: "Performing complete cycle(A|L|C|O) {} times...",
            LEARNING_CONTINUATION_ITERS); }

        for _ in 0..LEARNING_CONTINUATION_ITERS {
            util::ptal_alco(area, PtalAlco::ALL, false);
        }

        if print_debug {
            util::print_all(area, "\n - Confirm 3 - ");
            print!("\n");
        }

        // FLAGS: [pyr: 208], [syns: 1's & 2's], [mcol: 1]; (unchanged)

        //=============================================================================
        //=============================== 4 ===================================
        //=============================================================================
        if print_debug { println!(
            "\n ====================== {}[{}] 4: Deactivation ====================== ", flpd_str, i); }

        // ZERO PTAL SYNAPSE SOURCE AXON
        if print_debug { printlnc!(yellow: "Deactivating distal source (neighbor) axon..."); }
        area.finish_queues();
        area.deactivate_axon(self.fake_neighbor_axn_idx);
        area.finish_queues();

        util::ptal_alco(area, PtalAlco::CYCLE, print_debug);
        util::ptal_alco(area, PtalAlco::ACTIVATE, print_debug);

        if print_debug {
            util::print_all(area, "\n - Confirm 4 - ");
            print!("\n");
        }

        // Double-check that all synapses within the `focus_syns` range have the
        // potentiation flag (`SYN_STPOT_FLAG`):
        assert!(util::eval_range(self.focus_syns.clone(), &area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().syns().flag_sets(),
            |x| x == cmn::SYN_STPOT_FLAG ), "tests::learning::learning_iter: \
            Synapse flags are not properly set to potentiation.");

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
        if print_debug {  printlnc!(yellow: "Deactivating proximal source axon..."); }
        area.finish_queues();
        area.deactivate_axon(self.prx_src_axn_idx);
        area.finish_queues();
        // ZERO PTAL SYNAPSE SOURCE AXON
        // printlnc!(yellow: "Deactivating distal source (neighbor) axon...");
        // area.deactivate_axon(fake_neighbor_axn_idx);

        // util::ptal_alco(area, ALL, print_debug);
        util::ptal_alco(area, PtalAlco::ACTIVATE | PtalAlco::LEARN | PtalAlco::OUTPUT, print_debug);

        if print_debug {
            util::print_all(area, "\n - Confirm 5 - ");
            print!("\n");
            // unimplemented!();
        }

        // [FIXME] TODO: Need a more sophisticated test that tracks the current syn strengths:
        // assert!(util::eval_range(&area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().syns().strengths, self.focus_syns.clone(),
        //     |x| x > 0 ));
        // assert!(util::eval_range(&area.pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens().syns().strengths, self.off_focus_syns.clone(),
        //     |x| x < 0 ));

        // [FIXME] TODO: Check that synapses flags are all zero
        // [FIXME] TODO: Check that synapses strengths are + for focus and - for off-focus


        // FLAGS: [pyr: 0], [syns: 0's], [mcol: 0]; (all zeroed)
    }


    fn flip_focus_syns(&mut self) {
        // Zero the existing src slcs and offs for our dendrite:
        self.clean_up(false, true);

        // Make sure our ranges are valid:
        //
        // [FIXME] TODO: Add a check which compares the total number of
        // synapses in the two ranges with that of syn_coords.
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
        let area = self.cortex.areas_mut().by_key_mut(testbed::PRIMARY_AREA_NAME).unwrap();
        // area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().src_slc_ids()
        //    .cmd().fill(self.unused_slc_id, None).enq().unwrap();

        // * TODO: Reimplement using fill:
        for syn_idx in self.focus_syns.clone() {
            area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().set_src_offs(self.fake_v_ofs, self.fake_u_ofs, syn_idx as usize);
            area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().set_src_slc(self.fake_neighbor_slc, syn_idx as usize);
        }

        // Finish queues:
        area.finish_queues();
    }


    // Wipes every synapse on the entire cell-tuft;
    fn clean_up(&mut self, zero_strengths: bool, zero_flag_sets: bool) {
        //=============================================================================
        //=============================== CLEAN UP ===================================
        //=============================================================================
        if PRNT_INFO { println!("\n ====================== Clean-up ====================== \n"); }

        let area = self.cortex.areas_mut().by_key_mut(testbed::PRIMARY_AREA_NAME).unwrap();

        // Finish queues:
        area.finish_queues();

        printlnc!(yellow: "Cleaning up...");

        area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().src_slc_ids().cmd()
            .fill(self.unused_slc_id, Some(self.syn_coords.syn_idx_range_celtft().len()))
            .offset(self.syn_coords.syn_idx_range_celtft().start).enq().unwrap();

        area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().src_col_v_offs().cmd()
            .fill(0, Some(self.syn_coords.syn_idx_range_celtft().len()))
            .offset(self.syn_coords.syn_idx_range_celtft().start).enq().unwrap();

        area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().src_col_u_offs().cmd()
            .fill(0, Some(self.syn_coords.syn_idx_range_celtft().len()))
            .offset(self.syn_coords.syn_idx_range_celtft().start).enq().unwrap();

        if zero_strengths {
            area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().strengths().cmd()
            .fill(0, Some(self.syn_coords.syn_idx_range_celtft().len()))
            .offset(self.syn_coords.syn_idx_range_celtft().start).enq().unwrap();
        }

        if zero_flag_sets {
            area.pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut().syns_mut().flag_sets().cmd()
            .fill(0, Some(self.syn_coords.syn_idx_range_celtft().len()))
            .offset(self.syn_coords.syn_idx_range_celtft().start).enq().unwrap();
        }

        // Finish queues:
        area.finish_queues();

        if PRNT_INFO {
            util::print_all(area, "\n - Post-clean-up - ");
            print!("\n");
            // unimplemented!();
        }
    }
}

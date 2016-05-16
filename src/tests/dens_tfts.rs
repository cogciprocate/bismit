use area::{CorticalArea, CorticalAreaTest};
use map::{self, AreaMapTest};
use area::{SynapsesTest, DendritesTest, DenCoords, AxonSpaceTest};
use cmn::{self, DataCellLayer, DataCellLayerTest};
use super::{testbed, util};


const DENS_TEST_ITERATIONS: usize = 500;
const CELS_TEST_ITERATIONS: usize = 1; //50;
const PRINT_DETAILS: bool = false;




// TEST_CEL_TUFTS():
/*
        Test that input on each dendridic tuft is reaching the cell soma.
*/
#[test]
fn cel() {
    let mut cortex = testbed::cortex_with_lots_of_apical_tufts();
    let mut area = cortex.area_mut(testbed::PRIMARY_AREA_NAME);

    // Zero all dendrite and synapse buffers:
    area.ptal_mut().dens_mut().set_all_to_zero(true);
    area.axns().states.cmd().fill(0, None).enq().unwrap();

    // Set source slice to an unused slice for all synapses:
    let unused_slc_ranges = area.area_map().layers().layers_containing_tags_slc_range(map::UNUSED_TESTING);
    assert!(unused_slc_ranges.len() >= 3, "Make sure at least three axon layers have the UNUSED_TESTING flag.");
    let zeroed_slc_id = unused_slc_ranges[0].start;
    let unused_slc_id = unused_slc_ranges[1].start;

    area.ptal_mut().dens_mut().syns_mut().src_slc_ids().cmd().fill(zeroed_slc_id, None).enq().unwrap();
    area.ptal_mut().dens_mut().syns_mut().src_slc_ids().cmd().fill(zeroed_slc_id, None).enq().unwrap();

    // 'input' source slice which will be assigned to the synapses being tested:
    // let src_slc_ids = area.area_map().layers().layers_containing_tags_slc_range(map::FF_IN);
    // assert!(src_slc_ids.len() == 1);
    // let src_slc_id = ;

    // Primary spatial layer slice idz (base axon slice):
    // let prx_src_slc_id = area.psal().base_axn_slc();

    // DEBUG: Print slice map and synapse dims:
    println!("\nDEBUG INFO: \n{mt}{}, \n{mt}synapse dims: {:?}",
        area.area_map(), area.ptal().dens().syns().dims(), mt = cmn::MT);

    // Run tests:
    for i in 0..CELS_TEST_ITERATIONS {
        _test_rand_cel(area, zeroed_slc_id, unused_slc_id, i);
        // learning::_test_pyr_learning(area, zeroed_slc_id, prx_src_slc_id, unused_slc_ids[1], i);
    }
}


fn _test_rand_cel(area: &mut CorticalArea, zeroed_slc_id: u8, src_slc_id: u8, iter: usize) {
    // Get a random cell:
    let cel_coords = area.ptal_mut().rand_cel_coords();

    // For each tuft on that cell:
    for tft_id in area.ptal().dens().tft_id_range() {
        // And for each dendrite:
        for den_id_tft in area.ptal().dens().den_id_range() {

            let den_coords = DenCoords::new(tft_id, den_id_tft, &cel_coords, area.ptal().dens().dims());

            // Get synapse range corresponding to our dendrite:
            let den_syn_range = den_coords.syn_range(area.ptal().dens().syns().syns_per_den_l2());

            // Axon index corresponding to our cell and source slice:
            let src_axn_idx = area.area_map().axn_idx(src_slc_id, cel_coords.v_id, 
                0, cel_coords.u_id, 0).unwrap();

            //=============================================================================
            //========================= ACTIVATE AXON AND CYCLE ===========================
            //=============================================================================

            // Set source slice to our source slice for our dendrite's synapses only
            // area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(src_slc_id, 
            //     den_syn_range.clone()).unwrap();

            let fill_size = den_syn_range.end - den_syn_range.start;
            area.ptal_mut().dens_mut().syns_mut().src_slc_ids().cmd()
                .fill(src_slc_id, Some(fill_size)).offset(den_syn_range.start).enq().unwrap();

            // Write input:
            //area.write_to_axon(128, src_axn_idx);
            area.activate_axon(src_axn_idx);

            // Cycle entire cell:
            area.ptal_mut().cycle(None);    

            //=============================================================================
            //================================= EVALUATE ==================================
            //=============================================================================
                
            let den_idx = area.ptal().dens().den_idx(tft_id, cel_coords.idx(), den_id_tft);

            let mut den_state = vec![0];
            let mut cel_state = vec![0];
            area.ptal().dens().states().cmd().read(&mut den_state).offset(den_idx as usize).enq().unwrap();
            area.ptal().states().cmd().read(&mut cel_state).offset(cel_coords.idx() as usize).enq().unwrap();
            let den_state = den_state[0];
            let cel_state = cel_state[0];
            // area.ptal().dens().states.enqueue_read(&mut den_state[0..1], den_idx as usize);
            // let den_state = area.ptal().dens().states.read_idx_direct(den_idx as usize);
            // let cel_state = area.ptal().states.read_idx_direct(cel_coords.idx() as usize);


            // Ensure that the dendrite is active:
            if den_state == 0 || cel_state == 0 {
                // Print debugging info:
                println!("\niter: {}", iter);
                println!("{}", cel_coords);
                println!("{}", den_coords);
                println!("Axon Info: zeroed_slc_id: {}, src_slc_id: {}, src_axn_idx: {}", 
                    zeroed_slc_id, src_slc_id, src_axn_idx);
                println!("dens.state[{}]: '{}'", den_idx, den_state);
                print!("Synapse src_slc_ids: ");
                // area.ptal_mut().dens_mut().syns_mut().src_slc_ids
                //     .print(1, None, Some(den_syn_range.clone()), true);
                // util::print_all(area, " -- TEST_CEL_TUFTS() -- ");
                print!("\n");

                // Scream like a little girl:
                panic!("Error: dendrite not activated on test cell.");
            }

            // Make sure neighbors, etc. are inactive
            util::eval_others(&area.ptal_mut().dens_mut().states(), den_idx as usize, 0);
            util::eval_others(&area.ptal_mut().states(), cel_coords.idx() as usize, 0);

            //=============================================================================
            //================================= CLEAN UP ==================================
            //=============================================================================

            // area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(zeroed_slc_id, den_syn_range).unwrap();

            let fill_size = den_syn_range.end - den_syn_range.start;
            debug_assert_eq!(fill_size, den_syn_range.len());
            area.ptal_mut().dens_mut().syns_mut().src_slc_ids().cmd()
                .fill(zeroed_slc_id, Some(fill_size)).offset(den_syn_range.start).enq().unwrap();

            area.write_to_axon(0, src_axn_idx);
        }
    }

    // Clear out any residual activity:
    area.ptal_mut().cycle(None);

    // print!("\n");
    // panic!(" -- DEBUGGING -- ");
}






#[test]
fn dens() {
    let mut cortex = testbed::fresh_cortex();
    let mut area = cortex.area_mut(testbed::PRIMARY_AREA_NAME);

    // area.ptal_mut().dens_mut().syns_mut().set_all_to_zero();
    area.ptal_mut().dens_mut().set_all_to_zero(true);

    // SET SOURCE SLICE TO UNUSED SLICE FOR EVERY SYNAPSE:
    let zeroed_slc_range = area.area_map().layers()
        .layers_containing_tags_slc_range(map::UNUSED_TESTING)[0].clone();
    let zeroed_slc_id = zeroed_slc_range.start;
    area.ptal().dens().syns().src_slc_ids().cmd().fill(zeroed_slc_id, None).enq().unwrap();

    for _ in 0..DENS_TEST_ITERATIONS {

        //=============================================================================
        //=================================== INIT ====================================
        //=============================================================================

        // area.ptal_mut().dens_mut().set_all_to_zero(true);
        // area.ptal_mut().dens_mut().syns_mut().set_all_to_zero();
        // area.axns.states.cmd().fill(&[0], None).enq().unwrap();

        let cel_coords = area.ptal_mut().rand_cel_coords();
        let den_coords = area.ptal_mut().dens_mut().rand_den_coords(&cel_coords);

        // let den_dims = den_coords.dims().clone();

        // GET SOURCE SLICE TO USE TO SIMULATE INPUT:
        let cel_syn_range = den_coords.syn_idx_range_tft(area.ptal().dens().syns().syns_per_den_l2());
        let src_slc_ids = area.area_map().layers().layers_containing_tags_slc_range(map::FF_IN);
        assert!(src_slc_ids.len() == 1);
        let src_slc_id = src_slc_ids[0].start;

        // GET THE AXON INDEX CORRESPONDING TO OUR CELL AND SOURCE SLICE:
        let src_axn_idx = area.area_map().axn_idx(src_slc_id, cel_coords.v_id, 
                    0, cel_coords.u_id, 0).unwrap();

        // PRINT SOME DEBUG INFO IN CASE OF FAILURE:
        if PRINT_DETAILS {
            print!("\n");
            println!("{}", cel_coords);
            println!("{}", den_coords);
            println!("Axon Info: src_slc_id: {}, src_axn_idx: {}", src_slc_id, src_axn_idx);
        }

        //=============================================================================
        //========================= ACTIVATE AXON AND CYCLE ===========================
        //=============================================================================

        // SET SOURCE SLICE TO AFF IN SLICE FOR OUR CELL'S SYNAPSES ONLY:
        // area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(src_slc_id, 
        //     cel_syn_range.clone()).unwrap();


        area.ptal().dens().syns().src_slc_ids().cmd()
            .fill(src_slc_id, Some(cel_syn_range.len()))
            .offset(cel_syn_range.start)
            .enq().unwrap();

        // WRITE INPUT:
        area.activate_axon(src_axn_idx);

        // CYCLE SYNS AND DENS:
        area.ptal_mut().dens_mut().cycle(None);    

        //=============================================================================
        //================================= EVALUATE ==================================
        //=============================================================================

        // let mut result = vec![0]; REMOVE

        // CHECK EACH DENDRITE ON OUR TEST CELL:
        for den_idx in den_coords.cel_den_range_tftsec() {
            // area.ptal().dens().states.enqueue_read(&mut result[..], den_idx as usize); REMOVE
            // let den_state = result[0]; REMOVE
            // let den_state = area.ptal().dens().states.read_idx_direct(den_idx as usize);

            let den_state = util::read_idx_direct(den_idx as usize, area.ptal().dens().states());

            if PRINT_DETAILS {
                print!("\n");
                println!("dens.state[{}]: '{}'", den_coords.idx, den_state);
                // print_all(area, " - Dens - ");
                // print!("\n");
            }

            // ENSURE THAT THE DENDRITE IS ACTIVE:
            assert!(den_state != 0, "Error: dendrite not activated on test cell.");
        }

        // <<<<< TODO: TEST OTHER RANDOM OR NEARBY CELLS >>>>>

        //=============================================================================
        //================================= CLEAN UP ==================================
        //=============================================================================

        // area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(zeroed_slc_id, cel_syn_range).unwrap();

        area.ptal().dens().syns().src_slc_ids().cmd()
            .fill(zeroed_slc_id, Some(cel_syn_range.len()))
            .offset(cel_syn_range.start)
            .enq().unwrap();

        area.write_to_axon(0, src_axn_idx);
    }

    // print!("\n");
    // panic!(" -- DEBUGGING -- ");
}



// pub enum ElemSpec {
//     All,
//     List(Box<Vec<usize>>),
//     Range(Range<usize>),
//     Single(usize),
//     RandSingle(Range<usize>),
//     RandRange(Range<usize>),
// }

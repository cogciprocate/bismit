#![allow(dead_code)]

use std::iter;
use std::io::{self, Write};
use cortex::{Cortex, Dendrites, PyramidalLayer, DendritesTest, SynapsesTest,
    DataCellLayer, DataCellLayerTest};
use subcortex::{InputGenerator};
use cmn;
use tests::{util, testbed};


#[test]
fn cortex() {
    let layer_map_schemes = testbed::define_layer_map_schemes();
    let area_schemes = testbed::define_protoareas();

    let input_gen = InputGenerator::new(&layer_map_schemes, &area_schemes, "v0").unwrap();
    // let subcortex = Subcortex::new().nucleus(input_gen);
    let mut cortex = Cortex::builder(layer_map_schemes, area_schemes)
        .subcortical_nucleus(input_gen)
        .build().unwrap();

    cycles(&mut cortex, testbed::PRIMARY_AREA_NAME);
}



pub fn cycles(cortex: &mut Cortex, area_name: &str) {
    // let emsg = "\ntests::hybrid::test_cycles()";

    /*cortex.areas_mut().by_key_mut(area_name).ssc_layer_mut(testbed::PRIMARY_SPATIAL_SSC_LAYER_NAME).unwrap().dens.syns().src_col_v_offs.cmd().fill(&[0], None).enq().unwrap();
    cortex.areas_mut().by_key_mut(area_name).pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens.syns().src_col_v_offs.cmd().fill(&[0], None).enq().unwrap();

    cortex.areas_mut().by_key_mut(area_name).ssc_layer_mut(testbed::PRIMARY_SPATIAL_SSC_LAYER_NAME).unwrap().dens.cycle();
    cortex.areas_mut().by_key_mut(area_name).pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens.cycle();*/

        //#####  TRY THIS OUT SOMETIME  #####
    //let pyrs_input_len = cortex.areas_mut().by_key_mut(area_name).pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().len();
    //let mut vec_pyrs = iter::repeat(0).take().collect();
    //input_czar::vec_band_512_fill(&mut vec_pyrs);
    //let pyr_axn_ranges = cortex.areas_mut().by_key_mut(area_name).layer_input_ranges("iii", cortex.areas_mut().by_key_mut(area_name).pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens.syns().den_kind());
    //write_to_axons(axn_range, vec1);
    let vec1: Vec<u8> = iter::repeat(0).take(cortex.areas_mut().by_key_mut(area_name).unwrap().dims().columns() as usize).collect();

    // BRING BACK?
    // input_czar::sdr_stripes((cmn::SYNAPSE_SPAN_RHOMBAL_AREA as usize * 2), true, &mut vec1);

    println!("Primary Spatial Associative Layer...");
    //let psal_name = cortex.areas().by_key(area_name).ssc_layer(testbed::PRIMARY_SPATIAL_SSC_LAYER_NAME).unwrap().layer_name();
    //cortex.enqueue_write(area_name, psal_name, &vec1);
    cortex.areas_mut().by_key_mut(area_name).unwrap().ssc_layer_mut(testbed::PRIMARY_SPATIAL_SSC_LAYER_NAME).unwrap().soma().cmd().write(&vec1).offset(0).enq().unwrap();
    syn_and_den_states(&mut cortex.areas_mut().by_key_mut(area_name).unwrap().ssc_layer_mut(testbed::PRIMARY_SPATIAL_SSC_LAYER_NAME).unwrap().dens_mut());

    println!("Primary Temporal Associative Layer...");
    //let ptal_name = cortex.areas().by_key(area_name).pyr_layer(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().layer_name();
    //cortex.enqueue_write(area_name, ptal_name, &vec1);
    cortex.areas_mut().by_key_mut(area_name).unwrap().pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().soma().cmd().write(&vec1).offset(0).enq().unwrap();
    syn_and_den_states(&mut cortex.areas_mut().by_key_mut(area_name).unwrap().pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap().dens_mut());


    /////// [FIXME]: DISABLED DUE TO TUFT REDESIGNS:
    // pyr_preds(&mut cortex.areas_mut().by_key_mut(area_name).unwrap().pyr_layer_mut(testbed::PRIMARY_TEMPORAL_PYR_LAYER_NAME).unwrap());
}


// fn inhib(cortex: &mut Cortex) {

// }


// TEST PYRAMIDAL CELLS 'PREDICTIVENESS' AKA: SOMA STATES
// <<<<< TODO: NEEDS MASSIVE UPDATES TO PRETTY MUCH EVERY ASPECT >>>>>
// * TODO: Check every tuft.
fn pyr_preds(pyrs: &mut PyramidalLayer) {
    // let emsg = "\ntests::hybrid::test_pyr_preds()";

    io::stdout().flush().unwrap();
    pyrs.dens_mut().states().default_queue().unwrap().finish().unwrap();
    pyrs.dens_mut().states().cmd().fill(0, None).enq().unwrap();
    pyrs.dens_mut().states().default_queue().unwrap().finish().unwrap();

    // Currently looking at the first tuft only:
    let tft_id = 0;
    let dens_per_tuft = 1 << pyrs.dens().syns().tft_dims_by_tft()[tft_id].dens_per_tft_l2();

    println!("\n##### dens_per_tuft: {}", dens_per_tuft);
    //let dens_len = pyrs.dens_mut().states.len() as usize;
    let pyrs_len = pyrs.soma().len() as usize;
    let den_tuft_len = pyrs_len * dens_per_tuft;

    // WRITE `255` TO THE DENDRITES CORRESPONDING TO THE FIRST AND LAST CELL
    // FOR THE FIRST TUFT ONLY
    pyrs.dens_mut().states().default_queue().unwrap().finish().unwrap();
    pyrs.dens_mut().states().cmd().fill(255, Some(dens_per_tuft)).offset(0).enq().unwrap();
    pyrs.dens_mut().states().default_queue().unwrap().finish().unwrap();

    let last_cel_den_idz =  den_tuft_len - dens_per_tuft;

    println!("\n\nDEBUG: pyrs.dens_mut().states().len(): {}\n", pyrs.dens_mut().states().len());

    pyrs.dens_mut().states().default_queue().unwrap().finish().unwrap();
    pyrs.dens_mut().states().cmd().fill(255, Some(den_tuft_len - last_cel_den_idz))
        .offset(last_cel_den_idz).enq().unwrap();
    pyrs.dens_mut().states().default_queue().unwrap().finish().unwrap();

    // CYCLE THE PYRAMIDAL CELL ONLY, WITHOUT CYCLING IT'S DENS OR SYNS (WHICH WOULD OVERWRITE THE ABOVE)
    pyrs.cycle_solo();

    pyrs.soma().default_queue().unwrap().finish().unwrap();

    // READ THE PYRAMIDAL CELL SOMA STATES (PREDS)
    // pyrs.soma_mut().fill_vec();
    let mut soma_vec = vec![0u8; pyrs.soma().len()];
    pyrs.soma().cmd().read(&mut soma_vec).enq().unwrap();
    //pyrs.dens_mut().states.print_simple();
    //pyrs.soma_mut().print_simple();

    // TEST TO MAKE SURE THAT *ONLY* THE FIRST AND LAST CELL HAVE NON-ZERO VALUES
    for idx in 0..pyrs_len {
        //print!("([{}]:{})", i, pyrs.soma()[i]);
        if idx == 0 || idx == (pyrs_len - 1) {
            assert!(soma_vec[idx] > 0);
        } else {
            assert!(soma_vec[idx] == 0);
        }
    }

    println!("   test_pyr_preds(): {} ", super::PASS_STR);
}


fn syn_and_den_states(dens: &mut Dendrites) {
    // let emsg = "\ntests::hybrid::test_syn_and_den_states()";

    io::stdout().flush().unwrap();
    dens.syns_mut().src_col_v_offs().default_queue().unwrap().finish().unwrap();
    dens.syns_mut().src_col_v_offs().cmd().fill(0, None).enq().unwrap();
    dens.syns_mut().src_col_v_offs().default_queue().unwrap().finish().unwrap();

    dens.syns().cycle_solo();
    dens.states().default_queue().unwrap().finish().unwrap();
    dens.cycle_solo();
    dens.states().default_queue().unwrap().finish().unwrap();

    // let syns_per_tuft_l2: usize = dens.syns().dims().per_tft_l2_left() as usize;
    // let dens_per_tft_l2: usize = dens.dims().per_tft_l2_left() as usize;
    let tft_id = 0;
    let syns_per_den_l2 = dens.syns().tft_dims_by_tft()[tft_id].syns_per_den_l2();
    let dens_per_tft_l2 = dens.syns().tft_dims_by_tft()[tft_id].dens_per_tft_l2();
    let syns_per_tft_l2 = syns_per_den_l2 + dens_per_tft_l2;

    let cels_per_group: usize = cmn::SYNAPSE_SPAN_RHOMBAL_AREA as usize;
    let syns_per_group: usize = cels_per_group << syns_per_tft_l2;
    let dens_per_group: usize = cels_per_group << dens_per_tft_l2;
    let actv_group_thresh = syns_per_group / 4;
    //let den_actv_group_thresh = dens_per_group;

    //println!("Threshold: {}", actv_group_thresh);

    let mut cel_idz: usize = 0;
    let mut syn_states_ttl: usize;
    let mut den_states_ttl: usize;

    // dens.confab();

    let vec_dens_states = util::read_into_new_vec(dens.states());
    let vec_syns_states = util::read_into_new_vec(dens.syns().states());


    let mut test_failed: bool = false;

    loop {
        if (cel_idz + cels_per_group) > dens.dims().cells() as usize {
            break;
        }

        syn_states_ttl = 0;
        den_states_ttl = 0;

        let syn_idz = cel_idz << syns_per_tft_l2;
        let den_idz = cel_idz << dens_per_tft_l2;


        println!("\nDEBUG: syn_idz: {}, syns_per_tuft: {}, syns_per_group: {}",
            syn_idz, 1 << syns_per_tft_l2, syns_per_group);

        println!("DEBUG: dens.states().len(): {}, dens.syns().states().len(): {}",
            dens.states().len(), dens.syns().states().len());

        println!("DEBUG: vec_dens_states.len(): {}, vec_syns_states.len(): {}",
            vec_dens_states.len(), vec_syns_states.len());

        print!("\n");


        for syn_idx in syn_idz..(syn_idz + syns_per_group) {
            syn_states_ttl += (vec_syns_states[syn_idx] >> 7) as usize;
        }

        for den_idx in den_idz..(den_idz + dens_per_group) {
            den_states_ttl += (vec_dens_states[den_idx]) as usize;
        }

        if (cel_idz & 512) == 0 {
            println!("   -Inactive-");

            if (syn_states_ttl < actv_group_thresh) || (den_states_ttl < actv_group_thresh) {
                test_failed = true;
            }

            /*assert!(syn_states_ttl > actv_group_thresh);
            assert!(den_states_ttl > actv_group_thresh);*/

        } else {
            println!("   -Active-");

            if (syn_states_ttl > actv_group_thresh) || (den_states_ttl > actv_group_thresh) {
                test_failed = true;
            }

            /*assert!(syn_states_ttl < actv_group_thresh);
            assert!(den_states_ttl < actv_group_thresh);*/

        }

        println!("SYN [{} - {}]: {}", cel_idz, (cel_idz + cels_per_group - 1), syn_states_ttl);
        print!("   DEN [{} - {}]: {}", cel_idz, (cel_idz + cels_per_group - 1), den_states_ttl);

        io::stdout().flush().unwrap();

        cel_idz += cels_per_group;
    }

    assert!(test_failed);

    println!("   test_syn_and_den_states(): {} ", super::PASS_STR);
}

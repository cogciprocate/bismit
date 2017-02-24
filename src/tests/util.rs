#![allow(non_snake_case)]
use std::ops::{Range};
// use std::fmt::Display;

use ocl::Buffer;
use ocl::traits::{OclPrm, OclScl};
use cortex::{CorticalArea, CorticalAreaTest, DendritesTest, SynapsesTest, MinicolumnsTest};
use cmn::{self, DataCellLayer, DataCellLayerTest};

const PRINT_DETAILS: bool = false;

/*=============================================================================
===============================================================================
=================================== UTILITY ===================================
===============================================================================
=============================================================================*/

bitflags! {
    // #[derive(Debug)]
    pub flags PtalAlcoSwitches: u32 {
        const NONE                = 0b00000000,
        const ACTIVATE             = 0b00000001,
        const LEARN                = 0b00000010,
        const CYCLE             = 0b00000100,
        const OUTPUT              = 0b00001000,
        const ALL                 = 0xFFFFFFFF,
    }
}


// ACTIVATE, LEARN, CYCLE, & OUTPUT
// pub fn al_cycle_depricate(area: &mut CorticalArea) {
//     area.mcols().activate();
//     area.ptal_mut().learn();
//     area.ptal_mut().cycle(None);
//     area.mcols().output();
// }

/// ACTIVATE, LEARN, CYCLE, & OUTPUT
///
/// If `print` is true, will print a message and finish the queue before and
/// after each kernel.
///
// pub fn ptal_alco(area: &mut CorticalArea, activ: bool, learn: bool, cycle: bool, output: bool) {
pub fn ptal_alco(area: &mut CorticalArea, switches: PtalAlcoSwitches, print: bool) {

    if print { area.mcols().kern_activate().default_queue().unwrap().finish().unwrap(); }

    if switches.contains(ACTIVATE) {
        if print { printlnc!(yellow: "Activating..."); }
        area.mcols().activate_solo();
    }

    if print { area.mcols().kern_activate().default_queue().unwrap().finish().unwrap(); }

    if switches.contains(LEARN) {
        if print { printlnc!(yellow: "Learning..."); }
        area.ptal_mut().learn_solo();
    }

    if print { area.mcols().kern_activate().default_queue().unwrap().finish().unwrap(); }

    if switches.contains(CYCLE) {
        if print { printlnc!(yellow: "Cycling..."); }
        // area.ptal_cycle();
        area.ptal().dens().syns().cycle_solo();
        area.ptal().dens().cycle_solo();
        area.ptal().cycle_solo();
    }

    if print { area.mcols().kern_activate().default_queue().unwrap().finish().unwrap(); }

    if switches.contains(OUTPUT) {
        if print { printlnc!(yellow: "Outputting..."); }
        area.mcols().output_solo();
    }

    if print { area.mcols().kern_activate().default_queue().unwrap().finish().unwrap(); }

    area.finish_queues();

    if print { println!("Finishing queues..."); }
}


// pub fn confirm_syns(area: &mut CorticalArea, syn_range: &Range<usize>, state_neq: u8,
//         flag_set_eq: u8, strength_eq: i8)
// {
//     for syn_idx in syn_range.clone() {
//         area.ptal_mut().dens_mut().syns_mut().states.fill_vec();
//         area.ptal_mut().dens_mut().syns_mut().flag_sets.fill_vec();
//         area.ptal_mut().dens_mut().syns_mut().strengths.fill_vec();
//         assert!(area.ptal_mut().dens_mut().syns_mut().states[syn_idx] != state_neq);
//         assert!(area.ptal_mut().dens_mut().syns_mut().flag_sets[syn_idx] == flag_set_eq);
//         assert!(area.ptal_mut().dens_mut().syns_mut().strengths[syn_idx] == strength_eq);
//     }
// }


// pub fn assert_neq_range<T: OclPrm>(env: &Buffer<T>, idx_range: Range<usize>, val_neq: T) -> bool {
//     for idx in idx_range {
//         if env.read_idx_direct(idx) == val_neq { return false };
//     }

//     true
// }

// pub fn assert_eq_range<T: OclPrm>(env: &Buffer<T>, idx_range: Range<usize>, val_eq: T) -> bool {
//     for idx in idx_range.clone() {
//         if env.read_idx_direct(idx) != val_eq { return false };
//     }

//     true
// }

// ASSERT_RANGE():
// - [FIXME] TODO: Use env.read_direct and read the entire range at once into a Vec.
// - [FIXME] TODO: See if using an iterator (map?) function would be more idiomatic.
pub fn eval_range<T: OclPrm, F>(idx_range: Range<usize>, buf: &Buffer<T>, comparison: F) -> bool
    where F: Fn(T)-> bool
{
    let vec = read_idx_range_direct(idx_range.clone(), buf);

    for (idx, val) in vec.into_iter().enumerate() {
        // let val = read_idx_direct(idx, env);

        if !comparison(val) {
            println!("util::eval_range: The element at index: '{}' \
                has failed the provided comparison argument.", idx);
            return false
        };
    }

    true
}

pub fn read_idx_direct<T: OclPrm>(idx: usize, buf: &Buffer<T>) -> T {
    let mut val: [T; 1] = [Default::default()];
    buf.cmd().read(&mut val).offset(idx).enq().unwrap();
    // buf.default_queue().unwrap().finish().unwrap();
    val[0]
}

pub fn read_idx_range_direct<T: OclPrm>(idx_range: Range<usize>, buf: &Buffer<T>) -> Vec<T> {
    let mut vec = vec![Default::default(); idx_range.len()];
    buf.cmd().read(&mut vec).offset(idx_range.start).enq().unwrap();
    vec
}

// pub fn fill_vec<T: OclPrm>(buf: &Buffer<T>, vec: &mut Vec<T>) -> Event {
//     // let mut event = Event::new();
//     buf.cmd().read(vec).enq().unwrap();
// }

pub fn read_into_new_vec<T: OclPrm>(buf: &Buffer<T>) -> Vec<T> {
    let mut vec = vec![Default::default(); buf.len()];
    buf.cmd().read(&mut vec).enq().unwrap();
    vec
}


pub fn print_all(area: &mut CorticalArea, desc: &'static str) {
    //println!("\n - Confirm 1A - Activate");
    println!("{}", desc);
    area.print_axns();
    area.ptal().dens().syns().print_all();
    area.ptal().dens().print_all();
    area.ptal().print_all();
    area.mcols().print_all();
    area.print_aux();
}

pub fn print_all_syn_range(range: Range<usize>, area: &mut CorticalArea, desc: &'static str) {
    println!("{}", desc);
    area.print_axns();
    area.ptal().dens().syns().print_range(Some(range));
    area.ptal().dens().print_all();
    area.ptal().print_all();
    area.mcols().print_all();
    area.print_aux();
    unimplemented!();
}

// pub fn print_range(range: Range<usize>) -> String {
//     format!("{}..{}", range.start, range.end)
// }


pub fn compare_buffers<T: OclScl>(env1: &Buffer<T>, env2: &Buffer<T>) -> bool {
    if PRINT_DETAILS { print!("\nVector comparison:\n"); }
    assert!(env1.len() == env2.len());

    // env1.fill_vec();
    let vec1 = read_into_new_vec(env1);
    // env2.fill_vec();
    let vec2 = read_into_new_vec(env2);

    let mut failure = false;

    for i in 0..vec1.len() {
        let (e1_val, e2_val) = (vec1[i], vec2[i]);

        if e1_val != e2_val {
            failure = true;
            if PRINT_DETAILS { print!("{}", cmn::C_RED); }
        } else {
            if PRINT_DETAILS { print!("{}", cmn::C_DEFAULT); }
        }

        if PRINT_DETAILS { print!("[n:{}, v4:{}]{}", e1_val, e2_val, cmn::C_DEFAULT); }
    }

    if PRINT_DETAILS { print!("\n"); }

    failure
}


// TEST_NEARBY(): Ensure that elements near a focal index are equal to a particular value.
//        - idz and idm (first and last elements) are also checked along with their nearby elements
// <<<<< [FIXME] TODO: THIS FUNCTION NEEDS SERIOUS STREAMLINING & OPTIMIZATION >>>>>
pub fn eval_others<T: OclScl>(env: &Buffer<T>, foc_idx: usize, other_val: T) {    // -> Result<(), &'static str>
    // let mut checklist = Vec::new();
    let check_margin = 384;

    // assert!(env[foc_idx] == foc_val);

    // index[0]
    // let idz = 0;
    // index[n]
    let idn = env.len();

    assert!(idn > 0);
    assert!(foc_idx < idn);

    // env.fill_vec();
    let vec = read_into_new_vec(env);

    if idn <= check_margin * 4 {
        // CHECK THE WHOLE LIST (except for foc_idx)
        unimplemented!();
    } else {
        let start_mrg = check_margin;
        let start_mrg_2 = check_margin * 2;

        let end_mrg = idn - check_margin;
        let end_mrg_2 = idn - (check_margin * 2);

        let foc_idx_l = if foc_idx >= start_mrg_2 {
            foc_idx - check_margin
        } else if foc_idx >= start_mrg {
            start_mrg
        } else {
            foc_idx
        };

        let foc_idx_r = if foc_idx < end_mrg_2 {
            foc_idx + check_margin
        } else if foc_idx < end_mrg {
            end_mrg
        } else {
            foc_idx
        };

        let iter = (0usize..start_mrg)             // start of list
            .chain(foc_idx_l..foc_idx)            // left of foc_idx
            .chain(foc_idx..foc_idx_r)            // right of foc_idx
            .chain(end_mrg..idn)                // end of list
            .filter(|&i| i != foc_idx);            // filter foc_idx itself from list

        for i in iter {
            // debug_assert!(i != foc_idx);
            // checklist.push(i);
            assert_eq!(vec[i], other_val);
        }

        // println!("\n##### checklist: {:?} len: {}", checklist, checklist.len());
    }

}


#[test]
fn eval_others_UNIMPLEMENTED() {

}



// let foc_idx_l = match foc_idx {
        //     idz...start_mrg => foc_idx,
        //     start_mrg...start_mrg_2 => start_mrg,
        //     _ => foc_idx - start_mrg,
        // };

        // let foc_idx_r = match foc_idx {
        //     end_mrg...idn => foc_idx,
        //     end_mrg_2...end_mrg => end_mrg,
        //     _ => foc_idx + check_margin,
        // };

use std::ops::Range;
// use rand;

use cmn::{self, CorticalDims, DataCellLayer};
use map::{AreaMap};
use ocl::{self, ProQue, SpatialDims, Buffer, EventList, Result as OclResult};
use ocl::traits::OclPrm;
use cortex::{AxonSpace, PyramidalLayer, SpinyStellateLayer};

// #[cfg(test)]
// pub use self::tests::{MinicolumnsTest};


pub struct Minicolumns {
    dims: CorticalDims,
    aff_out_axn_slc: u8,
    aff_out_axn_idz: u32,
    ff_layer_axn_idz: usize,
    kern_output: ocl::Kernel,
    kern_activate: ocl::Kernel,
    // rng: rand::XorShiftRng,
    pub flag_sets: Buffer<u8>,
    pub best_den_states: Buffer<u8>,
}

impl Minicolumns {
    pub fn new(dims: CorticalDims, area_map: &AreaMap, axons: &AxonSpace,
                ssts: &SpinyStellateLayer, pyrs: &PyramidalLayer, ocl_pq: &ProQue
            ) -> Minicolumns
    {
        assert!(dims.depth() == 1);
        assert!(dims.v_size() == pyrs.dims().v_size() && dims.u_size() == pyrs.dims().u_size());

        // UPDATE ME TO AREA_MAP SETUP
        let ff_layer_axn_idz = ssts.axn_range().start;
        let pyr_depth = area_map.ptal_layer().depth();

        println!("{mt}{mt}MINICOLUMNS::NEW() dims: {:?}, pyr_depth: {}", dims, pyr_depth, mt = cmn::MT);

        let flag_sets = Buffer::<u8>::new(ocl_pq.queue().clone(), None, &dims, None).unwrap();
        let best_den_states = Buffer::<u8>::new(ocl_pq.queue().clone(), None, &dims, None).unwrap();

        // [FIXME]: TEMPORARY?:
        // [FIXME]: MAKE THIS CONSISTENT WITH 'aff_out_slc_range()':
        assert!(area_map.aff_out_slcs().len() == 1,
            "Afferent output slices currently limited to a maximum of 1.");

        let aff_out_axn_slc = area_map.aff_out_slcs()[0];
        let aff_out_axn_idz = area_map.axn_idz(aff_out_axn_slc);
        let pyr_lyr_axn_idz = area_map.axn_idz(pyrs.base_axn_slc());

        let kern_activate = ocl_pq.create_kernel("mcol_activate_pyrs").expect("[FIXME]: HANDLE ME")
            // .expect("Minicolumns::new()")
            .gws(SpatialDims::Three(pyrs.dims().depth() as usize, dims.v_size() as usize,
                dims.u_size() as usize))
            .arg_buf(&flag_sets)
            .arg_buf(&best_den_states)
            .arg_buf(&pyrs.tft_best_den_states())
            .arg_scl(ff_layer_axn_idz as u32)
            .arg_scl(pyr_lyr_axn_idz)
            // .arg_scl(pyrs.cell_scheme().dens_per_tft_l2)
            .arg_buf(&pyrs.flag_sets())
            .arg_buf(&pyrs.states())
            .arg_buf_named::<i32>("aux_ints_0", None)
            // .arg_buf_named::<i32>("aux_ints_1", None)
            .arg_buf(&axons.states);


        let kern_output = ocl_pq.create_kernel("mcol_output").expect("[FIXME]: HANDLE ME")
            // .expect("Minicolumns::new()")
            .gws(SpatialDims::Two(dims.v_size() as usize, dims.u_size() as usize))
            .arg_buf(&pyrs.soma())
            // .arg_scl(pyrs.tfts_per_cel())
            .arg_scl(ff_layer_axn_idz as u32)
            .arg_scl(pyr_depth)
            .arg_scl(aff_out_axn_slc)
            .arg_buf(&pyrs.tft_best_den_states())
            .arg_buf(&flag_sets)
            .arg_buf(&best_den_states)
            .arg_buf(&axons.states);


        Minicolumns {
            dims: dims,
            aff_out_axn_slc: aff_out_axn_slc,
            aff_out_axn_idz: aff_out_axn_idz,
            ff_layer_axn_idz: ff_layer_axn_idz,
            kern_output: kern_output,
            kern_activate: kern_activate,
            // rng: rand::weak_rng(),
            flag_sets: flag_sets,
            best_den_states: best_den_states,
        }
    }

    // <<<<< TODO: DEPRICATE >>>>>
    pub fn set_arg_buf_named<T: OclPrm>(&mut self, name: &'static str, env: &Buffer<T>)
            -> OclResult<()>
    {
        let activate_using_aux = true;
        let output_using_aux = false;

        if activate_using_aux {
            try!(self.kern_activate.set_arg_buf_named(name, Some(env)));
        }

        if output_using_aux {
            try!(self.kern_output.set_arg_buf_named(name, Some(env)));
        }

        Ok(())
    }

    #[inline]
    pub fn activate(&self) {
        self.kern_activate.enq().expect("[FIXME]: HANDLE ME!");
    }

    pub fn output(&self, new_events: Option<&mut EventList>) {
        match new_events {
            Some(ne) => {
                ne.clear_completed().expect("Minicolumns::output");
                // self.kern_output.enqueue_events(None, Some(ne))
                //     .expect("bismit::Minicolumns::output");

                self.kern_output.cmd().enew(ne).enq()
                    .expect("bismit::Minicolumns::output");
            },

            None => self.kern_output.enq().expect("[FIXME]: HANDLE ME!"),
        }
    }

    // pub fn confab(&mut self) {
    //     self.flag_sets.fill_vec();
    //     self.best_den_states.fill_vec();
    // }

    #[inline]
    pub fn ff_layer_axn_idz(&self) -> usize {
        self.ff_layer_axn_idz
    }

    // [FIXME]: CONVERT TO A RANGE (area_map.aff_out_slc_range)
    #[inline]
    pub fn aff_out_axn_slc(&self) -> u8 {
        self.aff_out_axn_slc
    }

    // AXN_OUTPUT_RANGE(): USED FOR TESTING / DEBUGGING PURPOSES
    pub fn aff_out_axn_range(&self) -> Range<usize> {
        self.aff_out_axn_idz as usize..self.aff_out_axn_idz as usize + self.dims.columns() as usize
    }
}


// #[cfg(test)]
// pub mod tests {
//     use std::ops::Range;
//     use super::Minicolumns;

//     pub trait MinicolumnsTest {
//         fn print_range(&mut self, range: Range<usize>);
//         fn print_all(&mut self);
//     }

//     impl MinicolumnsTest for Minicolumns {
//         fn print_range(&mut self, range: Range<usize>) {
//             print!("mcols.flag_sets: ");
//             self.flag_sets.print(1 << 0, Some((0, 255)),
//                 Some(range.clone()), false);

//             print!("mcols.best_den_states: ");
//             self.best_den_states.print(1 << 0, Some((0, 255)),
//                 Some(range.clone()), false);
//         }

//         fn print_all(&mut self) {
//             let range = 0..self.flag_sets.len();
//             self.print_range(range);
//         }
//     }

// }

#![allow(dead_code)]

use ocl::{Kernel, ProQue, SpatialDims, Buffer,};
use cmn::{ParaHexArray, Sdr};
use area::AxonSpace;
use map::{self, AreaMap};

pub struct SensoryFilter {
    filter_name: String,
    cl_file_name: Option<String>,
    area_name: &'static str,
    //dims: CorticalDims,
    input: Buffer<u8>,
    kern_cycle: Kernel,
}

impl SensoryFilter {
    pub fn new(
                filter_name: String, 
                cl_file_name: Option<String>, 
                area_map: &AreaMap,
                //area_name: &'static str,
                //dims: CorticalDims, 
                axns: &AxonSpace,
                //base_axn_slc: u8,
                ocl_pq: &ProQue, 
            ) -> SensoryFilter 
    {
        let layer_flags = map::FF_IN;
        // [NOTE]: Combine this with the call to `::slc_src_layer_dims` below:
        // let axn_slc_ranges = area_map.layers().layers_containing_tags_slc_range(layer_flags);
        // assert!(axn_slc_ranges.len() == 1);
        // assert!(axn_slc_ranges[0].len() == 1);
        // let axn_slc_range = axn_slc_ranges[0].clone();

        // let dims = area_map.slc_src_layer_dims(axn_slc_range.start, layer_flags).expect(&format!(
        //     "SensoryFilter::new(): No source slice layer with axon slice range: '{:?}' and \
        //     flags: '{:?}' found.", axn_slc_range, layer_flags));


        let layers = area_map.layers().layers_containing_tags(layer_flags);
        assert!(layers.len() == 1);
        let layer = layers[0];
        let axn_slc_range = layer.slc_range().expect("SensoryFilter::new(): \
            Invalid slice range.").clone();

        assert!(layers[0].sources().len() == 1);
        let ref src_layer = layers[0].sources()[0];
        let dims = src_layer.dims();
        
        assert!(dims.depth() == 1, "\nAfferent input layer depths of more than one for cortical \
            areas with sensory filters are not yet supported. Please set the depth of any \
            afferent input layers with filters to 1.");

        let input = Buffer::<u8>::new(ocl_pq.queue(), None, &dims, None).unwrap();        

        let kern_cycle = ocl_pq.create_kernel(&filter_name.clone()).expect("[FIXME]: HANDLE ME")
            // .expect("SensoryFilter::new()")
            .gws(SpatialDims::Three(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
            .lws(SpatialDims::Three(1, 8, 8 as usize))
            .arg_buf(&input)
            .arg_scl(axn_slc_range.start)
            .arg_buf(&axns.states);

        SensoryFilter {
            filter_name: filter_name,
            cl_file_name: cl_file_name,
            area_name: area_map.area_name(),
            //dims: dims,
            input: input,
            kern_cycle: kern_cycle,
        }
    }

    pub fn write(&self, sdr: &Sdr) {
        assert!(sdr.len() <= self.input.len());
        self.input.write(sdr).enq().expect("[FIXME]: HANDLE ME!");
    }

    pub fn cycle(&self) {
        //println!("Printing {} for {}:\n", &self.filter_name, self.area_name);
        self.kern_cycle.enq().expect("[FIXME]: HANDLE ME!");
    }
}

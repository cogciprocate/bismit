use std::fmt::Display;
use std::ops::Range;
// use std::collections::HashMap;
// use std::collections::{BTreeMap};

use ocl::builders::{BuildOpt, ProgramBuilder};
use map::{LayerMapSchemeList, AreaSchemeList, AreaScheme, LayerMapKind, FilterScheme,
    DendriteKind};
use cmn::{self, CorticalDims, MapStore};
use map::{self, SliceMap, LayerTags, LayerMap, LayerInfo};
use thalamus::ExternalPathway;


#[derive(Clone)]
pub struct AreaMap {
    area_name: &'static str,
    dims: CorticalDims,
    slices: SliceMap,
    layers: LayerMap,
    hrz_demarc: u8,
    eff_areas: Vec<&'static str>,
    aff_areas: Vec<&'static str>,
    filters: Option<Vec<FilterScheme>>,
}

impl AreaMap {
    pub fn new(pamap: &AreaScheme, layer_map_sl: &LayerMapSchemeList, area_sl: &AreaSchemeList,
            ext_paths: &MapStore<String, (ExternalPathway, Vec<LayerTags>)>) -> AreaMap
    {
        println!("\n{mt}AREAMAP::NEW(): Area: \"{}\", eff areas: {:?}, aff areas: {:?}", pamap.name,
            pamap.get_eff_areas(), pamap.get_aff_areas(), mt = cmn::MT);

        let layers = LayerMap::new(pamap, layer_map_sl, area_sl, ext_paths);

        let dims = pamap.dims().clone_with_depth(layers.depth());

        let slices = SliceMap::new(&dims, &layers);
        slices.print_debug();

        AreaMap {
            area_name: pamap.name,
            dims: dims,
            slices: slices,
            layers: layers,
            // [FIXME]: TEMPORARY:
            hrz_demarc: 128,
            eff_areas: pamap.get_eff_areas().clone(),
            aff_areas: pamap.get_aff_areas().clone(),
            filters: pamap.filters.clone(),
        }
    }

    // ADD OPTION FOR MORE CUSTOM KERNEL FILES OR KERNEL LINES
    pub fn gen_build_options(&self) -> ProgramBuilder {
        let mut build_options = cmn::base_build_options()
            .cmplr_def("HORIZONTAL_AXON_ROW_DEMARCATION", self.hrz_demarc as i32)
            .cmplr_def("AXN_SLC_COUNT", self.slices.depth() as i32)
            .cmplr_def("SLC_SCL_COEFF_L2", cmn::SLC_SCL_COEFF_L2)
            .bo(BuildOpt::include_def("AXN_SLC_IDZS", literal_list(self.slices.axn_idzs())))
            .bo(BuildOpt::include_def("AXN_SLC_V_SIZES", literal_list(self.slices.v_sizes())))
            .bo(BuildOpt::include_def("AXN_SLC_U_SIZES", literal_list(self.slices.u_sizes())))
            .bo(BuildOpt::include_def("AXN_SLC_V_SCALES", literal_list(self.slices.v_scales())))
            .bo(BuildOpt::include_def("AXN_SLC_U_SCALES", literal_list(self.slices.u_scales())))
            .bo(BuildOpt::include_def("AXN_SLC_V_MIDS", literal_list(self.slices.v_mids())))
            .bo(BuildOpt::include_def("AXN_SLC_U_MIDS", literal_list(self.slices.u_mids())))
        ;

        // Custom filter kernels
        match self.filters {
            Some(ref filter_schemes) => {
                for pf in filter_schemes.iter() {
                    match pf.cl_file_name() {
                        Some(ref clfn)  => {
                            build_options = build_options.src_file(clfn.clone());
                        },
                        None => (),
                    }
                }
            },
            None => (),
        };

        cmn::load_builtin_kernel_source(build_options)
    }

    // NEW
    pub fn layer_name_by_tags(&self, layer_tags: LayerTags) -> &'static str {
        let layer_info = self.layers.layers_meshing_tags(layer_tags);
        assert_eq!(layer_info.len(), 1);
        layer_info[0].name()
    }

    // UPDATE / DEPRICATE
    /// Returns a grouped list of source layer names for each distal dendritic tuft in a layer.
    pub fn layer_dst_srcs(&self, layer_name: &'static str) -> Vec<Vec<&'static str>> {
        let potential_tufts = match self.layers.layer_info_by_name(layer_name) {
            Some(li) => li.dst_src_lyrs(),
            None => panic!("AreaMap::layer_dst_srcs(): No layer named '{}' found.", layer_name),
        };

        let mut valid_tufts: Vec<Vec<&'static str>> = Vec::with_capacity(potential_tufts.len());

        for mut potential_tuft_src_lyrs in potential_tufts {
            let mut valid_src_lyrs = Vec::with_capacity(potential_tuft_src_lyrs.len());

            for lyr_name in potential_tuft_src_lyrs.drain(..) {
                let li = match self.layers.layer_info_by_name(lyr_name) {
                    Some(li) => li,
                    None => panic!("AreaMap::layer_dst_srcs(): No layer named '{}' found.",
                        layer_name),
                };

                if li.depth() > 0 {
                    valid_src_lyrs.push(lyr_name);
                }
            }

            if valid_src_lyrs.len() > 0 {
                valid_src_lyrs.shrink_to_fit();
                valid_tufts.push(valid_src_lyrs);
            }
        }

        valid_tufts
    }

    // NEW - UPDATE
    /// Returns a merged list of slice ids for all source layers.
    pub fn layer_slc_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
        let mut slc_ids = Vec::with_capacity(32);

        for layer_name in layer_names.iter() {
            let li = match self.layers.layer_info_by_name(layer_name) {
                Some(li) => li,
                None => panic!("AreaMap::layer_slc_ids(): No layer named '{}' found.",
                    layer_name),
            };

            if let Some(slc_range) = li.slc_range() {
                for i in slc_range.clone() {
                    slc_ids.push(i);
                }
            }
        }

        slc_ids
    }

    // NEW - UPDATE
    /// Returns a merged list of source slice ids for all source layers.
    pub fn layer_src_slc_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
        let li = match self.layers.layer_info_by_name(layer_name) {
            Some(li) => li,
            None => panic!("AreaMap::layer_src_slc_ids(): No layer named '{}' found.",
                layer_name),
        };

        let src_lyr_names = li.src_lyr_names(den_type);
        self.layer_slc_ids(src_lyr_names)
     }

     // NEW - UPDATE
     /// Returns a grouped list of source slice ids for each distal dendritic tuft in a layer.
     pub fn layer_dst_src_slc_ids(&self, layer_name: &'static str) -> Vec<Vec<u8>> {
         let src_tufts = self.layer_dst_srcs(layer_name);
         let mut dst_src_slc_ids = Vec::with_capacity(src_tufts.len());

         for tuft in src_tufts {
             dst_src_slc_ids.push(self.layer_slc_ids(tuft));
        }

        dst_src_slc_ids
    }

    // NEW - UPDATE / RENAME
    pub fn aff_out_slcs(&self) -> Vec<u8> {
        let mut output_slcs: Vec<u8> = Vec::with_capacity(8);

         // Push all matching slices:
         for layer in self.layers.iter() {
             if (layer.tags() & map::FF_OUT) == map::FF_OUT {
                 let v = self.layer_slc_ids(vec![layer.name()]);
                 output_slcs.extend_from_slice(&v);
             }
         }

         output_slcs.shrink_to_fit();

         // Ensure that the slice id list contains contiguous slice ids:
         for i in 0..output_slcs.len() {
             if i > 0 {
                 unsafe { debug_assert!(*output_slcs.get_unchecked(i - 1)
                     == *output_slcs.get_unchecked(i) - 1); }
            }
        }

        output_slcs
    }

    // NEW NEW NEW
    /// Returns the slice range of the afferent output axon slices (FF_OUT).
    pub fn aff_out_slc_range(&self) -> Range<u8> {
        let aff_out_slcs = self.aff_out_slcs();
        let idz = 0;
        let idn = aff_out_slcs.len() - 1;
        aff_out_slcs[idz]..(aff_out_slcs[idn] + 1)
    }

    // /// Returns the base slc_ids.
    // pub fn axn_base_slc_ids_by_tags(&self, layer_tags: LayerTags) -> Vec<u8> {
    //     // let layers = self.layers.layers_containing_tags(layer_tags);
    //     // let mut slc_ids = Vec::with_capacity(layers.len());

    //     // for &layer in layers.iter() {
    //     //     slc_ids.push(layer.slc_range().start);
    //     // }

    //     // slc_ids
    //     self.layers.layers_containing_tags(layer_tags).iter()
    //         // .filter(|l| l.slc_range().len() > 0)
    //         .map(|l| l.slc_range().start)
    //         .collect()
    // }

    // // UPDATE / DEPRICATE / MERGE WITH ABOVE
    // pub fn output_layer_info(&self) -> Vec<(LayerTags, u32)> {
    //     let layers = self.layers.layer_info(map::OUTPUT);
    //     let mut layer_info = Vec::with_capacity(layers.len());

    //     for &layer in layers.iter() {
    //         layer_info.push((layer.tags(), self.dims.columns()));
    //     }

    //     layer_info
    // }

    // // [TODO]: UPDATE / DEPRICATE / MERGE WITH ABOVE (axn_base_slc_ids_by_tags):
    // pub fn output_layer_info(&self) -> Vec<&LayerInfo> {
    //     self.layers.layers_containing_tags(map::OUTPUT)
    // }


    // NEW
    pub fn psal_layer(&self) -> &LayerInfo {
        let psal_layer_vec = self.layers.layers_containing_tags(map::PSAL);
        assert_eq!(psal_layer_vec.len(), 1);
        psal_layer_vec[0]
     }

     // NEW
     #[inline]
     pub fn ptal_layer(&self) -> &LayerInfo {
        let ptal_layer_vec = self.layers.layers_containing_tags(map::PTAL);
        assert_eq!(ptal_layer_vec.len(), 1);
        ptal_layer_vec[0]
     }

    // NEW
    pub fn axn_range_meshing_tags(&self, layer_tags: LayerTags) -> Option<Range<u32>> {
        let layers = self.layers.layers_meshing_tags(layer_tags);

        if layers.len() == 1 {
            let layer = layers[0];

            if layer.slc_range().is_some() {
                let base_slc_id = layers[0].slc_range().unwrap().start;
                let layer_idz = self.axn_idz(base_slc_id);
                let layer_len = layers[0].axn_count();

                debug_assert!({
                        let slc_idm = base_slc_id + layers[0].depth() - 1;
                        let slc_len = self.slices.slc_axn_count(slc_idm);
                        let axn_idz = self.axn_idz(slc_idm);
                        let axn_idn = axn_idz + slc_len;
                        // println!("\n\n# (layer_idz, layer_len) = ({}, {}), axn_idn = {}, \
                        //     slc_len = {}, axn_idz = {}, \n# layer: {:?}\n",
                        //     layer_idz, layer_len, axn_idn, slc_len, axn_idz, layers[0]);
                        (layer_idz + layer_len) == axn_idn
                    }, "AreaMap::axn_range(): Axon index mismatch.");

                Some(layer_idz..(layer_idz + layer_len))
            } else {
                None
            }
        } else if layers.len() == 0 {
            None
        } else {
            panic!("AreaMap::axn_range_meshing_tags(): Multiple layers matching \
                flags: '{}' for area: '{}'. Layers: {:?}", layer_tags, self.area_name, layers);
        }
    }

    // // [TEMPORARY] - REMOVE ME ASAP
    // pub fn axn_range_containing_tags(&self, layer_tags: LayerTags) -> Option<Range<u32>> {
    //     let layers = self.layers.layers_containing_tags(layer_tags);

    //     if layers.len() == 1 {
    //         let layer_idz = self.axn_idz(layers[0].slc_range().start);
    //         let layer_len = layers[0].axn_count();

    //         debug_assert!({
    //                 let slc_idm = layers[0].slc_range().start + layers[0].depth() - 1;
    //                 let slc_len = self.slices.slc_axn_count(slc_idm);
    //                 let axn_idz = self.axn_idz(slc_idm);
    //                 let axn_idn = axn_idz + slc_len;
    //                 // println!("\n\n# (layer_idz, layer_len) = ({}, {}), axn_idn = {}, \
    //                 //     slc_len = {}, axn_idz = {}, \n# layer: {:?}\n",
    //                 //     layer_idz, layer_len, axn_idn, slc_len, axn_idz, layers[0]);
    //                 (layer_idz + layer_len) == axn_idn
    //             }, "AreaMap::axn_range(): Axon index mismatch.");

    //         Some(layer_idz..(layer_idz + layer_len))
    //     } else if layers.len() == 0 {
    //         None
    //     } else {
    //         panic!("AreaMap::axn_range_meshing_tags(): Internal error. Multiple layers matching \
    //             flags: '{}' found.", layer_tags);
    //     }
    // }

    // NEW
    pub fn slc_src_layer_dims(&self, slc_id: u8, layer_tags: LayerTags) -> Option<&CorticalDims> {
        self.layers.slc_src_layer_info(slc_id, layer_tags).map(|sli| sli.dims())
    }

    // DEPRICATE
    pub fn aff_areas(&self) -> &Vec<&'static str> {
        &self.aff_areas
    }

    // DEPRICATE
    pub fn eff_areas(&self) -> &Vec<&'static str> {
        &self.eff_areas
    }

    pub fn area_name(&self) -> &'static str {
        self.area_name
    }

    pub fn axn_idz(&self, slc_id: u8) -> u32 {
        self.slices.idz(slc_id)
    }

    pub fn slices(&self) -> &SliceMap {
        &self.slices
    }

    pub fn layers(&self) -> &LayerMap {
        &self.layers
    }

    // UPDATE / DEPRICATE
    pub fn filters(&self) -> &Option<Vec<FilterScheme>> {
        &self.filters
    }

    pub fn dims(&self) -> &CorticalDims {
        &self.dims
    }

    // UPDATE / DEPRICATE
    pub fn lm_kind_tmp(&self) -> &LayerMapKind {
        &self.layers.region_kind()
    }
}


pub fn literal_list<T: Display>(vec: &Vec<T>) -> String {
    let mut literal = String::with_capacity((vec.len() * 5) + 20);

    let mut i = 0u32;
    for ele in vec.iter() {
        if i != 0 {
            literal.push_str(", ");
        }

        literal.push_str(&ele.to_string());
        i += 1;
    }

    literal
}


#[cfg(test)]
pub mod tests {
    use cmn;
    use std::fmt::{Display, Formatter, Result as FmtResult};
    use super::{AreaMap};

    pub trait AreaMapTest {
        fn axn_idx(&self, slc_id: u8, v_id: u32, v_ofs: i8, u_id: u32, u_ofs: i8)
                -> Result<u32, &'static str>;
        fn axn_col_id(&self, slc_id: u8, v_id_unscaled: u32, v_ofs: i8, u_id_unscaled: u32, u_ofs: i8)
                -> Result<u32, &'static str>;
    }

    impl AreaMapTest for AreaMap {
        /* AXN_IDX(): Some documentation for this can be found in bismit.cl
                 Basically all we're doing is scaling up or down the v and u coordinates based on a predetermined scaling factor. The scaling factor only applies when a foreign cortical area is a source for the axon's slice AND is a different size than the local cortical area. The scale factor is based on the relative size of the two areas. Most of the time the scaling factor is 1:1 (scale factor of 16). The algorithm below for calculating an axon index is the same as the one in the kernel and gives precisely the same results.
        */
        fn axn_idx(&self, slc_id: u8, v_id_unscaled: u32, v_ofs: i8, u_id_unscaled: u32, u_ofs: i8)
                -> Result<u32, &'static str>
        {
            let v_scale = self.slices.v_scales()[slc_id as usize];
            let u_scale = self.slices.u_scales()[slc_id as usize];

            // let v_id_scaled = (v_id_unscaled * v_scale) >> cmn::SLC_SCL_COEFF_L2;
            // let u_id_scaled = (u_id_unscaled * u_scale) >> cmn::SLC_SCL_COEFF_L2;

            let v_id_scaled = cmn::scale(v_id_unscaled as i32, v_scale);
            let u_id_scaled = cmn::scale(u_id_unscaled as i32, u_scale);

            let slc_count = self.slices().depth();
            let v_size = self.slices.v_sizes()[slc_id as usize];
            let u_size = self.slices.u_sizes()[slc_id as usize];

            if coords_are_safe(slc_count, slc_id, v_size, v_id_scaled as u32, v_ofs,
                    u_size, u_id_scaled as u32, u_ofs) {
                Ok(axn_idx_unsafe(self.axn_idz(slc_id), v_id_scaled as u32, v_ofs,
                    u_size, u_id_scaled as u32, u_ofs))
            } else {
                Err("Axon coordinates invalid.")
            }
        }

        fn axn_col_id(&self, slc_id: u8, v_id_unscaled: u32, v_ofs: i8, u_id_unscaled: u32, u_ofs: i8)
                -> Result<u32, &'static str>
        {
            let v_scale = self.slices.v_scales()[slc_id as usize];
            let u_scale = self.slices.u_scales()[slc_id as usize];

            // let v_id_scaled = (v_id_unscaled * v_scale) >> cmn::SLC_SCL_COEFF_L2;
            // let u_id_scaled = (u_id_unscaled * u_scale) >> cmn::SLC_SCL_COEFF_L2;

            let v_id_scaled = cmn::scale(v_id_unscaled as i32, v_scale);
            let u_id_scaled = cmn::scale(u_id_unscaled as i32, u_scale);

            let v_size = self.slices.v_sizes()[slc_id as usize];
            let u_size = self.slices.u_sizes()[slc_id as usize];

            // Make sure v and u are safe (give fake slice info to coords_are_safe()):
            if coords_are_safe(1, 0, v_size, v_id_scaled as u32, v_ofs,
                    u_size, u_id_scaled as u32, u_ofs) {
                // Give a fake, zero idz (since this is a column id we're returning):
                Ok(axn_idx_unsafe(0, v_id_scaled as u32, v_ofs,
                    u_size, u_id_scaled as u32, u_ofs))
            } else {
                Err("Axon coordinates invalid.")
            }
        }

    }

    impl Display for AreaMap {
        fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
            write!(fmtr, "area slices: {}", self.slices)
        }
    }

    pub fn coords_are_safe(slc_count: u8, slc_id: u8, v_size: u32, v_id: u32, v_ofs: i8,
            u_size: u32, u_id: u32, u_ofs: i8
        ) -> bool
    {
        (slc_id < slc_count) && coord_is_safe(v_size, v_id, v_ofs)
            && coord_is_safe(u_size, u_id, u_ofs)
    }

    pub fn coord_is_safe(dim_size: u32, coord_id: u32, coord_ofs: i8) -> bool {
        let coord_ttl = coord_id as i64 + coord_ofs as i64;
        (coord_ttl >= 0) && (coord_ttl < dim_size as i64)
    }

    pub fn axn_idx_unsafe(idz: u32, v_id: u32, v_ofs: i8, u_size: u32, u_id: u32, u_ofs: i8) -> u32 {
        let v = v_id as i64 + v_ofs as i64;
        let u = u_id as i64 + u_ofs as i64;
        (idz as i64 + (v * u_size as i64) + u) as u32
    }
}

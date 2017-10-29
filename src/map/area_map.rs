use std::fmt::Display;
use std::ops::{Range, Deref};
use ocl::builders::{BuildOpt, ProgramBuilder};
use cmn::{self, CorticalDims, MapStore, CmnResult};
use ::InputGenerator;
use map::{SliceMap, LayerTags, LayerMap, LayerInfo, LayerAddress, LayerMapSchemeList,
    AreaSchemeList, AreaScheme, LayerMapKind, FilterScheme, AxonTags, InputTrack};
use subcortex::Subcortex;


#[derive(Clone)]
pub struct AreaMap {
    area_id: usize,
    area_name: &'static str,
    dims: CorticalDims,
    slice_map: SliceMap,
    layer_map: LayerMap,
    eff_areas: Vec<&'static str>,
    aff_areas: Vec<&'static str>,
    other_areas: Vec<(&'static str, Option<Vec<(AxonTags, AxonTags)>>)>,
    filter_chain_schemes: Vec<(InputTrack, AxonTags, Vec<FilterScheme>)>,
}

impl AreaMap {
    pub fn new(area_id: usize, area_sch: &AreaScheme, layer_map_sl: &LayerMapSchemeList,
            area_sl: &AreaSchemeList, _ext_paths: &MapStore<String, (InputGenerator, Vec<LayerAddress>)>,
            subcortex: &Subcortex)
            -> CmnResult<AreaMap>
    {
        println!("\n{mt}AREAMAP::NEW(): Area: \"{}\", eff areas: {:?}, aff areas: {:?}", area_sch.name(),
            area_sch.get_eff_areas(), area_sch.get_aff_areas(), mt = cmn::MT);

        let layer_map = LayerMap::new(area_sch, layer_map_sl, area_sl, _ext_paths, subcortex)?;

        let dims = area_sch.dims().clone_with_depth(layer_map.depth());

        let slice_map = SliceMap::new(&dims, &layer_map);
        slice_map.print_debug();

        Ok(AreaMap {
            area_id: area_id,
            area_name: area_sch.name(),
            dims: dims,
            slice_map: slice_map,
            layer_map: layer_map,
            eff_areas: area_sch.get_eff_areas().clone(),
            aff_areas: area_sch.get_aff_areas().clone(),
            other_areas: area_sch.get_other_areas().clone(),
            filter_chain_schemes: area_sch.filter_chains().clone(),
        })
    }

    // ADD OPTION FOR MORE CUSTOM KERNEL FILES OR KERNEL LINES
    pub fn gen_build_options(&self) -> ProgramBuilder {
        let mut build_options = cmn::base_build_options()
            .cmplr_def("AXN_SLC_COUNT", self.slice_map.depth() as i32)
            .cmplr_def("SLC_SCL_COEFF_L2", cmn::SLC_SCL_COEFF_L2)
            .bo(BuildOpt::include_def("AXN_SLC_IDZS", literal_list(self.slice_map.axn_idzs())))
            .bo(BuildOpt::include_def("AXN_SLC_V_SIZES", literal_list(self.slice_map.v_sizes())))
            .bo(BuildOpt::include_def("AXN_SLC_U_SIZES", literal_list(self.slice_map.u_sizes())))
            .bo(BuildOpt::include_def("AXN_SLC_V_SCALES", literal_list(self.slice_map.v_scales())))
            .bo(BuildOpt::include_def("AXN_SLC_U_SCALES", literal_list(self.slice_map.u_scales())))
            .bo(BuildOpt::include_def("AXN_SLC_V_MIDS", literal_list(self.slice_map.v_mids())))
            .bo(BuildOpt::include_def("AXN_SLC_U_MIDS", literal_list(self.slice_map.u_mids())))
        ;

        // Custom filter kernels
        for &(_, _, ref filter_chain) in self.filter_chain_schemes.iter() {
            for pf in filter_chain.iter() {
                match pf.cl_file_name() {
                    Some(ref clfn)  => {
                        build_options = build_options.src_file(clfn.clone());
                    },
                    None => (),
                }
            }
        }

        cmn::load_builtin_kernel_source(build_options)
    }

    // NEW
    pub fn layer_name_by_tags(&self, layer_tags: LayerTags) -> &'static str {
        let layer_info = self.layer_map.layers_meshing_tags(layer_tags);
        assert!(layer_info.len() == 1, "AreaMap::layer_name_by_tags(): ({}) \
            tags matching: {} for area: \"{}\" found", layer_info.len(), layer_tags, self.area_name);
        layer_info[0].name()
    }

    // NEW - UPDATE / CONSOLIDATE
    /// Returns a merged list of slice ids for all source layers.
    //
    // [FIXME]: CONVERT TO layer_id.
    pub fn layer_slc_ids<S: Deref<Target=str> + Display>(&self, layer_names: &[S]) -> Vec<u8> {
        let mut slc_ids = Vec::with_capacity(32);

        for layer_name in layer_names.iter() {
            let li = match self.layer_map.layer_info_by_name(layer_name.clone()) {
                Some(li) => li,
                None => panic!("AreaMap::layer_slc_ids(): No layer named '{}' found.",
                    &layer_name),
            };

            if let Some(slc_range) = li.slc_range() {
                for i in slc_range.clone() {
                    slc_ids.push(i as u8);
                }
            }
        }

        slc_ids
    }

    /// Returns a list of tuples of (source slice id, synapse reach) for a
    /// tuft of a cellular layer.
    ///
    /// If `use_prevalance` is true, repeats the (id, reach) tuple as
    /// specified by the `TuftSourceLayer` prevalance parameter.
    ///
    pub fn cel_src_slc_id_rchs(&self, lyr_id: usize, tft_id: usize, use_prevalance: bool)
            -> Vec<(u8, i8)>
    {
        let li = self.layer_map.layer_info(lyr_id)
            .expect(&format!("AreaMap::layer_src_slc_ids(): No layer with id: '{}' found.",
                lyr_id));

        let mut id_rchs = Vec::with_capacity(32);
        let tft_src_lyrs = li.cel_tft_src_lyrs(tft_id);

        for src_lyr in tft_src_lyrs.iter() {
            let src_slc_ids = self.layer_slc_ids(&[src_lyr.name()]);
            let prevalence = if use_prevalance { src_lyr.prevalence() } else { 1 };

            for _ in 0..prevalence {
                for &id in src_slc_ids.iter() {
                    id_rchs.push((id, src_lyr.syn_reach()))
                }
            }
        }

        id_rchs
    }

    // NEW - UPDATE / RENAME
    pub fn aff_out_slc_ids(&self) -> Vec<u8> {
        let mut output_slcs: Vec<u8> = Vec::with_capacity(8);

         // Push all matching slices:
         for layer in self.layer_map.iter() {
             // if (layer.layer_tags() & map::FF_OUT) == map::FF_OUT {
            if layer.axn_domain().is_output() {
                let v = self.layer_slc_ids(&[layer.name().to_owned()]);
                output_slcs.extend_from_slice(&v);
             }
         }

         output_slcs.shrink_to_fit();

        //  // Ensure that the slice id list contains contiguous slice ids:
        //  for i in 0..output_slcs.len() {
        //      if i > 0 {
        //          unsafe { debug_assert!(*output_slcs.get_unchecked(i - 1)
        //              == *output_slcs.get_unchecked(i) - 1); }
        //     }
        // }

        output_slcs
    }

    // // NEW NEW NEW
    // /// Returns the slice range of the afferent output axon slices (FF_OUT).
    // pub fn aff_out_slc_range(&self) -> Range<usize> {
    //     let aff_out_slcs = self.aff_out_slcs();
    //     let idz = 0;
    //     let idn = aff_out_slcs.len() - 1;
    //     (aff_out_slcs[idz] as usize)..(aff_out_slcs[idn] as usize + 1)
    // }

    // NEW
    pub fn psal_layer(&self) -> &LayerInfo {
        let psal_layer_vec = self.layer_map.layers_containing_tags(LayerTags::PSAL);
        assert_eq!(psal_layer_vec.len(), 1);
        psal_layer_vec[0]
    }

    // NEW
    #[inline]
    pub fn ptal_layer(&self) -> &LayerInfo {
        let ptal_layer_vec = self.layer_map.layers_containing_tags(LayerTags::PTAL);
        assert_eq!(ptal_layer_vec.len(), 1);
        ptal_layer_vec[0]
    }

    /// Returns the axon range for a single layer with tags meshing with
    /// `layer_tags`.
    ///
    /// `src_lyr_sub_slcs` optionally specifies a particular range of sub
    /// slices mapping to a specific source layer (source `area_id` is
    /// required for redundant verification).
    ///
    /// [DEPRICATED]
    ///
    pub fn axn_range_meshing_tags_either_way(&self, layer_tags: LayerTags,
                src_lyr_sub_slcs: Option<(usize, Range<usize>)>) -> Option<Range<u32>>
    {
        let layers = self.layer_map.layers_meshing_tags_either_way(layer_tags);

        if layers.len() == 1 {
            let layer = layers[0];

            if let Some(lyr_slc_range) = layer.slc_range() {
                match src_lyr_sub_slcs {
                    // Sub-slices of Layer:
                    Some((area_id, slc_range)) => {
                        match layer.src_lyr_old(area_id, slc_range) {
                            Some(src_lyr) => {
                                let src_base_slc_id = src_lyr.tar_slc_range().start as u8;
                                let src_lyr_idz = self.axn_idz(src_base_slc_id as u8);
                                let src_lyr_len = src_lyr.axn_count();

                                // * TODO: ADDME: self.verify_axn_range()
                                debug_assert!({
                                        let slc_idm = src_base_slc_id + src_lyr.dims().depth() - 1;
                                        let slc_len = self.slice_map.slc_axn_count(slc_idm);
                                        let axn_idz = self.axn_idz(slc_idm);
                                        let axn_idn = axn_idz + slc_len;
                                        // // [DEBUG]:
                                        // println!("\n\n# (lyr_idz, lyr_len) = ({}, {}), axn_idn = {}, \
                                        //     slc_len = {}, axn_idz = {}, \n# layer: {:#?}\n",
                                        //     src_lyr_idz, src_lyr_len, axn_idn, slc_len, axn_idz, layer);
                                        (src_lyr_idz + src_lyr_len) == axn_idn
                                    }, "AreaMap::axn_range(): Axon index mismatch.");

                                Some(src_lyr_idz..(src_lyr_idz + src_lyr_len))
                            },
                            None => None,
                        }
                    },
                    // Entire Layer:
                    None => {
                        let base_slc_id = lyr_slc_range.start as u8;
                        let lyr_idz = self.axn_idz(base_slc_id);

                        let lyr_len = layer.ttl_axn_count();

                        // * TODO: ADDME: self.verify_axn_range()
                        debug_assert!({
                                let slc_idm = base_slc_id + layer.depth() - 1;
                                let slc_len = self.slice_map.slc_axn_count(slc_idm);
                                let axn_idz = self.axn_idz(slc_idm);
                                let axn_idn = axn_idz + slc_len;
                                // [DEBUG]:
                                // println!("\n\n# (lyr_idz, lyr_len) = ({}, {}), axn_idn = {}, \
                                //     slc_len = {}, axn_idz = {}, \n# layer: {:?}\n",
                                //     lyr_idz, lyr_len, axn_idn, slc_len, axn_idz, layer);
                                (lyr_idz + lyr_len) == axn_idn
                            }, "AreaMap::axn_range(): Axon index mismatch.");

                        Some(lyr_idz..(lyr_idz + lyr_len))
                    },
                }
            } else {
                None
            }
        } else if layers.len() == 0 {
            None
        } else {
            panic!("AreaMap::axn_range_meshing_tags(): Multiple layers matching \
                flags: '{}' for area: '{}'. \n\nLayers: \n{:#?}", layer_tags,
                self.area_name, layers);
        }
    }

    /// Returns the axon index range for a layer, or optionally a subset of
    /// that range pertaining to a specific source layer, if the layer exists.
    ///
    pub fn lyr_axn_range(&self, lyr_addr: &LayerAddress, src_lyr_addr: Option<&LayerAddress>)
            -> Option<Range<u32>>
    {
        assert!(lyr_addr.area_id() == self.area_id(), "AreaMap::lyr_axn_range: \
            The layer address area id provided ({}) does not match this area's id ({}).",
            lyr_addr.area_id(), self.area_id());

        if let Some(ref li) = self.layer_map.layer_info(lyr_addr.layer_id()) {
            if let Some(sl_addr) = src_lyr_addr {
                if let Some(sli) = li.src_lyr(sl_addr) {
                    let src_base_slc_id = sli.tar_slc_range().start as u8;
                    let src_lyr_axn_idz = self.axn_idz(src_base_slc_id);
                    let src_lyr_axn_len = sli.axn_count();
                    let src_lyr_axn_range = src_lyr_axn_idz..(src_lyr_axn_idz + src_lyr_axn_len);

                    debug_assert!(self.verify_axn_range(src_lyr_axn_range.clone(),
                        src_base_slc_id, li.depth()), "AreaMap::lyr_axn_range: \
                        Axon index range mismatch.");

                    Some(src_lyr_axn_range)
                } else {
                    None
                }
            } else if let Some(lyr_slc_range) = li.slc_range() {
                let base_slc_id = lyr_slc_range.start as u8;
                let lyr_axn_idz = self.axn_idz(base_slc_id);
                let lyr_axn_len = li.ttl_axn_count();
                let lyr_axn_range = lyr_axn_idz..(lyr_axn_idz + lyr_axn_len);

                debug_assert!(self.verify_axn_range(lyr_axn_range.clone(),
                    base_slc_id, li.depth()), "AreaMap::lyr_axn_range: \
                    Axon index range mismatch.");

                Some(lyr_axn_range)

            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn verify_axn_range(&self, axn_range: Range<u32>, base_slc_id: u8, depth: u8) -> bool {
        let slc_idm = base_slc_id + depth - 1;
        let slc_len = self.slice_map.slc_axn_count(slc_idm);
        let axn_idz = self.axn_idz(slc_idm);
        let axn_idn = axn_idz + slc_len;
        // [DEBUG]:
        // println!("\n\n# (lyr_idz, lyr_len) = ({}, {}), axn_idn = {}, \
        //     slc_len = {}, axn_idz = {}, \n# layer: {:?}\n",
        //     lyr_idz, lyr_len, axn_idn, slc_len, axn_idz, layer);
        axn_range.start == axn_idz && axn_range.end == axn_idn
    }

    // NEW
    pub fn slc_src_layer_dims(&self, slc_id: u8, layer_tags: LayerTags) -> Option<&CorticalDims> {
        self.layer_map.slc_src_layer_info(slc_id, layer_tags).map(|sli| sli.dims())
    }

    // DEPRICATE
    pub fn aff_areas(&self) -> &Vec<&'static str> {
        &self.aff_areas
    }

    // DEPRICATE
    pub fn eff_areas(&self) -> &Vec<&'static str> {
        &self.eff_areas
    }


    pub fn filter_chain_schemes(&self) -> &[(InputTrack, AxonTags, Vec<FilterScheme>)] {
        &self.filter_chain_schemes
    }

    // UPDATE / DEPRICATE
    pub fn lm_kind_tmp(&self) -> &LayerMapKind {
        &self.layer_map.region_kind()
    }

    pub fn area_id(&self) -> usize { self.area_id }
    pub fn area_name(&self) -> &'static str { self.area_name }
    pub fn axn_idz(&self, slc_id: u8) -> u32 { self.slice_map.idz(slc_id) }
    pub fn slice_map(&self) -> &SliceMap { &self.slice_map }
    pub fn layer_map(&self) -> &LayerMap { &self.layer_map }
    pub fn layer(&self, layer_id: usize) -> Option<&LayerInfo> { self.layer_map.layer_info(layer_id) }
    pub fn dims(&self) -> &CorticalDims { &self.dims }
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
         /// AXN_IDX(): Some documentation for this can be found in `bismit.cl`.
         ///
         ///         Basically all we're doing is scaling up or down the v and
         ///         u coordinates based on a predetermined scaling factor. The
         ///         scaling factor only applies when a foreign cortical or
         ///         sub-cortical area is a source for the axon's slice AND is
         ///         a different size than the local cortical area. The scale
         ///         factor is based on the relative size of the two areas.
         ///         Most of the time the scaling factor is 1:1 (scale factor
         ///         of 16). The algorithm below for calculating an axon index
         ///         is the same as the one in the kernel and gives precisely
         ///         the same results.
         ///
        fn axn_idx(&self, slc_id: u8, v_id_unscaled: u32, v_ofs: i8, u_id_unscaled: u32, u_ofs: i8)
                -> Result<u32, &'static str>
        {
            let v_scale = self.slice_map.v_scales()[slc_id as usize];
            let u_scale = self.slice_map.u_scales()[slc_id as usize];

            let v_id_scaled = cmn::scale(v_id_unscaled as i32, v_scale);
            let u_id_scaled = cmn::scale(u_id_unscaled as i32, u_scale);

            let slc_count = self.slice_map().depth();
            let v_size = self.slice_map.v_sizes()[slc_id as usize];
            let u_size = self.slice_map.u_sizes()[slc_id as usize];

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
            let v_scale = self.slice_map.v_scales()[slc_id as usize];
            let u_scale = self.slice_map.u_scales()[slc_id as usize];

            let v_id_scaled = cmn::scale(v_id_unscaled as i32, v_scale);
            let u_id_scaled = cmn::scale(u_id_unscaled as i32, u_scale);

            let v_size = self.slice_map.v_sizes()[slc_id as usize];
            let u_size = self.slice_map.u_sizes()[slc_id as usize];

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
            write!(fmtr, "slice_map: {}", self.slice_map)
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

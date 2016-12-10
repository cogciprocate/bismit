use std::ops::Range;
use ocl::{self, SpatialDims};
use ocl::traits::MemLen;
use cmn::{self, CorticalDims, SliceDims};
use map::{area_map, LayerMap, AxonTopology, SliceTractMap};

const DEBUG_PRINT: bool = false;

#[derive(Debug, Clone)]
pub struct SliceMap {
    axn_idzs: Vec<u32>,
    layer_names: Vec<&'static str>,
    axn_kinds: Vec<AxonTopology>,
    v_sizes: Vec<u32>,
    u_sizes: Vec<u32>,
    v_scales: Vec<u32>,
    u_scales: Vec<u32>,
    v_mids: Vec<u32>,
    u_mids: Vec<u32>,
    dims: Vec<SliceDims>,
    physical_len: u32
}

impl SliceMap {
    pub fn new(area_dims: &CorticalDims, layers: &LayerMap) -> SliceMap {
        let slc_map = layers.slc_map();
        let depth = layers.depth() as usize;

        debug_assert_eq!(slc_map.len(), depth);

        let mut axn_idzs = Vec::with_capacity(depth);
        let mut layer_names = Vec::with_capacity(depth);
        let mut axn_kinds = Vec::with_capacity(depth);
        let mut v_scales = Vec::with_capacity(depth);
        let mut u_scales = Vec::with_capacity(depth);
        let mut v_sizes = Vec::with_capacity(depth);
        let mut u_sizes = Vec::with_capacity(depth);
        let mut v_mids = Vec::with_capacity(depth);
        let mut u_mids = Vec::with_capacity(depth);
        let mut dims = Vec::with_capacity(depth);

        let mut axn_idz_ttl = 0u32;
        // For checking purposes:
        let mut slc_id_ttl = 0u8;

        for (&slc_id, &layer) in slc_map.iter() {
            let mut add_slice = |slc_dims: SliceDims| {
                assert!(slc_id as usize == axn_idzs.len(), "SliceMap::new(): \
                    slice_id of the slice currently being added: '{}' must be equal to the \
                    number of slices already added: '{}'", slc_id, axn_idzs.len());

                axn_idzs.push(axn_idz_ttl);
                axn_idz_ttl += slc_dims.columns();

                layer_names.push(layer.name());
                axn_kinds.push(layer.axn_kind());
                v_sizes.push(slc_dims.v_size());
                u_sizes.push(slc_dims.u_size());
                v_scales.push(slc_dims.v_scale());
                u_scales.push(slc_dims.u_scale());
                v_mids.push(slc_dims.v_mid());
                u_mids.push(slc_dims.u_mid());
                dims.push(slc_dims);
            };

            let layer_sources = layer.sources();

            if layer_sources.len() > 0 {
                // This loop must succeed in adding a new slice only once.
                for layer_source in layer_sources {
                    // Only add a slice to the final slice map if current
                    // slc_id is within the source layer's target slice range
                    if slc_id >= layer_source.tar_slc_range().start
                        && slc_id < layer_source.tar_slc_range().end
                    {
                        debug_assert!(slc_id == slc_id_ttl);
                        // debug_assert_eq!(layer.axn_kind(), layer_source.axn_kind());

                        if layer.axn_kind() != layer_source.axn_kind() {
                            // Ensure that we are going from Spatial -> Horizontal:
                            if layer_source.axn_kind() == AxonTopology::Spatial &&
                                    layer.axn_kind() == AxonTopology::Horizontal
                            {
                                assert!(layer_source.dims().v_size() <= 254 &&
                                    layer_source.dims().u_size() <= 254,
                                    "SliceMap::new(): Horizontal layer sources must have dimensions \
                                    less than or equal to 254v x 254u.");
                            } else {
                                panic!("SliceMap::new(): Layers may only convert from \
                                    Spatial -> Horizontal");
                            }
                        }

                        if DEBUG_PRINT {
                            println!("SLICEMAP::NEW(): Using source layer dims: {:?} \
                                for layer: {} in area: {}", layer_source.dims(),
                                layer.name(), layer_source.area_id());
                        }
                        slc_id_ttl += 1;
                        add_slice(SliceDims::new(area_dims, Some(layer_source.dims()),
                            layer.axn_kind())
                            .expect("SliceMap::new(): Error creating SliceDims."));
                    }
                }
            } else {
                debug_assert!(slc_id == slc_id_ttl);
                match layer.irregular_layer_dims() {
                    Some(dims) => {
                        if DEBUG_PRINT {
                            println!("SLICEMAP::NEW(): Adding irregular layer dims: {:?} \
                                for layer: {}", dims, layer.name());
                        }
                        slc_id_ttl += 1;
                        add_slice(SliceDims::new(dims, None, layer.axn_kind())
                            .expect("SliceMap::new()"))
                    },
                    None => {
                        if DEBUG_PRINT {
                            println!("SLICEMAP::NEW(): Boring area layer dims: {:?} \
                                for layer: {}", area_dims, layer.name());
                        }
                        slc_id_ttl += 1;
                        add_slice(SliceDims::new(area_dims, None, layer.axn_kind())
                            .expect("SliceMap::new()"))
                    },
                }
            }
        }

        debug_assert_eq!(axn_idzs.len(), layer_names.len());
        debug_assert_eq!(axn_idzs.len(), axn_kinds.len());
        debug_assert_eq!(axn_idzs.len(), dims.len());
        debug_assert_eq!(axn_idzs.len(), v_sizes.len());
        debug_assert_eq!(axn_idzs.len(), u_sizes.len());
        debug_assert_eq!(axn_idzs.len(), v_scales.len());
        debug_assert_eq!(axn_idzs.len(), u_scales.len());
        debug_assert_eq!(axn_idzs.len(), v_mids.len());
        debug_assert_eq!(axn_idzs.len(), u_mids.len());
        debug_assert_eq!(axn_idzs.len(), depth);
        debug_assert_eq!(axn_idzs.len(), slc_id_ttl as usize);

        SliceMap {
            axn_idzs: axn_idzs,
            layer_names: layer_names,
            axn_kinds: axn_kinds,
            dims: dims,
            v_sizes: v_sizes,
            u_sizes: u_sizes,
            v_scales: v_scales,
            u_scales: u_scales,
            v_mids: v_mids,
            u_mids: u_mids,
            physical_len: axn_idz_ttl,
        }
    }

    pub fn print_debug(&self) {
        println!(
            "{mt}{mt}SLICEMAP::PRINT_DEBUG(): Area slices: \
            \n{mt}{mt}{mt}layer_names:  {:?}, \
            \n{mt}{mt}{mt}axn_idzs:     [{}], \
            \n{mt}{mt}{mt}v_sizes:      [{}], \
            \n{mt}{mt}{mt}u_sizes:      [{}], \
            \n{mt}{mt}{mt}v_scales:     [{}], \
            \n{mt}{mt}{mt}u_scales:     [{}], \
            \n{mt}{mt}{mt}v_mids:       [{}], \
            \n{mt}{mt}{mt}u_mids:       [{}]",
            self.layer_names,
            area_map::literal_list(&self.axn_idzs),
            area_map::literal_list(&self.v_sizes),
            area_map::literal_list(&self.u_sizes),
            area_map::literal_list(&self.v_scales),
            area_map::literal_list(&self.u_scales),
            area_map::literal_list(&self.v_mids),
            area_map::literal_list(&self.u_mids),
            mt = cmn::MT
        );

        println!("");
    }

    #[inline]
    pub fn idz(&self, slc_id: u8) -> u32 {
        self.axn_idzs[slc_id as usize]
    }

    #[inline]
    pub fn layer_name(&self, slc_id: u8) -> &'static str {
        self.layer_names[slc_id as usize]
    }

    #[inline]
    pub fn slc_axn_count(&self, slc_id: u8) -> u32 {
        self.v_sizes[slc_id as usize] * self.u_sizes[slc_id as usize]
    }

    #[inline]
    pub fn axn_range(&self, slc_id: u8) -> Range<usize> {
        let idz = self.idz(slc_id) as usize;
        idz..(idz + self.slc_axn_count(slc_id) as usize)
    }

    #[inline]
    pub fn tract_map_range(&self, slc_range: Range<usize>) -> SliceTractMap {
        assert!(slc_range.end <= 255);
        SliceTractMap::new(&self.layer_names[slc_range.clone()], &self.v_sizes[slc_range.clone()],
            &self.u_sizes[slc_range.clone()])
    }

    #[inline]
    pub fn tract_map(&self) -> SliceTractMap {
        self.tract_map_range(0..self.axn_idzs.len())
    }

    #[inline] pub fn slc_count(&self) -> usize { self.axn_idzs.len() }
    #[inline] pub fn depth(&self) -> u8 { self.axn_idzs.len() as u8 }
    #[inline] pub fn axn_count(&self) -> u32 { self.physical_len }
    #[inline] pub fn axn_idzs(&self) -> &Vec<u32> { &self.axn_idzs }
    #[inline] pub fn layer_names(&self) -> &Vec<&'static str> { &self.layer_names }
    #[inline] pub fn axn_kinds(&self) -> &Vec<AxonTopology> { &self.axn_kinds }
    #[inline] pub fn v_sizes(&self) -> &Vec<u32> { &self.v_sizes }
    #[inline] pub fn u_sizes(&self) -> &Vec<u32> { &self.u_sizes }
    #[inline] pub fn v_scales(&self) -> &Vec<u32> { &self.v_scales }
    #[inline] pub fn u_scales(&self) -> &Vec<u32> { &self.u_scales }
    #[inline] pub fn v_mids(&self) -> &Vec<u32> { &self.v_mids }
    #[inline] pub fn u_mids(&self) -> &Vec<u32> { &self.u_mids }
    #[inline] pub fn dims(&self) -> &Vec<SliceDims> { &self.dims }
}

impl MemLen for SliceMap {
    #[inline]
    fn to_len(&self) -> usize {
        self.axn_count() as usize
    }

    fn to_len_padded(&self, incr: usize) -> usize {
        ocl::util::padded_len(self.axn_count() as usize, incr)
    }

    fn to_lens(&self) -> [usize; 3] {
        // self.dims.to_lens().expect("bismit::SliceMap::to_size")
        [self.axn_count() as usize, 1, 1]
    }
}

impl Into<SpatialDims> for SliceMap {
    fn into(self) -> SpatialDims {
        self.to_lens().into()
    }
}

impl<'a> Into<SpatialDims> for &'a SliceMap {
    fn into(self) -> SpatialDims {
        self.to_lens().into()
    }
}


#[cfg(test)]
pub mod tests {
    use std::fmt::{Display, Formatter, Result as FmtResult};
    use super::{SliceMap};

    pub trait SliceMapTest {
        fn print(&self);
    }

    impl SliceMapTest for SliceMap {
        fn print(&self) {
            unimplemented!();
        }
    }

    impl Display for SliceMap {
        fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
            let mut output = String::with_capacity(30 * self.slc_count());

            for i in 0..self.slc_count() {
                output.push_str(&format!("[{}: '{}', {}]", i, self.layer_names()[i],
                    self.axn_idzs()[i]));
            }

            fmtr.write_str(&output)
        }
    }
}



            // println!("{mt}{mt}SLICEMAP::NEW(): Adding inter-area slice '{}': slc_id: {}, src_area_name: {}, \
            //     v_size: {}, u_size: {}.", layer.name(), slc_id, sli.area_name(),
            //     slc_dims.v_size(), slc_dims.u_size(), mt = cmn::MT);

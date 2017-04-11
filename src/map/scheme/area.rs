use map::{FilterScheme, InputScheme, AxonTags, InputTrack};
use cmn::{self, CorticalDims, MapStore};



#[derive(PartialEq, Debug, Clone)]
pub struct AreaScheme {
    area_id: Option<usize>,
    name: &'static str,
    layer_map_name: &'static str,
    dims: CorticalDims,
    input: InputScheme,
    filter_chains: Vec<(InputTrack, AxonTags, Vec<FilterScheme>)>,
    aff_areas: Vec<&'static str>,
    eff_areas: Vec<&'static str>,
    // (area name, list of optional axon tag masquerades (original, replacement))):
    other_areas: Vec<(&'static str, Option<Vec<(AxonTags, AxonTags)>>)>
}

impl AreaScheme {
    pub fn new(name: &'static str, layer_map_name: &'static str, dim: u32) -> AreaScheme {
        AreaScheme::irregular(name, layer_map_name, [dim, dim])
    }

    pub fn irregular(name: &'static str, layer_map_name: &'static str, dims: [u32; 2])
            -> AreaScheme
    {
        AreaScheme {
            area_id: None,
            name: name,
            layer_map_name: layer_map_name,
            dims: CorticalDims::new(dims[0], dims[1], 0, None),
            input: InputScheme::None,
            filter_chains: Vec::with_capacity(4),
            aff_areas: Vec::with_capacity(4),
            eff_areas: Vec::with_capacity(0),
            other_areas: Vec::with_capacity(0),
        }
    }

    pub fn input(mut self, input: InputScheme) -> AreaScheme {
        self.input = input;
        self
    }

    // pub fn filter_chain(mut self, tags: LayerTags, filter_chain: Vec<FilterScheme>) -> AreaScheme {
    //     self.filter_chains.push((tags, filter_chain));
    //     self
    // }

    pub fn filter_chain<A, F>(mut self, input_track: InputTrack, axn_tags: A,
            filter_chain: &[F]) -> AreaScheme
            where A: Into<AxonTags>, F: Into<FilterScheme> + Clone
    {
        // let filter_chain = filter_chain.into_iter().map(move |f| f.into()).collect();
        let mut filter_chain_vec: Vec<FilterScheme> = Vec::with_capacity(filter_chain.len());

        for f in filter_chain.into_iter() {
            filter_chain_vec.push(f.clone().into());
        }

        self.add_filter_chain(input_track, axn_tags, filter_chain_vec);
        self
    }

    pub fn eff_areas(mut self, eff_areas: Vec<&'static str>) -> AreaScheme {
        self.eff_areas = eff_areas;
        self
    }

    pub fn other_area(mut self, area_name: &'static str, new_tags: Option<&[(AxonTags, AxonTags)]>)
            -> AreaScheme
            // where A: Into<AxonTags> + Clone
    {
        let new_tags_owned = new_tags.map(|nt| {
            nt.into_iter()
                .map(|masq| {
                    let (orig, repl) = masq.clone();
                    (orig.into(), repl.into())
                })
                .collect()
        });

        self.other_areas.push((area_name, new_tags_owned));
        self
    }

    // pub fn set_filter_chain(&mut self, tags: LayerTags, filter_chain: Vec<FilterScheme>) {
    //     self.filter_chains.push((tags, filter_chain));
    // }

    pub fn add_filter_chain<A: Into<AxonTags>>(&mut self, input_track: InputTrack, axn_tags: A,
            filter_chain: Vec<FilterScheme>)
    {
        self.filter_chains.push((input_track, axn_tags.into(), filter_chain.into()));
    }

    pub fn set_eff_areas(&mut self, eff_areas: Vec<&'static str>) {
        self.eff_areas = eff_areas;
    }

    #[inline]
    pub fn get_other_areas(&self) -> &Vec<(&'static str, Option<Vec<(AxonTags, AxonTags)>>)> {
        &self.other_areas
    }

    // #[inline]
    // pub fn filter_chains(&self) -> &Vec<(LayerTags, Vec<FilterScheme>)> {
    //     &self.filter_chains
    // }

    #[inline]
    pub fn filter_chains(&self) -> &Vec<(InputTrack, AxonTags, Vec<FilterScheme>)> {
        &self.filter_chains
    }

    #[inline] pub fn area_id(&self) -> usize { self.area_id.expect("Area ID not set!") }
    #[inline] pub fn name(&self) -> &'static str { self.name }
    #[inline] pub fn layer_map_name(&self) -> &'static str { self.layer_map_name }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn get_input(&self) -> &InputScheme { &self.input }
    #[inline] pub fn get_eff_areas(&self) -> &Vec<&'static str> { &self.eff_areas }
    #[inline] pub fn get_aff_areas(&self) -> &Vec<&'static str> { &self.aff_areas }

}



pub struct AreaSchemeList {
    areas: MapStore<&'static str, AreaScheme>,
    frozen: bool,
}

impl <'a>AreaSchemeList {
    pub fn new() -> AreaSchemeList {
        AreaSchemeList { areas: MapStore::new(), frozen: false }
    }

    fn add(&mut self, mut protoarea: AreaScheme) {
        if self.frozen { panic!("AreaSchemeList is frozen."); }
        let name = protoarea.name;
        protoarea.area_id = Some(self.areas.len());
        self.areas.insert(name, protoarea)
            .map(|_| panic!("AreaScheme::add(): Duplicate areas: (area: \"{}\")", name));
    }

    pub fn area(mut self, protoarea: AreaScheme) -> AreaSchemeList {
        self.add(protoarea);
        self
    }

    pub fn freeze(&mut self) {
        let mut aff_list: Vec<(&'static str, &'static str)> = Vec::with_capacity(5);

        for (area_id, area) in self.areas.values().iter().enumerate() {
            assert!(area.area_id() == area_id);

            for eff_area_name in &area.eff_areas {
                aff_list.push((eff_area_name, area.name()));
            }
        }

        assert!(aff_list.len() <= cmn::MAX_FEEDFORWARD_AREAS, "areas::AreaSchemeList::freeze(): \
                An area cannot have more than {} afferent areas.", cmn::MAX_FEEDFORWARD_AREAS);

        for (area_name, aff_area_name) in aff_list {
            let emsg = format!("map::areas::AreaSchemeList::freeze(): Area: '{}' not found. ", area_name);
            self.areas.by_key_mut(&area_name).expect(&emsg).aff_areas.push(aff_area_name);
        }

        self.areas.shrink_to_fit();
        self.frozen = true;
    }

    pub fn get_area_by_key(&self, area_name: &'static str) -> Option<&AreaScheme> {
        self.areas.by_key(&area_name)
    }

    #[inline] pub fn areas(&self) -> &[AreaScheme] { &self.areas.values() }
}
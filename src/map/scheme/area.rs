// use std::collections::{HashMap};
use map::{FilterScheme, InputScheme, LayerTags};
use cmn::{self, CorticalDims, MapStore};



#[derive(PartialEq, Debug, Clone)]
pub struct AreaScheme {
    pub name: &'static str,
    pub layer_map_name: &'static str,
    pub dims: CorticalDims,
    //pub region_kind: LayerMapKind,
    pub input: InputScheme,
    // inputs: Vec<InputScheme>,
    pub filter_chains: Vec<(LayerTags, Vec<FilterScheme>)>,
    aff_areas: Vec<&'static str>,
    eff_areas: Vec<&'static str>,
    other_areas: Vec<&'static str>,
}

impl AreaScheme {
    pub fn new(
                name: &'static str,
                layer_map_name: &'static str,
                dim: u32,
            ) -> AreaScheme
    {
        // [FIXME] TODO: This is out of date. Need to instead verify that
        // 'side' is > CellScheme::den_*_syn_reach. This must be done when
        // assembling the final map.
        //
        // assert!(side >= cmn::SYNAPSE_REACH * 2);

        AreaScheme::irregular(name, layer_map_name, [dim, dim])
    }

    pub fn irregular(
                    name: &'static str,
                    layer_map_name: &'static str,
                    dims: [u32; 2],
                ) -> AreaScheme {
        AreaScheme {
            name: name,
            layer_map_name: layer_map_name,
            dims: CorticalDims::new(dims[0], dims[1], 0, 0, None),
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

    pub fn filter_chain(mut self, tags: LayerTags, filter_chain: Vec<FilterScheme>) -> AreaScheme {
        self.filter_chains.push((tags, filter_chain));
        self
    }

    pub fn eff_areas(mut self, eff_areas: Vec<&'static str>) -> AreaScheme {
        self.eff_areas = eff_areas;
        self
    }

    pub fn other_areas(mut self, areas: Vec<&'static str>) -> AreaScheme {
        self.other_areas = areas;
        self
    }

    pub fn set_filter_chain(&mut self, tags: LayerTags, filter_chain: Vec<FilterScheme>) {
        self.filter_chains.push((tags, filter_chain));
    }

    pub fn set_eff_areas(&mut self, eff_areas: Vec<&'static str>) {
        self.eff_areas = eff_areas;
    }

    #[inline] pub fn name(&self) -> &'static str { self.name }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn get_input(&self) -> &InputScheme { &self.input }
    #[inline] pub fn get_eff_areas(&self) -> &Vec<&'static str> { &self.eff_areas }
    #[inline] pub fn get_aff_areas(&self) -> &Vec<&'static str> { &self.aff_areas }
    #[inline] pub fn get_other_areas(&self) -> &Vec<&'static str> { &self.other_areas }
}



pub struct AreaSchemeList {
    // maps: HashMap<&'static str, AreaScheme>,
    areas: MapStore<&'static str, AreaScheme>,
    frozen: bool,
}

impl <'a>AreaSchemeList {
    pub fn new() -> AreaSchemeList {
        // AreaSchemeList { maps: HashMap::new(), frozen: false }
        AreaSchemeList { areas: MapStore::new(), frozen: false }
    }

    fn add(&mut self, protoarea: AreaScheme) {
        if self.frozen { panic!("AreaSchemeList is frozen."); }
        let name = protoarea.name;
        //let dims = protoarea.dims;
        self.areas.insert(name, protoarea)
            .map(|_| panic!("AreaScheme::add(): Duplicate areas: (area: \"{}\")", name));
    }

    pub fn area(mut self, protoarea: AreaScheme) -> AreaSchemeList {
        if self.frozen { panic!("AreaSchemeList is frozen."); }
        self.add(protoarea);
        self
    }

    pub fn freeze(&mut self) {
        let mut aff_list: Vec<(&'static str, &'static str)> = Vec::with_capacity(5);

        for area in self.areas.values().iter() {
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

    // #[inline] pub fn maps(&self) -> &HashMap<&'static str, AreaScheme> { &self.areas }
    #[inline] pub fn areas(&self) -> &[AreaScheme] { &self.areas.values() }
}
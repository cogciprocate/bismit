use std::collections::{HashMap};

use map::{FilterScheme, InputScheme, LayerTags};
use cmn::{self, CorticalDims};


pub struct AreaSchemeList {
    maps: HashMap<&'static str, AreaScheme>,
}

impl <'a>AreaSchemeList {
    pub fn new() -> AreaSchemeList {
        AreaSchemeList { maps: HashMap::new() }
    }

    fn add(&mut self, protoarea: AreaScheme) {
        let name = protoarea.name;
        //let dims = protoarea.dims;
        self.maps.insert(name, protoarea)
            .map(|_| panic!("AreaScheme::add(): Duplicate areas: (area: \"{}\")", name));
    }

    pub fn area(mut self, protoarea: AreaScheme) -> AreaSchemeList {
        self.add(protoarea);
        self
    }

    //     FREEZE(): CURRENTLY NO CHECKS TO MAKE SURE THIS HAS BEEN CALLED! -
    pub fn freeze(&mut self) {
        let mut aff_list: Vec<(&'static str, &'static str)> = Vec::with_capacity(5);

        for (area_name, area) in self.maps.iter() {
            for eff_area_name in &area.eff_areas {
                aff_list.push((eff_area_name, area_name));
            }
        }

        assert!(aff_list.len() <= cmn::MAX_FEEDFORWARD_AREAS, "areas::AreaSchemeList::freeze(): \
                An area cannot have more than {} afferent areas.", cmn::MAX_FEEDFORWARD_AREAS);

        for (area_name, aff_area_name) in aff_list {
            let emsg = format!("map::areas::AreaSchemeList::freeze(): Area: '{}' not found. ", area_name);
            self.maps.get_mut(area_name).expect(&emsg).aff_areas.push(aff_area_name);
        }
    }

    #[inline] pub fn maps(&self) -> &HashMap<&'static str, AreaScheme> { &self.maps }
}


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
}


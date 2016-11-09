use std::collections::{HashMap};

use map::{FilterScheme, InputScheme};
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

    // [TODO]: RENAME TO `::area`
    pub fn add_area(mut self, protoarea: AreaScheme) -> AreaSchemeList {
        self.add(protoarea);
        self
    }

    // [TODO]: DEPRICATE
    pub fn area(mut self,
                name: &'static str,
                layer_map_name: &'static str,
                side: u32,
                filters_opt: Option<Vec<FilterScheme>>,
                eff_areas_opt: Option<Vec<&'static str>>,
            ) -> AreaSchemeList
    {
        let mut new_area = AreaScheme::new(name, layer_map_name, side);

        if let Some(filters) = filters_opt {
            new_area.set_filters(filters);
        }

        if let Some(eff_areas) = eff_areas_opt {
            new_area.set_eff_areas(eff_areas);
        }

        self.add(new_area);
        self
    }

    // [TODO]: DEPRICATE
    pub fn area_ext(mut self,
                name: &'static str,
                layer_map_name: &'static str,
                side: u32,
                input_scheme: InputScheme,
                filters_opt: Option<Vec<FilterScheme>>,
                eff_areas_opt: Option<Vec<&'static str>>,
            ) -> AreaSchemeList
    {
        let mut new_area = AreaScheme::new(name, layer_map_name, side).input(input_scheme);

        if let Some(filters) = filters_opt {
            new_area.set_filters(filters);
        }

        if let Some(eff_areas) = eff_areas_opt {
            new_area.set_eff_areas(eff_areas);
        }

        self.add(new_area);
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
            // match self.maps.get_mut(area_name) {
            //     Some(area) => area.aff_areas.push(aff_area_name),
            //     None => (), // Could panic if we wanted to.
            // }
        }
    }

    pub fn maps(&self) -> &HashMap<&'static str, AreaScheme> {
        &self.maps
    }
}


#[derive(PartialEq, Debug, Clone)]
pub struct AreaScheme {
    pub name: &'static str,
    pub layer_map_name: &'static str,
    pub dims: CorticalDims,
    //pub region_kind: LayerMapKind,
    pub input: InputScheme,
    // inputs: Vec<InputScheme>,
    pub filters: Option<Vec<FilterScheme>>,
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
            filters: None,
            aff_areas: Vec::with_capacity(4),
            eff_areas: Vec::with_capacity(0),
        }
    }

    pub fn input(mut self, input: InputScheme) -> AreaScheme {
        self.input = input;
        self
    }

    pub fn filters(mut self, filters: Vec<FilterScheme>) -> AreaScheme {
        self.filters = Some(filters);
        self
    }

    pub fn eff_areas(mut self, eff_areas: Vec<&'static str>) -> AreaScheme {
        self.eff_areas = eff_areas;
        self
    }

    pub fn set_filters(&mut self, filters: Vec<FilterScheme>) {
        self.filters = Some(filters);
    }

    pub fn set_eff_areas(&mut self, eff_areas: Vec<&'static str>) {
        self.eff_areas = eff_areas;
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn dims(&self) -> &CorticalDims {
        &self.dims
    }

    pub fn get_input(&self) -> &InputScheme {
        &self.input
    }

    pub fn get_eff_areas(&self) -> &Vec<&'static str> {
        &self.eff_areas
    }

    pub fn get_aff_areas(&self) -> &Vec<&'static str> {
        &self.aff_areas
    }
}


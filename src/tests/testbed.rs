use cmn::CorticalDims;
use map::{self, LayerMapScheme, LayerMapSchemeList, LayerMapKind, AreaScheme, AreaSchemeList,
    CellScheme, EncoderScheme, AxonTopology, LayerKind, AxonDomain, InputTrack, LayerTags};
use ::{Thalamus, Subcortex, InputGenerator};
use ocl::{Context, ProQue};
use cortex::Cortex;

pub static PRIMARY_AREA_NAME: &'static str = "v1";
pub static INHIB_LAYER_NAME: &'static str = "iv_inhib";
pub static PRIMARY_SPATIAL_SSC_LAYER_NAME: &str = "iv";
pub static PRIMARY_TEMPORAL_PYR_LAYER_NAME: &str = "iii";
const CYCLES_PER_FRAME: usize = 1;

/*=============================================================================
===============================================================================
================================== CORTEX 1 ===================================
===============================================================================
=============================================================================*/

pub fn define_layer_map_schemes() -> LayerMapSchemeList {
    let mut layer_map_schemes: LayerMapSchemeList = LayerMapSchemeList::new();

    layer_map_schemes.add(LayerMapScheme::new("visual", LayerMapKind::Cortical)
        //.layer_old("test_noise", 1, map::DEFAULT, LayerKind::Axonal(AxonTopology::Spatial))
        .layer_old("motor_in", 1, LayerTags::DEFAULT,
            AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_NSP]), ]),
            LayerKind::Axonal(AxonTopology::Nonspatial))
        //.layer_old("olfac", 1, LayerTags::DEFAULT, LayerKind::Axonal(Horizontal))
        .layer_old("eff_in", 0, LayerTags::DEFAULT,
            AxonDomain::input(&[(InputTrack::Efferent, &[map::THAL_SP]), ]),
            LayerKind::Axonal(AxonTopology::Spatial))
        .layer_old("aff_in", 0, LayerTags::DEFAULT,
            AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP]), ]),
            LayerKind::Axonal(AxonTopology::Spatial))

        // .layer_old("mcols", 1, LayerTags::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
        //         CellScheme::minicolumn("iv", "iii", 9999))
        .layer_old("out", 1, LayerTags::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
            LayerKind::Axonal(AxonTopology::Spatial))

        .layer_old("unused", 1, LayerTags::DEFAULT, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))

        .layer_old(PRIMARY_SPATIAL_SSC_LAYER_NAME, 1, LayerTags::PSAL, AxonDomain::Local,
            CellScheme::ssc(&[("aff_in", 8, 1)], 5, 400)
        )

        .layer_old("iv_inhib", 0, LayerTags::DEFAULT, AxonDomain::Local,
            CellScheme::inhib("iv", 4, 0)
        )

        .layer_old(PRIMARY_TEMPORAL_PYR_LAYER_NAME, 3, LayerTags::PTAL, AxonDomain::Local,
                CellScheme::pyr(&[("iii", 8, 1)], 2, 3, 0, 1200)
                    .apical(1, &[("iii", 3, 1)], 1, 5, 0, 500)
        )
    );

    layer_map_schemes.add(LayerMapScheme::new("external", LayerMapKind::Subcortical)
        .layer_old("ganglion", 1, LayerTags::DEFAULT,
            AxonDomain::output(&[map::THAL_SP]),
            LayerKind::Axonal(AxonTopology::Spatial))
    );

    layer_map_schemes
}

pub fn define_area_schemes() -> AreaSchemeList {
    let area_side = 24 as u32;

    let area_schemes = AreaSchemeList::new()

        // .area_ext("v0", "external",
        //     // area_side * 2, area_side * 2,
        //     area_side,
        //                 // area_side / 2, area_side / 2,
        //     EncoderScheme::IdxStreamer {
        //         file_name: "train-images-idx3-ubyte".to_owned(),
        //         cyc_per: CYCLES_PER_FRAME,
        //         scale: 1.3,
        //         loop_frames: 1,
        //     },

        //     None,
        //     None,
        // )

        // .area("v1", "visual",
        //     // area_side * 2, area_side * 2,
        //     area_side,
        //     // area_side / 2, area_side / 2,
        //     // 128, 128,

        //     Some(vec![FilterScheme::new("retina", None)]),

        //     Some(vec!["v0"]),
        // )

        .area(AreaScheme::new("v0", "external", area_side)
            .encoder(EncoderScheme::IdxStreamer {
                file_name: "train-images-idx3-ubyte".to_owned(),
                cyc_per: CYCLES_PER_FRAME,
                scale: 1.3,
                loop_frames: 1,
            })
        )
        .area(AreaScheme::new("v1", "visual", area_side)
            .eff_areas(vec!["v0"])
            .filter_chain(InputTrack::Afferent, &[map::THAL_SP], &[("retina", None)]),
        )


        // .area("b1", "visual",
        //     // area_side * 2, area_side * 2,
        //     area_side, area_side,
        //     //32, 32,
        //     //256, 256,

        //      None,

        //      // Some(vec!["a1"]),
        //      None,
        // )

        // .area("a1", "visual", area_side, area_side, None, None)
    ;

    area_schemes
}


// FRESH_CORTEX(): Mmmm... Yummy.
pub fn fresh_cortex() -> Cortex {
    let layer_map_schemes = define_layer_map_schemes();
    let area_schemes = define_area_schemes();

    let input_gen = InputGenerator::new(&layer_map_schemes, &area_schemes, "v0").unwrap();
    // let subcortex = Subcortex::new().nucleus(input_gen);

    Cortex::builder(layer_map_schemes, area_schemes)
        .subcortical_nucleus(input_gen)
        .build().unwrap()
}


/*=============================================================================
===============================================================================
================================== CORTEX 2 ===================================
===============================================================================
=============================================================================*/

// LOTS OF TUFTS, THRESHOLD AT MIN
pub fn cortex_with_lots_of_apical_tufts() -> Cortex {
    let area_name = PRIMARY_AREA_NAME;
    let area_size = 24;
    let lmap_name = "lm_test";

    let mut layer_map_schemes = LayerMapSchemeList::new();

    layer_map_schemes.add(LayerMapScheme::new(lmap_name, LayerMapKind::Cortical)
        .layer_old("extra_in", 0, LayerTags::DEFAULT,
            AxonDomain::input(&[(InputTrack::Other, &[map::THAL_NSP]), ]),
            LayerKind::Axonal(AxonTopology::Nonspatial))
        .layer_old("eff_in", 0, LayerTags::DEFAULT,
            AxonDomain::input(&[(InputTrack::Efferent, &[map::THAL_SP]), ]),
            LayerKind::Axonal(AxonTopology::Spatial))
        .layer_old("aff_in", 0, LayerTags::DEFAULT,
            AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP]), ]),
            LayerKind::Axonal(AxonTopology::Spatial))
        .layer_old("out", 1, LayerTags::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
            LayerKind::Axonal(AxonTopology::Spatial))
        // .layer_old("mcols", 1, LayerTags::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
        //         CellScheme::minicolumn("iv", "iii", 9999))
        .layer_old("test0", 1, LayerTags::DEFAULT, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer_old("test1", 1, LayerTags::UNUSED, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer_old("test2", 1, LayerTags::UNUSED, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer_old("test3", 1, LayerTags::UNUSED, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer_old("test4", 1, LayerTags::UNUSED, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer_old("test5", 1, LayerTags::UNUSED, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer_old("unused", 1, LayerTags::UNUSED, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))

        // .layer_old("iv", 1, LayerTags::PSAL, AxonDomain::Local,
        //     CellScheme::ssc(5, vec!["unused"], 1, 8))
        // // .layer_old("iv_inhib", 0, LayerTags::DEFAULT,
        // //     CellScheme::inhib(4, "iv"))
        // .layer_old("iii", 2, LayerTags::PTAL, AxonDomain::Local,
        //     CellScheme::pyr(2, 4, vec!["unused"], 1, 8)
        //         .apical(1, vec!["test1"], 12)
        //         .apical(1, vec!["test2"], 11)
        //         // .apical(1, vec!["test3"])
        //         // .apical(1, vec!["test4"])
        //         // .apical(1, vec!["test5"])
        // )

        .layer_old(PRIMARY_SPATIAL_SSC_LAYER_NAME, 1, LayerTags::PSAL, AxonDomain::Local,
            CellScheme::ssc(&[("unused", 8, 1)], 4, 100))

        .layer_old(PRIMARY_TEMPORAL_PYR_LAYER_NAME, 3, LayerTags::PTAL, AxonDomain::Local,
            CellScheme::pyr(&[("unused", 8, 1)], 3, 4, 0, 100)
                .apical(1, &[("test1", 7, 1)], 3, 4, 0, 500)
        )

    );

    layer_map_schemes.add(LayerMapScheme::new("dummy_lm", LayerMapKind::Subcortical)
        .layer_old("ganglion", 1, LayerTags::DEFAULT,
            AxonDomain::output(&[map::THAL_SP]),
            LayerKind::Axonal(AxonTopology::Spatial))
    );

    let area_schemes = AreaSchemeList::new()
        // .area(area_name, lmap_name, 32, None, Some(vec!["dummy_area"]))
        // .area_ext("dummy_area", "dummy_lm", 67, EncoderScheme::None, None, None)
        .area(AreaScheme::new(area_name, lmap_name, area_size)
            .eff_areas(vec!["dummy_area"])
        )
        .area(AreaScheme::new("dummy_area", "dummy_lm", 44)
            // .input(EncoderScheme::None)
            .encoder(EncoderScheme::Custom)
        )
    ;

    let input_gen = InputGenerator::new(&layer_map_schemes, &area_schemes, "dummy_area").unwrap();
    // let subcortex = Subcortex::new().nucleus(input_gen);

    Cortex::builder(layer_map_schemes, area_schemes)
        .subcortical_nucleus(input_gen)
        .build().unwrap()
}


/*=============================================================================
===============================================================================
================================== TESTBED ====================================
===============================================================================
=============================================================================*/


// TESTBED {}: Stripped down cortex/cortical area
pub struct TestBed {
    pub ocl_context: Context,
    pub ocl_pq: ProQue,
    pub thal: Thalamus,
    pub dims: CorticalDims,
}

impl TestBed {
    pub fn new() -> TestBed {
        let layer_map_schemes = define_layer_map_schemes();
        let area_schemes = define_area_schemes();

        let v0 = InputGenerator::new(&layer_map_schemes, &area_schemes, "v0").unwrap();
        let subcortex = Subcortex::new().nucleus(v0);

        let ocl_context: Context = Context::builder()
            .build().expect("CorticalArea::new(): ocl_context creation error");

        let thal = Thalamus::new(layer_map_schemes, area_schemes, &subcortex, &ocl_context).unwrap();
        let area_map = thal.area_maps().by_key(PRIMARY_AREA_NAME).unwrap().clone();

        let ocl_pq = ProQue::builder()
            .context(ocl_context.clone())
            .prog_bldr(area_map.gen_build_options())
            .build().expect("Testbed::new(): ocl_pq.build()");

        let dims = area_map.dims().clone_with_incr(ocl_pq.max_wg_size().unwrap());

        TestBed {
            ocl_context: ocl_context,
            ocl_pq: ocl_pq,
            thal: thal,
            dims: dims,
        }
    }
}

impl Drop for TestBed {
    fn drop(&mut self) {
        print!("Releasing OpenCL components for test bed... ");
        print!(" ...complete. \n");
    }
}

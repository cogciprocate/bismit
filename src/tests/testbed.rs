use cmn::CorticalDims;
use map;
use map::{LayerMapScheme, LayerMapSchemeList, LayerMapKind, AreaScheme, AreaSchemeList,
    CellScheme, InputScheme, AxonTopology, LayerKind, AxonDomain, InputTrack};
use ::Thalamus;
use ocl::{Context, ProQue};
use cortex::Cortex;

pub static PRIMARY_AREA_NAME: &'static str = "v1";
pub static INHIB_LAYER_NAME: &'static str = "iv_inhib";
const CYCLES_PER_FRAME: usize = 1;

/*=============================================================================
===============================================================================
================================== CORTEX 1 ===================================
===============================================================================
=============================================================================*/

pub fn define_layer_map_schemes() -> LayerMapSchemeList {
    let mut layer_map_sl: LayerMapSchemeList = LayerMapSchemeList::new();

    layer_map_sl.add(LayerMapScheme::new("visual", LayerMapKind::Cortical)
        //.layer("test_noise", 1, map::DEFAULT, LayerKind::Axonal(AxonTopology::Spatial))
        .layer("motor_in", 1, map::DEFAULT,
            AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_NSP]), ]),
            LayerKind::Axonal(AxonTopology::Horizontal))
        //.layer("olfac", 1, map::DEFAULT, LayerKind::Axonal(Horizontal))
        .layer("eff_in", 0, map::DEFAULT,
            AxonDomain::input(&[(InputTrack::Efferent, &[map::THAL_SP]), ]),
            LayerKind::Axonal(AxonTopology::Spatial))
        .layer("aff_in", 0, map::DEFAULT,
            AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP]), ]),
            LayerKind::Axonal(AxonTopology::Spatial))

        // .layer("out", 1, map::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
        //     LayerKind::Axonal(AxonTopology::Spatial))
        .layer("mcols", 1, map::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
                CellScheme::minicolumn("iv", "iii"))

        .layer("unused", 1, map::DEFAULT, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))

        .layer("iv", 1, map::PSAL, AxonDomain::Local,
            CellScheme::spiny_stellate(&[("aff_in", 8, 1)], 5, 400)
        )

        .layer("iv_inhib", 0, map::DEFAULT, AxonDomain::Local,
            CellScheme::inhibitory(4, "iv")
        )

        .layer("iii", 3, map::PTAL, AxonDomain::Local,
                CellScheme::pyramidal(&[("iii", 8, 1)], 2, 3, 1200)
                    .apical(&[("iii", 3, 1)], 1, 5, 500)
        )
    );

    layer_map_sl.add(LayerMapScheme::new("external", LayerMapKind::Subcortical)
        .layer("ganglion", 1, map::DEFAULT,
            AxonDomain::output(&[map::THAL_SP]),
            LayerKind::Axonal(AxonTopology::Spatial))
    );

    layer_map_sl
}

pub fn define_protoareas() -> AreaSchemeList {
    let area_side = 24 as u32;

    let protoareas = AreaSchemeList::new()

        // .area_ext("v0", "external",
        //     // area_side * 2, area_side * 2,
        //     area_side,
        //                 // area_side / 2, area_side / 2,
        //     InputScheme::IdxStreamer {
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
            .input(InputScheme::IdxStreamer {
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

    protoareas
}


// FRESH_CORTEX(): Mmmm... Yummy.
pub fn fresh_cortex() -> Cortex {
    Cortex::new(define_layer_map_schemes(), define_protoareas(), None)
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

    let mut layer_map_sl = LayerMapSchemeList::new();

    layer_map_sl.add(LayerMapScheme::new(lmap_name, LayerMapKind::Cortical)
        .layer("extra_in", 0, map::DEFAULT,
            AxonDomain::input(&[(InputTrack::Other, &[map::THAL_NSP]), ]),
            LayerKind::Axonal(AxonTopology::Horizontal))
        .layer("eff_in", 0, map::DEFAULT,
            AxonDomain::input(&[(InputTrack::Efferent, &[map::THAL_SP]), ]),
            LayerKind::Axonal(AxonTopology::Spatial))
        .layer("aff_in", 0, map::DEFAULT,
            AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP]), ]),
            LayerKind::Axonal(AxonTopology::Spatial))
        // .layer("out", 1, map::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
        //     LayerKind::Axonal(AxonTopology::Spatial))
        .layer("mcols", 1, map::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
                CellScheme::minicolumn("iv", "iii"))
        .layer("test0", 1, map::DEFAULT, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer("test1", 1, map::UNUSED_TESTING, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer("test2", 1, map::UNUSED_TESTING, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer("test3", 1, map::UNUSED_TESTING, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer("test4", 1, map::UNUSED_TESTING, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer("test5", 1, map::UNUSED_TESTING, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))
        .layer("unused", 1, map::UNUSED_TESTING, AxonDomain::Local, LayerKind::Axonal(AxonTopology::Spatial))

        // .layer("iv", 1, map::PSAL, AxonDomain::Local,
        //     CellScheme::spiny_stellate(5, vec!["unused"], 1, 8))
        // // .layer("iv_inhib", 0, map::DEFAULT,
        // //     CellScheme::inhibitory(4, "iv"))
        // .layer("iii", 2, map::PTAL, AxonDomain::Local,
        //     CellScheme::pyramidal(2, 4, vec!["unused"], 1, 8)
        //         .apical(vec!["test1"], 12)
        //         .apical(vec!["test2"], 11)
        //         // .apical(vec!["test3"])
        //         // .apical(vec!["test4"])
        //         // .apical(vec!["test5"])
        // )

        .layer("iv", 1, map::PSAL, AxonDomain::Local,
            CellScheme::spiny_stellate(&[("unused", 8, 1)], 4, 100))

        .layer("iii", 3, map::PTAL, AxonDomain::Local,
            CellScheme::pyramidal(&[("unused", 8, 1)], 2, 3, 100)
                .apical(&[("test1", 7, 1)], 2, 3, 500)
                .apical(&[("test2", 6, 1)], 2, 3, 500)
                .apical(&[("test3", 5, 1)], 2, 3, 500)
                .apical(&[("test4", 4, 1)], 2, 3, 500)
                .apical(&[("test5", 3, 1)], 2, 3, 500)
        )

    );

    layer_map_sl.add(LayerMapScheme::new("dummy_lm", LayerMapKind::Subcortical)
        .layer("ganglion", 1, map::DEFAULT,
            AxonDomain::output(&[map::THAL_SP]),
            LayerKind::Axonal(AxonTopology::Spatial))
    );

    let area_sl = AreaSchemeList::new()
        // .area(area_name, lmap_name, 32, None, Some(vec!["dummy_area"]))
        // .area_ext("dummy_area", "dummy_lm", 67, InputScheme::None, None, None)
        .area(AreaScheme::new(area_name, lmap_name, area_size)
            .eff_areas(vec!["dummy_area"])
        )
        .area(AreaScheme::new("dummy_area", "dummy_lm", 44)
            .input(InputScheme::None { layer_count: 1 })
        )
    ;

    Cortex::new(layer_map_sl, area_sl, None)
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
        let layer_map_sl = define_layer_map_schemes();
        let area_schemes = define_protoareas();

        let ocl_context: Context = Context::builder()
            .build().expect("CorticalArea::new(): ocl_context creation error");

        let thal = Thalamus::new(layer_map_sl, area_schemes, &ocl_context).unwrap();
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

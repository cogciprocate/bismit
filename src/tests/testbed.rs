use cmn::CorticalDims;
use map;
use map::{LayerMapScheme, LayerMapSchemeList, LayerMapKind, AreaSchemeList, CellScheme, FilterScheme,
    InputScheme, AxonKind, LayerKind};
use thalamus::Thalamus;
use ocl::{Context, ProQue};
use cortex::Cortex;


pub static PRIMARY_AREA_NAME: &'static str = "v1";
pub static INHIB_LAYER_NAME: &'static str = "iv_inhib";
const CYCLES_PER_FRAME: usize = 1;


pub fn define_layer_scheme_maps() -> LayerMapSchemeList {
    let mut plmaps: LayerMapSchemeList = LayerMapSchemeList::new();

    plmaps.add(LayerMapScheme::new("visual", LayerMapKind::Cortical)
        //.layer("test_noise", 1, map::DEFAULT, LayerKind::Axonal(AxonKind::Spatial))
        .layer("motor_in", 1, map::DEFAULT, LayerKind::Axonal(AxonKind::Horizontal))
        //.layer("olfac", 1, map::DEFAULT, LayerKind::Axonal(Horizontal))
        .layer("eff_in", 0, map::FB_IN, LayerKind::Axonal(AxonKind::Spatial))
        .layer("aff_in", 0, map::FF_IN, LayerKind::Axonal(AxonKind::Spatial))
        .layer("out", 1, map::FF_OUT | map::FB_OUT, LayerKind::Axonal(AxonKind::Spatial))
        .layer("unused", 1, map::UNUSED_TESTING, LayerKind::Axonal(AxonKind::Spatial))
        .layer("iv", 1, map::PSAL,
            CellScheme::spiny_stellate(5, vec!["aff_in"], 600, 8))
        .layer("iv_inhib", 0, map::DEFAULT,
            CellScheme::inhibitory(4, "iv"))
        .layer("iii", 3, map::PTAL,
            CellScheme::pyramidal(2, 4, vec!["iii"], 1200, 8).apical(vec!["eff_in"], 12))
    );

    plmaps.add(LayerMapScheme::new("external", LayerMapKind::Thalamic)
        .layer("ganglion", 1, map::FF_OUT, LayerKind::Axonal(AxonKind::Spatial))
    );

    plmaps
}

pub fn define_protoareas() -> AreaSchemeList {
    let area_side = 32 as u32;

    let protoareas = AreaSchemeList::new()

        .area_ext("v0", "external",
            // area_side * 2, area_side * 2,
            area_side,
                        // area_side / 2, area_side / 2,
            InputScheme::IdxStreamer {
                file_name: "train-images-idx3-ubyte".to_owned(),
                cyc_per: CYCLES_PER_FRAME,
                scale: 1.3,
                loop_frames: 1,
            },

            None,
            None,
        )

        .area("v1", "visual",
            // area_side * 2, area_side * 2,
            area_side,
            // area_side / 2, area_side / 2,
            // 128, 128,

            Some(vec![FilterScheme::new("retina", None)]),

            Some(vec!["v0"]),
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
    Cortex::new(define_layer_scheme_maps(), define_protoareas(), None)
}


/*=============================================================================
===============================================================================
================================== CORTEX 2 ===================================
===============================================================================
=============================================================================*/

// LOTS OF TUFTS, THRESHOLD AT MIN
pub fn cortex_with_lots_of_apical_tufts() -> Cortex {
    let area_name = PRIMARY_AREA_NAME;
    let lmap_name = "lm_test";

    let mut plmaps = LayerMapSchemeList::new();

    plmaps.add(LayerMapScheme::new(lmap_name, LayerMapKind::Cortical)
        .layer("eff_in", 0, map::FB_IN, LayerKind::Axonal(AxonKind::Spatial))
        .layer("aff_in", 0, map::FF_IN, LayerKind::Axonal(AxonKind::Spatial))
        .layer("out", 1, map::FF_OUT | map::FB_OUT, LayerKind::Axonal(AxonKind::Spatial))
        .layer("test0", 1, map::DEFAULT, LayerKind::Axonal(AxonKind::Spatial))
        .layer("test1", 1, map::UNUSED_TESTING, LayerKind::Axonal(AxonKind::Spatial))
        .layer("test2", 1, map::UNUSED_TESTING, LayerKind::Axonal(AxonKind::Spatial))
        .layer("test3", 1, map::UNUSED_TESTING, LayerKind::Axonal(AxonKind::Spatial))
        // .layer("test4", 1, map::UNUSED_TESTING, LayerKind::Axonal(AxonKind::Spatial))
        // .layer("test5", 1, map::UNUSED_TESTING, LayerKind::Axonal(AxonKind::Spatial))
        .layer("unused", 1, map::UNUSED_TESTING, LayerKind::Axonal(AxonKind::Spatial))
        .layer("iv", 1, map::PSAL,
            CellScheme::spiny_stellate(5, vec!["unused"], 1, 8))
        // .layer("iv_inhib", 0, map::DEFAULT,
        //     CellScheme::inhibitory(4, "iv"))
        .layer("iii", 2, map::PTAL,
            CellScheme::pyramidal(2, 4, vec!["unused"], 1, 8)
                .apical(vec!["test1"], 12)
                .apical(vec!["test2"], 11)
                // .apical(vec!["test3"])
                // .apical(vec!["test4"])
                // .apical(vec!["test5"])
        )

    );

    plmaps.add(LayerMapScheme::new("dummy_lm", LayerMapKind::Thalamic)
        .layer("ganglion", 1, map::FF_OUT, LayerKind::Axonal(AxonKind::Spatial))
    );

    let pamaps = AreaSchemeList::new()
        .area(area_name, lmap_name, 32, None, Some(vec!["dummy_area"]))
        .area_ext("dummy_area", "dummy_lm", 67, InputScheme::None, None, None)
    ;

    Cortex::new(plmaps, pamaps, None)
}



// TESTBED {}: Stripped down cortex/cortical area
pub struct TestBed {
    pub ocl_context: Context,
    pub ocl_pq: ProQue,
    pub thal: Thalamus,
    pub dims: CorticalDims,
}

impl TestBed {
    pub fn new() -> TestBed {
        let plmaps = define_layer_scheme_maps();
        let area_schemes = define_protoareas();

        let thal = Thalamus::new(plmaps, area_schemes).unwrap();
        let area_map = thal.area_map(PRIMARY_AREA_NAME).clone();

        let ocl_context: Context = Context::builder()
            .build().expect("CorticalArea::new(): ocl_context creation error");

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

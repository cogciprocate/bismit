
use std::sync::mpsc::{Sender, Receiver};
use vibi::bismit::ocl::Buffer;
use vibi::bismit::{Cortex, Thalamus, SubcorticalNucleus};
use vibi::bismit::flywheel::{Command, Request, Response};
use vibi::bismit::map::{AxonDomainRoute, AreaMap};
use vibi::bismit::encode::ScalarSdrWriter;


pub(crate) struct Nucleus {
    area_name: String,
}

impl Nucleus {
    pub fn new<S: Into<String>>(area_name: S, lyr_name: &'static str, tar_area: &'static str,
            cortex: &Cortex) -> Nucleus
    {
        let area_name = area_name.into();

        let v0_ext_lyr_addr = *cortex.areas().by_key(area_name.as_str()).unwrap()
            .area_map().layer_map().layers().by_key(lyr_name).unwrap().layer_addr();
        let v1_in_lyr_buf = cortex.areas().by_key(tar_area).unwrap()
            .axns().create_layer_sub_buffer(v0_ext_lyr_addr, AxonDomainRoute::Input);
        let axns = cortex.areas().by_key(tar_area).unwrap()
            .axns().states().clone();
        let area_map = cortex.areas().by_key(area_name.as_str()).unwrap()
            .area_map().clone();

        Nucleus {
            area_name: area_name.into()
        }
    }
}

impl SubcorticalNucleus for Nucleus {
    fn area_name<'a>(&'a self) -> &'a str {
        &self.area_name
    }

    fn pre_cycle(&mut self, _thal: &mut Thalamus) {

    }

    fn post_cycle(&mut self, _thal: &mut Thalamus) {

    }
}


pub(crate) fn eval(cmd_tx: Sender<Command>, req_tx: Sender<Request>, res_rx: Receiver<Response>,
        axns: Buffer<u8>, area_map: AreaMap, area_side: u32)
{
    // let writer = ScalarSdrWriter::new((0, 31u32), 1, &(area_side, area_side, 1).into());






}
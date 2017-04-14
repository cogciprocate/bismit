
use std::sync::mpsc::{Sender, Receiver};
use vibi::bismit::ocl::Buffer;
use vibi::bismit::flywheel::{Command, Request, Response};
use vibi::bismit::{Thalamus, SubcorticalNucleus};


pub(crate) struct Nucleus {
    area_name: String,
}

impl Nucleus {
    pub fn new<S: Into<String>>(area_name: S) -> Nucleus {
        Nucleus { area_name: area_name.into() }
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
        axns: Buffer<u8>)
{




}
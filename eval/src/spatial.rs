
use std::sync::mpsc::{Sender, Receiver};
use vibi::bismit::ocl::Buffer;
use vibi::bismit::flywheel::{Command, Request, Response};


pub fn eval(cmd_tx: Sender<Command>, req_tx: Sender<Request>, res_rx: Receiver<Response>,
        axns: Buffer<u8>)
{




}
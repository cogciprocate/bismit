use ocl::Kernel;


pub struct Requisite {
    thing: u8,
}

impl Requisite {
    pub fn new() -> Requisite {
        Requisite { thing :0 }
    }
}




pub struct KernelWarden {
    requisites: Vec<Requisite>,
}

impl KernelWarden {
    pub fn new() -> KernelWarden {
        KernelWarden { requisites: Vec::with_capacity(16) }
    }

    pub fn requisite(&mut self, req_id: usize) -> Option<&mut Requisite> {
        self.requisites.get_mut(req_id)
    }

    pub fn new_requisite(&mut self, req: Requisite) -> usize {
        self.requisites.push(req);
        self.requisites.len()
    }
}


pub struct ReadWarden {
    requisites: Vec<Requisite>,
}

impl ReadWarden {
    pub fn read() -> ReadWarden {
        ReadWarden { requisites: Vec::with_capacity(16) }
    }

    pub fn requisite(&mut self, req_id: usize) -> Option<&mut Requisite> {
        self.requisites.get_mut(req_id)
    }

    pub fn new_requisite(&mut self, req: Requisite) -> usize {
        self.requisites.push(req);
        self.requisites.len()
    }
}


pub struct WriteWarden {
    requisites: Vec<Requisite>,
}

impl WriteWarden {
    pub fn write() -> WriteWarden {
        WriteWarden { requisites: Vec::with_capacity(16) }
    }

    pub fn new_requisite(&mut self, req: Requisite) -> usize {
        self.requisites.push(req);
        self.requisites.len()
    }

    pub fn requisite(&mut self, req_id: usize) -> Option<&mut Requisite> {
        self.requisites.get_mut(req_id)
    }
}





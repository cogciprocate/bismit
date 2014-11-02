
use opencl::mem::CLBuffer;

let ctx = clCreateContext(
			ptr::null(),
			1,
			&self.id,
			mem::transmute(ptr::null::<||>()),
			ptr::mut_null(),
			(&mut errcode),
);
let ctx = devices.get(0).create_context();
let (device, ctx, queue) = opencl::util::create_compute_context().unwrap();


let A: CLBuffer<int> = ctx.create_buffer(vec_a.len(), opencl::CL::CL_MEM_READ_ONLY);
let B: CLBuffer<int> = ctx.create_buffer(vec_a.len(), opencl::CL::CL_MEM_READ_ONLY);
let C: CLBuffer<int> = ctx.create_buffer(vec_a.len(), opencl::CL::CL_MEM_WRITE_ONLY);

pub struct CLBuffer<T> {
    pub cl_buffer: cl_mem
}

pub fn create_buffer<T>(&self, size: uint, flags: cl_mem_flags) -> CLBuffer<T> {
    unsafe {

        let buf = clCreateBuffer(self.ctx,
					flags,
					(size*mem::size_of::<T>()) as libc::size_t ,
					ptr::mut_null(),
					(&mut status)
        );

        CLBuffer{cl_buffer: buf}
    }
}




-----------



queue.write(&A, &vec_a, ());
queue.write(&B, &vec_b, ());



kernel.set_arg(0, &A);
kernel.set_arg(1, &B);
kernel.set_arg(2, &C);

pub fn set_arg<T: KernelArg>(
	&self, 
	i: uint, 
	x: &T
) {
    set_kernel_arg(
    	self, 
    	i as CL::cl_uint, 
    	x
	)
}

pub fn set_kernel_arg<T: KernelArg>(
			kernel: &Kernel,
			position: cl_uint,
			arg: &T
) {
    unsafe {
		size: size_t = mem::size_of::<cl_mem>() as size_t
		p: *const c_void:  = &CLBuffer.cl_buffer as *const cl_mem as *const c_void

        let ret = clSetKernelArg(
				kernel.kernel,
				position,
				size,
				p
		);

        check(ret, "Failed to set kernel arg!");
    }
}



----------


let command_queue = clCreateCommandQueue(context,device,CL_QUEUE_PROFILING_ENABLE,&mut err);


transmute for clCreateBuffer:
std::mem::transmute(c_ptr as *mut libc::c_void), 

	/*
	(Hypercolumn)
		<hypercolumn_states>
			*Strongest Columns
		(Columns)
			*Target Column Addresses
			<column_states>
				*Activation Strength
				*Input State
	(Messages)

	*/

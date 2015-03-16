
extern crate libc;

use std;
use std::ptr;
use std::mem;
use std::io::{ Read };
use std::fs::{ File };
use std::ffi;
use std::iter;
use envoy::{ Envoy };
use cl_h;
pub use cl_h::{cl_platform_id, cl_device_id, cl_context, cl_program, cl_kernel, cl_command_queue, cl_float, cl_mem, cl_char, cl_uchar, cl_short, cl_ushort, cl_int, cl_uint, cl_long, CLStatus};

pub const KERNELS_FILE_NAME: &'static str = "bismit.cl";
pub const BUILD_OPTIONS: &'static str = "-cl-denorms-are-zero -cl-fast-relaxed-math";		//	-cl-std=CL2.0
pub const GPU_DEVICE: usize = 1;


pub struct Ocl {
	pub platform: cl_platform_id,
	pub device: cl_device_id,
	pub context: cl_context,
	pub program: cl_program,
	pub command_queue: cl_command_queue,
}

impl Ocl {
	pub fn new() -> Ocl {
		let path_string = format!("{}/{}/{}", env!("P"), "bismit/src", KERNELS_FILE_NAME);
		let path_string_slice = path_string.as_slice();
		let kern_file_path = std::path::Path::new(path_string_slice);

		let mut kern_str: Vec<u8> = Vec::new();
		let kern_file = File::open(kern_file_path).unwrap().read_to_end(&mut kern_str);

		let kern_c_str = ffi::CString::from_vec(kern_str);

		let platform = new_platform();
		let devices: [cl_device_id; 2] = new_device(platform);
		let device: cl_device_id = devices[GPU_DEVICE];
		let context: cl_context = new_context(device);
		let program: cl_program = new_program(kern_c_str.as_ptr(), context, device);
		let command_queue: cl_command_queue = new_command_queue(context, device); 

		Ocl {
			platform: platform,
			device:  device,
			context:  context,
			program:  program,
			command_queue: command_queue,

		}
	}

	pub fn new_kernel(&self, name: &'static str, gws: WorkSize) -> Kernel {
		let mut err: cl_h::cl_int = 0;

		let kernel = unsafe {
			cl_h::clCreateKernel(self.program, ffi::CString::from_slice(name.as_bytes()).as_ptr(), &mut err)
		};

		must_succ("Ocl::new_kernel()", err);

		Kernel {
			kernel: kernel,
			name: name,
			arg_index: 0,
			command_queue: self.command_queue,
			context: self.context,
			gwo: WorkSize::Unspecified,
			gws: gws,
			lws: WorkSize::Unspecified,

		}
	}

	pub fn clone(&self) -> Ocl {
		Ocl {
			platform: self.platform,
			device:  self.device,
			context:  self.context,
			program:  self.program,
			command_queue: self.command_queue,

		}
	}

	pub fn new_write_buffer<T>(&self, data: &Vec<T>) -> cl_h::cl_mem {
		new_write_buffer(data, self.context)
	}

	pub fn new_read_buffer<T>(&self, data: &Vec<T>) -> cl_h::cl_mem {
		new_read_buffer(data, self.context)
	}


	pub fn enqueue_write_buffer<T>(
					&self,
					src: &Envoy<T>,
	) {

		unsafe {
			let err = cl_h::clEnqueueWriteBuffer(
						self.command_queue,
						src.buf,
						cl_h::CL_TRUE,
						0,
						(src.vec.len() * mem::size_of::<T>()) as libc::size_t,
						src.vec.as_ptr() as *const libc::c_void,
						0 as cl_h::cl_uint,
						ptr::null(),
						ptr::null_mut(),
			);
			must_succ("clEnqueueWriteBuffer()", err);
		}
	}


	pub fn enqueue_read_buffer<T>(
					&self,
					data: &Vec<T>,
					buffer: cl_h::cl_mem, 
	) {
		enqueue_read_buffer(data, buffer, self.command_queue);
	}

	pub fn enqueue_copy_buffer<T>(
					&self,
					src: &Envoy<T>,		//	src_buffer: cl_mem,
					dst: &Envoy<T>,		//	dst_buffer: cl_mem,
					src_offset: usize,
					dst_offset: usize,
					len_copy_bytes: usize,
	) {
		unsafe {
			let err = cl_h::clEnqueueCopyBuffer(
				self.command_queue,
				src.buf,				//	src_buffer,
				dst.buf,				//	dst_buffer,
				mem::transmute(src_offset),
				mem::transmute(dst_offset),
				mem::transmute(len_copy_bytes),
				0,
				ptr::null(),
				ptr::null_mut(),
			);
			must_succ("clEnqueueCopyBuffer()", err);
		}
	}

	pub fn enqueue_kernel(
				&self,
				kernel: cl_h::cl_kernel, 
				gws: usize,
	) { 
		enqueue_kernel(kernel, self.command_queue, gws);
	}

	pub fn release_components(&self) {

		unsafe {
			cl_h::clReleaseCommandQueue(self.command_queue);
			cl_h::clReleaseProgram(self.program);
			cl_h::clReleaseContext(self.context);
		}

	}
}



pub struct Kernel {
	kernel: cl_h::cl_kernel, //make this private!!!!!
	name: &'static str,
	arg_index: u32,
	command_queue: cl_h::cl_command_queue,
	context: cl_h::cl_context,
	gwo: WorkSize,
	gws: WorkSize,
	lws: WorkSize,
}

impl Kernel {
	pub fn gwo(mut self, gwo: WorkSize) -> Kernel {
		if gwo.dims() == self.gws.dims() {
			self.gwo = gwo
		} else {
			panic!("ocl::Kernel::gwo(): Work size mismatch.");
		}
		self
	}

	pub fn lws(mut self, lws: WorkSize) -> Kernel {
		if lws.dims() == self.gws.dims() {
			self.lws = lws;
		} else {
			panic!("ocl::Kernel::gwo(): Work size mismatch.");
		}
		self
	}

	pub fn arg_env<T>(mut self, envoy: &Envoy<T>) -> Kernel {
		self.new_arg_envoy(envoy);
		self
	}

	pub fn arg_scl<T>(mut self, scalar: T) -> Kernel {
		self.new_arg_scalar(scalar);
		self
	}

	pub fn arg_loc<T>(mut self, type_sample: T, length: usize) -> Kernel {
		self.new_arg_local(type_sample, length);
		self
	}

	pub fn new_arg_envoy<T>(&mut self, envoy: &Envoy<T>) {
		let buf = &envoy.buf;

		self.set_kernel_arg(
			mem::size_of::<cl_h::cl_mem>() as libc::size_t, 
			(buf as *const cl_h::cl_mem) as *const libc::c_void,
		)
	}

	pub fn new_arg_scalar<T>(&mut self, scalar: T) {
		unsafe {
			self.set_kernel_arg(
				mem::size_of::<T>() as libc::size_t, 
				mem::transmute(&scalar),
			)
		}
	}

	pub fn new_arg_local<T>(&mut self, type_sample: T, length: usize) {

		self.set_kernel_arg(
			(mem::size_of::<T>() * length) as libc::size_t,
			ptr::null(),
		)
	}

	fn set_kernel_arg(&mut self, arg_size: libc::size_t, arg_value: *const libc::c_void) {
		let err = unsafe {
			cl_h::clSetKernelArg(
						self.kernel, 
						self.arg_index, 
						arg_size, 
						arg_value,
			)
		};
		must_succ("clSetKernelArg()", err);
		//println!("Adding Kernel Argument: {}", self.arg_index);
		self.arg_index += 1;
		//self
	}

	pub fn enqueue(&self) {

			// TODO: VERIFY THE DIMENSIONS OF ALL THE WORKSIZES

		let c_gws = self.gws.complete_worksize();
		let gws = (&c_gws as *const (usize, usize, usize)) as *const libc::size_t;

		let c_lws = self.lws.complete_worksize();
		let lws = (&c_lws as *const (usize, usize, usize)) as *const libc::size_t;


		unsafe {
			let err = cl_h::clEnqueueNDRangeKernel(
						self.command_queue,
						self.kernel,
						self.gws.dims(),				//	dims,
						self.gwo.as_ptr(),
						gws,
						lws,
						0,
						ptr::null(),
						ptr::null_mut(),
			);
			must_succ("clEnqueueNDRangeKernel()", err);
		}
	}
}


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum WorkSize {
	Unspecified,
	OneDim		(usize),
	TwoDim		(usize, usize),
	ThreeDim 	(usize, usize, usize),
}

impl WorkSize {
	pub fn dims(&self) -> cl_h::cl_uint {
		use self::WorkSize::*;
		match self {
			&ThreeDim(_, _, _) 	=> 3,
			&TwoDim(_, _) 		=> 2,
			&OneDim(_) 			=> 1,
			&Unspecified 		=> 0,
		}

	}

	pub fn complete_worksize(&self) -> (usize, usize, usize) {
		match self {
			&WorkSize::OneDim(x) => {
				(x, 1, 1)
			},
			&WorkSize::TwoDim(x, y) => {
				(x, y, 1)
			},
			&WorkSize::ThreeDim(x, y, z) => {
				(x, y, z)
			},
			_ => (1, 1, 1)
		}
	}

	/* THIS IS POTENTIALLY VERY BUGGY SINCE THE POINTER TO 's' IS LEFT DANGLING 
		FIX IT ASAP
	*/
	fn as_ptr(&self) -> *const libc::size_t {

		match self {
			&WorkSize::OneDim(x) => {
				let s = (x, 1, 1);
				(&s as *const (usize, usize, usize)) as *const libc::size_t
			},
			&WorkSize::TwoDim(x, y) => {
				let s = (x, y, 1);
				(&s as *const (usize, usize, usize)) as *const libc::size_t
			},
			&WorkSize::ThreeDim(x, y, z) => {
				let s = (x, y, z);
				(&s as *const (usize, usize, usize)) as *const libc::size_t
			},
			_ => ptr::null(),
		}
	}
}



fn to_error_str(err_code: cl_h::cl_int) -> String {
	let err_opt: Option<cl_h::CLStatus> = std::num::FromPrimitive::from_int(err_code as isize);
	match err_opt {
		Some(e) => e.to_string(),
		None => format!("Unknown Error Code: {}", err_code as isize)
	}
}


pub fn must_succ(message: &str, err: cl_h::cl_int) {
	if err != cl_h::CLStatus::CL_SUCCESS as cl_h::cl_int {
		panic!(format!("{} failed with code: {}", message, to_error_str(err)));
	}
}


// Create Platform and get ID
pub fn new_platform() -> cl_h::cl_platform_id {
	let mut num_platforms = 0 as cl_h::cl_uint;
	
	let mut err: cl_h::cl_int = unsafe { cl_h::clGetPlatformIDs(0, ptr::null_mut(), &mut num_platforms) };
	must_succ("clGetPlatformIDs()", err);

	unsafe {
		let mut platform: cl_h::cl_platform_id = 0 as cl_h::cl_platform_id;

		err = cl_h::clGetPlatformIDs(1, &mut platform, ptr::null_mut()); 
		must_succ("clGetPlatformIDs()", err);

		platform
	}
	
}

pub fn new_device(platform: cl_h::cl_platform_id) -> [cl_h::cl_device_id; 2] {
	let mut device: [cl_h::cl_device_id; 2] = [0 as cl_h::cl_device_id; 2];

	unsafe {
		let err = cl_h::clGetDeviceIDs(platform, cl_h::CL_DEVICE_TYPE_GPU, 2, device.as_mut_ptr(), ptr::null_mut());
		must_succ("clGetDeviceIDs()", err);
	}
	device
}

pub fn new_context(device: cl_h::cl_device_id) -> cl_h::cl_context {
	let mut err: cl_h::cl_int = 0;

	unsafe {
		let context: cl_h::cl_context = cl_h::clCreateContext(
						ptr::null(), 
						1, 
						&device, 
						mem::transmute(ptr::null::<fn()>()), 
						ptr::null_mut(), 
						&mut err);
		must_succ("clCreateContext()", err);
		context
	}

}

pub fn new_program(
				src_str: *const i8, 
				context: cl_h::cl_context, 
				device: cl_h::cl_device_id,
) -> cl_h::cl_program {
	let mut err: cl_h::cl_int = 0;

	unsafe {
		let program: cl_h::cl_program = cl_h::clCreateProgramWithSource(
					context, 
					1,
					&src_str,
					ptr::null(), 
					&mut err,
		);
		must_succ("clCreateProgramWithSource()", err);

		err = cl_h::clBuildProgram(
					program,
					0, 
					ptr::null(), 
					ffi::CString::from_slice(BUILD_OPTIONS.as_bytes()).as_ptr(), 
					mem::transmute(ptr::null::<fn()>()), 
					ptr::null_mut(),
		);
		if err != 0i32 {
			program_build_info(program, device);
		}
		must_succ("clBuildProgram()", err);

		program
	}
}

pub fn new_kernel(program: cl_h::cl_program, kernel_name: &str) -> cl_h::cl_kernel {
	let mut err: cl_h::cl_int = 0;
	unsafe {
		let kernel = cl_h::clCreateKernel(program, ffi::CString::from_slice(kernel_name.as_bytes()).as_ptr(), &mut err);
		must_succ("clCreateKernel()", err);
		kernel
	}
}

pub fn new_command_queue(
				context: cl_h::cl_context, 
				device: cl_h::cl_device_id,
) -> cl_h::cl_command_queue {
	let mut err: cl_h::cl_int = 0;

	unsafe {
		let cq: cl_h::cl_command_queue = cl_h::clCreateCommandQueue(
					context, 
					device, 
					cl_h::CL_QUEUE_PROFILING_ENABLE, 
					&mut err
		);
		must_succ("clCreateCommandQueue()", err);
		cq
	}
}


pub fn new_buffer<T>(data: &Vec<T>, context: cl_h::cl_context) -> cl_h::cl_mem {
	let mut err: cl_h::cl_int = 0;
	unsafe {
		let buf = cl_h::clCreateBuffer(
					context, 
					cl_h::CL_MEM_READ_WRITE | cl_h::CL_MEM_COPY_HOST_PTR, 
					(data.len() * mem::size_of::<T>()) as u64,
					data.as_ptr() as *mut libc::c_void, 
					//ptr::null_mut(),
					&mut err,
		);
		must_succ("new_buffer", err);
		buf
	}
}

pub fn new_write_buffer<T>(data: &Vec<T>, context: cl_h::cl_context) -> cl_h::cl_mem {
	let mut err: cl_h::cl_int = 0;
	unsafe {
		let buf = cl_h::clCreateBuffer(
					context, 
					cl_h::CL_MEM_READ_ONLY | cl_h::CL_MEM_COPY_HOST_PTR, 
					(data.len() * mem::size_of::<T>()) as u64,
					data.as_ptr() as *mut libc::c_void, 
					//ptr::null_mut(),
					&mut err,
		);
		must_succ("new_write_buffer", err);
		buf
	}
}

pub fn new_read_buffer<T>(data: &Vec<T>, context: cl_h::cl_context) -> cl_h::cl_mem {
	let mut err: cl_h::cl_int = 0;
	unsafe {
		let buf = cl_h::clCreateBuffer(
					context, 
					cl_h::CL_MEM_WRITE_ONLY, 
					(data.len() * mem::size_of::<T>()) as u64, 
					ptr::null_mut(), 
					&mut err,
		);
		must_succ("new_read_buffer", err);
		buf
	}
}


pub fn enqueue_write_buffer<T>(
					data: &Vec<T>,
					buffer: cl_h::cl_mem, 
					command_queue: cl_h::cl_command_queue,
					offset: usize,
	) {

		unsafe {
			let err = cl_h::clEnqueueWriteBuffer(
						command_queue,
						buffer,
						cl_h::CL_TRUE,
						mem::transmute(offset),
						(data.len() * mem::size_of::<T>()) as libc::size_t,
						data.as_ptr() as *const libc::c_void,
						0 as cl_h::cl_uint,
						ptr::null(),
						ptr::null_mut(),
			);
			must_succ("clEnqueueWriteBuffer()", err);
		}
	}


pub fn enqueue_read_buffer<T>(
				data: &Vec<T>,
				buffer: cl_h::cl_mem, 
				command_queue: cl_h::cl_command_queue,
) {
	unsafe {
		let err = cl_h::clEnqueueReadBuffer(
					command_queue, 
					buffer, 
					cl_h::CL_TRUE, 
					0, 
					(data.len() * mem::size_of::<T>()) as libc::size_t, 
					data.as_ptr() as *mut libc::c_void, 
					0, 
					ptr::null(), 
					ptr::null_mut(),
		);
		must_succ("clEnqueueReadBuffer()", err);
	}
}


pub fn set_kernel_arg<T>(arg_index: cl_h::cl_uint, buffer: T, kernel: cl_h::cl_kernel) {
	unsafe {
		let err = cl_h::clSetKernelArg(
					kernel, 
					arg_index, 
					mem::size_of::<T>() as u64, 
					mem::transmute(&buffer),
		);
		must_succ("clSetKernelArg()", err);
	}
}

pub fn enqueue_kernel(
				command_queue: cl_h::cl_command_queue, 
				kernel: cl_h::cl_kernel, 
				gws: usize,
) {
	unsafe {
		let err = cl_h::clEnqueueNDRangeKernel(
					command_queue,
					kernel,
					1 as cl_uint,
					ptr::null(),
					mem::transmute(&gws),
					ptr::null(),
					0,
					ptr::null(),
					ptr::null_mut(),
		);
		must_succ("clEnqueueNDRangeKernel()", err);
	}
}

pub fn enqueue_2d_kernel(
				command_queue: cl_h::cl_command_queue,
				kernel: cl_kernel, 
				//dims: cl_uint,
				gwo_o: Option<&(usize, usize)>,
				gws: &(usize, usize),
				lws: Option<&(usize, usize)>,
) {
	let gwo = match gwo_o {
		Some(x) =>	(x as *const (usize, usize)) as *const libc::size_t,
		None 	=>	ptr::null(),
	};

	let lws = match lws {
		Some(x) =>	(x as *const (usize, usize)) as *const libc::size_t,
		None 	=>	ptr::null(),
	};

	unsafe {
		let err = cl_h::clEnqueueNDRangeKernel(
					command_queue,
					kernel,
					2,				//	dims,
					gwo,
					(gws as *const (usize, usize)) as *const libc::size_t,
					lws,
					0,
					ptr::null(),
					ptr::null_mut(),
		);
		must_succ("clEnqueueNDRangeKernel()", err);
	}
}

pub fn enqueue_3d_kernel(
				command_queue: cl_h::cl_command_queue,
				kernel: cl_kernel, 
				gwo_o: Option<&(usize, usize, usize)>,
				gws: &(usize, usize, usize),
				lws: Option<&(usize, usize, usize)>,
) {
	let gwo = match gwo_o {
		Some(x) =>	(x as *const (usize, usize, usize)) as *const libc::size_t,
		None 	=>	ptr::null(),
	};

	let lws = match lws {
		Some(x) =>	(x as *const (usize, usize, usize)) as *const libc::size_t,
		None 	=>	ptr::null(),
	};

	unsafe {
		let err = cl_h::clEnqueueNDRangeKernel(
					command_queue,
					kernel,
					3,
					gwo,
					(gws as *const (usize, usize, usize)) as *const libc::size_t,
					lws,
					0,
					ptr::null(),
					ptr::null_mut(),
		);
		must_succ("clEnqueueNDRangeKernel()", err);
	}
}

pub fn cl_finish(command_queue: cl_h::cl_command_queue) -> cl_h::cl_int {
	unsafe{	cl_h::clFinish(command_queue) }
}


pub fn mem_object_info_size(object: cl_h::cl_mem) -> libc::size_t {
	unsafe {
		let mut size: libc::size_t = 0;
		let err = cl_h::clGetMemObjectInfo(
					object,
					cl_h::CL_MEM_SIZE,
					mem::size_of::<libc::size_t>() as libc::size_t,
					(&mut size as *mut u64) as *mut libc::c_void,
					ptr::null_mut()
		);
		must_succ("clGetMemObjectInfo", err);
		size
	}
}

pub fn len(object: cl_h::cl_mem) -> usize {
	mem_object_info_size(object) as usize / mem::size_of::<f32>()
}

pub fn release_mem_object(obj: cl_h::cl_mem) {
	unsafe {
		cl_h::clReleaseMemObject(obj);
	}
}

pub fn release_kernel(
	kernel: cl_h::cl_kernel, 
			) {
	unsafe {
		cl_h::clReleaseKernel(kernel);
	}
}

pub fn release_components(
				kernel: cl_h::cl_kernel, 
				command_queue: cl_h::cl_command_queue, 
				program: cl_h::cl_program, 
				context: cl_h::cl_context,
) {
	unsafe {
		cl_h::clReleaseKernel(kernel);
		cl_h::clReleaseCommandQueue(command_queue);
		cl_h::clReleaseProgram(program);
		cl_h::clReleaseContext(context);
	}
}
	

pub fn platform_info(platform: cl_h::cl_platform_id) {
	let mut size = 0 as libc::size_t;

	unsafe {
		let name = cl_h::CL_PLATFORM_NAME as cl_h::cl_device_info;
        let mut err = cl_h::clGetPlatformInfo(
					platform,
					name,
					0,
					ptr::null_mut(),
					&mut size,
		);
		must_succ("clGetPlatformInfo(size)", err);
		
		let mut param_value: Vec<u8> = iter::repeat(32u8).take(size as usize).collect();
        err = cl_h::clGetPlatformInfo(
					platform,
					name,
					size,
					param_value.as_mut_ptr() as *mut libc::c_void,
					ptr::null_mut(),
		);
        must_succ("clGetPlatformInfo()", err);
        println!("*** Platform Name ({}): {}", name, cstring_to_string(param_value));
    }
}

pub fn program_build_info(program: cl_h::cl_program, device_id: cl_h::cl_device_id) -> Box<String> {
	let mut size = 0 as libc::size_t;

	unsafe {
		let name = cl_h::CL_PROGRAM_BUILD_LOG as cl_h::cl_program_build_info;
        let mut err = cl_h::clGetProgramBuildInfo(
					program,
					device_id,
					name,
					0,
					ptr::null_mut(),
					&mut size,
		);
		must_succ("clGetProgramBuildInfo(size)", err);
			
        let mut program_build_info: Vec<u8> = iter::repeat(32u8).take(size as usize).collect();

        err = cl_h::clGetProgramBuildInfo(
					program,
					device_id,
					name,
					size,
					program_build_info.as_mut_ptr() as *mut libc::c_void,
					ptr::null_mut(),
		);
        must_succ("clGetProgramBuildInfo()", err);

        let pbi = cstring_to_string(program_build_info);

        println!("*** Program Info ({}): \n {}", name, pbi);

        let rs: Box<String> = Box::new(pbi);
        rs
	}
}

pub fn print_junk(
				platform: cl_h::cl_platform_id, 
				device: cl_h::cl_device_id, 
				program: cl_h::cl_program, 
				kernel: cl_h::cl_kernel,
) {
	println!("");
	let mut size = 0 as libc::size_t;

	// Get Platform Name
	platform_info(platform);
	// Get Device Name
	let name = cl_h::CL_DEVICE_NAME as cl_h::cl_device_info;

	let mut err = unsafe { cl_h::clGetDeviceInfo(
					device,
					name,
					0,
					ptr::null_mut(),
					&mut size,
	) }; 
	must_succ("clGetPlatformInfo(size)", err);
	unsafe {
        let mut device_info: Vec<u8> = iter::repeat(32u8).take(size as usize).collect();
        err = cl_h::clGetDeviceInfo(
					device,
					name,
					size,
					device_info.as_mut_ptr() as *mut libc::c_void,
					ptr::null_mut(),
		);
        must_succ("clGetDeviceInfo()", err);
        println!("*** Device Name ({}): {}", name, cstring_to_string(device_info));
	}

	//Get Program Info
	unsafe {
		let name = cl_h::CL_PROGRAM_SOURCE as cl_h::cl_program_info;
        err = cl_h::clGetProgramInfo(
					program,
					name,
					0,
					ptr::null_mut(),
					&mut size,
		);
		must_succ("clGetProgramInfo(size)", err);
			
        let mut program_info: Vec<u8> = iter::repeat(32u8).take(size as usize).collect();

        err = cl_h::clGetProgramInfo(
					program,
					name,
					size,
					program_info.as_mut_ptr() as *mut libc::c_void,
					//program_info as *mut libc::c_void,
					ptr::null_mut(),
		);
        must_succ("clGetProgramInfo()", err);
        println!("*** Program Info ({}): \n {}", name, cstring_to_string(program_info));
	}
	println!("");
	//Get Kernel Name
	unsafe {
		let name = cl_h::CL_KERNEL_NUM_ARGS as cl_h::cl_uint;

        err = cl_h::clGetKernelInfo(
					kernel,
					name,
					0,
					ptr::null_mut(),
					&mut size,
		);
		must_succ("clGetKernelInfo(size)", err);

        let kernel_info = 5 as cl_h::cl_uint;

        err = cl_h::clGetKernelInfo(
					kernel,
					name,
					size,
					mem::transmute(&kernel_info),
					ptr::null_mut(),
		);
		
        must_succ("clGetKernelInfo()", err);
        println!("*** Kernel Info: ({})\n{}", name, kernel_info);
	}
	println!("");
}



fn empty_cstring(s: usize) -> ffi::CString {
	std::ffi::CString::from_vec(iter::repeat(32u8).take(s).collect())
}

fn cstring_to_string(cs: Vec<u8>) -> String {
	String::from_utf8(cs).unwrap()
}

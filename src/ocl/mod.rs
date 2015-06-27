use std;
use std::ptr;
use std::mem;
use std::io::{ Read };
use std::fs::{ File };
use std::ffi;
use std::iter;
use std::collections::{ HashMap };
use std::fmt::{ Display };
use num::{ self, Integer, FromPrimitive };
use libc;


pub use self::cl_h::{ cl_platform_id, cl_device_id, cl_context, cl_program, 
	cl_kernel, cl_command_queue, cl_float, cl_mem, cl_event, cl_char, cl_uchar, 
	cl_short, cl_ushort, cl_int, cl_uint, cl_long, CLStatus, 
	clSetKernelArg, clEnqueueNDRangeKernel };
pub use self::kernel::{ Kernel };
pub use self::envoy::{ Envoy };
pub use self::work_size::{ WorkSize };
pub use self::build_options::{ BuildOptions, BuildOption };
pub use self::cortical_dimensions::{ CorticalDimensions };
use cmn;

mod cl_h;
mod envoy;
mod kernel;
mod work_size;
mod build_options;
mod cortical_dimensions;

pub const GPU_DEVICE: usize = 1;
pub const KERNELS_FILE_NAME: &'static str = "bismit.cl";

pub struct Ocl {
	pub platform: cl_platform_id,
	pub device: cl_device_id,
	pub context: cl_context,
	pub program: cl_program,
	pub command_queue: cl_command_queue,
}

impl Ocl {
	pub fn new(build_options: BuildOptions) -> Ocl {
		let path_string = format!("{}/{}/{}", env!("P"), "bismit/src", KERNELS_FILE_NAME);
		let path_string_slc = &path_string;
		let kern_file_path = std::path::Path::new(path_string_slc);
		let mut kern_str: Vec<u8> = Vec::new();
		let kern_file = File::open(kern_file_path).unwrap().read_to_end(&mut kern_str);
		let kern_c_str = ffi::CString::new(kern_str).ok().expect("Ocl::new(): kern_c_str");

		let platform = new_platform();
		let devices: [cl_device_id; 2] = new_device(platform);
		let device: cl_device_id = devices[GPU_DEVICE];
		let context: cl_context = new_context(device);
		let program: cl_program = new_program(kern_c_str.as_ptr(), build_options.to_string(), context, device);
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
			cl_h::clCreateKernel(
				self.program, 
				ffi::CString::new(name.as_bytes()).ok().unwrap().as_ptr(), 
				&mut err
			)
		};
		
		let err_pre = format!("Ocl::new_kernel({}):", name);
		must_succ(&err_pre, err);

		Kernel::new(kernel, name, self.command_queue, self.context, gws)	
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


	// <<<<< CONVERT FROM VEC TO SLICE >>>>>
	pub fn new_write_buffer<T>(&self, data: &[T]) -> cl_h::cl_mem {
		new_write_buffer(data, self.context)
	}

	// <<<<< CONVERT FROM VEC TO SLICE >>>>>
	pub fn new_read_buffer<T>(&self, data: &[T]) -> cl_h::cl_mem {
		new_read_buffer(data, self.context)
	}

	// <<<<< CONVERT FROM VEC TO SLICE >>>>>
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


	// <<<<< CONVERT FROM VEC TO SLICE >>>>>
	pub fn enqueue_read_buffer<T>(
					&self,
					data: &[T],
					buffer: cl_h::cl_mem, 
	) {
		enqueue_read_buffer(data, buffer, self.command_queue, 0);
	}

	// <<<<< CONVERT FROM VEC TO SLICE >>>>>
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

	pub fn get_max_work_group_size(&self) -> u32 {
		let max_work_group_size: u64 = 0;

		let mut err = unsafe { 
			cl_h::clGetDeviceInfo(
				self.device,
				cl_h::CL_DEVICE_MAX_WORK_GROUP_SIZE,
				mem::size_of::<u64>() as u64,
				mem::transmute(&max_work_group_size),
				ptr::null_mut(),
			) 
		}; 

		must_succ("clGetDeviceInfo", err);

		max_work_group_size as u32
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
				build_opt: String,
				context: cl_h::cl_context, 
				device: cl_h::cl_device_id,
) -> cl_h::cl_program {

	let ocl_build_options_slc: &str = &build_opt;

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
					ffi::CString::new(ocl_build_options_slc.as_bytes()).ok().expect("ocl::new_program(): clBuildProgram").as_ptr(), 
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
		let kernel = cl_h::clCreateKernel(program, ffi::CString::new(kernel_name.as_bytes()).ok().expect("ocl::new_kernel(): clCreateKernel").as_ptr(), &mut err);
		let err_pre = format!("clCreateKernel({}):", kernel_name);
		must_succ(&err_pre, err);
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


// <<<<< CONVERT FROM VEC TO SLICE >>>>>
pub fn new_buffer<T>(data: &[T], context: cl_h::cl_context) -> cl_h::cl_mem {
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

// <<<<< CONVERT FROM VEC TO SLICE >>>>>
pub fn new_write_buffer<T>(data: &[T], context: cl_h::cl_context) -> cl_h::cl_mem {
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

// <<<<< CONVERT FROM VEC TO SLICE >>>>>
pub fn new_read_buffer<T>(data: &[T], context: cl_h::cl_context) -> cl_h::cl_mem {
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

// <<<<< CONVERT FROM VEC TO SLICE >>>>>
pub fn enqueue_write_buffer<T>(
					data: &[T],
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


// <<<<< CONVERT FROM VEC TO SLICE >>>>>
pub fn enqueue_read_buffer<T>(
				data: &[T],
				buffer: cl_h::cl_mem, 
				command_queue: cl_h::cl_command_queue,
				offset: usize,
) {
	unsafe {
		let err = cl_h::clEnqueueReadBuffer(
					command_queue, 
					buffer, 
					cl_h::CL_TRUE, 
					mem::transmute(offset), 
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

/*pub fn enqueue_2d_kernel(
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
}*/

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



/*fn empty_cstring(s: usize) -> ffi::CString {
	ffi::CString::new(iter::repeat(32u8).take(s).collect()).ok().expect("ocl::empty_cstring()")
}*/

fn cstring_to_string(cs: Vec<u8>) -> String {
	String::from_utf8(cs).unwrap()
}


pub fn must_succ(message: &str, err_code: cl_h::cl_int) {
	if err_code != cl_h::CLStatus::CL_SUCCESS as cl_h::cl_int {
		//format!("##### \n{} failed with code: {}\n\n #####", message, err_string(err_code));
		panic!(format!("\n\n#####> {} failed with code: {}\n\n", message, err_string(err_code)));
	}
}

fn err_string(err_code: cl_int) -> String {
	match CLStatus::from_i32(err_code) {
		Some(cls) => format!("{:?}", cls),
		None => format!("[Unknown Error Code: {}]", err_code as isize),
	}
}



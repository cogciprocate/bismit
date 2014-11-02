extern crate libc;

use std;
use std::ptr;
use std::mem;
use cl_h;
pub use cl_h::{cl_int, cl_platform_id, cl_device_id, cl_context, cl_program, cl_kernel, cl_command_queue, cl_float, cl_mem, cl_char, cl_ushort, cl_uint, cl_uchar};


fn to_error_str(err_code: cl_h::cl_int) -> String {
	let err_opt: Option<cl_h::CLStatus> = FromPrimitive::from_int(err_code as int);
	match err_opt {
		Some(status) => status.to_string(),
		None => format!("Unknown Error Code: {}", err_code as int)
	}
}

pub fn must_succ(message: &str, err: cl_h::cl_int) {
	if err != cl_h::CL_SUCCESS as cl_h::cl_int {
		fail!(format!("{} failed with code: {}", message, to_error_str(err)))
	}
}


// Create Platform and get ID
pub fn new_platform() -> cl_h::cl_platform_id {
	let mut num_platforms = 0 as cl_h::cl_uint;
	
	let mut err: cl_h::cl_int = unsafe { cl_h::clGetPlatformIDs(0, ptr::mut_null(), &mut num_platforms) };
	must_succ("clGetPlatformIDs()", err);

	unsafe {
		let mut platform: cl_h::cl_platform_id = 0 as cl_h::cl_platform_id;

		err = cl_h::clGetPlatformIDs(1, &mut platform, ptr::mut_null()); 
		must_succ("clGetPlatformIDs()", err);

		platform
	}
	
}

pub fn new_device(platform: cl_h::cl_platform_id) -> cl_h::cl_device_id {
	let mut device: cl_h::cl_device_id = 0 as cl_h::cl_device_id;

	unsafe {
		let err = cl_h::clGetDeviceIDs(platform, cl_h::CL_DEVICE_TYPE_GPU, 1, &mut device, ptr::mut_null());
		must_succ("clGetDeviceIDs()", err);
	}
	device
}

pub fn new_context(device: cl_h::cl_device_id) -> cl_h::cl_context {
	let mut err: cl_h::cl_int = 0;

	unsafe {
		let context: cl_h::cl_context = cl_h::clCreateContext(ptr::null(), 1, &device, mem::transmute(ptr::null::<||>()), ptr::mut_null(), &mut err);
		must_succ("clCreateContext()", err);
		context
	}

}

pub fn new_program(
				src_str: &str, 
				context: cl_h::cl_context, 
				device: cl_h::cl_device_id,
			) -> cl_h::cl_program {
	let mut err: cl_h::cl_int = 0;

	unsafe {
		let program: cl_h::cl_program = src_str.to_c_str().with_ref(|src_str| {
			let prog = cl_h::clCreateProgramWithSource(
						context, 
						1,
						&src_str,
						ptr::null(), 
						&mut err,
			);
			must_succ("clCreateProgramWithSource()", err);
			prog
		});

		err = cl_h::clBuildProgram(
					program,
					0, 
					ptr::null(), 
					"-cl-denorms-are-zero -cl-fast-relaxed-math".to_c_str().as_ptr(), 
					mem::transmute(ptr::null::<||>()), 
					ptr::mut_null(),
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
		let kernel = cl_h::clCreateKernel(program, kernel_name.to_c_str().as_ptr(), &mut err);
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

pub fn new_write_buffer<T>(data: &Vec<T>, context: cl_h::cl_context) -> cl_h::cl_mem {
	let mut err: cl_h::cl_int = 0;
	unsafe {
		let buf = cl_h::clCreateBuffer(
					context, 
					cl_h::CL_MEM_READ_ONLY | cl_h::CL_MEM_COPY_HOST_PTR, 
					(data.len() * mem::size_of::<T>()) as u64,
					data.as_ptr() as *mut libc::c_void, 
					//ptr::mut_null(),
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
					ptr::mut_null(), 
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
			) {
	unsafe {
		let err = cl_h::clEnqueueWriteBuffer(
					command_queue,
					buffer,
					cl_h::CL_TRUE,
					0,
					(data.len() * mem::size_of::<T>()) as libc::size_t,
					data.as_ptr() as *const libc::c_void,
					0 as cl_h::cl_uint,
					ptr::null(),
					ptr::mut_null(),
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
					ptr::mut_null(),
		);
		must_succ("clEnqueueReadBuffer()", err);
	}
}

pub fn set_kernel_arg(arg_index: cl_h::cl_uint, buffer: cl_h::cl_mem, kernel: cl_h::cl_kernel) {
	unsafe {
		let err = cl_h::clSetKernelArg(
					kernel, 
					arg_index, 
					mem::size_of::<cl_mem>() as u64, 
					mem::transmute(&buffer),
		);
		must_succ("clSetKernelArg()", err);
	}
}

pub fn enqueue_kernel(
				kernel: cl_h::cl_kernel, 
				command_queue: cl_h::cl_command_queue, 
				gws: uint,
			) {
	unsafe {
		let err = cl_h::clEnqueueNDRangeKernel(
					command_queue,
					kernel,
					1,
					ptr::null(),
					mem::transmute(&gws),
					ptr::null(),
					0,
					ptr::null(),
					ptr::mut_null(),
		);
		must_succ("clEnqueueNDRangeKernel()", err);
	}
}


pub fn mem_object_info_size(object: cl_h::cl_mem) -> libc::size_t {
	unsafe {
		let mut size: libc::size_t = 0;
		let err = cl_h::clGetMemObjectInfo(
					object,
					cl_h::CL_MEM_SIZE,
					mem::size_of::<libc::size_t>() as libc::size_t,
					(&mut size as *mut u64) as *mut libc::c_void,
					ptr::mut_null()
		);
		must_succ("clGetMemObjectInfo", err);
		size
	}
}

pub fn len(object: cl_h::cl_mem) -> uint {
	mem_object_info_size(object) as uint / mem::size_of::<f32>()
}

pub fn release_mem_object(obj: cl_h::cl_mem) {
	unsafe {
		cl_h::clReleaseMemObject(obj);
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
					ptr::mut_null(),
					&mut size,
		);
		must_succ("clGetPlatformInfo(size)", err);
		let mut plat_info: std::c_str::CString = std::string::String::from_char(size as uint, 'a').to_c_str();
        err = cl_h::clGetPlatformInfo(
					platform,
					name,
					size,
					plat_info.as_mut_ptr() as *mut libc::c_void,
					ptr::mut_null(),
		);
        must_succ("clGetPlatformInfo()", err);
        println!("*** Platform Name ({}): {}", name, plat_info);
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
					ptr::mut_null(),
					&mut size,
		);
		must_succ("clGetProgramBuildInfo(size)", err);
			
        let mut program_build_info: std::c_str::CString = std::string::String::from_char(size as uint, 'a').to_c_str();

        err = cl_h::clGetProgramBuildInfo(
					program,
					device_id,
					name,
					size,
					program_build_info.as_mut_ptr() as *mut libc::c_void,
					ptr::mut_null(),
		);
        must_succ("clGetProgramBuildInfo()", err);
        println!("*** Program Info ({}): \n {}", name, program_build_info);

        let rs: Box<String> = box program_build_info.as_str().to_string();
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
					ptr::mut_null(),
					&mut size,
	) };
	must_succ("clGetPlatformInfo(size)", err);
	unsafe {
        let mut device_info: std::c_str::CString = std::string::String::from_char(size as uint, ' ').to_c_str();
        err = cl_h::clGetDeviceInfo(
					device,
					name,
					size,
					device_info.as_mut_ptr() as *mut libc::c_void,
					ptr::mut_null(),
		);
        must_succ("clGetDeviceInfo()", err);
        println!("*** Device Name ({}): {}", name, device_info);
	}

	//Get Program Info
	unsafe {
		let name = cl_h::CL_PROGRAM_SOURCE as cl_h::cl_program_info;
        err = cl_h::clGetProgramInfo(
					program,
					name,
					0,
					ptr::mut_null(),
					&mut size,
		);
		must_succ("clGetProgramInfo(size)", err);
			
        let mut program_info: std::c_str::CString = std::string::String::from_char(size as uint, 'a').to_c_str();
        //let mut program_info: cl_program_info = 0 as cl_program_info;
        
        //println!("program_info string length: {}", program_info.len())
        err = cl_h::clGetProgramInfo(
					program,
					name,
					size,
					program_info.as_mut_ptr() as *mut libc::c_void,
					//program_info as *mut libc::c_void,
					ptr::mut_null(),
		);
        must_succ("clGetProgramInfo()", err);
        println!("*** Program Info ({}): \n {}", name, program_info);
	}
	println!("");
	//Get Kernel Name
	unsafe {
		let name = cl_h::CL_KERNEL_NUM_ARGS as cl_h::cl_uint;

        err = cl_h::clGetKernelInfo(
					kernel,
					name,
					0,
					ptr::mut_null(),
					&mut size,
		);
		must_succ("clGetKernelInfo(size)", err);

        //let mut kernel_info: std::c_str::CString = std::string::String::from_char(size as uint, ' ').to_c_str();
        let kernel_info = 5 as cl_h::cl_uint;
        //let kiptr = &kernel_info;

        //let mut test: *mut libc::c_void = (kernel_info.as_mut_ptr()) as *mut libc::c_void;

        //println!("kernel_info string length: {}", kernel_info.len())
        err = cl_h::clGetKernelInfo(
					kernel,
					name,
					size,
					//kernel_info.as_mut_ptr() as *mut libc::c_void,
					mem::transmute(&kernel_info),
					ptr::mut_null(),
		);
		
        must_succ("clGetKernelInfo()", err);
        println!("*** Kernel Info: ({})\n{}", name, kernel_info);
	}
	println!("");
}

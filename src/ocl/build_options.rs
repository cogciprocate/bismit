pub struct BuildOptions {
	options: Vec<BuildOption>,
	string: String,
	kernel_file_names: Vec<String>,
}

impl BuildOptions {
	pub fn new(cl_options: &'static str) -> BuildOptions {
		let mut bo = BuildOptions {
			options: Vec::with_capacity(1 << 5),
			string: String::with_capacity(1 << 11),
			kernel_file_names: Vec::with_capacity(20),
		};

		bo.str(cl_options)
	}

	pub fn str(mut self, st: &'static str) -> BuildOptions {
		self.string.push_str(st);
		self
	}

	pub fn opt(mut self, name: &'static str, val: i32) -> BuildOptions {
		self.options.push(BuildOption::new(name, val));
		self
	}

	pub fn add(mut self, bo: BuildOption) -> BuildOptions {
		self.options.push(bo);
		self
	}

	pub fn kern(&mut self, file_name: String) {
		self.kernel_file_names.push(file_name);
	}

	pub fn as_slc(&mut self) -> &str {
		&self.string
	}

	pub fn to_string(mut self) -> String {
		for option in self.options.iter_mut() {
			self.string.push_str(option.as_slc());
		}
		//println!("\n\tBuildOptions::as_slc(): length: {}, \n \tstring: {}", self.string.len(), self.string);
		self.string
	}

	pub fn kernel_file_names(&self) -> &Vec<String> {
		&self.kernel_file_names
	}
}



pub struct BuildOption {
	name: &'static str,
	val: i32,
	string: String,
}

impl BuildOption {
	pub fn new(name: &'static str, val: i32) -> BuildOption {
		BuildOption {
			name: name,
			val: val,
			string: String::with_capacity(name.len()),
		}
	}

	pub fn as_slc(&mut self) -> &str {
		self.string = format!(" -D{}={}", self.name, self.val);

		&self.string
	}
}

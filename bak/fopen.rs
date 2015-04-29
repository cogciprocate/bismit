use std::path
use std::io

fn fopen() {
	//let mut files: Vec<std::io::IoResult<std::io::File>> = Vec::with_capacity(3);

	let mut cwd_path: std::path::Path = std::path::Path::new(".");

	let mut cwd_files: std::io::IoResult<Vec<Path>> = std::io::fs::readdir(&cwd_path);

	for file_path in cwd_files.unwrap().mut_iter() {
		let mut fpp = path_printable(file_path);

		let needle: &str = ".txt";
		println!("{:s} ({}): {}", *fpp, fpp.len(), fpp.as_slice().ends_with(needle));

		if (fpp.as_slice().ends_with(needle)) {
			let file = std::io::File::open(file_path);
			let mut reader = std::io::BufferedReader::new(file);
			let mut i: int = 0;
			let mut division_total: int = 0;

			loop {
				match reader.read_line() {
					Ok(line) => {
						i += 1;
						division_total += from_str(line.as_slice().trim()).unwrap();
						//println!("{}", line);

					}
					Err(e) => {
						println!("read_line(): {}", e);
						break;
					}
				}
			}
			println!("{} Runs for {} -- Total: {}", i, *fpp, division_total);
		}
	}
}

fn path_printable(path: &std::path::Path) -> Box<String> {
	box String::from_utf8(Vec::from_slice(path.filename().unwrap())).unwrap()
}

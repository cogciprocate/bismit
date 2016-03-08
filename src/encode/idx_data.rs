use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::{PathBuf, Path};
use std::iter;
use std::ops::{Index, IndexMut, Range, RangeTo, RangeFrom, RangeFull};
// use find_folder::Search;

pub struct IdxData {
    file_path: PathBuf,
    file_reader: BufReader<File>,
    data: Vec<u8>,
    dims: Vec<usize>,
}

impl IdxData {
    /// Reads a .idx file all at once into a vector `data` or piece at a time via the
    /// `.read_item()` function.
    ///
    /// # Panics
    /// File::open(), BufReader::read()[x2], BufReader::read_to_end(), Invalid idx file format,
    ///
    /// [FIXME]: Consolidate error handling and return CmnResult instead of panicing.
    pub fn new(file_path_str: &str, stream_mode: bool) -> IdxData {
        // let path_string = format!("{}/{}/{}", env!("P"), "bismit", file_name);
        let fp_raw = Path::new(file_path_str);

        let file_path = if fp_raw.is_file() {
            fp_raw
        } else {
            // TODO: BRING THIS BACK EVENTUALLY
            // if fp_raw.is_relative() {
            //     let mut file_path = PathBuf::new();
            //     let mut fp_iter = fp_raw.iter();
            //     // println!("#### FP_COMP.len(): {}", fp_comp.len());
            //     let root_folder = fp_iter.next().expect("1").to_str().expect("2");
            //     println!("#### ROOT_FOLDER: {:?}", root_folder);
            //     let file_root = Search::ParentsThenKids(3, 3).for_folder(root_folder).expect("3");
            //     println!("#### FILE_ROOT: {}", file_root.display());
            // } else {
            //     // TODO: SWITCH TO ERR RETURN
            //     panic!("IdxData::new(): Invalid file path: '{}'.", fp_raw.display());
            // }
            // TEMPORARY:
            fp_raw
        };

        // let file_path = PathBuf::from(&file_path_str);
        let path_display = file_path.display();

        let file = match File::open(&file_path) {
            Err(why) => panic!("\ncouldn't open '{}': {}", path_display, Error::description(&why)),
            Ok(file) => file,
        };

        let mut reader = BufReader::new(file);
        let mut header_magic: Vec<u8> = iter::repeat(0).take(4).collect();

        match reader.read(&mut header_magic[..]) {
            Err(why) => panic!("\ncouldn't read '{}': {}", path_display, Error::description(&why)),
            Ok(_) => (), //println!("{} contains:\n{:?}\n{} bytes read.", path_display, header_magic, bytes),
        }

        let magic_data_type = header_magic[2];
        let magic_dims = header_magic[3] as usize;
        assert!(magic_data_type == 8, format!("IDX file: '{}' does not contain unsigned bytes.", path_display));

        let mut header_dim_sizes_bytes: Vec<u8> = iter::repeat(0).take(magic_dims * 4).collect();

        match reader.read(&mut header_dim_sizes_bytes[..]) {
            Err(why) => panic!("\ncouldn't read '{}': {}", path_display, Error::description(&why)),
            Ok(_) => (), //println!("{} contains:\n{:?}\n{} bytes read.", path_display, header_dim_sizes_bytes, bytes),
        }
        
        // let ttl_header_len = 4 + (magic_dims * 4);
        let mut dim_sizes: Vec<usize> = iter::repeat(0).take(magic_dims).collect();

        for i in 0..magic_dims {
            let header_ofs = 4 * i;
            dim_sizes[i] = 
                (header_dim_sizes_bytes[header_ofs] as usize) << 24 
                | (header_dim_sizes_bytes[header_ofs + 1] as usize) << 16 
                | (header_dim_sizes_bytes[header_ofs + 2] as usize) << 8 
                | (header_dim_sizes_bytes[header_ofs + 3] as usize)
            ;
        }

        let mut buffer_cap: usize = 1;

        for &size in &dim_sizes {
            buffer_cap *= size as usize;
        }

        let mut idx_buffer: Vec<u8> = Vec::with_capacity(buffer_cap);
        
        if !stream_mode {
            // TODO: CONVERT TO STREAM
            match reader.read_to_end(&mut idx_buffer) {
                Err(why) => panic!("\ncouldn't read '{}': {}", path_display, Error::description(&why)),
                Ok(bytes) => println!("{}: {} bytes read.", path_display, bytes),
            }
        }

        let header_len_bytes = header_magic.len() + header_dim_sizes_bytes.len();

        match reader.seek(SeekFrom::Start(header_len_bytes as u64)) {
            Err(why) => panic!("\ncouldn't seek to '[{}]': {}", header_len_bytes, 
                Error::description(&why)),
            Ok(_) => (),
        }

        println!("IDXREADER: initialized with dimensions: {:?}", dim_sizes);

        IdxData {                          
            file_path: file_path.to_path_buf(),
            file_reader: reader,
            data: idx_buffer,
            dims: dim_sizes, 
        }
    }

    // TODO: RETURN RESULT
    pub fn read(&mut self, buf: &mut [u8]) {
        match self.file_reader.read_exact(buf) {
            Err(why) => panic!("\ncouldn't read '{}': {}", self.file_path.display(), 
                Error::description(&why)),
            Ok(_) => (), //println!("{} contains:\n{:?}\n{} bytes read.", path_display, header_dim_sizes_bytes, bytes),
        }
    }

    // Feels like this probably exists in std somewhere...
    pub fn read_into_vec(&mut self, bytes_to_read: usize, vec: &mut Vec<u8>) {
        let prev_len = vec.len();
        let new_len = prev_len + bytes_to_read;

        vec.reserve(bytes_to_read);
        debug_assert!(new_len <= vec.capacity());
        unsafe { vec.set_len(new_len); }

        let idx_range = prev_len..new_len;

        match self.file_reader.read_exact(&mut vec[idx_range]) {
            Err(why) => panic!("\ncouldn't read '{}': {}", self.file_path.display(), 
                Error::description(&why)),
            Ok(_) => (), //println!("{} contains:\n{:?}\n{} bytes read.", path_display, header_dim_sizes_bytes, bytes),
        }
    }

    #[inline]
    pub fn file_path(&self) -> &Path {
        self.file_path.as_path()
    }

    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }

    #[inline]
    pub fn dims(&self) -> &[usize] {
        &self.dims[..]
    }
}

impl Index<usize> for IdxData {
    type Output = u8;
    #[inline]
    fn index<'a>(&'a self, index: usize) -> &'a u8 {
        // &self.data[index]
        &(*self.data)[index]
    }
}
impl IndexMut<usize> for IdxData {
    #[inline]
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut u8 {
        // &mut self.data[index]
        &mut (*self.data)[index]
    }
}

impl Index<Range<usize>> for IdxData {
    type Output = [u8];

    #[inline]
    fn index(&self, index: Range<usize>) -> &[u8] {
        Index::index(&*self.data, index)
    }
}
impl Index<RangeTo<usize>> for IdxData {
    type Output = [u8];

    #[inline]
    fn index(&self, index: RangeTo<usize>) -> &[u8] {
        Index::index(&*self.data, index)
    }
}
impl Index<RangeFrom<usize>> for IdxData {
    type Output = [u8];

    #[inline]
    fn index(&self, index: RangeFrom<usize>) -> &[u8] {
        Index::index(&*self.data, index)
    }
}
impl Index<RangeFull> for IdxData {
    type Output = [u8];

    #[inline]
    fn index(&self, _index: RangeFull) -> &[u8] {
        &self.data
    }
}


// THE IDX FILE FORMAT

// the IDX file format is a simple format for vectors and multidimensional matrices of various numerical types.
// The basic format is

// magic number 
// size in dimension 0 
// size in dimension 1 
// size in dimension 2 
// ..... 
// size in dimension N 
// data

// The magic number is an integer (MSB first). The first 2 bytes are always 0.

// The third byte codes the type of the data: 
// 0x08: unsigned byte 
// 0x09: signed byte 
// 0x0B: short (2 bytes) 
// 0x0C: int (4 bytes) 
// 0x0D: float (4 bytes) 
// 0x0E: double (8 bytes)

// The 4-th byte codes the number of dimensions of the vector/matrix: 1 for vectors, 2 for matrices....

// The sizes in each dimension are 4-byte integers (MSB first, high endian, like in most non-Intel processors).

// The data is stored like in a C array, i.e. the index in the last dimension changes the fastest. 
  
// Happy hacking.


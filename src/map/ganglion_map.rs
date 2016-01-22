use std::ops::Range;

#[derive(Debug, Clone)]
pub struct GanglionMap {	
	tags: Vec<&'static str>,	
	v_sizes: Vec<u32>,
	u_sizes: Vec<u32>,
	idzs: Vec<u32>,
	physical_len: u32,
}

impl GanglionMap {
	pub fn new(
				tags: &[&'static str],
				v_sizes: &[u32],
				u_sizes: &[u32]) 
			-> GanglionMap 
	{
		assert!(tags.len() == v_sizes.len());
		assert!(tags.len() == u_sizes.len());
		let mut idzs = Vec::with_capacity(tags.len());
		let mut physical_len = 0u32;

		for i in 0..v_sizes.len() {
			idzs.push(physical_len);

			unsafe {				
				physical_len += *v_sizes.get_unchecked(i) * *u_sizes.get_unchecked(i);
			}
		}

		debug_assert!(tags.len() == idzs.len());

		GanglionMap {
			tags: tags.to_vec(),			
			v_sizes: v_sizes.to_vec(),
			u_sizes: u_sizes.to_vec(),
			idzs: idzs,
			physical_len: physical_len,
		}
	}

	pub fn slc_range(&self) -> Range<u8> {
		0..self.tags.len() as u8
	}

	// TODO: Make fancy with iterators.
	pub fn axn_count(&self, slc_range: Range<u8>) -> usize {
		let mut count = 0;

		for i in slc_range.clone() {
			count += self.v_sizes[i as usize] * self.u_sizes[i as usize];
		}

		count as usize
	}
}

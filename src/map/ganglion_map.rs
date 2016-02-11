use std::ops::Range;

/// Map of axons within a cortical area.
///
/// Generally used when exporting snapshots of axon space for visualization, debugging, etc.
///
// FEEL FREE TO RENAME
//
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

    #[inline]
    pub fn slc_id_range(&self) -> Range<u8> {
        0..self.tags.len() as u8
    }

    // TODO: Make fancy with iterators.
    pub fn axn_count(&self, slc_id_range: Range<u8>) -> usize {
        let mut count = 0;

        for i in slc_id_range.clone() {
            count += self.v_sizes[i as usize] * self.u_sizes[i as usize];
        }

        count as usize
    }

    pub fn axn_id_range(&self, slc_id_range: Range<u8>) -> Range<usize> {
        let start = slc_id_range.start as usize;
        let end = slc_id_range.end as usize;
        assert!(start < self.idzs.len());
        assert!(end <= self.idzs.len());

        // let axn_id_range = unsafe { 
        //     let axn_idz_start = *self.idzs.get_unchecked(start) as usize;
        //     let axn_idz_end = (*self.idzs.get_unchecked(end)
        //         + (*self.v_sizes.get_unchecked(end) * *self.u_sizes.get_unchecked(end))) as usize;
        //     axn_idz_start..axn_idz_end
        // };

        let axn_id_start = self.idzs[start] as usize;
        let axn_id_end = self.idzs[end - 1] as usize
            + (self.v_sizes[end - 1] as usize * self.u_sizes[end - 1] as usize);

        // (*self.idzs.get_unchecked(end)
        //     + (*self.v_sizes.get_unchecked(end) * *self.u_sizes.get_unchecked(end))) as usize;

        let axn_id_range = axn_id_start..axn_id_end;

        // println!("###### GanglionMap::axn_id_range(slc_id_range: {:?}):", slc_id_range);
        // println!("######    axn_id_range: {:?}", axn_id_range);

        axn_id_range
    }
}

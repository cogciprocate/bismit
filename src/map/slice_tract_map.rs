use std::ops::Range;
use std::fmt;

/// Map of axons within a cortical area.
///
/// Generally used when exporting snapshots of axon space for visualization, debugging, etc.
///
// FEEL FREE TO RENAME
//
#[derive(Debug, Clone)]
pub struct SliceTractMap {
    tags: Vec<String>,
    v_sizes: Vec<u32>,
    u_sizes: Vec<u32>,
    idzs: Vec<u32>,
    physical_len: u32,
}

impl SliceTractMap {
    pub fn new(
            tags: &[String],
            v_sizes: &[u32],
            u_sizes: &[u32])
            -> SliceTractMap
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

        SliceTractMap {
            tags: tags.to_vec(),
            v_sizes: v_sizes.to_vec(),
            u_sizes: u_sizes.to_vec(),
            idzs: idzs,
            physical_len: physical_len,
        }
    }

    pub fn slc_id_range(&self) -> Range<usize> {
        0..self.tags.len()
    }

    pub fn slc_dims(&self, slc_id: u8) -> (u32, u32) {
        assert!((slc_id as usize) < self.v_sizes.len(), "Slice id out of range.");
        (self.v_sizes[slc_id as usize], self.u_sizes[slc_id as usize])
    }

    // TODO: Make fancy with iterators.
    pub fn axn_count(&self, slc_id_range: Range<usize>) -> usize {
        let mut count = 0;

        for i in slc_id_range.clone() {
            count += self.v_sizes[i as usize] * self.u_sizes[i as usize];
        }

        count as usize
    }

    pub fn axn_id_range(&self, slc_id_range: Range<usize>) -> Range<usize> {
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

        axn_id_start..axn_id_end
    }

    pub fn tags_reversed(&self) -> Vec<String> {
        self.tags.iter().rev().map(|t| t.clone()).collect()
    }
}

impl fmt::Display for SliceTractMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.tags)
    }
}
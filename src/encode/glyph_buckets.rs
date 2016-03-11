use std::iter;
use find_folder::Search;
use super::IdxData;

const PRINT_DEBUG: bool = false;
const PRINT_EVERY: usize = 10000;

pub struct GlyphBuckets {
    buckets: Vec<Vec<u8>>,
    cursors: Vec<usize>,
    glyph_dims: (usize, usize),
}

impl GlyphBuckets {
    pub fn new() -> GlyphBuckets {
        let bucket_count = 10;
        let label_file = Search::ParentsThenKids(3, 3).for_folder("data").unwrap()
            .join("train-labels-idx1-ubyte");
        let image_file = Search::ParentsThenKids(3, 3).for_folder("data").unwrap()
            .join("train-images-idx3-ubyte");
        let labels = IdxData::new(label_file, false);
        let mut images = IdxData::new(image_file, true);

        assert!(images.dims().len() == 3, "GlyphBuckets::new(): Source idx file must have three \
            dimensions.");

        assert!(images.dims()[0] == labels.dims()[0], "GlyphBuckets::new(): The images file \
            ('{}') must contain the same number of elements as the labels file ('{}').", 
            images.file_path().display(), labels.file_path().display());

        let image_count = images.dims()[0];
        let image_dims = (images.dims()[1], images.dims()[2]);
        let image_len = image_dims.0 * image_dims.1;
        // Extra 20% bucket space:
        let bucket_len_approx = image_len * (image_count + image_count / 5) / bucket_count;

        let mut buckets: Vec<Vec<u8>> = (0..).take(bucket_count).map(
            |_| Vec::with_capacity(bucket_len_approx)).collect();

        for i in 0..image_count {
            let label: u8 = labels[i];
            assert!(label as usize <= bucket_count, "GlyphBuckets::new(): \
                label ({}) exceeds bucket count ({}).", label, bucket_count);

            // let img_idz = i * image_len;
            // let img_idn = img_idz + image_len;            

            // let image: &[u8] = &images[img_idz..img_idn];
            // buckets[label as usize].extend_from_slice(image);
            let mut bucket = &mut buckets[label as usize];
            // let prev_len = bucket.len();
            // let tar_range = prev_len..(prev_len + image_len);

            // bucket.reserve(image_len);
            // debug_assert!(prev_len + image_len >= bucket.capacity());
            // unsafe { bucket.set_len(prev_len + image_len); }
            images.read_into_vec(image_len, &mut bucket);

            if PRINT_DEBUG && i % PRINT_EVERY == 0 { 
            // if i >= 10000 && i < 10010 {
                println!("\nimage[{}]: bucket.len(): {}, bucket.capacity(): {}, label: {}", 
                    i, bucket.len(), bucket.capacity(), label);

                let new_img_range = (bucket.len() - image_len)..bucket.len();
                super::print_image(&mut bucket[new_img_range.clone()], image_dims);
            }
        }

        if PRINT_DEBUG { println!("\nLoaded {} images into buckets as follows {{capacity(len)}}: ", 
                image_count); }

        for i in 0..buckets.len() {
            buckets[i].shrink_to_fit();
            if PRINT_DEBUG { println!("bucket[{}]: {}({})", 
                i, buckets[i].capacity(), buckets[i].len()); }
        }

        let cursors = iter::repeat(0).take(buckets.len()).collect();

        GlyphBuckets {
            buckets: buckets,
            glyph_dims: image_dims,
            cursors: cursors,
        }
    }

    #[inline]
    pub fn glyph_dims(&self) -> (usize, usize) {
        self.glyph_dims
    }

    #[inline]
    pub fn glyph_len(&self) -> usize {
        self.glyph_dims.0 * self.glyph_dims.1
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.buckets.len()
    }

    #[inline]
    pub fn next_glyph(&mut self, bucket_id: usize/*, buf: &mut [u8]*/) -> &[u8] {
        assert!(bucket_id < self.buckets.len(), "GlyphBuckets::next_glyph(): \
            bucket_id ({}) exceeds bucket count ({}).", bucket_id, self.buckets.len());
        // assert!(buf.len() == self.glyph_len(), "GlyphBuckets::next_glyph(): \
        //     buffer length ({}) does not equal glyph length ({}).", buf.len(), self.glyph_len());
        let idz = self.cursors[bucket_id];
        let idn = idz + self.glyph_len();
        // buf.clone_from_slice(&self.buckets[bucket_id][idz..idn]);
        self.incr_cursor(bucket_id);
        
        &self.buckets[bucket_id][idz..idn]
    }

    #[inline]
    fn incr_cursor(&mut self, bucket_id: usize) {
        self.cursors[bucket_id] += self.glyph_len();

        if self.cursors[bucket_id] == self.buckets[bucket_id].len() {
            self.cursors[bucket_id] = 0;
            if PRINT_DEBUG { println!("Resetting cursor: {}", bucket_id) }
        } else if self.cursors[bucket_id] > self.buckets[bucket_id].len() {
            panic!("GlyphBuckets::incr_cursor(): Bucket length inconsistency while \
                resetting cursor for bucket: {}. {{ cursor: {}, bucket length: {}, \
                glyph length: {} }}", bucket_id, self.cursors[bucket_id], 
                self.buckets[bucket_id].len(), self.glyph_len());
        }
    }
}

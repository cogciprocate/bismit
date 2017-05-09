#![allow(dead_code, unused_variables, unused_mut)]




#[test]
pub fn overlap() {
    let cell_count = 128 * 128;
    // Number of 2nd order neighbors (neighbors of neighbors) which are also
    // neighbors for each cell.
    let mut cell_2ons = vec![0usize; cell_count];
    // Coordinates of the center of the cell-mesh-group for a mesh layer.
    let mut mesh_cntrs = Vec::with_capacity(cell_count);


}

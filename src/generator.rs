use noise::{NoiseFn, Perlin};
use ndarray::*;

pub fn height_generator(x: i32, z: i32) -> ArrayBase<OwnedRepr<u32>, Dim<[usize; 3]>> {
    let perlin = Perlin::new();

    let mut height_map: Array3<u32> = Array::ones((16, 63, 16));
    let air = ArrayView::from(&[0; 49408]).into_shape((16, 193, 16)).unwrap();
    height_map.append(Axis(1), air);
    let mut y: i32;
    for i in x..x+16 {
        for j in z..z+16 {
            y = (perlin.get([i as f64 / 100.0, j as f64 / 100.0])*10.0).round() as i32;
            height_map.slice_mut(s![i-x, 0..64+y, j-z]).fill(2);
        }
    }
    return height_map
}
// cuTile captures this device syntax; the host sees only the generated launcher.
#[cutile::module]
pub(super) mod jacobi_kernel {
    use cutile::core::*;

    #[cutile::entry()]
    fn jacobi(
        output: &mut Tensor<f32, { [16, 16] }>,
        center: &Tensor<f32, { [-1, -1] }>,
        north: &Tensor<f32, { [-1, -1] }>,
        south: &Tensor<f32, { [-1, -1] }>,
        east: &Tensor<f32, { [-1, -1] }>,
        west: &Tensor<f32, { [-1, -1] }>,
        rhs: &Tensor<f32, { [-1, -1] }>,
        omega: f32,
        h_squared: f32,
    ) {
        let c = center.load_like(output);
        let n = north.load_like(output);
        let s = south.load_like(output);
        let e = east.load_like(output);
        let w = west.load_like(output);
        let r = rhs.load_like(output);
        let om = omega.broadcast(output.shape());
        let h2 = h_squared.broadcast(output.shape());
        let one = 1.0_f32.broadcast(output.shape());
        let quarter = 0.25_f32.broadcast(output.shape());
        let candidate = quarter * (n + s + e + w - h2 * r);
        output.store((one - om) * c + om * candidate);
    }
}

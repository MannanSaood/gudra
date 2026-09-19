// cuTile captures this device syntax; the host sees only the generated launcher.
#[cutile::module]
pub(super) mod jacobi_kernel {
    // The DSL rewrites Tensor into rank-specific types and uses core traits.
    #[allow(clippy::wildcard_imports)]
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
        let center_values = center.load_like(output);
        let north_values = north.load_like(output);
        let south_values = south.load_like(output);
        let east_values = east.load_like(output);
        let west_values = west.load_like(output);
        let rhs_values = rhs.load_like(output);
        let weight = omega.broadcast(output.shape());
        let spacing_squared = h_squared.broadcast(output.shape());
        let one = 1.0_f32.broadcast(output.shape());
        let quarter = 0.25_f32.broadcast(output.shape());
        let candidate = quarter
            * (north_values + south_values + east_values + west_values
                - spacing_squared * rhs_values);
        output.store((one - weight) * center_values + weight * candidate);
    }
}

use gudra::gpu::Field2D;

pub fn escape(field: Field2D) {
    let _ = field.storage; // error: E0616
}

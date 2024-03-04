// use std::io::Result;
use std::path::Path;

fn main() {
    // -> Result<()> {
    prost_build::compile_protos(
        &[Path::new("..").join("proto").join("ble_gatt.proto")],
        &[Path::new("..").join("proto")],
    )
    .unwrap();
    // Ok(())
}

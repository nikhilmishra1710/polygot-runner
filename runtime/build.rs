// runtime/build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo to re-run this script if the proto file changes
    println!("cargo:rerun-if-changed=../proto/execution.proto");

    // Compile the proto file using tonic
    tonic_build::compile_protos("../proto/execution.proto")?;

    Ok(())
}

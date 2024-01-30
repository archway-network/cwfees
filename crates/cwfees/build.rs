use std::io::Result;
fn main() -> Result<()> {
    prost_build::Config::new()
        .out_dir("src/")
        .compile_protos(&["src/cwfees.proto"], &["src/"])?;
    Ok(())
}

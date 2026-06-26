use std::{env, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    println!("cargo:rerun-if-changed=proto/blog.proto");

    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("blog_descriptor.bin"))
        .compile_protos(&["proto/blog.proto"], &["proto"])?;

    Ok(())
}

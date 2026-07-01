use std::{env, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    println!("cargo:rerun-if-changed=proto/blog/v1/blog.proto");

    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("blog_descriptor.bin"))
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .field_attribute(
            ".blog.v1.Post.created_at",
            "#[serde(with = \"crate::timestamp_serde\")]",
        )
        .field_attribute(
            ".blog.v1.Post.updated_at",
            "#[serde(with = \"crate::timestamp_serde\")]",
        )
        .field_attribute(
            ".blog.v1.User.created_at",
            "#[serde(with = \"crate::timestamp_serde\")]",
        )
        .compile_protos(&["proto/blog/v1/blog.proto"], &["proto"])?;

    Ok(())
}

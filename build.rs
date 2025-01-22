use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(&out_dir.join("todo_list_descriptor.bin"))
        .compile(&["proto/todo_list.proto"], &["proto"])?;
    Ok(())
}

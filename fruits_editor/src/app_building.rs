use std::{
    error::Error,
    path::{Path, PathBuf},
};

pub fn build_app(project_path: &str) {
    let project_name = PathBuf::from(project_path);
    let project_name = project_name.file_name().unwrap().to_str().unwrap();

    try_create_launcher_project(project_path).unwrap();

    build_cargo_project([project_path, "launcher"].iter().collect::<PathBuf>()).unwrap();
    build_cargo_project([project_path, "scripts"].iter().collect::<PathBuf>()).unwrap();

    // Cargo output name is OS-specific (Linux: lib{name}.so, Windows: {name}.dll).
    // We deploy a consistent name: lib_app{DLL_SUFFIX} on all platforms.
    use std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX};
    let scripts_lib_cargo_name = format!("{DLL_PREFIX}lib_app{DLL_SUFFIX}");
    let scripts_lib_deploy_name = format!("lib_app{DLL_SUFFIX}");
    let launcher_exe_name = format!("launcher{EXE_SUFFIX}");
    let build_exe_name = format!("{project_name}{EXE_SUFFIX}");
    let scripts_lib_cargo_name = scripts_lib_cargo_name.as_str();
    let scripts_lib_deploy_name = scripts_lib_deploy_name.as_str();
    let launcher_exe_name = launcher_exe_name.as_str();
    let build_exe_name = build_exe_name.as_str();

    copy_file(
        [project_path, "scripts", "target", "release", scripts_lib_cargo_name]
            .iter()
            .collect::<PathBuf>(),
        [project_path, "builds", scripts_lib_deploy_name].iter().collect::<PathBuf>(),
    )
    .unwrap();

    copy_file(
        [project_path, "launcher", "target", "release", launcher_exe_name]
            .iter()
            .collect::<PathBuf>(),
        [project_path, "builds", build_exe_name]
            .iter()
            .collect::<PathBuf>(),
    )
    .unwrap();

    let assets_src = [project_path, "assets"].iter().collect::<PathBuf>();
    let assets_dst = [project_path, "builds", "assets"].iter().collect::<PathBuf>();

    if std::fs::exists(assets_src).unwrap_or(true) {
        copy_dir_all([project_path, "assets"].iter().collect::<PathBuf>(), assets_dst).unwrap();
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn build_cargo_project(project_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let mut path = PathBuf::from(project_path.as_ref());

    path.push("Cargo.toml");

    let exit_status = std::process::Command::new("cargo")
        .args(["build", "--release", "--manifest-path", path.to_str().unwrap()])
        .stderr(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .status()?;

    if !exit_status.success() {
        return Err("cargo build was unsuccessful".into());
    }

    Ok(())
}

fn copy_file(src_path: impl AsRef<Path>, dst_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let dst_path = dst_path.as_ref();

    if let Some(dst_parent_path) = dst_path.parent() {
        _ = std::fs::create_dir_all(dst_parent_path)
    };

    std::fs::copy(src_path, dst_path)?;

    Ok(())
}

fn try_create_launcher_project(fruits_project_path: impl AsRef<Path>) -> Result<bool, Box<dyn Error>> {
    let mut path = PathBuf::from(fruits_project_path.as_ref());
    path.push("launcher");

    if let Ok(true) = std::fs::exists(&path) {
        return Ok(false);
    }

    std::fs::create_dir_all(&path)?;

    let mut cargo_path = path.clone();
    cargo_path.push("Cargo.toml");

    std::fs::write(
        cargo_path,
        r#"
[package]
name = "launcher"
version = "0.1.0"
edition = "2024"

[dependencies]
fruits_engine = { git = "https://github.com/are-you-fruits-studio/fruits_engine.git", tag = "v0.8.0" }
            "#
        .replace("\r\n", "\n")
        .replace("\r", "\n")
        .trim(),
    )?;

    let mut src_path = path.clone();
    src_path.push("src");

    std::fs::create_dir_all(&src_path)?;

    let mut main_path = src_path.clone();
    main_path.push("main.rs");

    std::fs::write(
        main_path,
        r#"
fn main() {
    fruits_engine::launch_app_dynamically();
}
            "#
        .replace("\r\n", "\n")
        .replace("\r", "\n")
        .trim(),
    )?;

    Ok(true)
}

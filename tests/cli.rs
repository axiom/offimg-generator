use std::{
    fs,
    os::unix::fs::symlink,
    path::Path,
    process::{Command, Output},
};

use image::ImageFormat;
use offimg_generator::{HEIGHT, WIDTH, bmp};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_offimg"))
        .args(args)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .unwrap()
}

fn convert(input: &Path, output: &Path, extra: &[&str]) -> Output {
    let mut args = vec!["convert", input.to_str().unwrap(), output.to_str().unwrap()];
    args.extend(extra);
    cli(&args)
}

fn assert_failed(output: Output, message: &str) {
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(message), "{stderr}");
}

#[test]
fn help_explains_arguments_and_defaults() {
    let result = cli(&["--help"]);
    assert!(result.status.success());
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("convert")
    );
    let result = cli(&["convert", "--help"]);
    assert!(result.status.success());
    let help = String::from_utf8(result.stdout).unwrap();
    for word in [
        "INPUT",
        "OUTPUT",
        "--force",
        "--fit",
        "--dither",
        "default: fit",
        "default: threshold",
    ] {
        assert!(help.contains(word), "{help}");
    }
}

#[test]
fn converts_fixture_without_a_display_and_refuses_unauthorized_overwrite() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.bmp");
    let output = directory.path().join("output.bmp");
    fs::write(&input, include_bytes!("../numbers.bmp")).unwrap();
    let result = convert(&input, &output, &[]);
    assert!(result.status.success(), "{:?}", result);
    let bytes = fs::read(&output).unwrap();
    assert_eq!(bytes.len(), bmp::FILE_SIZE);
    assert_eq!(&bytes[28..30], &[1, 0]);
    let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::Bmp)
        .unwrap()
        .into_luma8();
    let original = image::load_from_memory(include_bytes!("../numbers.bmp"))
        .unwrap()
        .into_luma8();
    assert_eq!(decoded.dimensions(), (WIDTH, HEIGHT));
    assert_eq!(decoded, original);

    fs::write(&output, b"keep this").unwrap();
    assert_failed(convert(&input, &output, &[]), "--force");
    assert_eq!(fs::read(&output).unwrap(), b"keep this");
    assert!(convert(&input, &output, &["--force"]).status.success());
    assert_eq!(fs::read(&output).unwrap(), bytes);
    assert_eq!(fs::read(&input).unwrap(), include_bytes!("../numbers.bmp"));
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
}

#[test]
fn rejects_source_identity_symlinks_hardlinks_and_relative_aliases_even_with_force() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.bmp");
    fs::write(&input, include_bytes!("../numbers2.bmp")).unwrap();
    let sym = directory.path().join("symlink.bmp");
    let hard = directory.path().join("hardlink.bmp");
    symlink(&input, &sym).unwrap();
    fs::hard_link(&input, &hard).unwrap();
    fs::create_dir(directory.path().join("sub")).unwrap();
    for output in [
        &input,
        &sym,
        &hard,
        &directory.path().join("sub/../input.bmp"),
    ] {
        assert_failed(convert(&input, output, &["--force"]), "source image");
        assert_eq!(fs::read(&input).unwrap(), include_bytes!("../numbers2.bmp"));
    }
}

#[test]
fn errors_are_nonzero_and_preserve_files() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("broken.png");
    let output = directory.path().join("output.bmp");
    fs::write(&input, b"not an image").unwrap();
    assert_failed(convert(&input, &output, &[]), "unsupported image");
    assert!(!output.exists());
    fs::write(&output, b"keep me").unwrap();
    assert_failed(convert(&input, &output, &["--force"]), "unsupported image");
    assert_eq!(fs::read(&output).unwrap(), b"keep me");
    assert_failed(
        convert(&input, &output, &["--fit", "stretch"]),
        "framing must be",
    );
    assert_failed(
        convert(&input, &output, &["--dither", "random"]),
        "dither must be",
    );
    assert_failed(
        convert(&directory.path().join("missing"), &output, &[]),
        "cannot load",
    );

    fs::write(&input, include_bytes!("../numbers.bmp")).unwrap();
    assert_failed(
        convert(&input, &directory.path().join("absent/output.bmp"), &[]),
        "cannot export",
    );
    assert_failed(
        convert(&input, directory.path(), &["--force"]),
        "cannot export",
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
}

#[test]
fn failed_atomic_export_preserves_destination_and_cleans_up() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.bmp");
    let output = directory.path().join("output.bmp");
    fs::write(&input, include_bytes!("../numbers.bmp")).unwrap();
    fs::write(&output, b"keep me").unwrap();
    let invalid = offimg_generator::mono::MonoImage::from_fn(1, 1, |_, _| true);
    assert!(offimg_generator::export::save(&invalid, &input, &output, true).is_err());
    assert_eq!(fs::read(&output).unwrap(), b"keep me");
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
}

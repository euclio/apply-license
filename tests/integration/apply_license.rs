use std::fs;

use anyhow::Result;
use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn single_license_with_author() -> Result<()> {
    let dir = tempdir()?;
    let dir = dir.path();

    let mut cmd = Command::cargo_bin("apply-license")?;
    cmd.current_dir(&dir)
        .args(&["--author", "John Doe", "--license", "MIT"])
        .assert()
        .success();

    let license = dir.join("LICENSE");
    assert!(license.exists());
    assert!(!dir.join("LICENSE-MIT").exists());

    let license_text = fs::read_to_string(license)?;
    assert!(license_text.contains("THE SOFTWARE IS PROVIDED \"AS IS\""));
    assert!(license_text.contains("John Doe"));

    Ok(())
}

#[test]
fn multiple_license_with_author() -> Result<()> {
    let dir = tempdir()?;
    let dir = dir.path();

    let mut cmd = Command::cargo_bin("apply-license")?;
    cmd.current_dir(&dir)
        .args(&["--author", "John Doe", "--license", "MIT/Apache-2.0"])
        .assert()
        .success();

    assert!(!dir.join("LICENSE").exists());

    let mit_license = dir.join("LICENSE-MIT");
    assert!(mit_license.exists());
    assert!(fs::read_to_string(mit_license)?.contains("THE SOFTWARE IS PROVIDED \"AS IS\""));

    let apache_license = dir.join("LICENSE-APACHE");
    assert!(fs::read_to_string(apache_license)?.contains("Apache License"));

    Ok(())
}

use std::fs;

use anyhow::Result;
use assert_cmd::Command;
use tempfile::tempdir;
use toml_edit::{Document, Item, Value};

#[test]
fn cargo_project_with_author() -> Result<()> {
    let dir = tempdir()?;
    let dir = dir.path();

    Command::new("cargo")
        .current_dir(dir)
        .args(&["init", "--name", "foo"])
        .assert()
        .success();

    let cargo_toml_contents = fs::read_to_string(dir.join("Cargo.toml"))?;

    let mut document = cargo_toml_contents.parse::<Document>()?;
    document["package"]["authors"] = Item::Value(Value::from_iter(vec!["John Doe"]));

    fs::write(dir.join("Cargo.toml"), document.to_string())?;

    Command::cargo_bin("cargo-apply-license")?
        .current_dir(dir)
        .args(&["apply-license"])
        .assert()
        .success();

    Ok(())
}

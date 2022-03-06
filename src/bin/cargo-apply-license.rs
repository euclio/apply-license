use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use clap::{Args, Parser};
use toml_edit::{value, Document};

static DEFAULT_LICENSE: &str = "MIT OR Apache-2.0";

/// Apply open-source licenses to your cargo project.
///
/// Parses author and license information from your Cargo.toml.
#[derive(Debug, Parser)]
#[clap(bin_name = "cargo")]
enum Cli {
    ApplyLicense(ApplyLicenseArgs),
}

#[derive(Debug, Args)]
#[clap(override_usage = "\
    cargo apply-license")]
struct ApplyLicenseArgs {
    /// Path to Cargo.toml
    #[clap(long = "manifest-path", name = "PATH")]
    manifest_path: Option<PathBuf>,

    /// An SPDX license expression. If specified, overrides the value in Cargo.toml.
    #[clap(long = "license")]
    license: Option<String>,
}

fn main() -> Result<()> {
    let Cli::ApplyLicense(args) = Cli::parse();

    let mut metadata_cmd = MetadataCommand::new();

    if let Some(manifest_path) = &args.manifest_path {
        metadata_cmd.manifest_path(manifest_path);
    }

    let metadata = metadata_cmd
        .exec()
        .context("unable to parse cargo metadata")?;

    let authors = &metadata.packages[0].authors;
    let authors = authors
        .iter()
        .map(|author| author.as_str())
        .collect::<Vec<_>>();
    let names = apply_license::parse_author_names(&authors)?;

    let manifest_path = args
        .manifest_path
        .as_ref()
        .map(Path::new)
        .unwrap_or_else(|| Path::new("Cargo.toml"));
    let mut manifest: Document = fs::read_to_string(manifest_path)?.parse()?;

    let (original_license, licenses) = {
        let license_value = &mut manifest["package"]["license"];
        let original_license = license_value.as_str().map(ToOwned::to_owned);

        let license_expr = args
            .license
            .unwrap_or_else(|| String::from(DEFAULT_LICENSE));
        let license_value = license_value.or_insert(value(license_expr));
        let licenses = apply_license::parse_spdx(&license_value.as_str().unwrap())?;
        (original_license, licenses)
    };

    for (name, contents) in apply_license::render_license_text(&licenses, &names)? {
        fs::write(name, contents)?;
    }

    if original_license.as_ref().map(|s| &**s) != manifest["package"]["license"].as_str() {
        fs::write(manifest_path, manifest.to_string())?;
    }

    Ok(())
}

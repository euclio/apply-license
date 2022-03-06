use std::fs;

use anyhow::Result;
use clap::Parser;

/// Apply open-source licenses to your project.
#[derive(Debug, Parser)]
struct Cli {
    /// The authors of the crate. Can be specified multiple times.
    #[clap(long = "author", short = 'a', min_values = 1)]
    authors: Vec<String>,

    /// The SPDX license expression for the license or licenses to apply.
    #[clap(long = "license", short = 'l')]
    license: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let licenses = apply_license::parse_spdx(&args.license)?;
    for (name, contents) in apply_license::render_license_text(&licenses, &args.authors)? {
        fs::write(name, contents)?;
    }

    Ok(())
}

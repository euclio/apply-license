use std::fs;

use anyhow::Result;
use structopt::StructOpt;

/// Apply open-source licenses to your project.
#[derive(Debug, StructOpt)]
struct Opt {
    /// The authors of the crate. Can be specified multiple times.
    #[structopt(long = "author", short = "a", raw(min_values = "1"))]
    authors: Vec<String>,

    /// An SPDX license expression.
    #[structopt(long = "license", short = "l")]
    license: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let licenses = apply_license::parse_spdx(&opt.license)?;
    for (name, contents) in apply_license::render_license_text(&licenses, &opt.authors)? {
        fs::write(name, contents)?;
    }

    Ok(())
}

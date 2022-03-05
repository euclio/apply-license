use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use chrono::{Datelike, Local};
use handlebars::Handlebars;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    /// A list of licenses with text included in the program.
    static ref LICENSES: Vec<License> = {
        let licenses_toml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/licenses.toml"));

        let mut licenses: BTreeMap<String, Vec<License>> =
            toml_edit::easy::from_str(licenses_toml).unwrap();

        licenses.remove("license").unwrap()
    };
}

/// An open-source license.
#[derive(Debug, PartialEq, Deserialize)]
pub struct License {
    /// The identifier for the license on the command line, if multiple licenses are present.
    ///
    /// For example, `LICENSE-APACHE` vs `LICENSE-MIT`.
    pub identifier: String,

    /// The [SPDX license identifier](https://github.com/spdx/license-list-data/tree/v2.4).
    pub spdx: String,

    /// A handlebars template of the license text.
    pub text: String,
}

/// Parses author names from a list of author names, which might include git-style author names
/// such as `John Doe <jd@example.com>`.
pub fn parse_author_names<'a>(authors: &[&'a str]) -> Result<Vec<&'a str>> {
    if authors.is_empty() {
        bail!("at least one author is required");
    }

    let names = authors
        .into_iter()
        .map(|author| match parse_git_style_author(author) {
            Some(name) => name,
            None => author,
        })
        .collect();

    Ok(names)
}

/// Returns true if the given license ID is known by SPDX 2.4.
fn is_valid_spdx_id(id: &str) -> bool {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct LicenseList {
        licenses: Vec<License>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct License {
        license_id: String,
    }

    lazy_static! {
        static ref SPDX_LICENSE_LIST: LicenseList = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/spdx-licenses.json"
        )))
        .unwrap();
    }

    SPDX_LICENSE_LIST
        .licenses
        .iter()
        .any(|license| license.license_id == id)
}

/// Parse a list of license identifiers from an SPDX license expression.
///
/// The cargo manifest format allows combining license expressions with `/`, so we allow it as
/// well, though it's not valid SPDX.
pub fn parse_spdx(license_expr: &str) -> Result<Vec<&'static License>> {
    let split: Box<dyn Iterator<Item = &str>> = if license_expr.contains("/") {
        Box::new(license_expr.split("/"))
    } else {
        Box::new(license_expr.split_whitespace())
    };

    split
        .flat_map(|token| match token {
            "WITH" | "OR" | "AND" => None,
            token => Some(token),
        })
        .map(|id| {
            if is_valid_spdx_id(id) {
                LICENSES
                    .iter()
                    .find(|license| license.spdx == id)
                    .ok_or_else(|| anyhow!("SPDX ID '{}' is valid, but unsupported by this program. Please open a PR!", id))
            } else {
                Err(anyhow!("invalid SPDX license ID: {}", id))
            }
        })
        .collect()
}

/// Given a list of authors and SPDX license identifiers, returns a map from file name to contents.
///
/// If only one license file is present, writes the file name will be `LICENSE`. If two or more
/// licenses are present, then each file will be named `LICENSE-{id}` (e.g., `LICENSE-MIT`).
pub fn render_license_text<S: Borrow<str>>(
    licenses: &[&License],
    authors: &[S],
) -> Result<BTreeMap<PathBuf, String>> {
    let mut reg = Handlebars::new();

    for license in LICENSES.iter() {
        reg.register_template_string(&license.spdx, &license.text)
            .expect("syntax error in license template");
    }

    #[derive(Debug, Serialize)]
    struct TemplateData {
        year: i32,
        copyright_holders: String,
    }

    licenses
        .into_iter()
        .map(|license| {
            let name = if licenses.len() == 1 {
                String::from("LICENSE")
            } else {
                format!("LICENSE-{}", license.identifier)
            };

            let contents = reg.render(
                &license.spdx,
                &TemplateData {
                    year: Local::today().year(),
                    copyright_holders: authors.join(", "),
                },
            )?;

            Ok((PathBuf::from(name), contents))
        })
        .collect()
}

fn parse_git_style_author(name: &str) -> Option<&str> {
    lazy_static! {
        static ref GIT_NAME_RE: Regex = Regex::new(r"(?P<name>.+) <(?P<email>.+)>").unwrap();
    }

    GIT_NAME_RE
        .captures(name)
        .map(|caps| caps.name("name").unwrap().as_str())
}

#[cfg(test)]
mod tests {
    use crate::{is_valid_spdx_id, parse_spdx, License, LICENSES};

    fn get_license(id: &str) -> &'static License {
        LICENSES.iter().find(|l| l.spdx == id).unwrap()
    }

    #[test]
    fn parse_licenses() {
        assert!(LICENSES.iter().any(|l| l.spdx == "MIT"));
    }

    #[test]
    fn valid_spdx_ids() {
        assert!(is_valid_spdx_id("MIT"));
        assert!(!is_valid_spdx_id("foobar"));
    }

    #[test]
    fn simple() {
        assert_eq!(parse_spdx("GPL-3.0").unwrap(), &[get_license("GPL-3.0")]);
    }

    #[test]
    fn compound() {
        assert_eq!(
            parse_spdx("MIT OR Apache-2.0").unwrap(),
            &[get_license("MIT"), get_license("Apache-2.0")],
        );
    }

    #[test]
    fn cargo_manifest_licenses() {
        assert_eq!(
            parse_spdx("MIT/Apache-2.0").unwrap(),
            &[get_license("MIT"), get_license("Apache-2.0")]
        );
    }
}

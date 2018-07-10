# apply-license

`apply-license` is a simple command-line tool that strives to make applying
open-source licenses to your software as easy and automated as possible.

It generates the appropriate license files in your directory (i.e., `LICENSE` if
you are using only one license, and `LICENSE-<id>` for projects with more than
one license applied.) The license text will contain the appropriate authorship
and the current year.

## Installation

To install the tool, use `cargo`. You can install `cargo` with
[rustup](https://rustup.rs/).

```sh
$ cargo install apply-license
```

This will install the `apply-license` and `cargo-apply-license` binaries to your
`PATH`.

## Usage

If you're working with a cargo project, using `apply-license` couldn't be
easier. Simply execute:

```sh
$ cargo apply-license
```

This command will parse your `Cargo.toml` to determine authorship and license
information. If you haven't specified a license, it will default to "MIT OR
Apache-2.0".

This package also includes a standalone binary for non-cargo projects. It works
similarly to `cargo-apply-license`, but you'll have to specify the license
expression and authorship yourself:

```
$ apply-license -a "John Doe" -l MIT
```

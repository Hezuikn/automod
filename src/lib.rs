//! [![github]](https://github.com/dtolnay/automod)&ensp;[![crates-io]](https://crates.io/crates/automod)&ensp;[![docs-rs]](https://docs.rs/automod)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! <br>
//!
//! **Pull in every source file in a directory as a module.**
//!
//! # Syntax
//!
//! ```
//! # const IGNORE: &str = stringify! {
//! automod::dir!("path/to/directory");
//! # };
//! ```
//!
//! This macro expands to one or more `mod` items, one for each source file in
//! the specified directory.
//!
//! The path is given relative to the directory containing Cargo.toml.
//!
//! It is an error if the given directory contains no source files.
//!
//! # Example
//!
//! Suppose that we would like to keep a directory of regression tests for
//! individual numbered issues:
//!
//! - tests/
//!   - regression/
//!     - issue1.rs
//!     - issue2.rs
//!     - ...
//!     - issue128.rs
//!
//! We would like to be able to toss files in this directory and have them
//! automatically tested, without listing them in some explicit list of modules.
//! Automod solves this by adding *tests/regression.rs* containing:
//!
//! ```
//! # const IGNORE: &str = stringify! {
//! mod regression {
//!     automod::dir!("tests/regression");
//! }
//! # };
//! ```
//!
//! The macro invocation expands to:
//!
//! ```
//! # const IGNORE: &str = stringify! {
//! mod issue1;
//! mod issue2;
//! /* ... */
//! mod issue128;
//! # };
//! ```

#![allow(clippy::enum_glob_use, clippy::needless_pass_by_value)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr, Visibility};

struct Arg {
    vis: Visibility,
    path: LitStr,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Arg {
            vis: input.parse()?,
            path: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn dir(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Arg);
    let vis = &input.vis;
    let rel_path = input.path.value();

    let dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(manifest_dir) => PathBuf::from(manifest_dir).join(rel_path),
        None => PathBuf::from(rel_path),
    };

    let expanded = source_files(&dir, &dir)
        .into_iter()
        .map(|(path, name)| {
            let ident = Ident::new(&name.replace('-', "_"), Span::call_site());
            quote! {
                #[path = #path]
                #vis mod #ident;
            }
        })
        .collect::<TokenStream2>();

    //println!("{expanded}");

    TokenStream::from(expanded)
}

fn source_files(top_dir: &Path, current_dir: &Path) -> Vec<(String, String)> {
    let mut paths = Vec::new();

    for entry in fs::read_dir(current_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name_path = path
            .canonicalize()
            .unwrap()
            .strip_prefix(Path::new(top_dir).canonicalize().unwrap())
            .unwrap()
            .with_extension("");
        let name = name_path
            .components()
            .map(|x| match x {
                std::path::Component::Normal(x) => x.to_str().unwrap(),
                _ => panic!(),
            })
            .collect::<Vec<_>>()
            .join("_");

        if entry.file_type().unwrap().is_dir() {
            let mod_file = path.join("mod.rs");
            if mod_file.exists() && mod_file.is_file() {
                paths.push((mod_file.into_os_string().into_string().unwrap(), name));
            } else {
                paths.append(&mut source_files(top_dir, &path));
            }
        } else if entry.file_type().unwrap().is_file() {
            let file_name = path.file_name().unwrap();
            if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "main.rs" {
                continue;
            }

            if path.extension() == Some(OsStr::new("rs")) {
                paths.push((path.into_os_string().into_string().unwrap(), name));
            }
        }
    }

    paths.sort();
    return paths;
}

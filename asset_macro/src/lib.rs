extern crate proc_macro;

use proc_macro::TokenStream;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_str, LitStr, Token};

fn sanitize_name(raw: &str) -> String {
    raw.replace('-', "_")
}

fn get_directory_tokens(
    path: PathBuf,
    dir: &str,
    is_pub: bool,
    name_override: Option<String>,
) -> TokenStream2 {
    let mut items = Vec::new();
    let dir_path = path.to_str().expect("Failed to convert OS path");
    let dir_name = path
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or_else(|| panic!("Failed to obtain name of directory at {}", dir_path));

    for entry in path
        .read_dir()
        .unwrap_or_else(|err| panic!("Failed to read directory {}: `{}`", dir_path, err))
    {
        let entry = entry.unwrap_or_else(|err| {
            panic!(
                "Failed to fetch info of file in directory `{}`: {}",
                dir_path, err
            )
        });
        items.push(get_entry_tokens(entry, dir))
    }

    let actual_name = match name_override {
        None => sanitize_name(dir_name),
        Some(name) => name,
    };
    let dir_name = parse_str::<Ident>(actual_name.as_str()).unwrap_or_else(|err| {
        panic!(
            "Failed to generate identifier for a directory `{}` ({}): {}",
            dir_path, dir_name, err
        )
    });

    if is_pub {
        quote! {
            pub mod #dir_name {
                #(#items)*
            }
        }
    } else {
        quote! {
            mod #dir_name {
                #(#items)*
            }
        }
    }
}

fn get_entry_tokens(file: DirEntry, dir: &str) -> TokenStream2 {
    let path = file.path();
    let full_path = path.to_str().expect("Failed to convert OS string");

    let file_type = file
        .file_type()
        .unwrap_or_else(|err| panic!("Failed to get type of file `{}`: {}", full_path, err));
    if file_type.is_dir() {
        return get_directory_tokens(path, dir, true, None);
    }

    let stripped_full_path = file
        .path()
        .strip_prefix(dir)
        .unwrap_or_else(|_| panic!("File `{}` is outside of assets folder", full_path))
        .to_str()
        .unwrap_or_else(|| panic!("Failed to convert OS string, at file `{}`", full_path))
        .replace('\\', "/");

    let full_path_no_ext = path.with_extension("");
    let file_name = full_path_no_ext
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or_else(|| panic!("Failed to obtain name of file at {}", full_path))
        .to_uppercase();

    let file_name = syn::parse_str::<syn::Ident>(sanitize_name(file_name.as_str()).as_str())
        .unwrap_or_else(|err| {
            panic!(
                "Failed to generate identifier for a file `{}` ({}): {}",
                full_path, file_name, err
            )
        });

    let asset_literal = LitStr::new(stripped_full_path.as_str(), proc_macro2::Span::call_site());

    quote! {
        pub const #file_name: &'static str = #asset_literal;
    }
}

mod kv {
    syn::custom_keyword!(from);
}

// #[derive(Parse)]
struct DeclarationInput {
    is_pub: bool,
    root_module_name: String,
    path: String,
}

impl Parse for DeclarationInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let is_pub = input.parse::<Token![pub]>().is_ok();
        input.parse::<Token![mod]>()?;
        let root_module_name = input.parse::<Ident>()?.to_string();
        let path = if input.parse::<Token![=]>().is_ok() {
            input.parse::<LitStr>()?.value()
        } else {
            root_module_name.clone()
        };
        Ok(DeclarationInput {
            is_pub,
            root_module_name,
            path,
        })
    }
}

struct MacroInput {
    root_path: String,
    declarations: Punctuated<DeclarationInput, Token![;]>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![in]>()?;
        let root_path = input.parse::<LitStr>()?.value();
        input.parse::<Token![:]>()?;
        let declarations = input.parse_terminated(DeclarationInput::parse, Token![;])?;
        Ok(MacroInput {
            root_path,
            declarations,
        })
    }
}

#[proc_macro]
pub fn generate_assets(args: TokenStream) -> TokenStream {
    let arg = parse_macro_input!(args as MacroInput);
    // Get the directory of the current crate
    let dir = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to fetch manifest dir");

    let asset_dir = Path::new(&dir).join(arg.root_path);
    let base_dir = asset_dir.to_str().expect("Failed to obtain root path");

    let mut items = Vec::new();
    for declaration in arg.declarations {
        items.push(get_directory_tokens(
            asset_dir.join(PathBuf::from(declaration.path)),
            base_dir,
            declaration.is_pub,
            Some(declaration.root_module_name),
        ))
    }

    TokenStream::from(quote! {
        // pub mod assets {
            #(#items)*
        // }
    })
}

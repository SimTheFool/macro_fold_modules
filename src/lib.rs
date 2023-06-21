extern crate proc_macro;
extern crate quote;
extern crate syn;

mod helpers;

use crate::helpers::{layout::get_directory_layout, parse::get_implem};
use helpers::errors::MacroError;
use proc_macro::{TokenStream, TokenTree};
use quote::quote;

#[proc_macro]
pub fn macro_fold_modules(input: TokenStream) -> TokenStream {
    let inputs = parse_input(input);

    let code = inputs.and_then(|s| main(s));

    match code {
        Ok(code) => quote! {
            #code
        }
        .into(),
        Err(e) => {
            let error = e.to_string();
            quote! {
                compile_error!(#error);
            }
            .into()
        }
    }
}

fn main(inputs: MacroInput) -> Result<syn::File, MacroError> {
    let (derive_idents, path) = inputs;

    let (directory_name, files_infos) =
        get_directory_layout(&path.value()).map_err(|e| MacroError::LayoutError(e))?;

    let new_implem = get_implem(files_infos, directory_name, &derive_idents)
        .map_err(|e| MacroError::FileParseError(e))?;

    Ok(new_implem)
}

type MacroInput = (Vec<syn::Ident>, syn::LitStr);

fn parse_input(input: TokenStream) -> Result<MacroInput, MacroError> {
    let derive_idents = input
        .clone()
        .into_iter()
        .filter_map(|t| match t {
            TokenTree::Ident(_) => {
                let ident = syn::parse::<syn::Ident>(TokenStream::from(t))
                    .map_err(|e| MacroError::InputError(format!("Cannot parse ident: {}", e)));

                Some(ident)
            }
            _ => None,
        })
        .collect::<Result<Vec<syn::Ident>, MacroError>>()?;

    let path = input
        .clone()
        .into_iter()
        .filter_map(|t| match t {
            TokenTree::Literal(_) => {
                let literal = syn::parse::<syn::LitStr>(TokenStream::from(t))
                    .map_err(|e| MacroError::InputError(format!("Cannot parse path: {}", e)));

                Some(literal)
            }
            _ => None,
        })
        .collect::<Result<Vec<syn::LitStr>, MacroError>>()?;

    let path = path
        .get(0)
        .ok_or_else(|| MacroError::InputError("No path were specified".to_string()))?;

    let path = path.to_owned();

    Ok((derive_idents, path))
}

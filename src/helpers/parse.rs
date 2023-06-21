use convert_case::{Case, Casing};

use super::{errors::FileParseError, layout::FileInfos, visit::visit_file};
use crate::syn::parse::Parser;
use quote::quote;

type FileVisiteResult<'a> = (&'a FileInfos, Option<syn::ImplItemFn>);

pub fn get_implem(
    infos: Vec<FileInfos>,
    directory_name: String,
    derive_idents: &Vec<syn::Ident>,
) -> Result<syn::File, FileParseError> {
    let init_method_name = "new";

    let new_implems = infos
        .iter()
        .map(|infos| match visit_file(infos, init_method_name) {
            Ok(Some(x)) => Ok((infos, Some(x))),
            Ok(None) => Ok((infos, None)),
            Err(e) => Err(e),
        })
        .collect::<Result<Vec<FileVisiteResult>, FileParseError>>()?;

    let first_init_implem = &new_implems.get(0).unwrap().1;

    let are_all_identical_implems = new_implems.iter().all(|(_, x)| x == first_init_implem);
    if !are_all_identical_implems {
        return Err(FileParseError::NotAllNewMethodsAreIdentical);
    }

    let struct_ident = syn::parse_str::<syn::Ident>(&directory_name.to_case(Case::Pascal)).unwrap();
    let struct_fields = infos
        .iter()
        .map(|(file_name, _)| {
            let field_type = syn::parse_str::<syn::Type>(&file_name.to_case(Case::Pascal))
                .map_err(|e| FileParseError::Other(e.to_string()));

            let field_name = syn::parse_str::<syn::Ident>(&file_name.to_case(Case::Snake))
                .map_err(|e| FileParseError::Other(e.to_string()));

            match (field_name, field_type) {
                (Ok(field_name), Ok(field_type)) => Ok((field_name, field_type)),
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            }
        })
        .collect::<Result<Vec<(syn::Ident, syn::Type)>, FileParseError>>()?;

    let mods_idents = struct_fields
        .iter()
        .map(|(file_ident, _)| file_ident)
        .collect::<Vec<&syn::Ident>>();

    let init_fields = new_implems
        .iter()
        .filter_map(|x| get_init_statements(x, init_method_name))
        .collect::<Vec<syn::FieldValue>>();

    let struct_implem = implem_struct_template(&struct_ident, first_init_implem, &init_fields);
    let struct_item = struct_template(&struct_ident, &struct_fields, derive_idents)?;
    let mods_item = mods_import_template(&mods_idents)?;
    let mods_uses = mods_uses_template(&mods_idents)?;

    let result = syn::parse2::<syn::File>(quote! (
        #(#mods_item)*
        #(#mods_uses)*
        #struct_item
        #struct_implem
    ))
    .map_err(|e| FileParseError::Other(e.to_string()))?;

    Ok(result)
}

fn mods_uses_template(mods_idents: &Vec<&syn::Ident>) -> Result<Vec<syn::ItemUse>, FileParseError> {
    let mods_uses = mods_idents
        .iter()
        .map(|ident| {
            let struct_ident =
                syn::parse_str::<syn::Ident>(&ident.clone().to_string().to_case(Case::Pascal))
                    .map_err(|e| FileParseError::Other(e.to_string()));

            match struct_ident {
                Ok(struct_ident) => syn::parse2::<syn::ItemUse>(quote! (
                    use #ident::#struct_ident;
                ))
                .map_err(|e| FileParseError::Other(e.to_string())),
                Err(e) => Err(e),
            }
        })
        .collect::<Result<Vec<syn::ItemUse>, FileParseError>>()?;

    Ok(mods_uses)
}

fn mods_import_template(
    mods_idents: &Vec<&syn::Ident>,
) -> Result<Vec<syn::ItemMod>, FileParseError> {
    let mods_imports = mods_idents
        .iter()
        .map(|ident| {
            syn::parse2::<syn::ItemMod>(quote! (
                pub mod #ident;
            ))
            .map_err(|e| FileParseError::Other(e.to_string()))
        })
        .collect::<Result<Vec<syn::ItemMod>, FileParseError>>()?;

    Ok(mods_imports)
}

fn struct_template(
    struct_ident: &syn::Ident,
    fields: &Vec<(syn::Ident, syn::Type)>,
    derive_idents: &Vec<syn::Ident>,
) -> Result<syn::ItemStruct, FileParseError> {
    let declaration = fields
        .iter()
        .map(|(field_name, field_type)| {
            syn::Field::parse_named
                .parse2(quote! {
                    pub #field_name: #field_type
                })
                .map_err(|e| FileParseError::Other(e.to_string()))
        })
        .collect::<Result<Vec<syn::Field>, FileParseError>>()?;

    let derive_list = syn::parse2::<syn::MetaList>(quote! {
        derive(#(#derive_idents),*)
    })
    .map_err(|e| FileParseError::Other(e.to_string()))?;

    let derive = syn::Attribute {
        pound_token: syn::token::Pound::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: syn::token::Bracket::default(),
        meta: syn::Meta::List(derive_list),
    };

    //println!("!!!!!! {:#?}", yyy);

    let struct_template: syn::ItemStruct = syn::parse_quote! {
        #derive
        pub struct #struct_ident {
            #(#declaration),*
        }
    };

    Ok(struct_template)
}

fn implem_struct_template(
    struct_ident: &syn::Ident,
    init_method: &Option<syn::ImplItemFn>,
    inits_fields: &Vec<syn::FieldValue>,
) -> Option<syn::ItemImpl> {
    let init_method = match init_method {
        Some(x) => Some(syn::ImplItemFn {
            block: syn::parse_quote! {
                {
                    #struct_ident {
                        #(#inits_fields),*
                    }
                }
            },
            ..x.clone()
        }),
        None => None,
    };

    match init_method {
        Some(x) => {
            let implem: syn::ItemImpl = syn::parse_quote! {
                impl #struct_ident {
                    #x
                }
            };

            Some(implem)
        }
        None => None,
    }
}

fn get_init_statements(
    visit_result: &FileVisiteResult,
    init_method_name: &str,
) -> Option<syn::FieldValue> {
    let ((file_name, _), init_method_empty_implem) = visit_result;

    let method_name = syn::parse_str::<syn::Ident>(&init_method_name.to_case(Case::Snake)).unwrap();
    let struct_name = syn::parse_str::<syn::Ident>(&file_name.to_case(Case::Pascal)).unwrap();
    let struct_variable_name =
        syn::parse_str::<syn::Ident>(&file_name.to_case(Case::Snake)).unwrap();

    match init_method_empty_implem {
        Some(x) => {
            let inputs: Vec<&syn::Ident> = x
                .sig
                .inputs
                .iter()
                .filter_map(|x| match x {
                    syn::FnArg::Typed(syn::PatType { pat, .. }) => match pat.as_ref() {
                        syn::Pat::Ident(syn::PatIdent { ident, .. }) => Some(ident),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();

            let init_invokation: syn::FieldValue = syn::parse_quote! {
                #struct_variable_name: #struct_name::#method_name(#(#inputs),*)
            };

            return Some(init_invokation);
        }
        None => None,
    }
}

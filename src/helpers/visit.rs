use convert_case::{Case, Casing};
use syn::{visit::Visit, Visibility};

use super::{errors::FileParseError, layout::FileInfos};

struct PathVisitor {
    pub path_ident: Option<String>,
    pub matcher: String,
}

impl syn::visit::Visit<'_> for PathVisitor {
    fn visit_path_segment(&mut self, path_segment: &syn::PathSegment) {
        if path_segment.ident.to_string() != self.matcher {
            return;
        }
        self.path_ident = Some(path_segment.ident.to_string());
    }
}

struct MethodImplVisitor {
    pub init_method_empty_implem: Option<syn::ImplItemFn>,
    pub method_name: String,
}

impl syn::visit::Visit<'_> for MethodImplVisitor {
    fn visit_impl_item_fn(&mut self, method: &'_ syn::ImplItemFn) {
        if method.sig.ident.to_string() != self.method_name {
            return;
        }

        if !matches!(method.vis, Visibility::Public(_)) {
            return;
        }

        let self_output_path_visitor = &mut PathVisitor {
            path_ident: None,
            matcher: "Self".to_string(),
        };
        self_output_path_visitor.visit_return_type(&method.sig.output);
        if self_output_path_visitor.path_ident.is_none() {
            return;
        }

        let init_method_empty_implem = syn::ImplItemFn {
            block: syn::Block {
                brace_token: syn::token::Brace::default(),
                stmts: Vec::new(),
            },
            ..method.clone()
        };

        self.init_method_empty_implem = Some(init_method_empty_implem);
    }
}

pub fn visit_file(
    infos: &FileInfos,
    init_method: &str,
) -> Result<Option<syn::ImplItemFn>, FileParseError> {
    let (file_name, file_path) = infos;
    let targeted_struct_name = file_name.to_case(Case::Pascal);

    let file_content = std::fs::read_to_string(file_path)
        .map_err(|e| FileParseError::CannotReadFile(e.to_string()))?;

    let source_code =
        syn::parse_file(&file_content).map_err(|e| FileParseError::CannotParseFile {
            path: file_path.clone(),
            error: e.to_string(),
        })?;

    let valid_structures_nb = source_code
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Struct(structure) => match structure {
                syn::ItemStruct { ident, .. } if ident.to_string() == targeted_struct_name => {
                    Some(structure)
                }
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<&syn::ItemStruct>>()
        .len();

    if valid_structures_nb != 1 {
        return Err(FileParseError::NoOrTooManyStruct(
            targeted_struct_name.clone(),
        ));
    }

    let init_method_empty_implem: Option<syn::ImplItemFn> = None;
    let init_method_empty_implem =
        source_code
            .items
            .iter()
            .fold(init_method_empty_implem, |acc, item| match item {
                syn::Item::Impl(impl_block) => {
                    let mut impl_path_visitor = PathVisitor {
                        path_ident: None,
                        matcher: targeted_struct_name.clone(),
                    };
                    impl_path_visitor.visit_item_impl(impl_block);
                    if None == impl_path_visitor.path_ident {
                        return acc;
                    }

                    let mut new_method_visitor = MethodImplVisitor {
                        init_method_empty_implem: None,
                        method_name: init_method.to_string(),
                    };
                    new_method_visitor.visit_item_impl(impl_block);
                    return match new_method_visitor.init_method_empty_implem {
                        Some(visit_result) => Some(visit_result),
                        None => acc,
                    };
                }
                _ => acc,
            });

    Ok(init_method_empty_implem)
}

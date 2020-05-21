// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::service::MacroArgs;
use proc_macro2::{Span, TokenStream as TokenStream2};

// TODO: If # of methods is larger than certain limit,
// then introduce closure list for method dispatch.
pub fn generate_dispatch(
    MacroArgs {
        env_path,
    }: &MacroArgs,
    the_trait: &syn::ItemTrait,
) -> Result<TokenStream2, TokenStream2> {
    let trait_ident = the_trait.ident.clone();
    let mut if_else_clauses = TokenStream2::new();

    // Make an if statement for service's each method
    for item in the_trait.items.iter() {
        let method = match item {
            syn::TraitItem::Method(x) => x,
            non_method => {
                return Err(
                    syn::Error::new_spanned(non_method, "Service trait must have only methods").to_compile_error()
                )
            }
        };
        let id_ident = super::id::id_method_ident(the_trait, method);

        // Argument will be represented as a tuple. We deserialize the data as a tuple here
        let mut the_let_pattern = syn::PatTuple {
            attrs: Vec::new(),
            paren_token: syn::token::Paren(Span::call_site()),
            elems: syn::punctuated::Punctuated::new(),
        };
        // They need annotation
        let mut type_annotation = syn::TypeTuple {
            paren_token: syn::token::Paren(Span::call_site()),
            elems: syn::punctuated::Punctuated::new(),
        };
        // We apply the arguments on the designated method, performing an actuall call.
        let mut the_args: syn::punctuated::Punctuated<syn::Expr, syn::token::Comma> =
            syn::punctuated::Punctuated::new();

        let no_self = "All your method must take &self";
        if let syn::FnArg::Typed(_) =
            method.sig.inputs.first().ok_or_else(|| syn::Error::new_spanned(method, no_self).to_compile_error())?
        {
            return Err(syn::Error::new_spanned(method, no_self).to_compile_error())
        }

        for (j, arg_source) in method.sig.inputs.iter().skip(1).enumerate() {
            let the_iden = quote::format_ident!("a{}", j + 1);
            the_let_pattern.elems.push(syn::Pat::Ident(syn::PatIdent {
                attrs: Vec::new(),
                by_ref: None,
                mutability: None,
                ident: the_iden,
                subpat: None,
            }));
            the_let_pattern.elems.push_punct(syn::token::Comma(Span::call_site()));

            let arg_type = match arg_source {
                syn::FnArg::Typed(syn::PatType {
                    attrs: _,
                    pat: _,
                    colon_token: _,
                    ty: t,
                }) => &**t,
                _ => panic!(),
            };

            if let Some(unrefed_type) = super::types::is_ref(arg_type)
                .map_err(|e| syn::Error::new_spanned(arg_source, &e).to_compile_error())?
            {
                type_annotation.elems.push(unrefed_type);
            } else {
                type_annotation.elems.push(arg_type.clone());
            }

            type_annotation.elems.push_punct(syn::token::Comma(Span::call_site()));

            let arg_ident = quote::format_ident!("a{}", j + 1);
            let the_arg = if super::types::is_ref(arg_type)
                .map_err(|e| syn::Error::new_spanned(arg_source, &e).to_compile_error())?
                .is_some()
            {
                quote! {
                    &#arg_ident
                }
            } else {
                quote! {
                    #arg_ident
                }
            };
            the_args.push(syn::parse2(the_arg).unwrap());
        }

        let stmt_deserialize = quote! {
            let #the_let_pattern: #type_annotation = serde_cbor::from_reader(&arguments[std::mem::size_of::<#env_path::PacketHeader>()..]).unwrap();
        };

        let method_name = method.sig.ident.clone();
        let stmt_call = quote! {
            let result = object.#method_name(#the_args);
        };

        let the_return = quote! {
            serde_cbor::to_writer(return_buffer, &result).unwrap();
        };

        if_else_clauses.extend(quote! {
            if method == #id_ident.load(#env_path::ID_ORDERING) {
                #stmt_deserialize
                #stmt_call
                #the_return
                return;
            }
        });
    }
    if_else_clauses.extend(quote! {
        panic!("Invalid handle call. Fatal Error.")
    });

    let trait_id_ident = super::id::id_trait_ident(&the_trait);
    let result = quote! {
        impl #env_path::ExportService<dyn #trait_ident> for dyn #trait_ident {
            fn export(port_id: #env_path::PortId, handle: std::sync::Arc<dyn #trait_ident>) -> #env_path::HandleInstance {
                #env_path::service_context::register(port_id, intertrait::cast::CastArc::cast::<dyn #env_path::Service>(handle).expect("Trait casting failed"))
            }
        }
        impl #env_path::DispatchService<dyn #trait_ident> for dyn #trait_ident {
            fn dispatch(object: &dyn #trait_ident, method: #env_path::MethodId, arguments: &[u8],
            return_buffer: std::io::Cursor<&mut Vec<u8>>) {
                #if_else_clauses
            }
        }
        impl #env_path::IdOfService<dyn #trait_ident> for dyn #trait_ident {
            fn id() -> #env_path::TraitId{
                #trait_id_ident.load(#env_path::ID_ORDERING)
            }
        }
    };
    Ok(result)
}

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

use crate::create_env_path;
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Token;

fn helper(service_trait: &syn::Path, source_struct: &syn::ItemStruct) -> Result<TokenStream2, TokenStream2> {
    let env_path = create_env_path();

    let got_no_handle = (|| -> Result<bool, TokenStream2> {
        for field in &source_struct.fields {
            let ident = field.ident.as_ref().ok_or_else(|| {
                syn::Error::new_spanned(field, "Your struct can't be a tuple struct").to_compile_error()
            })?;
            if *ident == quote::format_ident!("handle") {
                return Ok(false)
            }
        }
        Ok(true)
    })()?;
    if got_no_handle {
        return Err(syn::Error::new_spanned(source_struct, "Your struct must have field `handle: HandleInstance`")
            .to_compile_error())
    }

    let struct_name = source_struct.ident.clone();

    Ok(quote! {
        #[cast_to([sync] #env_path::Service)]
        #[derive(Debug)]
        #source_struct
        impl #env_path::ServiceDispatcher for #struct_name {
            fn dispatch(&self, method: #env_path::MethodId, arguments: &[u8], return_buffer: std::io::Cursor<&mut Vec<u8>>) {
                <dyn #service_trait as #env_path::DispatchService<dyn #service_trait>>::dispatch(self, method, arguments, return_buffer);
            }
        }
        impl #env_path::Service for #struct_name {
            fn get_handle(&self) -> &#env_path::HandleInstance {
                &self.handle
            }
            fn get_handle_mut(&mut self) -> &mut #env_path::HandleInstance {
                &mut self.handle
            }
            fn get_trait_id(&self) -> #env_path::TraitId {
                <dyn #service_trait as #env_path::IdOfService<dyn #service_trait>>::id()
            }
        }
    })
}

struct MacroArgs {
    service_trait: syn::Path,
}
impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        if input.is_empty() {
            return Err(input.error("You must supply one argument (Service trait)"))
        }
        let mut args = Punctuated::<syn::Path, Token![,]>::parse_terminated(input)?;
        if args.len() != 1 {
            return Err(input.error("You must supply one argument (Service trait)"))
        }
        let service_trait = args.pop().unwrap().into_value();
        Ok(MacroArgs {
            service_trait,
        })
    }
}

pub fn service_impl_adv(args: TokenStream2, input: TokenStream2) -> TokenStream2 {
    let args: MacroArgs = syn::parse2(args).unwrap();

    let source_struct = match syn::parse2::<syn::ItemStruct>(input.clone()) {
        Ok(x) => x,
        Err(_) => {
            return syn::Error::new_spanned(input, "You can use #[service_impl] only on a struct").to_compile_error()
        }
    };

    match helper(&args.service_trait, &source_struct) {
        Ok(x) => x,
        Err(x) => x,
    }
}

pub fn service_impl(args: TokenStream2, input: TokenStream2) -> TokenStream2 {
    service_impl_adv(quote! {#args}, input)
}

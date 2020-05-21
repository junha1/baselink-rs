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

use crate::helper::*;
use proc_macro2::TokenStream as TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Token;

pub struct MacroArgs {
    pub env_path: syn::Path,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        if input.is_empty() {
            return Err(input.error("You must supply one argument (env Path)"))
        }
        let mut args = Punctuated::<syn::Path, Token![,]>::parse_terminated(input)?;
        if args.len() != 1 {
            return Err(input.error("You must supply one argument (env Path)"))
        }
        let env_path = args.pop().unwrap().into_value();
        Ok(MacroArgs {
            env_path,
        })
    }
}

// TODO: Take an optional additional identifier to generate unique key for id registeration.
// This will allow user to have different service traits with the same name.
// (But of course in different name spaces)
pub fn service_adv(args: TokenStream2, input: TokenStream2) -> TokenStream2 {
    let source_trait = match syn::parse2::<syn::ItemTrait>(input.clone()) {
        Ok(x) => x,
        Err(_) => return syn::Error::new_spanned(input, "You can use #[service] only on a trait").to_compile_error(),
    };
    let args: MacroArgs = syn::parse2(args).unwrap();

    let id = {
        let result = id::generate_id_registeration(&args, &source_trait);
        match result {
            Ok(x) => x,
            Err(x) => return x,
        }
    };
    let dispatch = {
        let result = dispatch::generate_dispatch(&args, &source_trait);
        match result {
            Ok(x) => x,
            Err(x) => return x,
        }
    };
    let import = {
        let result = call::generate_imported_struct(&args, &source_trait);
        match result {
            Ok(x) => x,
            Err(x) => return x,
        }
    };

    quote! {
        #source_trait
        #id
        #dispatch
        #import
    }
}

pub fn service(args: TokenStream2, input: TokenStream2) -> TokenStream2 {
    if !args.is_empty() {
        return syn::Error::new_spanned(input, "#[service] does not take any argument").to_compile_error()
    }
    service_adv(quote! {remote_trait::env}, input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream as TokenStream2;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;
    use std::str::FromStr;

    pub fn service_string(source: &str) -> TokenStream2 {
        service(TokenStream2::new(), TokenStream2::from_str(source).unwrap())
    }

    #[test]
    fn example1() {
        let source = {
            let mut f = File::open(&Path::new("./src/example/ex1.rs")).unwrap();
            let mut buffer = String::new();
            f.read_to_string(&mut buffer).unwrap();
            service_string(&buffer)
        };
        let expected = {
            let mut f = File::open(&Path::new("./src/example/ex1_ex.rs")).unwrap();
            let mut buffer = String::new();
            f.read_to_string(&mut buffer).unwrap();
            TokenStream2::from_str(&buffer).unwrap()
        };
        assert_eq!(format!("{}", source), format!("{}", expected))
    }
}

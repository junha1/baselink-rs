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

#[macro_use]
extern crate quote;

fn create_env_path() -> syn::Path {
    syn::parse2(quote! {remote_trait_object::macro_env}).unwrap()
}

mod helper;
mod service;
mod service_impl;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from(service::service(TokenStream2::from(args), TokenStream2::from(input)))
}

#[proc_macro_attribute]
pub fn service_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from(service_impl::service_impl(TokenStream2::from(args), TokenStream2::from(input)))
}

#[proc_macro_attribute]
pub fn service_adv(args: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from(service::service_adv(TokenStream2::from(args), TokenStream2::from(input)))
}

#[proc_macro_attribute]
pub fn service_impl_adv(args: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from(service_impl::service_impl_adv(TokenStream2::from(args), TokenStream2::from(input)))
}

#[proc_macro_attribute]
pub fn service_debug(args: TokenStream, input: TokenStream) -> TokenStream {
    println!("{}", service::service_adv(TokenStream2::from(args), TokenStream2::from(input)));
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn service_impl_debug(args: TokenStream, input: TokenStream) -> TokenStream {
    println!("{}", service_impl::service_impl_adv(TokenStream2::from(args), TokenStream2::from(input)));
    TokenStream::new()
}

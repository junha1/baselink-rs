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

use super::Dispatch;
use super::*;
use std::sync::Arc;

/// An opaque service to register on the context.
///
/// It is constructed with a service object, wrapped in whichever smart pointer you want.
/// Depending on use of `&mut self` in the methods in your service trait, some or all `Box<>`, `Arc<>`, `Arc<RwLock<>>` will implement
/// [`IntoSkeleton`] automatically by the proc macro. Please see [this section](https://naver.com) for more detail about smart pointers.
///
/// `Skeleton` is useful when you want to erase the trait, and hold it as an opaque service that will be registered later.
/// Note that you will never need this if you do only plain export / import using [`ServiceRef`], [`ServiceToExport`], or [`ServiceToImport`].
///
/// [`IntoSkeleton`]: trait.IntoSkeleton.html
/// [`ServiceRef`]: https://naver.com
/// [`ServiceToExport`]: https://naver.com
/// [`ServiceToImport`]: https://naver.com
pub struct Skeleton {
    pub(crate) raw: Arc<dyn Dispatch>,
}

impl Skeleton {
    pub fn new<T: ?Sized + Service>(service: impl IntoSkeleton<T>) -> Self {
        service.into_skeleton()
    }
}

// This belongs to macro_env
pub fn create_skeleton(raw: Arc<dyn Dispatch>) -> Skeleton {
    Skeleton {
        raw,
    }
}

// These traits are associated with some specific service trait.
// These tratis will be implement by `dyn ServiceTrait` where `T = dyn ServiceTrait` as well.
// Macro will implement this trait with the target(expanding) service trait.

/// Conversion into a `Skeleton`, from a smart pointer of a service object.
///
/// By attaching `[remote_trait_object::service]` on a trait, smart pointers of the trait will automatically implement this
/// This is required if you want to create a [`Skeleton`] or [`ServiceToExport`].
///
/// [`ServiceToExport`]: https://naver.com
// Unused T is for avoiding violation of the orphan rule
// T will be local type for the crate, and that makes it possible to
// impl IntoSkeleton<dyn MyService> for Arc<dyn MyService>
pub trait IntoSkeleton<T: ?Sized + Service> {
    fn into_skeleton(self) -> Skeleton;
}

/// Unused T is for avoiding violation of the orphan rule, like `IntoSkeleton`
pub trait ImportRemote<T: ?Sized + Service>: Sized {
    fn import_remote(port: Weak<dyn Port>, handle: HandleToExchange) -> Self;
}

/// Exports a skeleton and returns a handle to it.
///
/// Once you create an instance of skeleton, you will eventulally export it calling this.
/// Take the handle to the other side's context and call [`import_service_from_handle`] to import into a remote object.
pub fn export_service_into_handle(context: &crate::context::Context, service: Skeleton) -> HandleToExchange {
    context.get_port().upgrade().unwrap().register_service(service.raw)
}

/// Imports a handle into a remote object.
///
/// Once you receive an instance of [`HandleToExchange`], you will eventually import in calling this.
/// Such handle must be from the other side's context.
pub fn import_service_from_handle<T: ?Sized + Service, P: ImportRemote<T>>(
    context: &crate::context::Context,
    handle: HandleToExchange,
) -> P {
    P::import_remote(context.get_port(), handle)
}

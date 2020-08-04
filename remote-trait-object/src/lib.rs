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

/*!
`remote-trait-object` is a general, powerful, and simple [remote method invocation](https://en.wikipedia.org/wiki/Distributed_object_communication) library
based on trait objects.

Note that it is commonly abbreviated as **rto**.

# Introduction

In short, this crate provides following functionalities:

## Call flow
![diagram](https://github.com/CodeChain-io/remote-trait-object/raw/master/remote-trait-object/flow.png) 

1. User calls a method in a remote object which is a trait object wrapped in a smart pointer.
2. It will invoke _call stub_ that the remote object owns.
3. The call will be delivered to the context from which the remote object is imported, after serialized into a byte packet:
the actual tranportation of data happens only at the context, which functions as a connection end.
4. The packet will be sent to the other end, (or context) by the _transport_.
5. After the other side's context receives the packet, it forwards the packet to the target servie object wrapped in a _skeleton_ in its registry.
6. The skeleton will dispatch the packet into an actual method call to the service object, which is a trait object wrapped in a smart pointer.

The result will go back to the user, again through the contexts and transport.

In this process, the key idea of this crate is that both _remote object_ and _service object_ are trait objects.
In other words, you can _export_ any trait object and _import_ it trait object as well.



# Connection

# Service

A service means an entity that can handle method calls from the client.

The reason this library is not about **RPC** (remote **procedure** call), but **RMI** (remote **method** invocation), is that
each call is involved with some particular _object_ that would have its own state.

Suppose there is some program and it is ready to handle an incoming call to some service object which the program has prepared.
Here such program is considered to be a _server_, at least during such particular action of call.
On the other side(client program), if one wants to make a call to such service object, such call must contain the information of 'to which object in the server?'.

Instead of letting the user find out which object to call and how to express the 'which object' part in a call packet, we introduce a 'remote object'.




Thus, a call must be both from the corresponding

As you can imagine, here the `_object_` is _trait object_.


In `remote-trait-object`, a _Service_ is actually a trait object.

# Smart pointers
We're talking about the trait objects like `dyn MyService`, which must be holded by some smart pointer.

Currently `remote-trait-object` supports three types of smart pointers for service objects and remote objects.

1. `Box<dyn MyService>`
2. `std::sync::Arc<dyn MyService>`
3. `std::sync::Arc<parking_lot::RwLock<dyn MyService>>`

When you export a service object, you can **export from whichever type** among them.

On the other hand when you import a remote object, you can **import into whichever type** among them.
Choosing smart pointer types is completely independent for exporting & importing sides.
Both can decided depending on their own requirements.

**Exporter (server)**
- Use `Box<>` when you have nothing to do with the object after you export it.
It will be registered in the [`Context`], and will be alive until the corresponding remote object is dropped.
You can never access to the object directly, since it will be _moved_ to the registry.

- Use `Arc<>` when you have something to do with the object after you export it, by doing `Arc::clone()` and holding somewhere.
In this case, both the corresponding remote object and some `Arc` copy in exporter side can access to the object,
though the latter can only access it immuatably.

- Use `Arc<RwLock<>>` when you have to access to the object **mutably**, in the similar situation with `Arc` case.

**Importer (client)**
- This is not different from the plain Rust progrmaming. Choose whichever type you want.

Note that `Arc<>` will not be supported if the trait has a method that takes `&mut self`.
You must use either `Box<>` or `Arc<RwLock<>>` in such case.

# Service trait
Service trait is the core idea of the `remote-trait-object`. Once you define a trait that you want to use it
as a interface between two ends, you can put `#[remote_trait_object::service]` to make it as a service trait.
It will generate all required code to construct remote object of it (with _call stub_ in it)
and to construct service object of it (with _skeleton_, and _dispatcher_ in it).

## Trait requirements
There are some rules to use a trait as a service trait.

1. Of course, the trait must be **object-safe** becase it will be used as a trait object.

1. It can't have any type item.

1. No generic parameter (including lifetime) is allowed, in both trait definition and methods.

1. All types appeared in method parameter or return value must implement [`serde`]'s [`Serialize`] and [`Deserialize`].
This library performs de/serialization of data using [`serde`], though the data format can be chosen.
Depending on your choice of macro arguments, this condition may differ slightly. See this [section](https://naver.com)

1. You can't return reference as a return type.
This hold for a composite type too. For example, you can't return `&i32` nor `(i32, i32, &i32)`.

1. You can pass only first-order reference as a parameter
For example, you can pass `&T` only if the `T` doesn't contain reference at all.
Note that T must be `Sized`. There are two exceptions that accepts `?Sized` `T`s: `str` and `[U]` where `U` doesn't contain reference at all.

## Compatibility
Although it is common to use same trait for both remote object and service object, it is possible to import a service into another trait.


## Example
```
use remote_trait_object as rto;

#[remote_trait_object_macro::service]
pub trait PizzaStore : rto::Service {
    fn order_pizza(&mut self, menu: &str, money: u64);
    fn ask_pizza_price(&self, menu: &str) -> u64;
}
```

# Export & Import services
One of the core featues of `remote-trait-object` is its simple and straightforward but extensive export & import of services.
Note that such process of exporting and importing is sometimes called _exchange_.

_**Exporting**_ a service means registering the trait object you're going to export in the [`Context`] as a service object, and passing
a _Handle_, which represents the index in the registry, to the importing side.

_**Importing**_ a service means 


Of course this library doens't make you manually register a service object, passing handle and so on, but provide you a much simpler and abstracted way.

There are three ways of exporting and importing a service.


## During initialization
When you create new `remote-trait-object` contexts, you can export and import one as initial services.
See details [here](./struct.Context.html#method.with_initial_service)

## As a parameter or a return value
The most common case of exporting / importing services would be this.

Once you created (or it could be just clone of some [`Arc`]) a trait object, you can
 
## Raw exchange
You will be **rarely** needed to perform a service exchange using raw _handle_.
If you use this method, you will do basically same thing as what the above methods would do internally, but have some extra controls over it.

See the [module-level documentation](mod.raw_exchange.html) for more

You may have [`Skeleton`], which is service to be registered, but **its trait erased**.
You can prepare one and hold it for a while, and register it on demand.
Creating an instance of [`Skeleton`] doesn't involve any context.
That means you can have a service object that both its trait and its context (to be exported later) remains undecided.

You may also have [`HandleToExchange`], which is a raw handle as a result of exporting a [`Skeleton`].
It should be imported as a remote object in the other side, but you can manage it freely until that moment.
It is useful when there is a third party besides two contexts of a single connection, who wants to perform service exchange by itself, not directly between contexts.

Raw exchange is not that frequently required. In most case using only method 1. and 2. will be sufficient.

[`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
*/

//fn order_pizza_with_credit_cart(&mut self, men: &str, card: ServiceRef<dyn CreditCard>);

#[macro_use]
extern crate log;

mod context;
mod forwarder;
mod packet;
mod port;
mod queue;
mod service;
#[cfg(test)]
mod tests;
pub mod transport;

pub use context::{Config, Context};
pub use service::id::setup_identifiers;
pub use service::serde_support::{ServiceRef, ServiceToExport, ServiceToImport};
pub use service::{SerdeFormat, Service};

pub mod raw_exchange {
    //! This module is needed only you want to perform some raw exchange (or export/import) of services.
    //!
    //! You may have [`Skeleton`], which is service to be registered, but **its trait erased**.
    //! You can prepare one and hold it for a while, and register it on demand.
    //! Creating an instance of [`Skeleton`] doesn't involve any context.
    //! That means you can have a service object that both its trait and its context (to be exported later) remains undecided.
    //!
    //! You may also have [`HandleToExchange`], which is a raw handle as a result of exporting a [`Skeleton`].
    //! It should be imported as a remote object in the other side, but you can manage it freely until that moment.
    //! It is useful when there is a third party besides two contexts of a single connection, who wants to perform service exchange by itself, not directly between contexts.
    //!
    //! Raw exchange is not that frequently required. In most case just using ordinary method
    //! Please check again that you surely need this method.

    pub use crate::service::export_import::{
        export_service_into_handle, import_service_from_handle, ImportRemote, IntoSkeleton, Skeleton,
    };
    pub use crate::service::HandleToExchange;
}

#[doc(hidden)]
pub mod macro_env {
    pub use super::raw_exchange::*;
    pub use super::*;
    pub use port::Port;
    pub use service::export_import::create_skeleton;
    pub use service::id::{IdMap, MethodIdAtomic, ID_ORDERING, MID_REG};
    pub use service::{Cbor as DefaultSerdeFormat, Dispatch, Handle, MethodId};
}

// Re-export macro
pub use remote_trait_object_macro::*;

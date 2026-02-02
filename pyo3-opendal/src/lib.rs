// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

// expose the opendal rust core as `core`.
// We will use `ocore::Xxx` to represents all types from opendal rust core.
#[cfg(feature = "runtime")]
pub use ::opendal_core as ocore;
#[cfg(feature = "runtime")]
use pyo3::prelude::*;

#[cfg(feature = "runtime")]
mod capability;
#[cfg(feature = "runtime")]
pub use capability::*;
#[cfg(feature = "runtime")]
pub mod layers;
#[cfg(feature = "runtime")]
pub use layers::*;
#[cfg(feature = "runtime")]
mod lister;
#[cfg(feature = "runtime")]
pub use lister::*;
#[cfg(feature = "runtime")]
mod metadata;
#[cfg(feature = "runtime")]
pub use metadata::*;
#[cfg(feature = "runtime")]
mod operator;
#[cfg(feature = "runtime")]
pub use operator::*;
#[cfg(feature = "runtime")]
mod file;
#[cfg(feature = "runtime")]
pub use file::*;
#[cfg(feature = "runtime")]
mod utils;
#[cfg(feature = "runtime")]
pub use utils::*;
#[cfg(feature = "runtime")]
mod errors;
#[cfg(feature = "runtime")]
pub use errors::*;
#[cfg(feature = "runtime")]
mod options;
#[cfg(feature = "runtime")]
pub use options::*;
#[cfg(feature = "runtime")]
pub mod export;
#[cfg(feature = "runtime")]
pub mod ffi;
#[cfg(feature = "runtime")]
use pyo3_stub_gen::derive::*;

pub mod codegen;

#[cfg(feature = "runtime")]
pub fn default_registry() -> &'static ocore::OperatorRegistry {
    &ocore::DEFAULT_OPERATOR_REGISTRY
}

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
pub use ::opendal_core as ocore;
use pyo3::prelude::*;

mod capability;
pub use capability::*;
pub mod layers;
pub use layers::*;
mod lister;
pub use lister::*;
mod metadata;
pub use metadata::*;
mod operator;
pub use operator::*;
mod file;
pub use file::*;
mod utils;
pub use utils::*;
mod errors;
pub use errors::*;
mod options;
pub use options::*;
#[cfg(feature = "codegen")]
pub mod codegen;
pub mod export;
pub mod ffi;
use pyo3_stub_gen::derive::*;

pub fn default_registry() -> &'static ocore::OperatorRegistry {
    &ocore::DEFAULT_OPERATOR_REGISTRY
}

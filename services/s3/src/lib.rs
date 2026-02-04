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

// expose the pyo3-opendal as `opyo3`.
// We will use `opyo3::Xxx` to represents all types from pyo3-opendal.
use ::pyo3_opendal as opyo3;
use pyo3::prelude::*;

use pyo3::types::PyDict;

use pyo3_stub_gen::define_stub_info_gatherer;

#[allow(deprecated)]
mod s3;
pub use s3::*;

use opyo3::PyRuntimeLayer;
use opyo3::default_registry;
use opyo3::define_build_operator;
use opyo3::export::OpendalOperator;
use opyo3::ocore::IntoOperatorUri;
use opyo3::ocore::Operator;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        opendal_service_s3::register_s3_service(default_registry());
    });
}

define_build_operator!();

#[pymodule(gil_used = false)]
fn _service_s3(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    opyo3::check_debug_build!(py, env!("CARGO_PKG_NAME"))?;

    init();

    // Add version
    opyo3::add_version!(m)?;

    m.add_function(wrap_pyfunction!(__build_operator__, m)?)?;
    m.add_class::<PyS3Service>()?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);

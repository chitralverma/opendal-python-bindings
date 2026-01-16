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
use pyo3_stub_gen::define_stub_info_gatherer;
mod factory;
pub use factory::*;

#[pymodule(gil_used = false)]
fn _layer_retry(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    opyo3::check_debug_build!(py, env!("CARGO_PKG_NAME"))?;

    // Add version
    opyo3::add_version!(m)?;

    // Export factory functions instead of operator classes
    // m.add_function(wrap_pyfunction!(create_fs_operator, m)?)?;
    // m.add_function(wrap_pyfunction!(create_fs_async_operator, m)?)?;
    m.add_class::<factory::PyRetryLayer>()?;

    // Type aliases for backward compatibility
    m.add("PyOperator", py.get_type::<opyo3::PyOperator>())?;
    m.add("PyAsyncOperator", py.get_type::<opyo3::PyAsyncOperator>())?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);

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

// expose the opendal-pyo3 as `opyo3`.
// We will use `opyo3::Xxx` to represents all types from opendal-pyo3.
use ::opendal_pyo3 as opyo3;
use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;
mod s3;
pub use s3::*;

#[pymodule(gil_used = false)]
fn _s3_service(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // use opyo3::*;

    // Add version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    // Operator module
    opyo3::add_pymodule!(py, m, "opendal_s3_service", "operator", [S3PyOperator])?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);

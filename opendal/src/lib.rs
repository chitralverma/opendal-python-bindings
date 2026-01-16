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

use pyo3::prelude::*;
use pyo3_opendal::*;
use pyo3_stub_gen::define_stub_info_gatherer;

#[pymodule(gil_used = false)]
fn _core(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    check_debug_build!(py, "opendal")?;

    // Add version
    add_version!(m)?;

    // Operator module
    add_pymodule!(py, m, "opendal", "operator", [PyOperator, PyAsyncOperator])?;

    // File module
    add_pymodule!(py, m, "opendal", "file", [File, AsyncFile])?;

    // Capability module
    add_pymodule!(py, m, "opendal", "capability", [PyCapability])?;

    // Layers module
    add_pymodule!(py, m, "opendal", "layers", [PyLayer])?;

    // Types module
    add_pymodule!(
        py,
        m,
        "opendal",
        "types",
        [Entry, EntryMode, Metadata, PresignedRequest]
    )?;

    m.add_class::<WriteOptions>()?;
    m.add_class::<ReadOptions>()?;
    m.add_class::<ListOptions>()?;
    m.add_class::<StatOptions>()?;

    // Exceptions module
    add_pyexceptions!(
        py,
        m,
        "opendal",
        "exceptions",
        [
            Error,
            Unexpected,
            Unsupported,
            ConfigInvalid,
            NotFound,
            PermissionDenied,
            IsADirectory,
            NotADirectory,
            AlreadyExists,
            IsSameFile,
            ConditionNotMatch
        ]
    )?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);

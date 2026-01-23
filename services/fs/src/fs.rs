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

//! Factory functions for creating FS service operators
//!
//! This module provides factory functions that create core operator types
//! configured for FS service, ensuring type compatibility with layers.

use opendal_service_fs::FS_SCHEME;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_opendal::export::OpendalOperator;
use pyo3_opendal::layers::PyRuntimeLayer;
use pyo3_opendal::ocore::Operator;
use std::collections::HashMap;

/// Factory function to create a new FS blocking operator
#[pyfunction]
#[pyo3(signature = (**kwargs))]
pub fn create_fs_operator(kwargs: Option<&Bound<PyDict>>) -> PyResult<OpendalOperator> {
    let mut map = HashMap::new();
    if let Some(kwargs) = kwargs {
        map = kwargs.extract::<HashMap<String, String>>()?;
    }

    let runtime = pyo3_async_runtimes::tokio::get_runtime();
    let handle = runtime.handle().clone();

    let op = Operator::via_iter(FS_SCHEME, map.clone())
        .map_err(|err| pyo3::exceptions::PyValueError::new_err(format!("build error: {err}")))?
        .layer(PyRuntimeLayer::new(handle));

    Ok(OpendalOperator::new(op, map, false))
}

/// Factory function to create a new FS async operator
#[pyfunction]
#[pyo3(signature = (**kwargs))]
pub fn create_fs_async_operator(kwargs: Option<&Bound<PyDict>>) -> PyResult<OpendalOperator> {
    let mut map = HashMap::new();
    if let Some(kwargs) = kwargs {
        map = kwargs.extract::<HashMap<String, String>>()?;
    }

    let runtime = pyo3_async_runtimes::tokio::get_runtime();
    let handle = runtime.handle().clone();

    let op = Operator::via_iter(FS_SCHEME, map.clone())
        .map_err(|err| pyo3::exceptions::PyValueError::new_err(format!("build error: {err}")))?
        .layer(PyRuntimeLayer::new(handle));

    Ok(OpendalOperator::new(op, map, true))
}

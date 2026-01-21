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
use pyo3_opendal::export::{OpendalAsyncOperator, OpendalOperator};
use pyo3_opendal::layers::RuntimeLayer;
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

    let op = Operator::via_iter(FS_SCHEME, map)
        .map_err(|err| pyo3::exceptions::PyValueError::new_err(format!("build error: {err}")))?
        .layer(RuntimeLayer::new(handle));

    let _guard = runtime.enter();
    let op = pyo3_opendal::ocore::blocking::Operator::new(op).map_err(|err| {
        pyo3::exceptions::PyValueError::new_err(format!("blocking build error: {err}"))
    })?;

    Ok(OpendalOperator::new(op))
}

/// Factory function to create a new FS async operator
#[pyfunction]
#[pyo3(signature = (**kwargs))]
pub fn create_fs_async_operator(kwargs: Option<&Bound<PyDict>>) -> PyResult<OpendalAsyncOperator> {
    let mut map = HashMap::new();
    if let Some(kwargs) = kwargs {
        map = kwargs.extract::<HashMap<String, String>>()?;
    }

    let runtime = pyo3_async_runtimes::tokio::get_runtime();
    let handle = runtime.handle().clone();

    let op = Operator::via_iter(FS_SCHEME, map)
        .map_err(|err| pyo3::exceptions::PyValueError::new_err(format!("build error: {err}")))?
        .layer(RuntimeLayer::new(handle));

    Ok(OpendalAsyncOperator::new(op))
}

/// FS-specific helper functions
#[pyclass]
pub struct FsHelper {
    // FS-specific helper methods can go here
}

#[pymethods]
impl FsHelper {
    /// Validate FS path format
    #[staticmethod]
    fn validate_path(path: &str) -> PyResult<bool> {
        // FS-specific validation logic
        Ok(!path.is_empty())
    }

    /// Get FS-specific capabilities
    #[staticmethod]
    fn get_capabilities() -> PyResult<Vec<String>> {
        Ok(vec![
            "read".to_string(),
            "write".to_string(),
            "delete".to_string(),
            "list".to_string(),
            "stat".to_string(),
        ])
    }
}

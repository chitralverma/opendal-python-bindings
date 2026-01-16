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

//! Type registry for sharing PyO3 types across modules
//!
//! This module provides a way to share base operator types across different
//! service modules, ensuring that all operators and layers use the exact same
//! type instances.

use pyo3::prelude::*;
use std::sync::OnceLock;

use super::{PyAsyncOperator, PyOperator};

/// Global registry for shared PyO3 types
pub struct SharedTypeRegistry {
    pub py_operator: Py<PyAny>,
    pub py_async_operator: Py<PyAny>,
}

static SHARED_REGISTRY: OnceLock<SharedTypeRegistry> = OnceLock::new();

/// Initialize the shared type registry
/// This should be called once from the core module when it's initialized
pub fn initialize_shared_types(py: Python) -> PyResult<()> {
    let registry = SharedTypeRegistry {
        py_operator: Py::new(py, PyOperator::new_empty())?,
        py_async_operator: Py::new(py, PyAsyncOperator::new_empty())?,
    };

    SHARED_REGISTRY.set(registry).map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Shared type registry already initialized",
        )
    })?;

    Ok(())
}

/// Get the shared PyOperator type
pub fn get_shared_py_operator() -> &'static Py<PyAny> {
    SHARED_REGISTRY
        .get()
        .expect("Shared type registry not initialized")
        .py_operator
        .as_ref()
}

/// Get the shared PyAsyncOperator type
pub fn get_shared_py_async_operator() -> &'static Py<PyAny> {
    SHARED_REGISTRY
        .get()
        .expect("Shared type registry not initialized")
        .py_async_operator
        .as_ref()
}

/// Check if the shared registry is initialized
pub fn is_initialized() -> bool {
    SHARED_REGISTRY.get().is_some()
}

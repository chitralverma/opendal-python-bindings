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

use crate::opyo3;
use opendal_pyo3::{PyAsyncOperator, PyOperator};
use opendal_service_fs::FS_SCHEME;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;
// use crate::opyo3::PyAsyncOperator;
use pyo3::types::PyDict;

use std::collections::HashMap;

/// The blocking equivalent of `FSPyAsyncOperator` for fs service.
///
/// `FsPyOperator` is the entry point for all blocking APIs for fs service.
///
/// See also
/// --------
/// FSPyAsyncOperator
#[gen_stub_pyclass]
#[pyclass(module = "opendal_fs_service.operator", extends=opyo3::PyOperator)]
#[derive(Clone)]
pub struct FsPyOperator {
    core: opyo3::ocore::blocking::Operator,
    __scheme: String,
    __map: HashMap<String, String>,
}

#[gen_stub_pymethods]
#[pymethods]
impl FsPyOperator {
    /// Create a new blocking `Operator` for fs service.
    ///
    /// Parameters
    /// ----------
    /// **kwargs : dict
    ///     The options for the service.
    ///
    /// Returns
    /// -------
    /// Operator
    ///     The new operator.
    #[gen_stub(override_return_type(type_repr = "FsPyOperator"))]
    #[new]
    #[pyo3(signature = (*, **kwargs))]
    pub fn new(kwargs: Option<&Bound<PyDict>>) -> PyResult<PyClassInitializer<Self>> {
        let map = kwargs
            .map(|v| {
                v.extract::<HashMap<String, String>>()
                    .expect("must be valid hashmap")
            })
            .unwrap_or_default();
        let py_operator = Self {
            core: opyo3::build_blocking_operator(FS_SCHEME, map.clone())?,
            __scheme: FS_SCHEME.to_string(),
            __map: map,
        };

        let class = PyClassInitializer::from(PyOperator {
            core: py_operator.core.clone(),
            __scheme: py_operator.__scheme.clone(),
            __map: py_operator.__map.clone(),
        })
        .add_subclass(py_operator);

        Ok(class)
    }
}

/// The async equivalent of `FSPyAsyncOperator` for fs service.
///
/// `FsPyAsyncOperator` is the entry point for all async APIs for fs service.
///
/// See also
/// --------
/// FSPyOperator
#[gen_stub_pyclass]
#[pyclass(module = "opendal_fs_service.operator", extends=opyo3::PyAsyncOperator)]
#[derive(Clone)]
pub struct FsPyAsyncOperator {
    core: opyo3::ocore::Operator,
    __scheme: String,
    __map: HashMap<String, String>,
}

#[gen_stub_pymethods]
#[pymethods]
impl FsPyAsyncOperator {
    /// Create a new blocking `Operator` for fs service.
    ///
    /// Parameters
    /// ----------
    /// **kwargs : dict
    ///     The options for the service.
    ///
    /// Returns
    /// -------
    /// Operator
    ///     The new operator.
    #[gen_stub(override_return_type(type_repr = "FsPyAsyncOperator"))]
    #[new]
    #[pyo3(signature = (*, **kwargs))]
    pub fn new(kwargs: Option<&Bound<PyDict>>) -> PyResult<PyClassInitializer<Self>> {
        let map = kwargs
            .map(|v| {
                v.extract::<HashMap<String, String>>()
                    .expect("must be valid hashmap")
            })
            .unwrap_or_default();
        let py_operator = Self {
            core: opyo3::build_operator(FS_SCHEME, map.clone())?,
            __scheme: FS_SCHEME.to_string(),
            __map: map,
        };

        let class = PyClassInitializer::from(PyAsyncOperator {
            core: py_operator.core.clone(),
            __scheme: py_operator.__scheme.clone(),
            __map: py_operator.__map.clone(),
        })
        .add_subclass(py_operator);

        Ok(class)
    }
}

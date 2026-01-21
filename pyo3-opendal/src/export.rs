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

use crate::ffi::{to_async_operator_capsule, to_operator_capsule};

use crate::ocore;

use pyo3::intern;

use pyo3::prelude::*;

use pyo3::IntoPyObjectExt;

use std::collections::HashMap;

/// A wrapper around [`ocore::blocking::Operator`] that implements [`IntoPyObject`] to convert to a
/// runtime-available `opendal.Operator`.
pub struct OpendalOperator {
    op: ocore::blocking::Operator,
    map: HashMap<String, String>,
}

impl OpendalOperator {
    pub fn new(op: ocore::blocking::Operator, map: HashMap<String, String>) -> Self {
        Self { op, map }
    }
}

impl<'py> IntoPyObject<'py> for OpendalOperator {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let opendal_mod = py.import(intern!(py, "opendal"))?;
        let capsule = to_operator_capsule(py, self.op)?;
        let map = self.map.into_py_any(py)?;
        opendal_mod
            .getattr(intern!(py, "Operator"))?
            .call_method1(intern!(py, "_from_capsule"), (capsule, map))
    }
}

/// A wrapper around [`ocore::Operator`] that implements [`IntoPyObject`] to convert to a
/// runtime-available `opendal.AsyncOperator`.
pub struct OpendalAsyncOperator {
    op: ocore::Operator,
    map: HashMap<String, String>,
}

impl OpendalAsyncOperator {
    pub fn new(op: ocore::Operator, map: HashMap<String, String>) -> Self {
        Self { op, map }
    }
}

impl<'py> IntoPyObject<'py> for OpendalAsyncOperator {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let opendal_mod = py.import(intern!(py, "opendal"))?;
        let capsule = to_async_operator_capsule(py, self.op)?;
        let map = self.map.into_py_any(py)?;
        opendal_mod
            .getattr(intern!(py, "AsyncOperator"))?
            .call_method1(intern!(py, "_from_capsule"), (capsule, map))
    }
}

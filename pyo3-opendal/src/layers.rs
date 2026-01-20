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

use crate::*;
use ocore::Operator;
use pyo3::types::PyCapsule;

pub trait PythonLayer: Send + Sync {
    fn layer(&self, op: Operator) -> Operator;
    fn layer_blocking(&self, op: ocore::blocking::Operator) -> ocore::blocking::Operator;
}

struct DummyLayer;

impl PythonLayer for DummyLayer {
    fn layer(&self, op: Operator) -> Operator {
        op
    }

    fn layer_blocking(&self, op: ocore::blocking::Operator) -> ocore::blocking::Operator {
        op
    }
}

/// Layers are used to intercept the operations on the underlying storage.
#[gen_stub_pyclass]
#[pyclass(module = "opendal.layers", name = "Layer", subclass, frozen)]
pub struct PyLayer(pub Box<dyn PythonLayer>);

#[gen_stub_pymethods]
#[pymethods]
impl PyLayer {
    #[new]
    pub fn new() -> PyResult<Self> {
        Ok(Self(Box::new(DummyLayer)))
    }

    /// Apply the layer to an async operator (passed as capsule) and return a new operator (as capsule).
    #[gen_stub(skip)]
    fn _layer_apply<'py>(
        &self,
        py: Python<'py>,
        op_capsule: &Bound<'py, PyCapsule>,
    ) -> PyResult<Bound<'py, PyCapsule>> {
        let op = crate::ffi::from_async_operator_capsule(op_capsule)?;
        let new_op = self.0.layer(op);
        crate::ffi::to_async_operator_capsule(py, new_op)
    }

    /// Apply the layer to a blocking operator (passed as capsule) and return a new operator (as capsule).
    #[gen_stub(skip)]
    fn _layer_apply_blocking<'py>(
        &self,
        py: Python<'py>,
        op_capsule: &Bound<'py, PyCapsule>,
    ) -> PyResult<Bound<'py, PyCapsule>> {
        let op = crate::ffi::from_operator_capsule(op_capsule)?;
        let new_op = self.0.layer_blocking(op);
        crate::ffi::to_operator_capsule(py, new_op)
    }
}

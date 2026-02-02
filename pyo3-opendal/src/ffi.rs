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

use std::ffi::{CStr, CString};

use pyo3::prelude::*;
use pyo3::types::{PyCapsule, PyCapsuleMethods};

use crate::layers::PythonLayer;
use crate::ocore;

const OPENDAL_OPERATOR_CAPSULE_NAME: &CStr = c"opendal_operator";
const OPENDAL_LAYER_CAPSULE_NAME: &CStr = c"opendal_layer";

/// Export an [`ocore::Operator`] to a PyCapsule.
pub fn to_operator_capsule(py: Python, op: ocore::Operator) -> PyResult<Bound<PyCapsule>> {
    PyCapsule::new(py, op, Some(CString::from(OPENDAL_OPERATOR_CAPSULE_NAME)))
}

/// Import an [`ocore::Operator`] from a PyCapsule.
pub fn from_operator_capsule(capsule: &Bound<PyCapsule>) -> PyResult<ocore::Operator> {
    let ptr = capsule
        .pointer_checked(Some(OPENDAL_OPERATOR_CAPSULE_NAME))?
        .cast::<ocore::Operator>();
    Ok(unsafe { ptr.as_ref().clone() })
}

/// Export a [`Box<dyn PythonLayer>`] to a PyCapsule.
///
/// Note: This consumes the box.
pub fn to_layer_capsule(py: Python, layer: Box<dyn PythonLayer>) -> PyResult<Bound<PyCapsule>> {
    PyCapsule::new(py, layer, Some(CString::from(OPENDAL_LAYER_CAPSULE_NAME)))
}

/// Import a [`Box<dyn PythonLayer>`] from a PyCapsule.
pub fn from_layer_capsule<'a>(
    capsule: &'a Bound<'a, PyCapsule>,
) -> PyResult<&'a Box<dyn PythonLayer>> {
    let ptr = capsule
        .pointer_checked(Some(OPENDAL_LAYER_CAPSULE_NAME))?
        .cast();
    Ok(unsafe { ptr.as_ref() })
}

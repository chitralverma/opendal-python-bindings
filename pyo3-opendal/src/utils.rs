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

use std::collections::HashMap;
use std::os::raw::c_int;

use crate::Unexpected;
use crate::ocore::Configurator;

use pyo3::IntoPyObjectExt;
use pyo3::ffi;
use pyo3::prelude::*;
use serde_json::Value;

/// A bytes-like object that implements buffer protocol.
#[pyclass(module = "opendal")]
pub struct Buffer {
    inner: Vec<u8>,
}

impl Buffer {
    pub fn new(inner: Vec<u8>) -> Self {
        Buffer { inner }
    }

    /// Consume self to build a bytes
    pub fn into_bytes(self, py: Python) -> PyResult<Py<PyAny>> {
        let buffer = self.into_py_any(py)?;

        unsafe { Py::<PyAny>::from_owned_ptr_or_err(py, ffi::PyBytes_FromObject(buffer.as_ptr())) }
    }

    /// Consume self to build a bytes
    pub fn into_bytes_ref(self, py: Python) -> PyResult<Bound<PyAny>> {
        let buffer = self.into_py_any(py)?;
        let view =
            unsafe { Bound::from_owned_ptr_or_err(py, ffi::PyBytes_FromObject(buffer.as_ptr()))? };

        Ok(view)
    }
}

#[pymethods]
impl Buffer {
    unsafe fn __getbuffer__(
        slf: PyRefMut<Self>,
        view: *mut ffi::Py_buffer,
        flags: c_int,
    ) -> PyResult<()> {
        let bytes = slf.inner.as_slice();
        let ret = unsafe {
            ffi::PyBuffer_FillInfo(
                view,
                slf.as_ptr() as *mut _,
                bytes.as_ptr() as *mut _,
                bytes.len().try_into().unwrap(),
                1, // read only
                flags,
            )
        };
        if ret == -1 {
            return Err(PyErr::fetch(slf.py()));
        }
        Ok(())
    }
}

/// Macro to create and register a PyO3 submodule with multiple classes.
///
/// Example:
/// ```rust
/// add_pymodule!(py, m, "services", [PyScheme, PyOtherClass]);
/// ```
#[macro_export]
macro_rules! add_pymodule {
    ($py:expr, $parent:expr, $parent_name:expr, $name:expr, [$($cls:ty),* $(,)?]) => {{
        let sub_module = pyo3::types::PyModule::new($py, $name)?;
        $(
            sub_module.add_class::<$cls>()?;
        )*
        $parent.add_submodule(&sub_module)?;
        $py.import("sys")?
            .getattr("modules")?
            .set_item(format!("{}.{}", $parent_name, $name), &sub_module)?;
        Ok::<_, pyo3::PyErr>(())
    }};
}

/// Macro to create and register a PyO3 submodule containing exception types.
///
/// Example:
/// ```rust
/// add_pyexceptions!(py, m, "exceptions", [Error, Unexpected]);
/// ```
#[macro_export]
macro_rules! add_pyexceptions {
    ($py:expr, $parent:expr, $parent_name:expr, $name:expr, [$($exc:ty),* $(,)?]) => {{
        let sub_module = pyo3::types::PyModule::new($py, $name)?;
        $(
            sub_module.add(stringify!($exc), $py.get_type::<$exc>())?;
        )*
        $parent.add_submodule(&sub_module)?;
        $py.import("sys")?
            .getattr("modules")?
            .set_item(format!("{}.{}", $parent_name, $name), &sub_module)?;
        Ok::<_, pyo3::PyErr>(())
    }};
}

#[macro_export]
macro_rules! add_version {
    ($mod:expr) => {{ $mod.add("__version__", env!("CARGO_PKG_VERSION")) }};
}

#[macro_export]
macro_rules! check_debug_build {
    ($py:expr, $name:expr) => {{
        #[cfg(debug_assertions)]
        {
            use pyo3::exceptions::PyRuntimeWarning;
            use pyo3::intern;
            use pyo3::types::PyTuple;

            let warnings_mod = $py.import(intern!($py, "warnings"))?;
            let warning = PyRuntimeWarning::new_err(format!(
                "{} has not been compiled in release mode. Performance will be degraded.",
                $name
            ));
            let args = PyTuple::new($py, vec![warning])?;
            warnings_mod.call_method1(intern!($py, "warn"), args)?;
        }
        Ok::<_, pyo3::PyErr>(())
    }};
}

#[macro_export]
macro_rules! define_build_operator {
    () => {
        #[pyfunction]
        #[pyo3(signature = (scheme, is_async, **kwargs))]
        pub fn __build_operator__(
            scheme: String,
            is_async: bool,
            kwargs: Option<&Bound<PyDict>>,
        ) -> PyResult<OpendalOperator> {
            use pyo3::types::PyDict;
            use std::collections::HashMap;

            let opts = kwargs
                .map(|v| {
                    v.extract::<HashMap<String, String>>()
                        .expect("must be valid hashmap")
                })
                .unwrap_or_default();

            let uri = (scheme, opts.clone())
                .into_operator_uri()
                .map_err(pyo3_opendal::format_pyerr)?;

            let runtime = pyo3_async_runtimes::tokio::get_runtime();
            let handle = runtime.handle().clone();

            let op = Operator::from_uri(uri.clone())
                .map_err(pyo3_opendal::format_pyerr)?
                .layer(PyRuntimeLayer::new(handle));

            Ok(OpendalOperator::new(op, opts, is_async))
        }
    };
}

pub trait FromConfigurator: Sized {
    fn from_configurator<C>(cfg: &C) -> PyResult<Self>
    where
        C: Configurator + serde::Serialize;
}

impl<T> FromConfigurator for T
where
    T: serde::de::DeserializeOwned,
{
    fn from_configurator<C>(cfg: &C) -> PyResult<Self>
    where
        C: Configurator + serde::Serialize,
    {
        let v = serde_json::to_value(cfg)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        serde_json::from_value(v)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
}

pub trait ToStringMap {
    fn to_string_map(&self) -> PyResult<HashMap<String, String>>;
}

impl<T: serde::Serialize> ToStringMap for T {
    fn to_string_map(&self) -> PyResult<HashMap<String, String>> {
        let json = serde_json::to_value(self).map_err(|_| {
            Unexpected::new_err(format!(
                "Internal serialization error for instance of {}",
                std::any::type_name::<T>()
            ))
        })?;

        let mut map = HashMap::new();

        if let Value::Object(obj) = json {
            for (k, v) in obj {
                match v {
                    Value::String(s) => {
                        map.insert(k, s);
                    }
                    Value::Null => {
                        // skip
                    }
                    other => {
                        map.insert(k, other.to_string());
                    }
                }
            }
        }

        Ok(map)
    }
}

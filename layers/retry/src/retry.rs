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
use opendal_layer_retry;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;
use std::time::Duration;

/// A layer that retries operations that fail with temporary errors.
///
/// Operations are retried if they fail with an error for which
/// `Error.is_temporary` returns `True`. If all retries are exhausted,
/// the error is marked as persistent and then returned.
///
/// Notes
/// -----
/// After an operation on a `Reader` or `Writer` has failed through
/// all retries, the object is in an undefined state. Reusing it
/// can lead to exceptions.
#[gen_stub_pyclass]
#[pyclass(name = "RetryLayer", extends=opyo3::PyLayer)]
#[derive(Clone)]
pub struct PyRetryLayer {
    pub l: opendal_layer_retry::RetryLayer,
}
// pub struct PyRetryLayer {
//     int_layer: opendal_layer_retry::RetryLayer,
//     max_times: Option<usize>,
//     factor: Option<f32>,
//     jitter: bool,
//     max_delay: Option<f64>,
//     min_delay: Option<f64>,
//     // FS-specific helper methods can go here
// }
impl opyo3::PythonLayer for PyRetryLayer {
    fn layer(&self, op: opyo3::ocore::Operator) -> opyo3::ocore::Operator {
        op.layer(self.l.clone())
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyRetryLayer {
    /// Create a new RetryLayer.
    ///
    /// Parameters
    /// ----------
    /// max_times : Optional[int]
    ///     Maximum number of retry attempts. Defaults to ``3``.
    /// factor : Optional[float]
    ///     Backoff factor applied between retries. Defaults to ``2.0``.
    /// jitter : bool
    ///     Whether to apply jitter to the backoff. Defaults to ``False``.
    /// max_delay : Optional[float]
    ///     Maximum delay (in seconds) between retries. Defaults to ``60.0``.
    /// min_delay : Optional[float]
    ///     Minimum delay (in seconds) between retries. Defaults to ``1.0``.
    ///
    /// Returns
    /// -------
    /// RetryLayer
    #[gen_stub(override_return_type(type_repr = "RetryLayer"))]
    #[new]
    #[pyo3(signature = (
        max_times = None,
        factor = None,
        jitter = false,
        max_delay = None,
        min_delay = None
    ))]
    fn new(
        max_times: Option<usize>,
        factor: Option<f32>,
        jitter: bool,
        max_delay: Option<f64>,
        min_delay: Option<f64>,
    ) -> PyResult<PyClassInitializer<Self>> {
        let mut retry = opendal_layer_retry::RetryLayer::default();
        if let Some(max_times) = max_times {
            retry = retry.with_max_times(max_times);
        }
        if let Some(factor) = factor {
            retry = retry.with_factor(factor);
        }
        if jitter {
            retry = retry.with_jitter();
        }
        if let Some(max_delay) = max_delay {
            retry = retry.with_max_delay(Duration::from_micros((max_delay * 1_000_000.0) as u64));
        }
        if let Some(min_delay) = min_delay {
            retry = retry.with_min_delay(Duration::from_micros((min_delay * 1_000_000.0) as u64));
        }

        let retry_layer = Self { l: retry };
        let class = PyClassInitializer::from(opyo3::PyLayer::new()?).add_subclass(retry_layer);

        Ok(class)
    }
}

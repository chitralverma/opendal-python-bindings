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

//! Factory functions for creating S3 service operators
//!
//! This module provides factory functions that create core operator types
//! configured for S3 service, ensuring type compatibility with layers.

use crate::opyo3;
use opendal_service_s3::S3_SCHEME;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_opendal::export::{OpendalAsyncOperator, OpendalOperator};
use std::collections::HashMap;

/// Factory function to create a new S3 blocking operator
#[pyfunction]
#[pyo3(signature = (**kwargs))]
pub fn create_s3_operator(kwargs: Option<&Bound<PyDict>>) -> PyResult<OpendalOperator> {
    let mut map = HashMap::new();
    if let Some(kwargs) = kwargs {
        map = kwargs.extract::<HashMap<String, String>>()?;
    }

    // Build S3 blocking operator
    let core = opyo3::build_blocking_operator(S3_SCHEME, map.clone())?;

    Ok(OpendalOperator::new(core))
}

/// Factory function to create a new S3 async operator
#[pyfunction]
#[pyo3(signature = (**kwargs))]
pub fn create_s3_async_operator(kwargs: Option<&Bound<PyDict>>) -> PyResult<OpendalAsyncOperator> {
    let mut map = HashMap::new();
    if let Some(kwargs) = kwargs {
        map = kwargs.extract::<HashMap<String, String>>()?;
    }

    // Build S3 async operator
    let core = opyo3::build_operator(S3_SCHEME, map.clone())?;

    Ok(OpendalAsyncOperator::new(core))
}

/// S3-specific helper functions
#[pyclass]
pub struct S3Helper {
    // S3-specific helper methods can go here
}

#[pymethods]
impl S3Helper {
    /// Validate S3 bucket name format
    #[staticmethod]
    fn validate_bucket_name(bucket: &str) -> PyResult<bool> {
        // S3-specific validation logic
        Ok(bucket.len() >= 3 && bucket.len() <= 63 && !bucket.is_empty())
    }

    /// Get S3-specific capabilities
    #[staticmethod]
    fn get_capabilities() -> PyResult<Vec<String>> {
        Ok(vec![
            "read".to_string(),
            "write".to_string(),
            "delete".to_string(),
            "list".to_string(),
            "stat".to_string(),
            "presign".to_string(),
            "multipart".to_string(),
        ])
    }
}

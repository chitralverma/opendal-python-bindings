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

use opendal_service_fs::FsConfig;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_opendal::FromConfigurator;
use pyo3_opendal::ToStringMap;
use pyo3_opendal::export::OpendalOperator;
use pyo3_opendal::ocore::Configurator;
use pyo3_opendal::ocore::Operator;
use pyo3_opendal::ocore::OperatorUri;
use pyo3_stub_gen::derive::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[gen_stub_pyclass]
#[pyclass(get_all, set_all, name = "FsService")]
#[derive(Clone, Default, Serialize, Deserialize)]
#[allow(deprecated)]
pub struct PyFsService {
    /// root dir for backend
    pub root: Option<String>,
    /// tmp dir for atomic write
    pub atomic_write_dir: Option<String>,
}

impl From<PyFsService> for FsConfig {
    #[allow(deprecated)]
    fn from(opts: PyFsService) -> Self {
        let mut cfg = FsConfig::default();
        cfg.root = opts.root;
        cfg.atomic_write_dir = opts.atomic_write_dir;
        cfg
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyFsService {
    #[staticmethod]
    #[pyo3(signature = (**kwargs))]
    pub fn from_config(kwargs: Option<&Bound<PyDict>>) -> PyResult<Self> {
        let map: HashMap<String, String> =
            kwargs.map(|d| d.extract()).transpose()?.unwrap_or_default();
        let cfg = FsConfig::from_iter(map).map_err(pyo3_opendal::format_pyerr)?;

        Self::from_configurator(&cfg)
    }

    #[staticmethod]
    #[pyo3(signature = (uri, **kwargs))]
    pub fn from_uri(uri: &str, kwargs: Option<&Bound<PyDict>>) -> PyResult<Self> {
        let map: HashMap<String, String> =
            kwargs.map(|d| d.extract()).transpose()?.unwrap_or_default();

        let cfg = OperatorUri::new(uri, map)
            .and_then(|u| FsConfig::from_uri(&u))
            .map_err(pyo3_opendal::format_pyerr)?;

        Self::from_configurator(&cfg)
    }

    #[gen_stub(override_return_type(type_repr = "opendal.AsyncOperator", imports=("opendal")))]
    pub fn to_async_operator(&self) -> PyResult<OpendalOperator> {
        let cfg: FsConfig = self.clone().into();
        let map = cfg.to_string_map()?;
        let op = Operator::from_config(cfg)
            .map_err(pyo3_opendal::format_pyerr)?
            .finish();

        Ok(OpendalOperator::new(op, map, true))
    }

    #[gen_stub(override_return_type(type_repr = "opendal.Operator", imports=("opendal")))]
    pub fn to_operator(&self) -> PyResult<OpendalOperator> {
        let op = self.to_async_operator()?;
        Ok(OpendalOperator::new(op.op, op.map, false))
    }
}

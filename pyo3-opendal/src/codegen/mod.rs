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

#[cfg(feature = "codegen")]
pub mod layer;
#[cfg(feature = "codegen")]
pub mod parser;
#[cfg(feature = "codegen")]
pub mod service;
#[cfg(feature = "codegen")]
pub mod types;
#[cfg(feature = "codegen")]
pub mod utils;

#[cfg(feature = "stub-gen")]
pub fn generate_service_stub<F>(
    _service_name: &str,
    pkg_name: &str,
    stub_info_getter: F,
) -> pyo3_stub_gen::Result<()>
where
    F: FnOnce() -> pyo3_stub_gen::Result<pyo3_stub_gen::StubInfo>,
{
    // Generate Stubs
    let mut stub = stub_info_getter()?;
    let pkg_name_clean = pkg_name.replace('-', "_");
    stub.modules
        .retain(|k: &String, _| k.starts_with(&pkg_name_clean));
    stub.generate()?;
    Ok(())
}

#[cfg(feature = "stub-gen")]
pub fn generate_layer_stub<F>(
    _layer_name: &str,
    pkg_name: &str,
    stub_info_getter: F,
) -> pyo3_stub_gen::Result<()>
where
    F: FnOnce() -> pyo3_stub_gen::Result<pyo3_stub_gen::StubInfo>,
{
    // Generate Stubs
    let mut stub = stub_info_getter()?;
    let pkg_name_clean = pkg_name.replace('-', "_");
    stub.modules
        .retain(|k: &String, _| k.starts_with(&pkg_name_clean));
    stub.generate()?;
    Ok(())
}

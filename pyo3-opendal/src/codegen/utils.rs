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

use anyhow::{Result, anyhow};
use cargo_metadata::MetadataCommand;
use std::path::{Path, PathBuf};

pub fn find_dependency_path(package_path: &Path, dep_name: &str) -> Result<PathBuf> {
    let manifest_path = package_path.join("Cargo.toml").canonicalize()?;

    let metadata = MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()?;

    let pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == dep_name && p.source.is_some())
        .ok_or_else(|| anyhow!("Could not find dependency package {}", dep_name))?;

    let dep_manifest_path = pkg.manifest_path.clone().into_std_path_buf();
    Ok(dep_manifest_path.parent().unwrap().to_path_buf())
}

pub fn to_pascal(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut capitalize = true;

    for &b in name.as_bytes() {
        if b == b'_' || b == b'-' {
            capitalize = true;
        } else if capitalize {
            result.push((b as char).to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(b as char);
        }
    }

    result
}

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

use crate::generate::parser::{parse_service_config, ConfigType};
use anyhow::{anyhow, Result};
use cargo_metadata::MetadataCommand;
use minijinja::value::ViaDeserialize;
use minijinja::{Environment, context};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(service_name: &str, package_path: &str) -> Result<()> {
    // 1. Find dependency path
    let dep_path = find_dependency_path(package_path, service_name)?;
    let config_path = dep_path.join("src/config.rs");

    if !config_path.exists() {
        return Err(anyhow!("Config file not found at {:?}", config_path));
    }

    // 2. Parse config
    let service_config = parse_service_config(&config_path, service_name)?;

    // 3. Setup Jinja
    let mut env = Environment::new();
    env.add_template("service", include_str!("service.j2"))?;
    env.add_function("make_rust_type", make_rust_type);
    env.add_function("convert_rust_type", convert_rust_type);

    let tmpl = env.get_template("service")?;

    let service_pascal = service_to_pascal(service_name);

    let context = context! {
        service => service_name,
        service_pascal => service_pascal,
        config => service_config,
    };

    let output = tmpl.render(context)?;

    // Write to src/_service.rs
    let output_path = PathBuf::from(package_path).join(format!("src/{}.rs", service_name));
    fs::write(&output_path, output)?;

    println!("Generated bindings for opendal-service-{} at {:?}", service_name, output_path);
    Ok(())
}

fn find_dependency_path(package_path: &str, service_name: &str) -> Result<PathBuf> {
    let manifest_path = Path::new(package_path).join("Cargo.toml").canonicalize()?;

    let metadata = MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()?;

    // The package name in Cargo.toml is like "opendal-service-fs"
    let dep_name = format!("opendal-service-{}", service_name);

    // We need to find the ID of the dependency in the resolve graph or just find the package info
    // cargo metadata returns all packages in the workspace + dependencies
    // We filter for packages that have a source (are not local workspace members)
    let pkg = metadata.packages.iter()
        .find(|p| p.name == dep_name && p.source.is_some())
        .ok_or_else(|| anyhow!("Could not find dependency package {}", dep_name))?;

    let dep_manifest_path = pkg.manifest_path.clone().into_std_path_buf();
    Ok(dep_manifest_path.parent().unwrap().to_path_buf())
}

fn service_to_pascal(service: &str) -> String {
    let mut result = String::with_capacity(service.len());
    let mut capitalize = true;

    for &b in service.as_bytes() {
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

fn make_rust_type(ty: ViaDeserialize<ConfigType>, is_option: bool) -> String {
    let base_type = match ty.0 {
        ConfigType::Bool => "bool",
        ConfigType::String => "String",
        ConfigType::Duration => "String", // Duration is usually passed as string in config
        ConfigType::Usize => "usize",
        ConfigType::U64 => "u64",
        ConfigType::I64 => "i64",
        ConfigType::U32 => "u32",
        ConfigType::U16 => "u16",
        ConfigType::Vec => "Vec<String>",
    };

    let optional = is_option || ty.0 == ConfigType::Bool;

    if optional {
        format!("Option<{}>", base_type)
    } else {
        base_type.to_string()
    }
}

fn convert_rust_type(name: &str, _ty: ViaDeserialize<ConfigType>, _is_option: bool) -> String {
    // For now, we assume direct mapping is sufficient as we use primitive types.
    // In future this can handle type conversions if we change `make_rust_type`.
    format!("opts.{}", name)
}

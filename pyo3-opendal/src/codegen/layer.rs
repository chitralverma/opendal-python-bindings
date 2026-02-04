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
use quote::{format_ident, quote};
use std::fs;
use std::path::{Path, PathBuf};
use syn::{
    Expr, ExprLit, ImplItem, Item, Lit, Meta, ReturnType, Type, TypePath, Visibility, parse_file,
};

pub fn generate(layer_name: &str, package_path: &Path) -> Result<String> {
    // 1. Find dependency path
    let dep_path = find_dependency_path(package_path, layer_name)?;
    let src_path = dep_path.join(format!("src/{}.rs", layer_name.replace('-', "_")));

    // Fallback to lib.rs if specific file doesn't exist
    let src_path = if src_path.exists() {
        src_path
    } else {
        dep_path.join("src/lib.rs")
    };

    if !src_path.exists() {
        return Err(anyhow!("Source file not found at {:?}", src_path));
    }

    println!("cargo:rerun-if-changed={}", src_path.display());

    // 2. Parse layer struct and methods
    let layer_def = parse_layer_def(&src_path, layer_name)?;

    let layer_pascal = layer_to_pascal(layer_name);
    let layer_snake = layer_name.replace('-', "_");
    let py_layer_ident = format_ident!("Py{}Layer", layer_pascal);
    let layer_struct_ident = format_ident!("{}Layer", layer_pascal);
    let layer_module = format_ident!("opendal_layer_{}", layer_snake);
    let layer_pascal_lit = format!("{}Layer", layer_pascal);

    // 3. Generate arguments for new()
    let mut new_args = Vec::new();
    let mut new_assignments = Vec::new();
    let mut signature_args = Vec::new();
    let mut doc_params = Vec::new();

    // Sort methods by arg_name to ensure deterministic order
    let mut methods = layer_def.methods;
    methods.sort_by(|a, b| a.arg_name.cmp(&b.arg_name));

    for method in methods {
        let method_name = format_ident!("{}", method.name);
        let arg_name = format_ident!("{}", method.arg_name);
        let arg_type_str = method.arg_type.as_str();

        // Type mapping and filtering. Only support types that map easily to Python.
        let (py_type, default_val, is_bool, py_type_doc) = match arg_type_str {
            "bool" => (quote!(bool), quote!(false), true, "bool"),
            "String" => (quote!(String), quote!(None), false, "str"),
            "usize" | "u64" | "i64" | "u32" | "u16" | "isize" | "i32" | "i16" | "i8" | "u8" => {
                let t = format_ident!("{}", arg_type_str);
                (quote!(#t), quote!(None), false, "int")
            }
            "f32" | "f64" => {
                let t = format_ident!("{}", arg_type_str);
                (quote!(#t), quote!(None), false, "float")
            }
            "Duration" => (quote!(std::time::Duration), quote!(None), false, "float"),
            _ => {
                // Skip unsupported types
                eprintln!(
                    "Skipping method {} due to unsupported type {}",
                    method.name, arg_type_str
                );
                continue;
            }
        };

        // Generate docstring entry
        let doc_desc = if method.docs.is_empty() {
            format!("See `{}`.", method.name)
        } else {
            // Join lines, trimming and adding indentation.
            // If a line starts with `#`, treat it as a header and ensure it starts on a new line.
            let mut lines = Vec::new();
            for s in method.docs.iter() {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    lines.push(trimmed.to_string());
                }
            }
            lines.join("\n    ")
        };

        doc_params.push(format!(
            "{} : Optional[{}]\n    {}",
            method.arg_name, py_type_doc, doc_desc
        ));

        if method.is_toggle {
            new_args.push(quote! { #arg_name: bool });
            signature_args.push(quote! { #arg_name = #default_val });
            new_assignments.push(quote! {
                if #arg_name {
                    layer = layer.#method_name();
                }
            });
        } else if is_bool {
            new_args.push(quote! { #arg_name: bool });
            signature_args.push(quote! { #arg_name = #default_val });
            new_assignments.push(quote! {
                if #arg_name {
                    layer = layer.#method_name(#arg_name);
                }
            });
        } else {
            // Treat as Option
            new_args.push(quote! { #arg_name: Option<#py_type> });
            signature_args.push(quote! { #arg_name = #default_val });
            new_assignments.push(quote! {
                if let Some(v) = #arg_name {
                    layer = layer.#method_name(v);
                }
            });
        }
    }

    // Construct docstring
    let doc_string = if doc_params.is_empty() {
        format!(
            "Create a new {}.\n\nReturns\n-------\n{}",
            layer_pascal_lit, layer_pascal_lit
        )
    } else {
        format!(
            "Create a new {}.\n\nParameters\n----------\n{}\n\nReturns\n-------\n{}",
            layer_pascal_lit,
            doc_params.join("\n"),
            layer_pascal_lit
        )
    };

    let doc_attributes = doc_string.lines().map(|line| quote!(#[doc = #line]));

    let code = quote! {
        //! This file is automatically generated by `pyo3_opendal::codegen::generate_layer_stub`
        #![allow(clippy::possible_missing_else)]

        use #layer_module::#layer_struct_ident;
        use crate::opyo3;
        use pyo3::prelude::*;
        use pyo3_stub_gen::derive::*;

        #[gen_stub_pyclass]
        #[pyclass(name = #layer_pascal_lit, extends=opyo3::PyLayer)]
        #[derive(Clone)]
        pub struct #py_layer_ident(#layer_struct_ident);

        impl opyo3::PythonLayer for #py_layer_ident {
            fn layer(&self, op: opyo3::ocore::Operator) -> opyo3::ocore::Operator {
                op.layer(self.0.clone())
            }
        }

        #[gen_stub_pymethods]
        #[pymethods]
        impl #py_layer_ident {
            #(#doc_attributes)*
            #[gen_stub(override_return_type(type_repr = "opendal.layers.Layer", imports=("opendal")))]
            #[new]
            #[pyo3(signature = (#(#signature_args),*))]
            #[allow(unused)]
            fn new(#(#new_args),*) -> PyResult<PyClassInitializer<Self>> {
                let mut layer = #layer_struct_ident::default();
                #(#new_assignments)*

                let class = PyClassInitializer::from(opyo3::PyLayer::new()?).add_subclass(Self(layer));
                Ok(class)
            }
        }
    };

    Ok(code.to_string())
}

#[derive(Default, Clone, Debug)]
struct MethodDef {
    name: String,
    arg_name: String,
    arg_type: String,
    docs: Vec<String>,
    is_toggle: bool,
}

#[derive(Default, Clone, Debug)]
struct LayerDef {
    methods: Vec<MethodDef>,
}

fn parse_layer_def(path: &Path, layer_name: &str) -> Result<LayerDef> {
    let content = fs::read_to_string(path)?;
    let ast = parse_file(&content)?;

    let layer_pascal = layer_to_pascal(layer_name);
    let struct_name = format!("{}Layer", layer_pascal);

    let mut methods = Vec::new();

    for item in ast.items {
        if let Item::Impl(impl_block) = item {
            // Check if impl is for the Layer struct
            if let Type::Path(TypePath { path, .. }) = &*impl_block.self_ty {
                if let Some(segment) = path.segments.last() {
                    if segment.ident == struct_name {
                        // Iterate items
                        for impl_item in impl_block.items {
                            if let ImplItem::Fn(method) = impl_item {
                                let vis = method.vis;
                                if matches!(vis, Visibility::Public(_)) {
                                    let sig = method.sig;
                                    // We are looking for builder methods: `pub fn name(mut self, arg: Type) -> Self`
                                    // Check receiver
                                    let has_receiver = sig
                                        .inputs
                                        .iter()
                                        .any(|arg| matches!(arg, syn::FnArg::Receiver(_)));
                                    if !has_receiver {
                                        continue;
                                    }

                                    // Check return type is Self
                                    match &sig.output {
                                        ReturnType::Type(_, ty) => {
                                            if let Type::Path(tp) = &**ty {
                                                if let Some(seg) = tp.path.segments.last() {
                                                    if seg.ident != "Self"
                                                        && seg.ident != struct_name
                                                    {
                                                        continue;
                                                    }
                                                }
                                            }
                                        }
                                        _ => continue,
                                    }

                                    // Check args: should have 1 arg besides self
                                    // (or more, but usually builder is 1)
                                    // Also allow 0 args (toggle-like: with_jitter(mut self) -> Self)
                                    let args: Vec<_> = sig
                                        .inputs
                                        .iter()
                                        .filter_map(|arg| {
                                            if let syn::FnArg::Typed(pat_type) = arg {
                                                Some(pat_type)
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();

                                    if args.len() == 1 {
                                        let arg = args[0];
                                        let arg_name = if let syn::Pat::Ident(pat_ident) = &*arg.pat
                                        {
                                            pat_ident.ident.to_string()
                                        } else {
                                            "arg".to_string()
                                        };

                                        let arg_type = if let Type::Path(tp) = &*arg.ty {
                                            if let Some(seg) = tp.path.segments.last() {
                                                seg.ident.to_string()
                                            } else {
                                                "unknown".to_string()
                                            }
                                        } else {
                                            "unknown".to_string()
                                        };

                                        // Extract docs
                                        let docs: Vec<String> = method
                                            .attrs
                                            .iter()
                                            .filter(|attr| attr.path().is_ident("doc"))
                                            .filter_map(|attr| {
                                                if let Meta::NameValue(meta) = &attr.meta {
                                                    if let Expr::Lit(ExprLit {
                                                        lit: Lit::Str(s),
                                                        ..
                                                    }) = &meta.value
                                                    {
                                                        return Some(s.value().trim().to_string());
                                                    }
                                                }
                                                None
                                            })
                                            .collect();

                                        methods.push(MethodDef {
                                            name: sig.ident.to_string(),
                                            arg_name,
                                            arg_type,
                                            docs,
                                            is_toggle: false,
                                        });
                                    } else if args.is_empty() {
                                        // Handle toggle methods (e.g. with_jitter)
                                        let name = sig.ident.to_string();
                                        if name.starts_with("with_") {
                                            let arg_name =
                                                name.strip_prefix("with_").unwrap().to_string();

                                            // Extract docs
                                            let docs: Vec<String> = method
                                                .attrs
                                                .iter()
                                                .filter(|attr| attr.path().is_ident("doc"))
                                                .filter_map(|attr| {
                                                    if let Meta::NameValue(meta) = &attr.meta {
                                                        if let Expr::Lit(ExprLit {
                                                            lit: Lit::Str(s),
                                                            ..
                                                        }) = &meta.value
                                                        {
                                                            return Some(
                                                                s.value().trim().to_string(),
                                                            );
                                                        }
                                                    }
                                                    None
                                                })
                                                .collect();

                                            methods.push(MethodDef {
                                                name,
                                                arg_name,
                                                arg_type: "bool".to_string(),
                                                docs,
                                                is_toggle: true,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(LayerDef { methods })
}

fn find_dependency_path(package_path: &Path, layer_name: &str) -> Result<PathBuf> {
    let manifest_path = package_path.join("Cargo.toml").canonicalize()?;

    let metadata = MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()?;

    let dep_name = format!("opendal-layer-{}", layer_name);

    let pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == dep_name && p.source.is_some())
        .ok_or_else(|| anyhow!("Could not find dependency package {}", dep_name))?;

    let dep_manifest_path = pkg.manifest_path.clone().into_std_path_buf();
    Ok(dep_manifest_path.parent().unwrap().to_path_buf())
}

fn layer_to_pascal(layer: &str) -> String {
    let mut result = String::with_capacity(layer.len());
    let mut capitalize = true;

    for &b in layer.as_bytes() {
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

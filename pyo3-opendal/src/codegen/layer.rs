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

use crate::codegen::types::get_type_info_from_str;
use crate::codegen::utils::{find_dependency_path, to_pascal};
use anyhow::{Result, anyhow};
use quote::{format_ident, quote};
use std::fs;
use std::path::Path;
use syn::{
    Expr, ExprLit, ImplItem, Item, Lit, Meta, ReturnType, Type, TypePath, Visibility, parse_file,
};

pub fn generate(layer_name: &str, package_path: &Path) -> Result<String> {
    // 1. Find dependency path
    let dep_name = format!("opendal-layer-{}", layer_name);
    let dep_path = find_dependency_path(package_path, &dep_name)?;
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

    let layer_pascal = to_pascal(layer_name);
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

    let methods = layer_def.methods;
    // Separate factory methods (e.g. `new`) from builder methods (e.g. `with_xxx`)
    let (factory_methods, mut builder_methods): (Vec<_>, Vec<_>) =
        methods.into_iter().partition(|m| m.is_factory);

    // Sort builder methods by arg_name to ensure deterministic order
    builder_methods.sort_by(|a, b| a.arg_name.cmp(&b.arg_name));

    // Handle factory method arguments
    // We prioritize `new` if it exists, otherwise assume Default::default()
    let mut factory_call = quote! { let mut layer = #layer_struct_ident::default(); };
    let mut factory_arg_names = std::collections::HashSet::new();

    if let Some(new_method) = factory_methods.iter().find(|m| m.name == "new") {
        let method_name = format_ident!("{}", new_method.name);
        let arg_name = format_ident!("{}", new_method.arg_name);
        let arg_type_str = new_method.arg_type.as_str();

        let type_info = get_type_info_from_str(arg_type_str);

        let py_type = type_info.rust_type;
        let py_type_doc = type_info.py_type_doc;

        // Generate docstring entry
        let doc_desc = process_docs(&new_method.docs, &new_method.name);
        doc_params.push(format!(
            "{} : {}
    {}",
            new_method.arg_name, py_type_doc, doc_desc
        ));

        new_args.push(quote! { #arg_name: #py_type });
        signature_args.push(quote! { #arg_name }); // Required arg, no default value

        factory_call = quote! { let mut layer = #layer_struct_ident::#method_name(#arg_name); };
        factory_arg_names.insert(new_method.arg_name.clone());
    }

    for method in builder_methods {
        // Skip if this arg was already handled by factory method
        if factory_arg_names.contains(&method.arg_name) {
            continue;
        }

        let method_name = format_ident!("{}", method.name);
        let arg_name = format_ident!("{}", method.arg_name);
        let arg_type_str = method.arg_type.as_str();

        // Type mapping and filtering. Only support types that map easily to Python.
        let type_info = get_type_info_from_str(arg_type_str);

        let py_type = type_info.rust_type;
        let default_val = type_info.default_val;
        let is_bool = type_info.is_bool;
        let py_type_doc = type_info.py_type_doc;

        if py_type_doc.is_empty() {
            eprintln!(
                "Skipping method {} due to unsupported type {}",
                method.name, arg_type_str
            );
            continue;
        } // Skipped type

        // Generate docstring entry
        let doc_desc = process_docs(&method.docs, &method.name);

        doc_params.push(format!(
            "{} : {}, optional
    {}",
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
                #factory_call
                #(#new_assignments)*

                let class = PyClassInitializer::from(opyo3::PyLayer::new()?).add_subclass(Self(layer));
                Ok(class)
            }
        }
    };

    Ok(code.to_string())
}

fn process_docs(docs: &[String], method_name: &str) -> String {
    if docs.is_empty() {
        format!("See `{}`.", method_name)
    } else {
        // Join lines, trimming and adding indentation.
        // If a line starts with `#`, treat it as a header and ensure it starts on a new line.
        let mut lines = Vec::new();
        for s in docs.iter() {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
        }
        lines.join("\n    ")
    }
}

#[derive(Default, Clone, Debug)]
struct MethodDef {
    name: String,
    arg_name: String,
    arg_type: String,
    docs: Vec<String>,
    is_toggle: bool,
    is_factory: bool,
}

#[derive(Default, Clone, Debug)]
struct LayerDef {
    methods: Vec<MethodDef>,
}

fn parse_layer_def(path: &Path, layer_name: &str) -> Result<LayerDef> {
    let content = fs::read_to_string(path)?;
    let ast = parse_file(&content)?;

    let layer_pascal = to_pascal(layer_name);
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
                                    let is_new = sig.ident == "new";

                                    // Check receiver
                                    let has_receiver = sig
                                        .inputs
                                        .iter()
                                        .any(|arg| matches!(arg, syn::FnArg::Receiver(_)));

                                    if !has_receiver && !is_new {
                                        // Skip static methods that are not `new`
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

                                    // Extract docs
                                    let docs: Vec<String> = method
                                        .attrs
                                        .iter()
                                        .filter(|attr| attr.path().is_ident("doc"))
                                        .filter_map(|attr| {
                                            if let Meta::NameValue(meta) = &attr.meta {
                                                if let Expr::Lit(ExprLit {
                                                    lit: Lit::Str(s), ..
                                                }) = &meta.value
                                                {
                                                    return Some(s.value().trim().to_string());
                                                }
                                            }
                                            None
                                        })
                                        .collect();

                                    // Check args:
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

                                        methods.push(MethodDef {
                                            name: sig.ident.to_string(),
                                            arg_name,
                                            arg_type,
                                            docs,
                                            is_toggle: false,
                                            is_factory: is_new,
                                        });
                                    } else if args.is_empty() {
                                        if is_new {
                                            // new() with no args -> implies default() basically
                                            // We can track it as a factory but with no args?
                                            // Current logic handles Default::default() as fallback.
                                            continue;
                                        }

                                        // Handle toggle methods (e.g. with_jitter)
                                        let name = sig.ident.to_string();
                                        if name.starts_with("with_") {
                                            let arg_name =
                                                name.strip_prefix("with_").unwrap().to_string();

                                            methods.push(MethodDef {
                                                name,
                                                arg_name,
                                                arg_type: "bool".to_string(),
                                                docs,
                                                is_toggle: true,
                                                is_factory: false,
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

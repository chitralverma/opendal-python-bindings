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

use crate::codegen::parser::ConfigType;
use quote::{format_ident, quote};

pub struct TypeInfo {
    pub rust_type: proc_macro2::TokenStream,
    pub py_type_doc: String,
    pub default_val: proc_macro2::TokenStream,
    pub is_bool: bool,
}

pub fn get_type_info_from_str(type_str: &str) -> TypeInfo {
    match type_str {
        "bool" => TypeInfo {
            rust_type: quote!(bool),
            py_type_doc: "bool".to_string(),
            default_val: quote!(false),
            is_bool: true,
        },
        "String" => TypeInfo {
            rust_type: quote!(String),
            py_type_doc: "str".to_string(),
            default_val: quote!(None),
            is_bool: false,
        },
        "usize" | "u64" | "i64" | "u32" | "u16" | "isize" | "i32" | "i16" | "i8" | "u8" => {
            let t = format_ident!("{}", type_str);
            TypeInfo {
                rust_type: quote!(#t),
                py_type_doc: "int".to_string(),
                default_val: quote!(None),
                is_bool: false,
            }
        }
        "f32" | "f64" => {
            let t = format_ident!("{}", type_str);
            TypeInfo {
                rust_type: quote!(#t),
                py_type_doc: "float".to_string(),
                default_val: quote!(None),
                is_bool: false,
            }
        }
        "Duration" => TypeInfo {
            rust_type: quote!(std::time::Duration),
            py_type_doc: "datetime.timedelta".to_string(),
            default_val: quote!(None),
            is_bool: false,
        },
        _ => TypeInfo {
            rust_type: quote!(),
            py_type_doc: "".to_string(),
            default_val: quote!(),
            is_bool: false,
        },
    }
}

pub fn get_type_info_from_config_type(config_type: ConfigType) -> TypeInfo {
    match config_type {
        ConfigType::Bool => TypeInfo {
            rust_type: quote!(bool),
            py_type_doc: "bool".to_string(),
            default_val: quote!(false),
            is_bool: true,
        },
        ConfigType::String => TypeInfo {
            rust_type: quote!(String),
            py_type_doc: "str".to_string(),
            default_val: quote!(None),
            is_bool: false,
        },
        ConfigType::Duration => TypeInfo {
            // TODO: In the future, we should support converting from datetime.timedelta to String
            // For now, services expect String for duration
            rust_type: quote!(String),
            py_type_doc: "str".to_string(),
            default_val: quote!(None),
            is_bool: false,
        },
        ConfigType::Usize => TypeInfo {
            rust_type: quote!(usize),
            py_type_doc: "int".to_string(),
            default_val: quote!(None),
            is_bool: false,
        },
        ConfigType::U64 => TypeInfo {
            rust_type: quote!(u64),
            py_type_doc: "int".to_string(),
            default_val: quote!(None),
            is_bool: false,
        },
        ConfigType::I64 => TypeInfo {
            rust_type: quote!(i64),
            py_type_doc: "int".to_string(),
            default_val: quote!(None),
            is_bool: false,
        },
        ConfigType::U32 => TypeInfo {
            rust_type: quote!(u32),
            py_type_doc: "int".to_string(),
            default_val: quote!(None),
            is_bool: false,
        },
        ConfigType::U16 => TypeInfo {
            rust_type: quote!(u16),
            py_type_doc: "int".to_string(),
            default_val: quote!(None),
            is_bool: false,
        },
        ConfigType::Vec => TypeInfo {
            rust_type: quote!(Vec<String>),
            py_type_doc: "List[str]".to_string(),
            default_val: quote!(None),
            is_bool: false,
        },
    }
}

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

mod generate;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cmd {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate bindings for a specific service.
    GenerateService {
        /// Service name (e.g. "fs")
        #[arg(short, long)]
        service: String,
        /// Path to the service binding crate (e.g. "services/fs")
        #[arg(short, long)]
        path: String,
    },
}

fn main() -> anyhow::Result<()> {
    logforth::starter_log::stderr().apply();

    match Cmd::parse().command {
        Commands::GenerateService { service, path } => generate::service::run(&service, &path),
    }
}
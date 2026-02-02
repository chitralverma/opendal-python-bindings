# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

# Default recipe to 'help' to display this help screen
default: help

# ==============================================================================
# Configuration & Variables
# ==============================================================================

set ignore-comments := true

workspace_root := `uv workspace dir --preview-features workspace-dir`

# ==============================================================================
# Setup & Maintenance
# ==============================================================================

# Install/refresh all Python dependencies using uv.
[group('maintenance')]
setup:
    @echo "{{ BOLD }}--- Installing/ validating dependencies ---{{ NORMAL }}"
    @uv sync --managed-python --all-packages --all-groups --all-extras --compile-bytecode

# Clean up all caches, build artifacts, and the venv
[group('maintenance')]
clean:
    @echo "{{ BOLD }}--- Cleaning Rust build artifacts (Cargo) ---{{ NORMAL }}"
    @cargo clean --quiet
    @echo "{{ BOLD }}--- Removing build directories, other caches, python bytecode and compiled extensions ---{{ NORMAL }}"
    @find {{ workspace_root }} \
        \( -type d \( -name __pycache__ -o -name .venv -o -name .build -o -name dist -o -name .pytest_cache -o -name .mypy_cache -o -name .hypothesis -o -name .ruff_cache \) -prune -exec rm -rf {} + \) \
        -o \
        \( -type f \( -name '*.py[co]' -o -name '_*.so' \) -delete \)

# ==============================================================================
# Dev & Build
# ==============================================================================

# Generate Python type stubs
[group('dev')]
stub-gen:
    @echo "{{ BOLD }}--- Generating stubs for (opendal) ---{{ NORMAL }}"
    @cargo run --quiet --package opendal-python --bin stub_gen
    @find "{{ workspace_root }}/services" -maxdepth 1 -mindepth 1 -type d \
        -exec bash -c 'SERVICE_DIR="$0"; \
        SERVICE_NAME=$(basename "$SERVICE_DIR"); \
        echo "{{ BOLD }}--- Generating stubs for (opendal-service-${SERVICE_NAME}) ---{{ NORMAL }}"; \
        cargo run --quiet --manifest-path ${SERVICE_DIR}/Cargo.toml --bin stub_gen_"${SERVICE_NAME}";' {} \;
    -@bash -c 'shopt -s globstar; uv run ruff check **/*.pyi --fix --unsafe-fixes --silent || true'
    @just fmt

# Compile and produce release wheel(s) for all members with optimizations
[group('release')]
build-release *args: stub-gen
    @echo "{{ BOLD }}--- Building release wheel (opendal) ---{{ NORMAL }}"
    @uv run maturin build -m {{ workspace_root }}/opendal/Cargo.toml --release --out {{ workspace_root }}/dist {{ args }}
    @find "{{ workspace_root }}/services" "{{ workspace_root }}/layers" -type f -name Cargo.toml \
        -exec bash -c 'MANIFEST_PATH="$0"; \
        CRATE_NAME=$(cargo pkgid --manifest-path "$0" | cut -d "#" -f 2 | cut -d "@" -f 1); \
        echo "{{ BOLD }}--- Building release wheel (${CRATE_NAME}) ---{{ NORMAL }}"; \
        uv run maturin build -m "${MANIFEST_PATH}" --release --out "{{ workspace_root }}/dist" {{ args }};' {} \;

# Build and install release wheel(s) for all members in the current venv
[group('release')]
install-release *args: stub-gen
    @echo "{{ BOLD }}--- Installing release wheel (opendal) ---{{ NORMAL }}"
    @uv run maturin develop --uv -m {{ workspace_root }}/opendal/Cargo.toml --release {{ args }}
    @find "{{ workspace_root }}/services" "{{ workspace_root }}/layers" -type f -name Cargo.toml \
        -exec bash -c 'MANIFEST_PATH="$0"; \
        CRATE_NAME=$(cargo pkgid --manifest-path "$0" | cut -d "#" -f 2 | cut -d "@" -f 1); \
        echo "{{ BOLD }}--- Installing release wheel (${CRATE_NAME}) ---{{ NORMAL }}"; \
        uv run maturin develop --uv -m "${MANIFEST_PATH}" --release {{ args }};' {} \;

# Run benchmarks with release wheels
[group('release')]
bench: install-release
    @echo "{{ BOLD }}--- Bench release ---{{ NORMAL }}"
    @find {{ workspace_root }}/opendal/benchmark -type f -name '*.py' -exec uv run '{}' \;

# Build only a source distribution(s) without compiling for all members
[group('release')]
sdist *args: stub-gen
    @echo "{{ BOLD }}--- Building source distribution without compiling (opendal) ---{{ NORMAL }}"
    @uv run maturin sdist -m {{ workspace_root }}/opendal/Cargo.toml --out {{ workspace_root }}/dist {{ args }}
    @find "{{ workspace_root }}/services" "{{ workspace_root }}/layers" -type f -name Cargo.toml \
        -exec bash -c 'MANIFEST_PATH="$0"; \
        CRATE_NAME=$(cargo pkgid --manifest-path "$0" | cut -d "#" -f 2 | cut -d "@" -f 1); \
        echo "{{ BOLD }}--- Building source distribution without compiling (${CRATE_NAME}) ---{{ NORMAL }}"; \
        uv run maturin sdist -m "${MANIFEST_PATH}" --out "{{ workspace_root }}/dist" {{ args }};' {} \;

# Compile and produce development wheel(s) for all members
[group('dev')]
build-dev *args: stub-gen
    @echo "{{ BOLD }}--- Building development wheel (opendal) ---{{ NORMAL }}"
    @uv run maturin build -m {{ workspace_root }}/opendal/Cargo.toml --out {{ workspace_root }}/dist {{ args }}
    @find "{{ workspace_root }}/services" "{{ workspace_root }}/layers" -type f -name Cargo.toml \
        -exec bash -c 'MANIFEST_PATH="$0"; \
        CRATE_NAME=$(cargo pkgid --manifest-path "$0" | cut -d "#" -f 2 | cut -d "@" -f 1); \
        echo "{{ BOLD }}--- Building development wheel (${CRATE_NAME}) ---{{ NORMAL }}"; \
        uv run maturin build -m "${MANIFEST_PATH}" --out "{{ workspace_root }}/dist" {{ args }};' {} \;

# Build and install development wheel(s) for all members in the current venv
[group('dev')]
install-dev *args: stub-gen
    @echo "{{ BOLD }}--- Installing development wheel (opendal) ---{{ NORMAL }}"
    @uv run maturin develop --uv -m {{ workspace_root }}/opendal/Cargo.toml {{ args }}
    @find "{{ workspace_root }}/services" "{{ workspace_root }}/layers" -type f -name Cargo.toml \
        -exec bash -c 'MANIFEST_PATH="$0"; \
        CRATE_NAME=$(cargo pkgid --manifest-path "$0" | cut -d "#" -f 2 | cut -d "@" -f 1); \
        echo "{{ BOLD }}--- Installing development wheel (${CRATE_NAME}) ---{{ NORMAL }}"; \
        uv run maturin develop --uv -m "${MANIFEST_PATH}" {{ args }};' {} \;

# Run tests
[group('dev')]
[working-directory('opendal')]
test *args:
    @just install-dev
    @echo "{{ BOLD }}--- Running tests ---{{ NORMAL }}"
    @uv run pytest -v {{ args }}

# Build mkdocs for all members
[group('dev')]
build-docs *args: stub-gen
    # TODO: fix this later
    @uv run mkdocs build {{ args }}

# Generate a new opendal service package from template
[group('dev')]
generate-service service_name: setup
    @echo "{{ BOLD }}--- Generating service package (opendal-service-{{ replace(service_name, '_', '-') }}) ---{{ NORMAL }}"
    @uv run copier copy --data service_name={{ service_name }} {{ workspace_root }}/templates/service {{ workspace_root }}/services/{{ service_name }}
    @uv run python {{ workspace_root }}/scripts/add_service_to_opendal.py {{ replace(service_name, '_', '-') }} {{ workspace_root }}
    @cargo run --quiet --manifest-path {{ workspace_root }}/services/{{ service_name }}/Cargo.toml --bin stub_gen_{{ replace(service_name, '-', '_') }}
    -@bash -c 'shopt -s globstar; uv run ruff check **/*.pyi --fix --unsafe-fixes --silent || true'
    @just fmt
    @echo "{{ BOLD }}--- Service package (opendal-service-{{ replace(service_name, '_', '-') }}) is ready ---{{ NORMAL }}"

# ==============================================================================
# Code Quality & Formatting
# ==============================================================================

# Run all lint checks for Rust and Python
[group('lint')]
lint: setup
    @echo "{{ BOLD }}--- Running Rust linter (Clippy) ---{{ NORMAL }}"
    ## remove `-A clippy::incompatible_msrv` until https://github.com/rust-lang/rust-clippy/issues/15792 is fixed and released
    @cargo clippy -- -D warnings -D clippy::dbg_macro -A clippy::incompatible_msrv
    @echo "{{ BOLD }}--- Running Python linter (Ruff) ---{{ NORMAL }}"
    @uv run ruff check

# Format all code (Rust, Python, etc.)
[group('lint')]
fmt: setup
    @echo "{{ BOLD }}--- Formatting Rust ---{{ NORMAL }}"
    @cargo fmt --all
    @echo "{{ BOLD }}--- Formatting Python ---{{ NORMAL }}"
    @uv run ruff format --quiet
    @echo "{{ BOLD }}--- Checking license headers (Hawkeye) ---{{ NORMAL }}"
    @hawkeye format --config {{ workspace_root }}/licenserc.toml --fail-if-updated false
    @echo "{{ BOLD }}--- Formatting Misc ---{{ NORMAL }}"
    @taplo fmt --config {{ workspace_root }}/.taplo.toml
    @just --fmt --unstable

# Run all code formatting and quality checks
[group('lint')]
pre-commit: fmt lint
    @echo "{{ BOLD }}--- Checking license headers (Hawkeye) ---{{ NORMAL }}"
    @hawkeye check --config {{ workspace_root }}/licenserc.toml
    @echo "{{ BOLD }}--- Running Misc checks ---{{ NORMAL }}"
    @taplo lint --config {{ workspace_root }}/.taplo.toml
    @just --fmt --unstable --check

# Display this help screen
help:
    @just --list

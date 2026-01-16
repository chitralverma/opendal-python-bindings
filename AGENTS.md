# AGENTS.md

This file contains guidelines and commands for agentic coding assistants working on the OpenDAL Python bindings project.

## Project Overview

This is a **Python bindings project for Apache OpenDAL** using PyO3 for Rust-Python integration. The project features:
- **Modular architecture** mirroring OpenDAL's split into separate service/layer crates
- Async-first design with sync wrappers
- Type stubs for all Python modules
- Comprehensive testing with capability-based filtering

### Architecture Context

Following OpenDAL's recent modularization where services and layers were split into separate crates (s3, concurrent_layer, etc.), this Python bindings project aims to achieve the same modularity. This is the first manual attempt to create equivalent modular Python bindings.

**End Goal**: Make OpenDAL Python bindings fully modular so that services like `opendal-service-fs` become optional dependencies (extras) of the main `opendal` package. Users should be able to use `uv install opendal[service-fs]` to get only the implementations they need.

### Current Architecture & Progress

- **Mixed workspace**: Uses both UV and Cargo workspaces coordinated together
- **UV-first builds**: All code built via UV using Maturin backend, avoiding double Rust compilation
- **Separate packages**: Each service and layer is its own Python package
- **Optional dependencies**: Services are configured as optional extras of the main `opendal` package
- **Implemented services**: Currently only `fs` and `s3` services are complete
- **Working operators**: Refactored operators work independently with the modular architecture

### Workspace Structure

The project uses a coordinated workspace approach:
- **UV workspace**: Manages Python dependencies and package relationships
- **Cargo workspace**: Manages Rust dependencies and compilation
- **Maturin integration**: Bridges UV and Cargo for Python-Rust extension building
- **Service packages**: Located in `services/` (e.g., `services/fs/`, `services/s3/`)
- **Layer packages**: Located in `layers/` for operator layers

## Build & Development Commands

### Primary Commands (use Just task runner)

```bash
# Setup and install dependencies
just setup                    # Install all Python dependencies via UV

# Building
just build-dev               # Build development wheels
just build-release           # Build optimized release wheels
just install-dev             # Build and install development wheels
just install-release         # Build and install release wheels

# Testing
just test                     # Run all tests
just test path/to/test.py    # Run single test file
just test -k "test_name"     # Run specific test by name
just test -v                  # Verbose test output

# Code Quality
just lint                     # Run all linters (Clippy + Ruff)
just fmt                      # Format all code (Rust + Python + TOML)
just pre-commit               # Run formatting + linting + checks

# Type Stubs
just stub-gen                 # Generate Python type stubs from Rust

# Cleanup
just clean                    # Remove all build artifacts and caches
```

### Alternative Commands (without Just)

```bash
# Python-specific commands
uv run pytest -v             # Run tests directly
uv run ruff check             # Python linting
uv run ruff format            # Python formatting
cargo clippy                  # Rust linting
cargo fmt                     # Rust formatting
```

## Testing Guidelines

### Test Structure
- Tests located in `opendal/tests/`
- Separate sync and async test files
- Session-scoped fixtures for efficiency
- Environment-based configuration via `.env`

### Running Single Tests
```bash
# Run specific test file
just test tests/test_async_operator.py

# Run specific test method
just test -k "test_write"

# Run with verbose output
just test -v tests/test_async_operator.py::test_write
```

### Test Configuration
- Uses `pytest-asyncio` for async test support
- Custom fixtures in `conftest.py`:
  - `service_name` - from OPENDAL_TEST env var
  - `setup_config` - service configuration from environment
  - `async_operator` / `operator` - test operators with layers
- Custom marker: `@pytest.mark.need_capability()` for capability-based test filtering

### Environment Setup
Tests require environment variables for service configuration:
```bash
export OPENDAL_TEST=fs
export opendal_fs_root=/tmp/test
```

## Code Style Guidelines

### Python Code Style

#### Formatting & Linting
- **Line length**: 88 characters (strict)
- **Docstring style**: NumPy convention
- **Type annotations**: Required (strict checking)
- **Import style**: No relative imports, use `isort` rules
- **Tool**: Ruff for both linting and formatting

#### File Structure
```python
# Licensed to the Apache Software Foundation (ASF) under one
# ... (full Apache license header) ...

"""Module docstring following NumPy style."""

import typing
from third_party import module
from project.module import specific_import

# Code follows PEP 8 with 88-char line limit
```

#### Import Conventions
- Ban relative imports (`from .module import` not allowed)
- Known first-party: `["opendal"]`
- Use `typing` for type annotations
- Import specific functions/classes when possible

#### Type Annotations
- Required for all function signatures
- Use `typing` module for generic types
- Use `@typing.final` for classes that shouldn't be subclassed
- Use `# ruff: noqa: ANN003` for **kwargs when necessary

#### Docstrings
- NumPy style convention
- Include parameter types, returns, and examples
- Use `r"""raw"""` for docstrings with backticks

#### Error Handling
- Use custom exceptions from `opendal.exceptions`
- Follow existing error patterns in the codebase
- Include proper error messages with context

### Rust Code Style

#### Standards
- **Edition**: 2024
- **Formatting**: Standard `rustfmt`
- **Linting**: Clippy with strict warnings
- **License**: Apache 2.0 headers required

#### PyO3 Integration
- Use `#[pyclass]` for Python-exposed classes
- Use `#[pymethods]` for Python-exposed methods
- Handle Python exceptions properly
- Follow async/await patterns with tokio runtime

## Project Structure

```
opendal-python-bindings/
├── opendal/                    # Main Python package
│   ├── src/                    # Rust PyO3 bindings layer
│   │   └── opendal/            # Python source code
│   ├── tests/                  # Test files
│   ├── Cargo.toml              # Rust configuration
│   └── pyproject.toml          # Python configuration
├── services/                   # Modular service packages
│   ├── fs/                     # POSIX Filesystem service
│   └── s3/                     # S3 service
├── layers/                     # Operator layers
│   └── retry/                  # Retry layer
├── justfile                    # Task runner recipes
├── ruff.toml                   # Python linting configuration
└── licenserc.toml              # License header configuration
```

## Development Workflow

1. **Setup**: Run `just setup` to install dependencies
2. **Make Changes**: Edit Python or Rust code
3. **Generate Stubs**: Run `just stub-gen` after Rust changes
4. **Format**: Run `just fmt` to format all code
5. **Lint**: Run `just lint` to check code quality
6. **Test**: Run `just test` to verify functionality
7. **Pre-commit**: Run `just pre-commit` before committing

## Important Notes

- **License Headers**: All files must include Apache 2.0 license header
- **Type Stubs**: Must be regenerated after Rust changes with `just stub-gen`
- **Modular Services**: Each service (fs, s3, etc.) is a separate package
- **Async-First**: Sync APIs use async runtime internally
- **Workspace**: Uses UV workspace for dependency management
- **No Relative Imports**: Enforced by Ruff configuration

## Modular Architecture Guidelines

### Service Package Structure
Each service (fs, s3, etc.) must:
- Be a separate Python package in `services/` directory
- Have its own `Cargo.toml` and `pyproject.toml`
- Be listed as optional extra in main `opendal/pyproject.toml`
- Export service-specific operator classes
- Follow naming convention: `opendal-service-{service}`

### Build System Integration
- **UV-first**: Always build through UV, not directly with Cargo
- **Maturin backend**: UV uses Maturin for Python-Rust extension building
- **Single compilation**: Avoid double Rust compilation through coordinated workspaces
- **Workspace coordination**: UV and Cargo workspaces must be kept in sync

### Package Dependencies
- Services and layers depend on `opendal` for base pyo3 bindings
- Main `opendal` package lists services and layers as optional dependencies
- Users install with extras: `uv install opendal[fs-service,s3-service,retry-layer]`
- Layer packages follow same pattern as services

### Current Implementation Status
- **Complete**: `fs` and `s3` services with working operators
- **In Progress**: Layer packages structure
- **TODO**: Remaining services (gcs, azblob, etc.)
- **Working**: Refactored operators that dynamically load service implementations

## Testing Services

The project supports multiple storage services. Test configuration is environment-based:
- Set `OPENDAL_TEST` to specify service (fs, s3, etc.)
- Configure service-specific environment variables (e.g., `opendal_s3_region`)
- Use `OPENDAL_DISABLE_RANDOM_ROOT=true` to disable random test directories

## Common Issues & Solutions

- **Import Errors**: Ensure type stubs are generated with `just stub-gen`
- **Build Failures**: Run `just clean` then `just setup`
- **Test Skips**: Check environment variables for service configuration
- **Linting Failures**: Run `just fmt` to auto-fix most formatting issues

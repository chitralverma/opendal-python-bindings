# Architecture Proposal: Path 1 - Single Extension Module

## Overview

This approach consolidates all PyO3 type definitions into a single extension module (`opendal._core`), eliminating the duplicate type registration issue that currently prevents layer application across modules.

## Current Problem

Each service module (`opendal_fs_service`, `opendal_s3_service`, etc.) statically links `opendal-pyo3`, creating duplicate Python type registrations:
- `FsPyAsyncOperator` extends a **different** `PyAsyncOperator` instance than the one in `opendal.operator`
- Layers from `opendal.layers` cannot be applied to service-specific operators
- Type identity checks fail: `isinstance(layer, Layer)` returns `True`, but PyO3 sees them as incompatible types

## Solution Design

### 1. Single Extension Module (`opendal._core`)

**Keep all PyO3 classes in one place:**
```
opendal/
├── Cargo.toml           # Single Python extension module
├── src/
│   ├── lib.rs           # Registers ALL PyO3 types
│   ├── opendal/
│   │   ├── __init__.py  # Pure Python exports
│   │   ├── operator.py  # Service-specific Python wrappers
│   │   └── services/
│   │       ├── __init__.py
│   │       ├── fs.py    # Pure Python wrapper for FS
│   │       └── s3.py    # Pure Python wrapper for S3
```

### 2. Service Modules Become Pure Python

**services/fs/pyproject.toml:**
```toml
[project]
name = "opendal-fs-service"
dependencies = ["opendal>=0.47.0"]

# NO Rust/PyO3 build - pure Python package
```

**services/fs/src/opendal_fs_service/__init__.py:**
```python
"""FS service implementation for OpenDAL."""
from opendal.operator import PyAsyncOperator, PyOperator
from opendal_service_fs import FS_SCHEME  # Import from Rust crate if needed

class FsAsyncOperator(PyAsyncOperator):
    """Async operator for FS service."""
    
    def __init__(self, **kwargs):
        # Call parent constructor with FS scheme
        super().__init__(FS_SCHEME, **kwargs)

class FsOperator(PyOperator):
    """Blocking operator for FS service."""
    
    def __init__(self, **kwargs):
        super().__init__(FS_SCHEME, **kwargs)

# For backwards compatibility
FsPyAsyncOperator = FsAsyncOperator
FsPyOperator = FsOperator
```

### 3. Main Module Structure

**opendal/src/lib.rs:**
```rust
// Register ALL types in one module
#[pymodule(gil_used = false)]
fn _core(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Operators
    add_pymodule!(py, m, "opendal", "operator", [PyOperator, PyAsyncOperator])?;
    
    // Layers (same as before)
    add_pymodule!(py, m, "opendal", "layers", 
        [Layer, RetryLayer, ConcurrentLimitLayer, MimeGuessLayer])?;
    
    // Other types...
    Ok(())
}
```

**opendal/src/opendal/operator.py:**
```python
"""Service-specific operator constructors."""
from opendal.operator import PyAsyncOperator, PyOperator

def create_fs_operator(**kwargs) -> PyOperator:
    """Create a blocking FS operator."""
    return PyOperator("fs", **kwargs)

def create_fs_async_operator(**kwargs) -> PyAsyncOperator:
    """Create an async FS operator."""
    return PyAsyncOperator("fs", **kwargs)

# Similarly for S3, etc.
```

### 4. Installation & Dependencies

**User installs opendal with extras:**
```bash
# Just base (memory backend only)
uv pip install opendal

# With FS support
uv pip install opendal[fs]

# With S3 support  
uv pip install opendal[s3]

# With multiple services
uv pip install opendal[fs,s3]
```

**pyproject.toml:**
```toml
[project]
name = "opendal"
dependencies = []

[project.optional-dependencies]
fs = ["opendal-fs-service>=0.47.0"]
s3 = ["opendal-s3-service>=0.47.0"]
```

### 5. Rust Service Crates

Service crates remain as pure Rust libraries:
```
opendal-service-fs/      # Rust-only crate
├── Cargo.toml
└── src/
    └── lib.rs           # Just Rust code, no PyO3

opendal-service-s3/      # Rust-only crate  
├── Cargo.toml
└── src/
    └── lib.rs
```

The main `opendal` crate links these Rust libraries and exposes them via PyO3.

## Implementation Steps

### Phase 1: Consolidate Types
1. Move all operator/layer type definitions to `opendal` crate
2. Remove PyO3 code from service packages
3. Update `opendal/src/lib.rs` to register all types

### Phase 2: Convert Services to Pure Python
1. Convert `services/fs/` to pure Python package
2. Convert `services/s3/` to pure Python package
3. Update import paths

### Phase 3: Update Build System
1. Modify `justfile` to only build main `opendal` extension
2. Update service package configs (remove maturin)
3. Test installation with extras

### Phase 4: Testing & Migration
1. Update tests to use new import paths
2. Add migration guide for users
3. Verify layer application works

## Benefits

✅ **Fixes layer issue**: Single type registry, no duplicate types
✅ **Simpler architecture**: One extension module, easier to maintain
✅ **Faster builds**: No need to recompile Rust for each service
✅ **Better Python integration**: Services are just Python classes
✅ **Clearer separation**: Rust does core functionality, Python does service wiring

## Drawbacks

⚠️ **Major refactor**: Requires significant code restructuring
⚠️ **Breaking change**: Service import paths change
⚠️ **Single build**: Can't distribute services independently (but extras solve this)

## Migration Guide for Users

**Before:**
```python
import opendal
from opendal_fs_service import FsPyAsyncOperator

op = FsPyAsyncOperator(root="/tmp")
```

**After:**
```python
import opendal

# Option 1: Direct construction
op = opendal.AsyncOperator("fs", root="/tmp")

# Option 2: Explicit import (if we keep wrappers)
from opendal.services.fs import FsAsyncOperator
op = FsAsyncOperator(root="/tmp")
```

## File Changes Summary

```
DELETE: services/fs/Cargo.toml (Rust build config)
DELETE: services/fs/src/lib.rs
DELETE: services/fs/src/fs.rs
CREATE: services/fs/pyproject.toml (Python-only)
CREATE: services/fs/src/opendal_fs_service/__init__.py

MODIFY: opendal/src/lib.rs (register all types)
CREATE: opendal/src/opendal/services/ (Python wrappers)
MODIFY: justfile (remove service Rust builds)
```

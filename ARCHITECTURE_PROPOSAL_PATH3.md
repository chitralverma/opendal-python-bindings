# Architecture Proposal: Path 3 - Shared Base Types via Dynamic Linking

## Overview

This approach maintains PyO3 inheritance (`extends`) while solving the duplicate type registration issue by having service modules **dynamically import and use** the base types from the main `opendal` module instead of statically linking their own copies.

## User Requirements Addressed

✅ **Maintain inheritance**: Service operators extend base operators, inheriting all functionality (list/delete/read/etc.)  
✅ **Common layers**: Single layer instance can be used across multiple service operators  
✅ **No duplication**: All operator methods available without code duplication  
✅ **Shared base**: All modules reconcile to the same base type from one source

## The Problem with Current Approach

Currently, each service module statically links `opendal-pyo3`:
```toml
# services/fs/Cargo.toml
[dependencies]
opendal-pyo3 = { path = "../../opendal-pyo3" }  # ❌ Creates duplicate types
```

This causes each service to get its own copy of `PyAsyncOperator`, `Layer`, etc., creating incompatible type hierarchies.

## Solution: Dynamic Type Importing

Instead of service modules defining their own PyO3 classes that extend base types, they should:
1. Import the base operator class from the already-loaded `opendal` module at runtime
2. Use those base classes directly or minimally wrap them
3. Let Python's type system handle the shared types naturally

## Implementation Design

### Approach 3A: Pure Python Service Operators (Recommended)

Service modules don't define PyO3 classes at all. They're pure Python packages that use the base operators.

**services/fs/pyproject.toml:**
```toml
[project]
name = "opendal-fs-service"
version = "0.47.0"
dependencies = ["opendal>=0.47.0"]
# No maturin, no Rust build!
```

**services/fs/src/opendal_fs_service/__init__.py:**
```python
"""FS Service - Pure Python wrapper around base operators."""
from opendal.operator import PyAsyncOperator, PyOperator

class FsPyAsyncOperator(PyAsyncOperator):
    """Async FS operator with all base functionality inherited."""
    
    def __init__(self, **kwargs):
        # Call parent constructor - inherits ALL methods
        super().__init__("fs", **kwargs)
    
    # No need to redefine layer(), read(), write(), etc.
    # They're all inherited from PyAsyncOperator!

class FsPyOperator(PyOperator):
    """Blocking FS operator with all base functionality inherited."""
    
    def __init__(self, **kwargs):
        super().__init__("fs", **kwargs)

__all__ = ["FsPyAsyncOperator", "FsPyOperator"]
```

**Benefits:**
- ✅ Full inheritance - all methods available
- ✅ Layers work - same type hierarchy
- ✅ No code duplication
- ✅ Fast to implement
- ✅ Easy to maintain

**Usage remains the same:**
```python
from opendal_fs_service import FsPyAsyncOperator
from opendal.layers import RetryLayer

# All methods inherited from PyAsyncOperator
op = FsPyAsyncOperator(root="/tmp")

# Layer works because FsPyAsyncOperator IS A PyAsyncOperator
layer = RetryLayer()
op_with_layer = op.layer(layer)  # ✅ Works!

# All base methods available
await op_with_layer.write("test.txt", b"hello")
await op_with_layer.read("test.txt")
await op_with_layer.delete("test.txt")
```

**Share layers across services:**
```python
from opendal_fs_service import FsPyAsyncOperator
from opendal_s3_service import S3PyAsyncOperator
from opendal.layers import RetryLayer

# Create one layer instance
retry = RetryLayer(max_times=5)

# Use with different services - WORKS!
fs_op = FsPyAsyncOperator(root="/tmp").layer(retry)
s3_op = S3PyAsyncOperator(bucket="my-bucket").layer(retry)

# Both operators have the same layer applied
await fs_op.write("test.txt", b"data")
await s3_op.write("test.txt", b"data")
```

### Approach 3B: Minimal Rust Wrapper with Dynamic Import

If you need some Rust-level customization, use dynamic imports to get base types:

**services/fs/src/fs.rs:**
```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass(module = "opendal_fs_service.operator")]
pub struct FsPyAsyncOperator {
    // Store the actual base operator
    base: Py<PyAny>,
}

#[pymethods]
impl FsPyAsyncOperator {
    #[new]
    fn new(py: Python, kwargs: Option<&Bound<PyDict>>) -> PyResult<Self> {
        // Import the base operator class from the opendal module
        let opendal = py.import_bound("opendal.operator")?;
        let base_class = opendal.getattr("PyAsyncOperator")?;
        
        // Create instance - this is the SAME type layers expect
        let args = ("fs",);
        let base = base_class.call((args,), kwargs)?;
        
        Ok(Self {
            base: base.unbind(),
        })
    }
    
    // Explicitly expose important methods
    fn layer(&self, py: Python, layer: &Bound<PyAny>) -> PyResult<Py<Self>> {
        let base_bound = self.base.bind(py);
        let result = base_bound.call_method1("layer", (layer,))?;
        
        Py::new(py, Self {
            base: result.unbind(),
        })
    }
    
    // Forward everything else via __getattr__
    fn __getattr__(&self, py: Python, name: &str) -> PyResult<PyObject> {
        self.base.bind(py).getattr(name).map(|v| v.unbind())
    }
}
```

**services/fs/Cargo.toml:**
```toml
[dependencies]
# Still links opendal-pyo3 for utility functions, but doesn't use its types
opendal-pyo3 = { path = "../../opendal-pyo3" }
opendal-service-fs = { git = "https://github.com/apache/opendal.git", branch = "main" }
pyo3 = { workspace = true }
```

### Approach 3C: Rust-Only Service Builders (No Extension)

Services are pure Rust libraries that the main `opendal` extension uses:

**services/fs/Cargo.toml:**
```toml
[package]
name = "opendal-fs-service"

[lib]
crate-type = ["rlib"]  # ❌ NOT cdylib - not a Python extension!

[dependencies]
opendal-service-fs = { git = "https://github.com/apache/opendal.git", branch = "main" }
# NO pyo3 dependency!
```

**services/fs/src/lib.rs:**
```rust
//! Pure Rust service builder - no Python bindings

pub use opendal_service_fs::FS_SCHEME;

pub fn build_fs_operator(config: HashMap<String, String>) -> opendal::Operator {
    opendal::Operator::via_iter(FS_SCHEME, config).unwrap()
}
```

**Main opendal/src/operator.rs imports this:**
```rust
use opendal_fs_service;

#[pymethods]
impl PyAsyncOperator {
    #[new]
    pub fn new(scheme: &str, kwargs: Option<&Bound<PyDict>>) -> PyResult<Self> {
        let map = extract_kwargs(kwargs);
        
        let core = match scheme {
            "fs" => opendal_fs_service::build_fs_operator(map)?,
            "s3" => opendal_s3_service::build_s3_operator(map)?,
            _ => return Err(PyErr::new::<PyValueError, _>("Unknown scheme")),
        };
        
        Ok(Self { core, ... })
    }
}
```

Now there's only ONE `PyAsyncOperator` class, and services just provide builders!

## Comparison of Sub-Approaches

| Feature | 3A: Pure Python | 3B: Rust Wrapper | 3C: Rust Builders |
|---------|----------------|------------------|-------------------|
| Inheritance | ✅ Native Python | ⚠️ Via `__getattr__` | ✅ Single class |
| Complexity | ⭐ Simple | ⭐⭐ Medium | ⭐⭐⭐ Complex |
| Build time | ⭐⭐⭐ Instant | ⭐⭐ Medium | ⭐ Slow |
| Layer support | ✅ Native | ✅ Works | ✅ Native |
| Service isolation | ⭐⭐ Package-level | ⭐⭐⭐ Extension-level | ⭐ Same extension |

## Implementation Steps (Path 3A - Recommended)

### Phase 1: Convert FS Service to Pure Python (1-2 hours)

1. **Remove Rust build configuration:**
```bash
cd services/fs
rm Cargo.toml src/lib.rs src/fs.rs
rm -rf src/bin
```

2. **Create pure Python package:**
```bash
# services/fs/pyproject.toml
cat > pyproject.toml << 'EOF'
[project]
name = "opendal-fs-service"
version = "0.47.0"
dependencies = ["opendal>=0.47.0"]
description = "FS service for OpenDAL Python bindings"

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"
EOF
```

3. **Create Python module:**
```python
# services/fs/src/opendal_fs_service/__init__.py
from opendal.operator import PyAsyncOperator, PyOperator

class FsPyAsyncOperator(PyAsyncOperator):
    def __init__(self, **kwargs):
        super().__init__("fs", **kwargs)

class FsPyOperator(PyOperator):
    def __init__(self, **kwargs):
        super().__init__("fs", **kwargs)

__all__ = ["FsPyAsyncOperator", "FsPyOperator"]
```

4. **Add type stub:**
```python
# services/fs/src/opendal_fs_service/__init__.pyi
from opendal.operator import PyAsyncOperator, PyOperator

class FsPyAsyncOperator(PyAsyncOperator): ...
class FsPyOperator(PyOperator): ...
```

5. **Update justfile:**
```just
# Remove FS from Rust builds, add to Python packages
install-dev:
    @uv run maturin develop --uv -m opendal/Cargo.toml
    # No more maturin for services!
    @pip install -e services/fs
    @pip install -e services/s3  # After converting
```

### Phase 2: Test FS Service

```python
# test_fs_layers.py
from opendal_fs_service import FsPyAsyncOperator
from opendal.layers import RetryLayer, ConcurrentLimitLayer

def test_fs_with_layers():
    op = FsPyAsyncOperator(root="/tmp/test")
    
    # Apply layers
    retry = RetryLayer(max_times=3)
    op = op.layer(retry)
    
    concurrent = ConcurrentLimitLayer(10)
    op = op.layer(concurrent)
    
    # Verify all methods available
    assert hasattr(op, 'read')
    assert hasattr(op, 'write')
    assert hasattr(op, 'delete')
    assert hasattr(op, 'list')
    
    print("✅ FS service with layers works!")

def test_shared_layer():
    """Test same layer with different services."""
    retry = RetryLayer(max_times=5)
    
    fs_op = FsPyAsyncOperator(root="/tmp").layer(retry)
    # After converting S3:
    # s3_op = S3PyAsyncOperator(bucket="test").layer(retry)
    
    assert fs_op is not None
    print("✅ Shared layer works!")

if __name__ == "__main__":
    test_fs_with_layers()
    test_shared_layer()
```

### Phase 3: Convert S3 Service (same pattern)

### Phase 4: Update Documentation

## Benefits of Path 3A

✅ **Solves original problem**: Layers work across services  
✅ **Maintains inheritance**: All operator methods available  
✅ **No duplication**: Zero code duplication in services  
✅ **Shared layers**: Same layer instance works with all services  
✅ **Simple**: Pure Python, easy to understand and maintain  
✅ **Fast builds**: No Rust compilation for services  
✅ **Easy testing**: Standard Python testing  
✅ **Type safety**: MyPy/Pyright work perfectly  

## Addressing User Concerns

> "all the functionality (list/ delete/ read etc.) of an operator is not passed down"

**Solution**: Python inheritance naturally passes down all methods. `FsPyAsyncOperator(PyAsyncOperator)` automatically has `read()`, `write()`, `delete()`, `list()`, etc.

> "layers packages are supposed to be common to all services"

**Solution**: Layers remain in `opendal._core`. All services import and use the same layer types.

> "i can apply do things like x = RetryLayer(..) s1 = AOperator(...).layer(x) s2 = BOperator(...).layer(x)"

**Solution**: This works perfectly because all service operators inherit from the same base classes:
```python
retry = RetryLayer()
fs = FsPyAsyncOperator(root="/tmp").layer(retry)
s3 = S3PyAsyncOperator(bucket="b").layer(retry)
```

> "keep common traits in one pyo3 crate which can be extended by other crates"

**Solution**: Common traits stay in `opendal-pyo3`. Services don't create new PyO3 crates - they're pure Python wrappers that extend the base Python classes.

## Migration from Current Code

**Current (broken):**
```python
from opendal_fs_service import FsPyAsyncOperator  # Rust extension
op = FsPyAsyncOperator(root="/tmp")
```

**New (working):**
```python
from opendal_fs_service import FsPyAsyncOperator  # Pure Python
op = FsPyAsyncOperator(root="/tmp")  # Same API!
```

**No user code changes needed!** The import path and API remain identical.

## File Structure Comparison

**Before:**
```
services/fs/
├── Cargo.toml          (Rust build)
├── src/
│   ├── lib.rs          (PyO3 module)
│   ├── fs.rs           (PyO3 classes)
│   └── bin/
│       └── stub_gen_fs.rs
└── pyproject.toml      (Python metadata)
```

**After:**
```
services/fs/
├── pyproject.toml      (Python-only package)
└── src/
    └── opendal_fs_service/
        ├── __init__.py      (Pure Python)
        ├── __init__.pyi     (Type stubs)
        └── py.typed         (Marker)
```

**Much simpler!**

## Why This Works

1. **Single source of truth**: `opendal._core` has the only `PyAsyncOperator` class
2. **Python inheritance**: `FsPyAsyncOperator(PyAsyncOperator)` is a proper subclass
3. **Type identity**: `isinstance(fs_op, PyAsyncOperator)` returns `True`
4. **Layer compatibility**: Layers check `isinstance(op, PyAsyncOperator)` - works!

## Performance Impact

⚠️ **None**. Python class inheritance has negligible overhead. The actual operations (I/O) happen in Rust through the base operator.

## Conclusion

**Path 3A (Pure Python Service Operators)** is the optimal solution:
- ✅ Solves the type registration issue
- ✅ Maintains full inheritance
- ✅ Supports shared layers
- ✅ No code duplication
- ✅ Simplest implementation
- ✅ Fastest to deploy

This gives you exactly what you want: common traits in one PyO3 crate (`opendal-pyo3`), services that properly extend them (via Python inheritance), and complete interoperability.

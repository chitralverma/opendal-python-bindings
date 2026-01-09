# Architecture Proposal: Path 2 - Composition Over Inheritance

## Overview

This approach eliminates PyO3 inheritance (`extends`) for service-specific operators, using composition and delegation instead. Service operators wrap a base operator and forward method calls.

## Current Problem

Service operators use PyO3's `extends` to inherit from `PyAsyncOperator`:
```rust
#[pyclass(module = "opendal_fs_service.operator", extends=opyo3::PyAsyncOperator)]
pub struct FsPyAsyncOperator { ... }
```

This creates type incompatibility because each service module gets its own copy of the `PyAsyncOperator` type.

## Solution Design

### 1. Remove PyO3 Inheritance

**Before (current):**
```rust
#[pyclass(extends=PyAsyncOperator)]
pub struct FsPyAsyncOperator {
    core: Operator,
    __scheme: String,
    __map: HashMap<String, String>,
}
```

**After (composition):**
```rust
#[pyclass(module = "opendal_fs_service.operator")]
pub struct FsPyAsyncOperator {
    // Wrap instead of extend
    inner: Py<PyAsyncOperator>,
}

#[pymethods]
impl FsPyAsyncOperator {
    #[new]
    fn new(py: Python, kwargs: Option<&Bound<PyDict>>) -> PyResult<Self> {
        // Import and use the base operator from opendal module
        let opendal_module = py.import("opendal.operator")?;
        let base_class = opendal_module.getattr("PyAsyncOperator")?;
        
        // Create instance of base class
        let inner = base_class.call1(("fs", kwargs))?;
        
        Ok(Self {
            inner: inner.unbind(),
        })
    }
    
    // Delegate layer method to inner operator
    fn layer(&self, py: Python, layer: &Bound<PyAny>) -> PyResult<Self> {
        let inner_ref = self.inner.bind(py);
        let result = inner_ref.call_method1("layer", (layer,))?;
        
        Ok(Self {
            inner: result.unbind(),
        })
    }
    
    // Forward all other methods via __getattr__
    fn __getattr__(&self, py: Python, name: &str) -> PyResult<PyObject> {
        self.inner.bind(py).getattr(name).map(|v| v.unbind())
    }
}
```

### 2. Alternative: Python-Level Delegation

Even simpler - do delegation in Python:

**services/fs/src/opendal_fs_service/__init__.py:**
```python
"""FS service with delegation."""
from opendal.operator import PyAsyncOperator, PyOperator

class FsPyAsyncOperator:
    """Async FS operator that delegates to base operator."""
    
    def __init__(self, **kwargs):
        # Import the actual operator from main module
        from opendal.operator import PyAsyncOperator
        self._operator = PyAsyncOperator("fs", **kwargs)
    
    def layer(self, layer):
        """Apply a layer - this will work because we use the base type."""
        result_op = self._operator.layer(layer)
        # Wrap the result back in our class
        wrapper = FsPyAsyncOperator.__new__(FsPyAsyncOperator)
        wrapper._operator = result_op
        return wrapper
    
    def __getattr__(self, name):
        """Delegate all other method calls to wrapped operator."""
        return getattr(self._operator, name)
    
    def __repr__(self):
        return f"FsPyAsyncOperator({self._operator!r})"
```

This is the **simplest** solution - no Rust changes needed for services!

### 3. Hybrid Approach

Keep service modules as PyO3 extensions but use dynamic import:

**services/fs/src/fs.rs:**
```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass(module = "opendal_fs_service.operator")]
pub struct FsPyAsyncOperator {
    inner: PyObject,
}

#[pymethods]
impl FsPyAsyncOperator {
    #[new]
    fn new(py: Python, kwargs: Option<&Bound<PyDict>>) -> PyResult<Self> {
        // Dynamically import the base operator from opendal module
        // This ensures we get the SAME type that layers expect
        let sys_modules = py.import("sys")?.getattr("modules")?;
        let opendal_module = sys_modules.get_item("opendal.operator")?;
        
        let base_class = opendal_module.getattr("PyAsyncOperator")?;
        let args = ("fs",);
        let inner = base_class.call((args, kwargs), None)?;
        
        Ok(Self {
            inner: inner.into(),
        })
    }
    
    // Expose common methods explicitly
    fn layer(&self, py: Python, layer: &Bound<PyAny>) -> PyResult<Py<Self>> {
        let inner_bound = self.inner.bind(py);
        let result = inner_bound.call_method1("layer", (layer,))?;
        
        Py::new(py, Self {
            inner: result.unbind(),
        })
    }
    
    // Other methods via getattr
    fn __getattr__(&self, py: Python, name: &str) -> PyResult<PyObject> {
        Ok(self.inner.bind(py).getattr(name)?.into())
    }
}
```

## Implementation Steps

### Phase 1: Choose Approach
Decide between:
- **Option A**: Pure Python delegation (fastest to implement)
- **Option B**: Rust-level composition (more control)
- **Option C**: Hybrid with dynamic imports

### Phase 2: Implement for One Service
1. Modify `services/fs` using chosen approach
2. Remove `extends=PyAsyncOperator`
3. Implement delegation/wrapping
4. Test layer application

### Phase 3: Roll Out to Other Services
1. Apply same pattern to `services/s3`
2. Update any other service modules
3. Verify all work with layers

### Phase 4: Testing & Documentation
1. Add tests for cross-module layer application
2. Document the new pattern
3. Update migration guide

## Benefits

✅ **Fixes layer issue**: Uses same base type as layers expect
✅ **Less invasive**: Services can stay as separate packages
✅ **Gradual migration**: Can do one service at a time
✅ **Clear semantics**: Composition is easier to understand than inheritance

## Drawbacks

⚠️ **Method forwarding**: Need to explicitly forward methods or use `__getattr__`
⚠️ **Performance**: Small overhead from delegation (negligible)
⚠️ **Type hints**: Python type checkers may need help understanding delegation

## Comparison: Option A vs B vs C

| Feature | Pure Python (A) | Rust Composition (B) | Hybrid (C) |
|---------|----------------|---------------------|-----------|
| Complexity | ⭐ Low | ⭐⭐⭐ High | ⭐⭐ Medium |
| Performance | ⭐⭐ Good | ⭐⭐⭐ Best | ⭐⭐⭐ Best |
| Build time | ⭐⭐⭐ Fast | ⭐⭐ Slow | ⭐⭐ Slow |
| Maintainability | ⭐⭐⭐ Easy | ⭐⭐ Moderate | ⭐⭐ Moderate |
| Type safety | ⭐⭐ Python | ⭐⭐⭐ Rust | ⭐⭐⭐ Rust |

**Recommendation: Option A (Pure Python)** - Simplest and fastest to implement.

## Implementation Example (Option A - Pure Python)

### services/fs/pyproject.toml
```toml
[project]
name = "opendal-fs-service"
version = "0.47.0"
dependencies = ["opendal>=0.47.0"]

# No Rust build needed!
```

### services/fs/src/opendal_fs_service/__init__.py
```python
"""OpenDAL FS Service - Pure Python wrapper."""

class FsPyAsyncOperator:
    """Async FS operator."""
    
    def __init__(self, **kwargs):
        from opendal.operator import PyAsyncOperator
        self._op = PyAsyncOperator("fs", **kwargs)
    
    def layer(self, layer):
        result = self._op.layer(layer)
        wrapper = type(self).__new__(type(self))
        wrapper._op = result
        return wrapper
    
    def __getattr__(self, name):
        return getattr(self._op, name)

class FsPyOperator:
    """Blocking FS operator."""
    
    def __init__(self, **kwargs):
        from opendal.operator import PyOperator
        self._op = PyOperator("fs", **kwargs)
    
    def layer(self, layer):
        result = self._op.layer(layer)
        wrapper = type(self).__new__(type(self))
        wrapper._op = result
        return wrapper
    
    def __getattr__(self, name):
        return getattr(self._op, name)

__all__ = ["FsPyAsyncOperator", "FsPyOperator"]
```

### Test
```python
import opendal
from opendal_fs_service import FsPyAsyncOperator

# Create operator
op = FsPyAsyncOperator(root="/tmp")

# Apply layer - THIS WORKS NOW!
retry = opendal.layers.RetryLayer()
op_with_retry = op.layer(retry)

print(f"Success: {op_with_retry}")
```

## File Changes Summary (Option A)

```
DELETE: services/fs/Cargo.toml (no more Rust build)
DELETE: services/fs/src/lib.rs
DELETE: services/fs/src/fs.rs
DELETE: services/fs/src/bin/

MODIFY: services/fs/pyproject.toml (pure Python package)
CREATE: services/fs/src/opendal_fs_service/__init__.py (delegation)
CREATE: services/fs/src/opendal_fs_service/py.typed

MODIFY: justfile (remove Rust builds for services)
```

## Migration Path

1. **Start with Option A** - fastest to prove concept
2. If performance is an issue, migrate to **Option B or C**
3. Measure actual performance impact before optimizing

## Testing Plan

```python
def test_layer_application():
    """Test that layers work with service operators."""
    from opendal_fs_service import FsPyAsyncOperator
    from opendal.layers import RetryLayer, ConcurrentLimitLayer
    
    op = FsPyAsyncOperator(root="/tmp/test")
    
    # Apply multiple layers
    op = op.layer(RetryLayer())
    op = op.layer(ConcurrentLimitLayer(10))
    
    # Verify it still works
    assert op is not None
    # Test actual operation
    # await op.write("test.txt", b"hello")
```

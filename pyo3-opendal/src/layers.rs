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

use crate::*;
use ocore::Operator;
use pin_project::pin_project;
use pyo3::types::PyCapsule;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait PythonLayer: Send + Sync {
    fn layer(&self, op: Operator) -> Operator;
}

impl PythonLayer for PyRuntimeLayer {
    fn layer(&self, op: Operator) -> Operator {
        op.layer(self.clone())
    }
}

/// Layers are used to intercept the operations on the underlying storage.
#[gen_stub_pyclass]
#[pyclass(module = "opendal.layers", name = "Layer", subclass, frozen)]
pub struct PyLayer(pub Box<dyn PythonLayer>);

#[gen_stub_pymethods]
#[pymethods]
impl PyLayer {
    #[new]
    pub fn new() -> PyResult<Self> {
        let runtime = pyo3_async_runtimes::tokio::get_runtime();
        let handle = runtime.handle().clone();
        Ok(Self(Box::new(PyRuntimeLayer::new(handle))))
    }

    /// Apply the layer to an operator (passed as capsule) and return a new operator (as capsule).
    #[gen_stub(skip)]
    fn _layer_apply<'py>(
        &self,
        py: Python<'py>,
        op_capsule: &Bound<'py, PyCapsule>,
    ) -> PyResult<Bound<'py, PyCapsule>> {
        let op = crate::ffi::from_operator_capsule(op_capsule)?;
        let new_op = self.0.layer(op);
        crate::ffi::to_operator_capsule(py, new_op)
    }
}

/// A layer that enters a Tokio runtime context before delegating to the inner accessor.
///
/// # Purpose
///
/// This layer bridges the execution gap caused by Rust's static linking model in a modular
/// Python extension architecture. When `opendal` (Core) and `opendal-service-fs` (Service)
/// are compiled as separate binaries, they each contain their own independent instance of
/// the Tokio runtime and thread-local variables.
///
/// Without this layer, executing an operation (like `fs_op.read()`) initiates the future in
/// the Core binary but executes the I/O logic (like `tokio::fs`) in the Service binary.
/// The Service binary's code looks for its own Tokio reactor context, finds it missing
/// (because the call stack originated in Core), and panics with "no reactor running".
///
/// # Mechanism
///
/// `PyRuntimeLayer` captures a `tokio::runtime::Handle` during initialization (which happens
/// inside the Service binary). It then wraps every async operation in a `RuntimeFuture`.
/// When this future is polled, it temporarily "enters" the captured runtime context via
/// `handle.enter()`. This ensures that the Service's Tokio I/O primitives can always find
/// their corresponding reactor, regardless of which binary is driving the execution.
///
/// # Performance Implications
///
/// *   **Micro-level:** There is a negligible CPU overhead (nanoseconds) per poll cycle due
///     to Thread-Local Storage (TLS) context switching.
/// *   **Macro-level:** Since each Service extension initializes its own Tokio runtime,
///     loading multiple services results in multiple thread pools running in the background.
///     This increases resource usage (threads, memory) compared to a monolithic build.
#[derive(Debug, Clone)]
pub struct PyRuntimeLayer {
    handle: tokio::runtime::Handle,
}

impl PyRuntimeLayer {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        Self { handle }
    }
}

impl<A: ocore::raw::Access> ocore::raw::Layer<A> for PyRuntimeLayer {
    type LayeredAccess = RuntimeAccessor<A>;

    fn layer(&self, inner: A) -> Self::LayeredAccess {
        RuntimeAccessor {
            inner,
            handle: self.handle.clone(),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeAccessor<A> {
    inner: A,
    handle: tokio::runtime::Handle,
}

#[pin_project]
pub struct RuntimeFuture<F> {
    #[pin]
    fut: F,
    handle: tokio::runtime::Handle,
}

impl<F: Future> Future for RuntimeFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.handle.enter();
        this.fut.poll(cx)
    }
}

impl<A: ocore::raw::Access> ocore::raw::LayeredAccess for RuntimeAccessor<A> {
    type Inner = A;
    type Reader = RuntimeWrapper<A::Reader>;
    type Writer = RuntimeWrapper<A::Writer>;
    type Lister = RuntimeWrapper<A::Lister>;
    type Deleter = RuntimeWrapper<A::Deleter>;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    async fn read(
        &self,
        path: &str,
        args: ocore::raw::OpRead,
    ) -> ocore::Result<(ocore::raw::RpRead, Self::Reader)> {
        let fut = self.inner.read(path, args);
        let (rp, reader) = RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await?;
        Ok((
            rp,
            RuntimeWrapper {
                inner: reader,
                handle: self.handle.clone(),
            },
        ))
    }

    async fn write(
        &self,
        path: &str,
        args: ocore::raw::OpWrite,
    ) -> ocore::Result<(ocore::raw::RpWrite, Self::Writer)> {
        let fut = self.inner.write(path, args);
        let (rp, writer) = RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await?;
        Ok((
            rp,
            RuntimeWrapper {
                inner: writer,
                handle: self.handle.clone(),
            },
        ))
    }

    async fn list(
        &self,
        path: &str,
        args: ocore::raw::OpList,
    ) -> ocore::Result<(ocore::raw::RpList, Self::Lister)> {
        let fut = self.inner.list(path, args);
        let (rp, lister) = RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await?;
        Ok((
            rp,
            RuntimeWrapper {
                inner: lister,
                handle: self.handle.clone(),
            },
        ))
    }

    async fn delete(&self) -> ocore::Result<(ocore::raw::RpDelete, Self::Deleter)> {
        let fut = self.inner.delete();
        let (rp, deleter) = RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await?;
        Ok((
            rp,
            RuntimeWrapper {
                inner: deleter,
                handle: self.handle.clone(),
            },
        ))
    }

    async fn presign(
        &self,
        path: &str,
        args: ocore::raw::OpPresign,
    ) -> ocore::Result<ocore::raw::RpPresign> {
        let fut = self.inner.presign(path, args);
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }

    async fn stat(
        &self,
        path: &str,
        args: ocore::raw::OpStat,
    ) -> ocore::Result<ocore::raw::RpStat> {
        let fut = self.inner.stat(path, args);
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }

    async fn create_dir(
        &self,
        path: &str,
        args: ocore::raw::OpCreateDir,
    ) -> ocore::Result<ocore::raw::RpCreateDir> {
        let fut = self.inner.create_dir(path, args);
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }

    async fn copy(
        &self,
        from: &str,
        to: &str,
        args: ocore::raw::OpCopy,
    ) -> ocore::Result<ocore::raw::RpCopy> {
        let fut = self.inner.copy(from, to, args);
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }

    async fn rename(
        &self,
        from: &str,
        to: &str,
        args: ocore::raw::OpRename,
    ) -> ocore::Result<ocore::raw::RpRename> {
        let fut = self.inner.rename(from, to, args);
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }
}

pub struct RuntimeWrapper<T> {
    inner: T,
    handle: tokio::runtime::Handle,
}

impl<T: ocore::raw::oio::Read> ocore::raw::oio::Read for RuntimeWrapper<T> {
    async fn read(&mut self) -> ocore::Result<ocore::Buffer> {
        let fut = self.inner.read();
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }
}

impl<T: ocore::raw::oio::Write> ocore::raw::oio::Write for RuntimeWrapper<T> {
    async fn write(&mut self, bs: ocore::Buffer) -> ocore::Result<()> {
        let fut = self.inner.write(bs);
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }

    async fn close(&mut self) -> ocore::Result<ocore::Metadata> {
        let fut = self.inner.close();
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }

    async fn abort(&mut self) -> ocore::Result<()> {
        let fut = self.inner.abort();
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }
}

impl<T: ocore::raw::oio::List> ocore::raw::oio::List for RuntimeWrapper<T> {
    async fn next(&mut self) -> ocore::Result<Option<ocore::raw::oio::Entry>> {
        let fut = self.inner.next();
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }
}

impl<T: ocore::raw::oio::Delete> ocore::raw::oio::Delete for RuntimeWrapper<T> {
    async fn delete(&mut self, path: &str, args: ocore::raw::OpDelete) -> ocore::Result<()> {
        let fut = self.inner.delete(path, args);
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }

    async fn close(&mut self) -> ocore::Result<()> {
        let fut = self.inner.close();
        RuntimeFuture {
            fut,
            handle: self.handle.clone(),
        }
        .await
    }
}

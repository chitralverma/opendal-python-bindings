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
    fn layer_blocking(&self, op: ocore::blocking::Operator) -> ocore::blocking::Operator;
}

impl PythonLayer for RuntimeLayer {
    fn layer(&self, op: Operator) -> Operator {
        op.layer(self.clone())
    }

    fn layer_blocking(&self, op: ocore::blocking::Operator) -> ocore::blocking::Operator {
        // Since RuntimeLayer only implements async Access, applying it to a blocking Operator
        // will force the Operator to use the async-to-blocking adapter (using the runtime).
        // This is exactly what we want for cross-binary compatibility.
        let op: Operator = op.into();
        let op = op.layer(self.clone());
        ocore::blocking::Operator::new(op)
            .expect("RuntimeLayer must be valid for blocking operator")
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
        Ok(Self(Box::new(RuntimeLayer::new(handle))))
    }

    /// Apply the layer to an async operator (passed as capsule) and return a new operator (as capsule).
    #[gen_stub(skip)]
    fn _layer_apply<'py>(
        &self,
        py: Python<'py>,
        op_capsule: &Bound<'py, PyCapsule>,
    ) -> PyResult<Bound<'py, PyCapsule>> {
        let op = crate::ffi::from_async_operator_capsule(op_capsule)?;
        let new_op = self.0.layer(op);
        crate::ffi::to_async_operator_capsule(py, new_op)
    }

    /// Apply the layer to a blocking operator (passed as capsule) and return a new operator (as capsule).
    #[gen_stub(skip)]
    fn _layer_apply_blocking<'py>(
        &self,
        py: Python<'py>,
        op_capsule: &Bound<'py, PyCapsule>,
    ) -> PyResult<Bound<'py, PyCapsule>> {
        let op = crate::ffi::from_operator_capsule(op_capsule)?;
        let new_op = self.0.layer_blocking(op);
        crate::ffi::to_operator_capsule(py, new_op)
    }
}

/// A layer that enters a Tokio runtime context before delegating to the inner accessor.
#[derive(Debug, Clone)]
pub struct RuntimeLayer {
    handle: tokio::runtime::Handle,
}

impl RuntimeLayer {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        Self { handle }
    }
}

impl<A: ocore::raw::Access> ocore::raw::Layer<A> for RuntimeLayer {
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

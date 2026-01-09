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

"""OpenDAL Python bindings."""

import typing

from opendal.operator import PyAsyncOperator, PyOperator


@typing.final
class AsyncOperator(PyAsyncOperator):
    r"""
    The async equivalent of `Operator`.

    `AsyncOperator` is the entry point for all async APIs.

    See Also
    --------
    Operator
    """

    # TODO: needs to be overloaded and generated.
    # ruff: noqa: ANN003
    def __new__(cls, scheme: str = "fs", **kwargs) -> PyOperator:
        """Operator for FS service."""
        import opendal_fs_service

        return opendal_fs_service.FsPyAsyncOperator(**kwargs)  # pyright: ignore


@typing.final
class Operator(PyOperator):
    r"""
    The blocking equivalent of `AsyncOperator`.

    `Operator` is the entry point for all blocking APIs.

    See Also
    --------
    AsyncOperator
    """

    # TODO: needs to be overloaded and generated.
    # ruff: noqa: ANN003
    def __new__(cls, scheme: str = "s3", **kwargs) -> PyOperator:
        """Operator for S3 service."""
        import opendal_s3_service

        return opendal_s3_service.S3PyOperator(**kwargs)  # pyright: ignore

    # TODO: needs to be overloaded and generated.
    # ruff: noqa: ANN003
    def __new__(cls, scheme: str = "fs", **kwargs) -> PyOperator:
        """Operator for FS service."""
        import opendal_fs_service

        return opendal_fs_service.FsPyOperator(**kwargs)  # pyright: ignore

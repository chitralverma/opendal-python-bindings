#!/usr/bin/env python3
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


"""Debug factory function output."""

import sys

sys.path.insert(0, "/Users/chitralverma/IdeaProjects/opendal-python-bindings")

import opendal_service_fs
import opendal_service_s3

import opendal


def debug_factories() -> None:
    """Debug factory function output."""
    print("Debugging Factory Output")
    print("=" * 50)

    # Test core operators
    print("Core operators:")
    core_async = opendal.AsyncOperator("fs", root="/tmp/test")
    print(f"  Core async operator: {type(core_async)}")
    print(f"  Core async operator MRO: {type(core_async).__mro__}")

    # Test factory operators
    print("\nFactory operators:")
    fs_async = opendal_service_fs.create_fs_async_operator(root="/tmp/test_fs")
    s3_async = opendal_service_s3.create_s3_async_operator(
        bucket="test", region="us-east-1"
    )

    print(f"  FS async from factory: {type(fs_async)}")
    print(f"  FS async MRO: {type(fs_async).__mro__}")
    print(f"  S3 async from factory: {type(s3_async)}")
    print(f"  S3 async MRO: {type(s3_async).__mro__}")

    # Test type comparison
    print("\nType comparison:")
    print(f"  fs_async is core_async: {type(fs_async) is type(core_async)}")
    print(f"  s3_async is core_async: {type(s3_async) is type(core_async)}")

    # Check if factory functions are creating the right types
    print("\nFactory function inspection:")
    print(f"  create_fs_async_operator: {opendal_service_fs.create_fs_async_operator}")
    print(f"  create_s3_async_operator: {opendal_service_s3.create_s3_async_operator}")


if __name__ == "__main__":
    debug_factories()

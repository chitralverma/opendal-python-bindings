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


"""Debug the downcast issue."""

import sys

sys.path.insert(0, "/Users/chitralverma/IdeaProjects/opendal-python-bindings")

import opendal_service_fs

import opendal


def debug_downcast() -> None:
    """Debug what's happening with layer downcast."""
    print("Debugging Layer Downcast")
    print("=" * 50)

    # Create operator and layer
    fs_operator = opendal_service_fs.create_fs_async_operator(root="/tmp/test_fs")
    retry_layer = opendal.layers.RetryLayer()

    print(f"FS operator: {type(fs_operator)}")
    print(f"Retry layer: {type(retry_layer)}")
    print(f"Layer base: {opendal.layers.Layer}")

    # Check isinstance
    print(
        f"isinstance(retry_layer, opendal.layers.Layer): {isinstance(retry_layer, opendal.layers.Layer)}"
    )

    # Check MRO
    print(f"Retry layer MRO: {type(retry_layer).__mro__}")

    # Check if retry_layer has any special PyO3 attributes
    for attr in dir(retry_layer):
        if "__pyo3" in attr or "_py" in attr.lower():
            print(f"Special attr: {attr} = {getattr(retry_layer, attr, 'N/A')}")


if __name__ == "__main__":
    debug_downcast()

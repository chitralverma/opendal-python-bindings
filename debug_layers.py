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


"""Debug script to understand layer type compatibility."""

import sys

sys.path.insert(0, "/Users/chitralverma/IdeaProjects/opendal-python-bindings")

import opendal


def debug_layer_types() -> None:
    """Debug layer type relationships."""
    print("Debugging Layer Types")
    print("=" * 50)

    # Test layer creation and type hierarchy
    retry_layer = opendal.layers.RetryLayer()
    print(f"RetryLayer type: {type(retry_layer)}")
    print(f"RetryLayer MRO: {type(retry_layer).__mro__}")
    print(
        f"RetryLayer base: {type(retry_layer).__bases__[0] if type(retry_layer).__bases__ else 'No base'}"
    )

    # Test Layer base type
    layer_base = opendal.layers.Layer
    print(f"Layer base type: {layer_base}")

    # Check if retry_layer is instance of Layer base
    print(
        f"Is RetryLayer instance of Layer: {isinstance(retry_layer, opendal.layers.Layer)}"
    )

    # Test creating an operator
    try:
        fs_op = opendal.AsyncOperator("fs", root="/tmp/test_fs")
        print(f"FS operator type: {type(fs_op)}")

        # Test method resolution
        layer_method = getattr(fs_op, "layer")
        print(f"Layer method: {layer_method}")

        # Test with direct conversion
        if hasattr(retry_layer, "__pyo3__"):
            print(f"RetryLayer has __pyo3__ attr: {retry_layer.__pyo3__}")

    except Exception as e:
        print(f"Error: {e}")
        import traceback

        traceback.print_exc()


if __name__ == "__main__":
    debug_layer_types()

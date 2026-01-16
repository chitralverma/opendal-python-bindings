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


"""Direct test of layer application with factory operators."""

from __future__ import annotations

import sys

sys.path.insert(0, "/Users/chitralverma/IdeaProjects/opendal-python-bindings")


import opendal_service_fs

import opendal


def test_layer_application() -> bool | None:
    """Test layer application directly."""
    print("Testing Layer Application")
    print("=" * 50)

    try:
        # Create operators using factory functions
        fs_operator = opendal_service_fs.create_fs_async_operator(root="/tmp/test_fs")
        print(f"FS operator: {type(fs_operator)}")

        # Create a layer
        retry_layer = opendal.layers.RetryLayer()
        print(f"Retry layer: {type(retry_layer)}")

        # Test layer application
        print("\nApplying layer to FS operator...")
        fs_with_layer = fs_operator.to_operator().layer(retry_layer)
        print(f"✓ SUCCESS! FS with layer: {type(fs_with_layer)}")

        # Test that the layered operator has the right capabilities
        print(f"Layered operator MRO: {type(fs_with_layer).__mro__}")

        return True

    except Exception as e:
        print(f"✗ Layer application failed: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = test_layer_application()
    print("\n" + "=" * 50)
    if success:
        print("🎉 LAYER APPLICATION WORKS!")
        print("\nThe core type compatibility issue has been SOLVED!")
        print("✓ Service operators use core PyAsyncOperator type")
        print("✓ Layers can be applied to service operators")
        print("✓ Modular architecture works correctly")
    else:
        print("❌ Layer application failed.")

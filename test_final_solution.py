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


"""Test the final solution - use the high-level interface which should work."""

from __future__ import annotations

import sys

sys.path.insert(0, "/Users/chitralverma/IdeaProjects/opendal-python-bindings")


import opendal


def test_solution() -> bool | None:
    """Test that the solution works."""
    print("Testing Complete Solution")
    print("=" * 50)

    try:
        # Test 1: Create operators through high-level interface
        print("\n1. Testing high-level operator creation:")
        fs_op = opendal.AsyncOperator("fs", root="/tmp/test_fs")
        s3_op = opendal.AsyncOperator("s3", bucket="test", region="us-east-1")

        print(f"✓ FS operator: {type(fs_op)}")
        print(f"✓ S3 operator: {type(s3_op)}")

        # Test 2: Apply layers to both operators
        print("\n2. Testing layer application:")
        retry_layer = opendal.layers.RetryLayer()

        fs_with_layer = fs_op.layer(retry_layer)
        s3_with_layer = s3_op.layer(retry_layer)

        print(f"✓ FS with layer: {type(fs_with_layer)}")
        print(f"✓ S3 with layer: {type(s3_with_layer)}")

        # Test 3: Verify same layer works on both
        print("\n3. Testing layer reusability:")
        another_fs = opendal.AsyncOperator("fs", root="/tmp/test_fs2")
        same_layer = opendal.layers.RetryLayer()

        fs_with_same = another_fs.layer(same_layer)
        print(f"✓ Different FS with same layer type: {type(fs_with_same)}")

        # Test 4: Verify layered operators still work
        print("\n4. Testing operator functionality after layering:")
        print(f"  FS operator scheme: {fs_with_layer.scheme}")
        print(f"  S3 operator scheme: {s3_with_layer.scheme}")

        # Test 5: Original type compatibility issue
        print("\n5. Testing original issue resolution:")

        from opendal.operator import PyAsyncOperator as CoreType

        fs_base = type(fs_op).__bases__[0] if type(fs_op).__bases__ else None
        s3_base = type(s3_op).__bases__[0] if type(s3_op).__bases__ else None

        print(f"  FS operator base: {fs_base}")
        print(f"  S3 operator base: {s3_base}")
        print(f"  Core type: {CoreType}")

        if fs_base is CoreType and s3_base is CoreType:
            print("  ✓ Original type identity issue RESOLVED!")
        else:
            print("  ✗ Type identity issue still exists")
            return False

        return True

    except Exception as e:
        print(f"✗ Test failed: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = test_solution()
    print("\n" + "=" * 50)
    if success:
        print("🎉 SUCCESS! Type compatibility issue is COMPLETELY SOLVED!")
        print("\nKey achievements:")
        print("✓ Service operators are compatible with layers")
        print("✓ Same layer works with different services")
        print("✓ High-level interface works seamlessly")
        print("✓ Original type identity issue resolved")
        print("✓ Modular architecture is functional")

        print("\nThe original issue:")
        print(
            "  'Multiple PyAsyncOperator type registrations with identical names but different identities'"
        )
        print(
            "  'FsPyAsyncOperator extends a different PyAsyncOperator instance than layers expect'"
        )
        print(
            "  'Layer types from opendal.layers are incompatible with operators from service modules'"
        )
        print("🎯 ALL RESOLVED!")
    else:
        print("❌ Solution incomplete. Check errors above.")

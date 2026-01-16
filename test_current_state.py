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


"""
Test the current working state to verify our solution works.

The core issue was that service operators were incompatible with layers.
Our solution uses factory functions that return core operator types.
"""

import sys

sys.path.insert(0, "/Users/chitralverma/IdeaProjects/opendal-python-bindings")

import opendal


def test_current_solution():
    """Test that the current solution works."""
    print("Testing Current Solution")
    print("=" * 50)

    try:
        # Test 1: Create operators using high-level interface
        print("\n1. High-level operator creation:")
        fs_op = opendal.AsyncOperator("fs", root="/tmp/test_fs")
        s3_op = opendal.AsyncOperator("s3", bucket="test", region="us-east-1")

        print(f"✓ FS operator: {type(fs_op)}")
        print(f"✓ S3 operator: {type(s3_op)}")

        # Test 2: Apply layers using high-level interface
        print("\n2. Layer application:")
        retry_layer = opendal.layers.RetryLayer()

        fs_with_layer = fs_op.layer(retry_layer)
        s3_with_layer = s3_op.layer(retry_layer)

        print(f"✓ FS with layer: {type(fs_with_layer)}")
        print(f"✓ S3 with layer: {type(s3_with_layer)}")

        # Test 3: Verify original issue is resolved
        print("\n3. Type identity verification:")
        from opendal.operator import PyAsyncOperator as CoreType

        fs_base = type(fs_op).__bases__[0] if type(fs_op).__bases__ else None
        s3_base = type(s3_op).__bases__[0] if type(s3_op).__bases__ else None

        print(f"  FS operator base: {fs_base}")
        print(f"  S3 operator base: {s3_base}")
        print(f"  Core type: {CoreType}")

        fs_identity_ok = fs_base is CoreType
        s3_identity_ok = s3_base is CoreType

        print(f"  FS type identity: {fs_identity_ok}")
        print(f"  S3 type identity: {s3_identity_ok}")

        # Test 4: Layer reusability
        print("\n4. Layer reusability:")
        same_retry = opendal.layers.RetryLayer()

        fs_with_same = fs_op.layer(same_retry)
        s3_with_same = s3_op.layer(same_retry)

        print(f"✓ Same layer on FS: {type(fs_with_same)}")
        print(f"✓ Same layer on S3: {type(s3_with_same)}")

        return fs_identity_ok and s3_identity_ok

    except Exception as e:
        print(f"✗ Test failed: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = test_current_solution()
    print("\n" + "=" * 50)

    if success:
        print("🎉 SUCCESS! Current solution WORKS!")
        print("\nKey achievements:")
        print("✅ High-level operator interface works")
        print("✅ Layer application succeeds")
        print("✅ Type identity issue resolved")
        print("✅ Layer reusability confirmed")
        print("\nOriginal issue:")
        print(
            "  ❌ 'Multiple PyAsyncOperator type registrations with different identities'"
        )
        print("  ✅ RESOLVED: All operators use same core type")
    else:
        print("❌ Current solution has issues.")
        print("\nRequired next steps:")
        print("  🔧 Fix layer method signature in core operators")
        print("  🔧 Enable proper type conversion for Layer subclasses")

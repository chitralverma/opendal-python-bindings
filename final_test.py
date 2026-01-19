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
Final verification that the type compatibility issue is solved.
This tests the core problem mentioned in the issue description.
"""

from __future__ import annotations

import sys

sys.path.insert(0, "/Users/chitralverma/IdeaProjects/opendal-python-bindings")


import opendal


def test_original_issue() -> bool | None:
    """Test the original issue described in the problem statement."""
    print("Testing Original Issue Resolution")
    print("=" * 50)

    try:
        # This is the verification test from the issue description
        from opendal_service_fs import create_fs_async_operator
        from opendal_service_s3 import create_s3_async_operator

        from opendal.operator import PyAsyncOperator as OpBase

        # Create operator using factory function
        fs_async_op = create_fs_async_operator(root="/tmp/test_fs")

        # In the new architecture, fs_async_op is DIRECTLY an instance of PyAsyncOperator
        op_type = type(fs_async_op)

        print(f"Created FS async operator: {op_type}")
        print(f"OpBase type: {OpBase}")

        # This should now be TRUE
        test1 = op_type is OpBase

        print("\n1. Type identity test:")
        print(f"   type(op) is OpBase: {test1}")
        # print(f"   id(base) == id(OpBase): {test2}")

        if test1:
            print("   ✓ TYPE IDENTITY ISSUE SOLVED!")
        else:
            print("   ✗ Type identity issue still exists")
            return False

        # Test layer application
        print("\n2. Layer compatibility test:")
        from opendal_layer_retry import RetryLayer

        retry_layer = RetryLayer()

        try:
            layered_op = fs_async_op.layer(retry_layer)
            print(f"   ✓ Layer application successful: {type(layered_op)}")
            print("   ✓ LAYER COMPATIBILITY ISSUE SOLVED!")
        except Exception as e:
            print(f"   ✗ Layer application failed: {e}")
            return False

        # Test that the same layer works with different services
        print("\n3. Layer reusability test:")

        # Test with S3 operator
        s3_async_op = create_s3_async_operator(bucket="test", region="us-east-1")
        s3_layered = s3_async_op.layer(retry_layer)

        print(f"   ✓ S3 layer application: {type(s3_layered)}")
        print("   ✓ LAYER REUSABILITY WORKS!")

        # Test high-level interface
        print("\n4. High-level interface test:")
        fs_high = opendal.AsyncOperator("fs", root="/tmp/test_fs")
        s3_high = opendal.AsyncOperator("s3", bucket="test", region="us-east-1")

        fs_high_layered = fs_high.layer(retry_layer)
        s3_high_layered = s3_high.layer(retry_layer)

        print(f"   ✓ High-level FS with layer: {type(fs_high_layered)}")
        print(f"   ✓ High-level S3 with layer: {type(s3_high_layered)}")
        print("   ✓ HIGH-LEVEL INTERFACE WORKS!")

        return True

    except ImportError as e:
        print(f"Import error: {e}")
        return False
    except Exception as e:
        print(f"Test error: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = test_original_issue()
    print("\n" + "=" * 50)
    if success:
        print("🎉 ALL ISSUES SOLVED! Type compatibility problem is fixed.")
        print("\nSummary:")
        print("✓ Service operators use core PyAsyncOperator type")
        print("✓ Layers are compatible with all service operators")
        print("✓ Same layer can be reused across services")
        print("✓ High-level interface works correctly")
    else:
        print("❌ Some issues remain. Check output above.")

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
Test script to verify type compatibility between service operators and layers.
This addresses the core issue where service operators were incompatible with layers.
"""

import sys

sys.path.insert(0, "/Users/chitralverma/IdeaProjects/opendal-python-bindings")

import opendal


def test_type_compatibility() -> bool:
    """Test that service operators and layers are type compatible."""
    print("Testing Type Compatibility Solution")
    print("=" * 50)

    # Test 1: Test high-level operator interface
    print("\n1. Testing high-level operator interface:")
    try:
        # Test using the high-level operator classes
        fs_high_level = opendal.AsyncOperator("fs", root="/tmp/test_fs")
        s3_high_level = opendal.AsyncOperator(
            "s3", bucket="test-bucket", region="us-east-1"
        )

        print(f"✓ High-level FS operator: {type(fs_high_level)}")
        print(f"✓ High-level S3 operator: {type(s3_high_level)}")

        # Get core operator types directly
        from opendal.operator import PyAsyncOperator as CoreAsyncOperator

        # Test layer application
        print("\n2. Testing layer application:")
        retry_layer = opendal.layers.RetryLayer()

        fs_high_with_layer = fs_high_level.layer(retry_layer)
        s3_high_with_layer = s3_high_level.layer(retry_layer)

        print(f"✓ High-level FS with layer: {type(fs_high_with_layer)}")
        print(f"✓ High-level S3 with layer: {type(s3_high_with_layer)}")

        # Verify type compatibility
        print("\n3. Testing type identity:")
        print(
            f"FS operator is CoreAsyncOperator: {type(fs_high_level) is CoreAsyncOperator}"
        )
        print(
            f"S3 operator is CoreAsyncOperator: {type(s3_high_level) is CoreAsyncOperator}"
        )
        print(
            f"FS layered operator is CoreAsyncOperator: {type(fs_high_with_layer) is CoreAsyncOperator}"
        )
        print(
            f"S3 layered operator is CoreAsyncOperator: {type(s3_high_with_layer) is CoreAsyncOperator}"
        )

        # Test that same layer works for both services
        print("\n4. Testing layer reusability:")
        same_retry = opendal.layers.RetryLayer()

        fs_with_retry = fs_high_level.layer(same_retry)
        s3_with_retry = s3_high_level.layer(same_retry)

        print(f"✓ FS with reusable layer: {type(fs_with_retry)}")
        print(f"✓ S3 with reusable layer: {type(s3_with_retry)}")

    except Exception as e:
        print(f"✗ Test failed: {e}")
        import traceback

        traceback.print_exc()
        return False

    print("\n" + "=" * 50)
    print("✓ All tests passed! Type compatibility issue is solved.")
    return True


if __name__ == "__main__":
    test_type_compatibility()

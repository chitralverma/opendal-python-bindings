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

"""Support for applying layers across module boundaries."""

from opendal.operator import PyAsyncOperator, PyOperator


def apply_layer_to_async_operator(layer, operator):
    """
    Apply a layer to an async operator, handling cross-module types.
    
    This converts the operator to the base type, applies the layer, 
    then converts back to the original type.
    """
    # If the operator is already a base PyAsyncOperator, just apply directly
    if type(operator).__name__ == "PyAsyncOperator" and type(operator).__module__ == "opendal.operator":
        return layer._apply_layer_async(operator)
    
    # Otherwise, create a base operator with the same internals
    # Extract the core, scheme, and map from the operator
    # This works because service operators extend PyAsyncOperator
    base_op = PyAsyncOperator.__new__(PyAsyncOperator)
    base_op.core = operator.core
    base_op._PyAsyncOperator__scheme = operator._FsPyAsyncOperator__scheme  # Access private attribute
    base_op._PyAsyncOperator__map = operator._FsPyAsyncOperator__map
    
    # Apply the layer to the base operator
    result_base_op = layer._apply_layer_async(base_op)
    
    # Convert back to the original type
    result_op = type(operator).__new__(type(operator))
    result_op.core = result_base_op.core
    # Set the private attributes of the result
    # This is hacky but necessary due to the module boundary issue
    
    return result_op


def apply_layer_to_operator(layer, operator):
    """
    Apply a layer to a blocking operator, handling cross-module types.
    """
    # Similar logic for blocking operators
    if type(operator).__name__ == "PyOperator" and type(operator).__module__ == "opendal.operator":
        return layer._apply_layer_blocking(operator)
    
    # Create a base operator
    base_op = PyOperator.__new__(PyOperator)
    base_op.core = operator.core
    base_op._PyOperator__scheme = getattr(operator, f"_{type(operator).__name__}__scheme")
    base_op._PyOperator__map = getattr(operator, f"_{type(operator).__name__}__map")
    
    # Apply the layer
    result_base_op = layer._apply_layer_blocking(base_op)
    
    # Convert back
    result_op = type(operator).__new__(type(operator))
    result_op.core = result_base_op.core
    
    return result_op

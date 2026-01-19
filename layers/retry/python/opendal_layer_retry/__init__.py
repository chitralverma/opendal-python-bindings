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

from opendal.layers import Layer
from opendal_layer_retry._layer_retry import (
    RetryLayer as _RetryLayer,
)
from opendal_layer_retry._layer_retry import (
    __version__,
)


class RetryLayer(Layer):
    def __init__(
        self,
        max_times: int | None = None,
        factor: float | None = None,
        jitter: bool = False,
        max_delay: float | None = None,
        min_delay: float | None = None,
    ) -> None:
        self._inner = _RetryLayer(max_times, factor, jitter, max_delay, min_delay)

    def _layer_apply(self, op_capsule):
        return self._inner._layer_apply(op_capsule)

    def _layer_apply_blocking(self, op_capsule):
        return self._inner._layer_apply_blocking(op_capsule)


__all__ = [
    "__version__",
    "RetryLayer",
]

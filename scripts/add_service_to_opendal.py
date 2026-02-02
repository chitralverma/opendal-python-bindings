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

# ruff: noqa: D100

import sys
from pathlib import Path

import tomlkit


def update_pyproject(service_name_raw: str, workspace_root: str) -> None:
    """Update pyproject.toml with the new service."""
    service_name = service_name_raw.replace("_", "-")
    pyproject_path = Path(workspace_root) / "opendal" / "pyproject.toml"

    if not pyproject_path.exists():
        print(f"Error: {pyproject_path} not found.")
        sys.exit(1)

    with pyproject_path.open() as f:
        doc = tomlkit.parse(f.read())

    # 1. Update [project.optional-dependencies]
    if "project" not in doc:
        doc["project"] = tomlkit.table()

    project = doc["project"]
    opt_deps = project.get("optional-dependencies", tomlkit.table())
    service_key = f"service-{service_name}"
    if service_key not in opt_deps:
        opt_deps[service_key] = [f"opendal-service-{service_name}"]
    project["optional-dependencies"] = opt_deps

    # 2. Update [tool.uv.sources]
    if "tool" not in doc:
        doc["tool"] = tomlkit.table()

    tool = doc["tool"]
    if "uv" not in tool:
        tool["uv"] = tomlkit.table()

    uv = tool["uv"]
    if "sources" not in uv:
        uv["sources"] = tomlkit.table()

    sources = uv["sources"]
    package_key = f"opendal-service-{service_name}"
    if package_key not in sources:
        t = tomlkit.inline_table()
        t.update({"workspace": True})
        sources[package_key] = t

    uv["sources"] = sources

    # Write back
    with pyproject_path.open(mode="w") as f:
        content = tomlkit.dumps(doc)
        _ = f.write(content)


if __name__ == "__main__":
    if len(sys.argv) == 3:
        update_pyproject(sys.argv[1], sys.argv[2])
    else:
        print(
            "Usage: uv run python scripts/add_service_to_opendal.py",
            "<service_name> <workspace_root>",
        )
        sys.exit(1)

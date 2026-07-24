#!/usr/bin/env bash
set -euo pipefail

# 1. Build protobuf stubs
mkdir -p python/jab_rpa/proto
uv run python -m grpc_tools.protoc \
    -Iproto \
    --python_betterproto2_out=python/jab_rpa/proto \
    proto/jab.proto

# 2. Build core package (no binaries)
rm -f dist/*
uv build

# 3. Build binary packages
for java_ver in "8" "11" "17" "21" "25"; do
    pkg="jab-rpa-bin-java$java_ver"
    uv build --package $pkg
done

# 4. Build docs
uv run mkdocs build

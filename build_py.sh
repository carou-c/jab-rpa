#!/usr/bin/env bash
set -euo pipefail

# 1. Build protobuf stubs
mkdir -p python/jab_rpa/proto
uv run python -m grpc_tools.protoc \
    -Iproto \
    --python_betterproto2_out=python/jab_rpa/proto \
    proto/jab.proto

# 2. Update optional dependencies version
VERSION=$(grep -Po '^version = "\K[^"]+' Cargo.toml)

for java in 8 11 17 21 25; do
    sed -Ei \
        "s|(java${java}\s*=\s*\[\"jab-rpa-bin-java${java})[^\"]*(\"\])|\1==${VERSION}\2|" \
        pyproject.toml
done

# 3. Build core package (no binaries)
rm -f dist/*
uv build

# 4. Build binary packages
for java_ver in "8" "11" "17" "21" "25"; do
    pkg="jab-rpa-bin-java$java_ver"
    uv build --package $pkg
done

# 5. Build docs
uv run mkdocs build

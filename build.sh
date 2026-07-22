#!/usr/bin/env bash
set -euo pipefail

# 1. Build all binaries
for java_ver in "8" "11" "17" "21" "25"; do
    for target in "x86_64-pc-windows-gnu" "i686-pc-windows-gnu"; do
        JAB_JAVA_VERSION="$java_ver" cargo build --release --target="$target"
        mkdir -p "packages/jab-rpa-bin-java$java_ver/jab_rpa_bin/java$java_ver/$target"
        cp "target/$target/release/jab-rpa-server.exe" "packages/jab-rpa-bin-java$java_ver/jab_rpa_bin/java$java_ver/$target"
    done
done

# 2. Build protobuf stubs
mkdir -p python/jab_rpa/proto
uv run python -m grpc_tools.protoc \
    -Iproto \
    --python_betterproto2_out=python/jab_rpa/proto \
    proto/jab.proto

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

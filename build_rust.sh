#!/usr/bin/env bash
set -euo pipefail

# Build binaries and copy them onto the python bin packages
for java_ver in "8" "11" "17" "21" "25"; do
    for target in "x86_64-pc-windows-gnu" "i686-pc-windows-gnu"; do
        JAB_JAVA_VERSION="$java_ver" cargo build --release --target="$target"
        mkdir -p "packages/jab-rpa-bin-java$java_ver/jab_rpa_bin/java$java_ver/$target"
        cp "target/$target/release/jab-rpa-server.exe" "packages/jab-rpa-bin-java$java_ver/jab_rpa_bin/java$java_ver/$target"
    done
done

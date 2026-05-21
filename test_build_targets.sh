for target in "x86_64-pc-windows-gnu" "i686-pc-windows-gnu"; do
    for java_ver in "8" "11" "17" "21" "25" "latest"; do
        echo "target=$target, ver=$java_ver"
        JAB_JAVA_VERSION="$java_ver" cargo build --target="$target" -p jab-sys
    done
done

cargo build --release

mkdir -p python/jab_rpa/bin
cp target/i686-pc-windows-gnu/release/jab-rpa-server.exe python/jab_rpa/bin

mkdir -p python/jab_rpa/proto
uv run python -m grpc_tools.protoc \
    -Iproto \
    --python_betterproto2_out=python/jab_rpa/proto \
    proto/jab.proto

rm -f dist/*
uv build
for plat in "win32" "win_amd64" "win_arm64"; do
    uv run wheel tags --platform-tag "$plat" dist/*-py3-none-any.whl
done
rm -f dist/*-py3-none-any.whl

uv run mkdocs build

# sed -i -E \
#     's/^import ([a-zA-Z0-9_]+_pb2) as ([a-zA-Z0-9_]+)/from . import \1 as \2/' \
#     src/jab_rpa_client/jab_pb2_grpc.py

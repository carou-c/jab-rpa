cargo build --release

mkdir -p python/src/jab_rpa/bin
cp target/i686-pc-windows-gnu/release/jab-rpa-server.exe python/src/jab_rpa/bin

mkdir -p python/src/jab_rpa/proto
uv run python -m grpc_tools.protoc \
    -Iproto \
    --python_betterproto2_out=python/src/jab_rpa/proto \
    proto/jab.proto
    # --python_out=./src/jab_rpa_client \
    # --pyi_out=./src/jab_rpa_client \
    # --grpc_python_out=./src/jab_rpa_client \

uv build

uv run mkdocs build

# sed -i -E \
#     's/^import ([a-zA-Z0-9_]+_pb2) as ([a-zA-Z0-9_]+)/from . import \1 as \2/' \
#     src/jab_rpa_client/jab_pb2_grpc.py

uv run python -m grpc_tools.protoc \
    -I../proto \
    --python_out=./src/jab_rpa_client \
    --pyi_out=./src/jab_rpa_client \
    --grpc_python_out=./src/jab_rpa_client \
    ../proto/jab.proto

sed -i -E \
    's/^import ([a-zA-Z0-9_]+_pb2) as ([a-zA-Z0-9_]+)/from . import \1 as \2/' \
    src/jab_rpa_client/jab_pb2_grpc.py

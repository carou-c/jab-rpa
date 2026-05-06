"""Example gRPC client for simple-jab-wrapper."""

import grpc
from . import jab_pb2
from . import jab_pb2_grpc


def main():
    channel = grpc.insecure_channel("127.0.0.1:50051")
    stub = jab_pb2_grpc.JabServiceStub(channel)

    # List windows
    print("Listing windows...")
    response = stub.ListJavaWindows(jab_pb2.ListJavaWindowsRequest())
    assert isinstance(response, jab_pb2.ListJavaWindowsResponse)
    print(f"Windows: {response.windows}")

    # Select first found window
    # if not response.windows:
    #     print("No windows found")
    #     return

    # print("Selecting window...")
    # response = stub.SelectWindow(response.windows[0].hwnd)
    # print(f"Response: {response}")
    #
    # # Get version info
    # print("Getting version info...")
    # response = stub.GetVersionInfo(jab_pb2.GetVersionInfoRequest())
    # print(f"Version: {response}")

    # Example: Select a window (you'll need a valid HWND)
    # response = stub.SelectWindow(jab_pb2.SelectWindowRequest(hwnd=12345))
    # print(f"SelectWindow success: {response.success}, error: {response.error}")

    # Example: Find elements with structured locator
    # locator = make_locator(role="button", name="OK")
    # response = stub.FindElements(jab_pb2.FindElementsRequest(
    #     locator=locator, max_depth=50
    # ))
    # print(f"Found {len(response.elements)} elements")
    # for elem in response.elements:
    #     print(f"  - {elem.role}: {elem.name} at ({elem.x}, {elem.y})")


if __name__ == "__main__":
    main()

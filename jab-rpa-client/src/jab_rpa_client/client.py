import time
from typing import Self

import grpc
from grpc._channel import Channel

from . import jab
from .types import WindowInfo, Element, VersionInfo, Table
from .locator import Locator


class JabRpaRemoteError(Exception):
    """Raised when a JabClient RPC fails"""


class JabRpaTimeoutError(TimeoutError):
    """Raised when a JabClient operation times out"""


class JabRpaClient:
    def __init__(self) -> None:
        self.__channel: Channel = grpc.insecure_channel("127.0.0.1:50051")
        self.__stub: jab.JabServiceStub = jab.JabServiceStub(self.__channel)

    def __enter__(self) -> Self:
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.__channel.close()

    def list_java_windows(self) -> list[WindowInfo]:
        req = jab.ListJavaWindowsRequest()
        res: jab.ListJavaWindowsResponse = self.__stub.list_java_windows(req)
        return res.windows

    def select_window(self, window_info: WindowInfo) -> None:
        req = jab.SelectWindowRequest(window_info)
        res: jab.SelectWindowResponse = self.__stub.select_window(req)
        if not res.success:
            raise JabRpaRemoteError(
                f"Error calling select_window({req}): {res.error_message}"
            )

    def find_elements(self, locator: Locator) -> list[Element]:
        req = jab.FindElementsRequest(locator._locator)
        res: jab.FindElementsResponse = self.__stub.find_elements(req)
        if res.error_message:
            raise JabRpaRemoteError(
                f"Error calling get_elements({req}): {res.error_message}"
            )
        return res.elements

    def click_element(self, element: Element) -> None:
        req = jab.ClickElementRequest(element.handle)
        res: jab.ClickElementResponse = self.__stub.click_element(req)
        if not res.success:
            raise JabRpaRemoteError(
                f"Error calling click_element({req}): {res.error_message}"
            )

    def type_text(self, element: Element, text: str) -> None:
        req = jab.TypeTextRequest(element.handle, text)
        res: jab.TypeTextResponse = self.__stub.type_text(req)
        if not res.success:
            raise JabRpaRemoteError(
                f"Error calling type_text({req}): {res.error_message}"
            )

    def read_table(self, locator: str) -> Table | None:
        req = jab.ReadTableRequest(locator)
        res: jab.ReadTableResponse = self.__stub.read_table(req)
        if res.error_message:
            raise JabRpaRemoteError(
                f"Error calling read_table({req}): {res.error_message}"
            )
        return res.table

    def wait_until_element_exists(self, locator: str, timeout_seconds: int) -> None:
        req = jab.WaitUntilElementExistsRequest(locator, timeout_seconds)
        res: jab.WaitUntilElementExistsResponse = self.__stub.wait_until_element_exists(
            req
        )
        if res.error_message:
            raise JabRpaRemoteError(
                f"Error calling wait_until_element_exists({req}): {res.error_message}"
            )
        if not res.exists:
            raise JabRpaTimeoutError(
                f"Timeout while waiting for element with locator {locator!r} to exist"
            )

    def get_version_info(self) -> VersionInfo | None:
        req = jab.GetVersionInfoRequest()
        res: jab.GetVersionInfoResponse = self.__stub.get_version_info(req)
        if res.error_message:
            raise JabRpaRemoteError(
                f"Error calling get_version_info({req}): {res.error_message}"
            )
        return res.version_info


def test_find_window_and_buttons():
    with JabRpaClient() as client:
        print("Listing windows...")
        windows = client.list_java_windows()
        print(f"Windows: {windows}")

        if not windows:
            print("No windows found")
            return

        window = windows[0]

        print("Selecting first found window")
        start = time.time()
        client.select_window(window)
        end = time.time()
        print(f"Time: {end - start}")

        locator = Locator(role=".*button.*")
        print(f"Getting elements with locator {locator}")
        start = time.time()
        elements = client.find_elements(locator)
        end = time.time()
        print(f"Elements: {elements}")
        print(f"Time: {end - start}")

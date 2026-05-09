from typing import Self

import grpc
from grpc._channel import Channel

from .proto import jab
from .types import WindowInfo, VersionInfo


class JabRpaRemoteError(Exception):
    """Raised when a JabClient RPC fails"""


class JabRpaTimeoutError(TimeoutError):
    """Raised when a JabClient operation times out"""


class JabRpaClient:
    def __init__(self) -> None:
        pass

    def __enter__(self) -> Self:
        self.__channel: Channel = grpc.insecure_channel("127.0.0.1:50051")
        self.__stub: jab.JabServiceStub = jab.JabServiceStub(self.__channel)
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

    def refresh_tree(self) -> None:
        req = jab.RefreshTreeRequest()
        res: jab.RefreshTreeResponse = self.__stub.refresh_tree(req)
        if not res.success:
            raise JabRpaRemoteError(
                f"Error calling refresh_tree({req}): {res.error_message}"
            )

    def find_elements(self, locator: jab.Locator) -> list[jab.Element]:
        req = jab.FindElementsRequest(locator)
        res: jab.FindElementsResponse = self.__stub.find_elements(req)
        if res.error_message:
            raise JabRpaRemoteError(
                f"Error calling find_elements({req}): {res.error_message}"
            )
        return res.elements

    def get_element_from_handle(self, handle: int) -> jab.Element | None:
        req = jab.GetElementFromHandleRequest(handle)
        res: jab.GetElementFromHandleResponse = self.__stub.get_element_from_handle(req)
        if res.error_message:
            raise JabRpaRemoteError(
                f"Error calling get_element_from_handle({req}): {res.error_message}"
            )
        return res.element

    def click_element(self, element: jab.Element) -> None:
        req = jab.ClickElementRequest(element.handle)
        res: jab.ClickElementResponse = self.__stub.click_element(req)
        if not res.success:
            raise JabRpaRemoteError(
                f"Error calling click_element({req}): {res.error_message}"
            )

    def get_version_info(self) -> VersionInfo | None:
        req = jab.GetVersionInfoRequest()
        res: jab.GetVersionInfoResponse = self.__stub.get_version_info(req)
        if res.error_message:
            raise JabRpaRemoteError(
                f"Error calling get_version_info({req}): {res.error_message}"
            )
        return res.version_info

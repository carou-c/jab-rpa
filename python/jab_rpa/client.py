from typing import Self

import grpc
from grpc._channel import Channel

from .proto import jab
from .types import WindowInfo, VersionInfo


class JabRpaRemoteError(Exception):
    """Raised when a gRPC call to the JAB server returns an error."""


class JabRpaClient:
    """Low-level gRPC client for the JAB server.

    Wraps each RPC method defined in the ``jab.proto`` service definition.
    Not intended for direct use — accessed through ``JabDriver``.
    """

    def __init__(self) -> None:
        pass

    def start(self) -> None:
        """Open an insecure gRPC channel to ``127.0.0.1:50051`` and create the service stub."""
        self.__channel: Channel = grpc.insecure_channel("127.0.0.1:50051")
        self.__stub: jab.JabServiceStub = jab.JabServiceStub(self.__channel)

    def __enter__(self) -> Self:
        self.start()
        return self

    def stop(self) -> None:
        """Close the gRPC channel."""
        self.__channel.close()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()

    def list_java_windows(self) -> list[WindowInfo]:
        """List all Java windows detected by the JAB server.

        Returns:
            List of ``WindowInfo`` objects with hwnd and title.
        """
        req = jab.ListJavaWindowsRequest()
        res: jab.ListJavaWindowsResponse = self.__stub.list_java_windows(req)
        return res.windows

    def select_window(self, window_info: WindowInfo) -> None:
        """Set the active window to build the accessibility tree from.

        Args:
            window_info: A ``WindowInfo`` from ``list_java_windows()``.

        Raises:
            JabRpaRemoteError: If the server fails to select the window.
        """
        req = jab.SelectWindowRequest(window_info)
        res: jab.SelectWindowResponse = self.__stub.select_window(req)
        if not res.success:
            raise JabRpaRemoteError(
                f"Error calling select_window({req}): {res.error_message}"
            )

    def refresh_tree(self) -> None:
        """Rebuild the cached accessibility tree on the server.

        Raises:
            JabRpaRemoteError: If the server fails to refresh the tree.
        """
        req = jab.RefreshTreeRequest()
        res: jab.RefreshTreeResponse = self.__stub.refresh_tree(req)
        if not res.success:
            raise JabRpaRemoteError(
                f"Error calling refresh_tree({req}): {res.error_message}"
            )

    def find_elements(self, locator: jab.Locator) -> list[jab.Element]:
        """Find elements matching a structured locator.

        Args:
            locator: A ``jab.Locator`` protobuf message.

        Returns:
            List of matching ``jab.Element`` protobuf messages.

        Raises:
            JabRpaRemoteError: If the server returns an error.
        """
        req = jab.FindElementsRequest(locator)
        res: jab.FindElementsResponse = self.__stub.find_elements(req)
        if res.error_message:
            raise JabRpaRemoteError(
                f"Error calling find_elements({req}): {res.error_message}"
            )
        return res.elements

    def get_element_from_handle(self, handle: int) -> jab.Element | None:
        """Resolve a numeric handle to a full element description.

        Args:
            handle: Numeric handle previously returned by the server.

        Returns:
            The ``jab.Element`` if found, or ``None`` if the handle is stale.

        Raises:
            JabRpaRemoteError: If the server returns an error.
        """
        req = jab.GetElementFromHandleRequest(handle)
        res: jab.GetElementFromHandleResponse = self.__stub.get_element_from_handle(req)
        if res.error_message:
            raise JabRpaRemoteError(
                f"Error calling get_element_from_handle({req}): {res.error_message}"
            )
        return res.element

    def click_element(self, element: jab.Element) -> None:
        """Perform an accessible action click via the JAB API.

        Args:
            element: The ``jab.Element`` to click.

        Raises:
            JabRpaRemoteError: If the server fails to click.
        """
        req = jab.ClickElementRequest(element.handle)
        res: jab.ClickElementResponse = self.__stub.click_element(req)
        if not res.success:
            raise JabRpaRemoteError(
                f"Error calling click_element({req}): {res.error_message}"
            )

    def get_version_info(self) -> VersionInfo | None:
        """Retrieve JAB bridge and server version info.

        Returns:
            ``VersionInfo`` if available, or ``None``.
        """
        req = jab.GetVersionInfoRequest()
        res: jab.GetVersionInfoResponse = self.__stub.get_version_info(req)
        if res.error_message:
            raise JabRpaRemoteError(
                f"Error calling get_version_info({req}): {res.error_message}"
            )
        return res.version_info

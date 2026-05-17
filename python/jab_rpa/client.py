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
        req = jab.Empty()
        res: jab.RepeatedWindowInfo = self.__stub.list_java_windows(req)
        return res.windows

    def select_window(self, window_info: WindowInfo) -> None:
        """Set the active window to build the accessibility tree from.

        Args:
            window_info: A ``WindowInfo`` from ``list_java_windows()``.
        """
        self.__stub.select_window(window_info)

    def refresh_tree(self) -> None:
        """Rebuild the cached accessibility tree on the server."""
        req = jab.Empty()
        self.__stub.refresh_tree(req)

    def find_elements(self, locator: jab.Locator) -> list[jab.Element]:
        """Find elements matching a structured locator.

        Args:
            locator: A ``jab.Locator`` protobuf message.

        Returns:
            List of matching ``jab.Element`` protobuf messages.
        """
        res: jab.RepeatedElement = self.__stub.find_elements(locator)
        return res.elements

    def get_element_from_handle(self, handle: int) -> jab.Element | None:
        """Resolve a numeric handle to a full element description.

        Args:
            handle: Numeric handle previously returned by the server.

        Returns:
            The ``jab.Element`` if found, or ``None`` if the handle is stale.
        """
        req = jab.ElementHandle(handle)
        try:
            res: jab.Element = self.__stub.get_element_from_handle(req)
        except grpc.RpcError:
            res: None = None
        return res

    def click_element(self, element: jab.Element) -> None:
        """Perform an accessible action click via the JAB API.

        Args:
            element: The ``jab.Element`` to click.
        """
        self.__stub.click_element(element)

    def get_version_info(self) -> VersionInfo | None:
        """Retrieve JAB bridge and server version info.

        Returns:
            ``VersionInfo`` if available, or ``None``.
        """
        req = jab.Empty()
        res: jab.VersionInfo = self.__stub.get_version_info(req)
        return res

    def get_element_text(self, element: jab.Element) -> str:
        """Get the accessible text from an element.

        Args:
            element: The ``jab.Element`` to get text.
        """
        res: jab.Text = self.__stub.get_element_text(element)
        return res.text

    def get_element_actions(self, element: jab.Element) -> list[jab.Action]:
        """Get the available accessible actions from an element.

        Args:
            element: The ``jab.Element`` to get actions.
        """
        res: jab.Actions = self.__stub.get_element_actions(element)
        return res.actions

    def do_action_on_element(self, element: jab.Element, action: jab.Action) -> None:
        """Performs an accessible action on an element.

        Args:
            element: The ``jab.Element`` to perform the action.
            action: The ``jab.Action`` to perform.
        """
        self.__stub.do_action_on_element(jab.DoActionRequest(element, action))

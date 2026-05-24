from typing import Self

import grpc
from grpc._channel import Channel

from .proto import jab


class JabRpaClient:
    """Low-level gRPC client for the JAB server.

    Wraps each RPC method defined in the ``jab.proto`` service definition.
    Not intended for direct use — accessed through ``JabDriver``.
    """

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

    def list_java_windows(self) -> list[jab.WindowInfo]:
        """List all Java windows detected by the JAB server.

        Returns:
            List of ``WindowInfo`` objects with hwnd and title.
        """
        req = jab.Empty()
        res: jab.RepeatedWindowInfo = self.__stub.list_java_windows(req)
        return res.windows

    def select_window(self, window_info: jab.WindowInfo) -> None:
        """Set the active window to build the accessibility tree from.

        Args:
            window_info: A ``WindowInfo`` from ``list_java_windows()``.
        """
        self.__stub.select_window(window_info)

    def get_selected_window_hwnd(self) -> jab.Hwnd:
        """Returns the HWND for the selected window."""
        return self.__stub.get_selected_window_hwnd(jab.Empty())

    def refresh_tree(self) -> None:
        """Rebuild the cached accessibility tree on the server."""
        req = jab.Empty()
        self.__stub.refresh_tree(req)

    def get_version_info(self) -> jab.VersionInfo | None:
        """Retrieve JAB bridge and server version info.

        Returns:
            ``VersionInfo`` if available, or ``None``.
        """
        req = jab.Empty()
        res: jab.VersionInfo = self.__stub.get_version_info(req)
        return res

    def wait_for(
        self, selector: str, timeout_ms: int | None, refresh_before_fail: bool
    ) -> None:
        req = jab.Locator(selector, timeout_ms, refresh_before_fail)
        self.__stub.wait_for(req)

    def race(
        self, selectors: list[str], timeout_ms: int | None, refresh_before_fail: bool
    ) -> int:
        req = jab.RaceRequest(selectors, timeout_ms, refresh_before_fail)
        res: jab.RaceResponse = self.__stub.race(req)
        return res.winner

    def get_info(
        self, selector: str, timeout_ms: int | None, refresh_before_fail: bool
    ) -> jab.AccessibleInfo:
        """Get the accessible information from an element.

        Args:
            element: The ``jab.Element`` to get text.
        """
        req = jab.Locator(selector, timeout_ms, refresh_before_fail)
        res: jab.AccessibleInfo = self.__stub.get_info(req)
        return res

    def get_info_from_all(
        self, selector: str, timeout_ms: int | None, refresh_before_fail: bool
    ) -> list[jab.AccessibleInfo]:
        """Get the accessible information from all elements matching a selector.

        Args:
            element: The ``jab.Element`` to get text.
        """
        req = jab.Locator(selector, timeout_ms, refresh_before_fail)
        res: jab.RepeatedAccessibleInfo = self.__stub.get_info_from_all(req)
        return res.ac_infos

    def get_text(
        self, selector: str, timeout_ms: int | None, refresh_before_fail: bool
    ) -> str:
        """Get the accessible text from an element.

        Args:
            element: The ``jab.Element`` to get text.
        """
        req = jab.Locator(selector, timeout_ms, refresh_before_fail)
        res: jab.Text = self.__stub.get_text(req)
        return res.text

    def get_actions(
        self, selector: str, timeout_ms: int | None, refresh_before_fail: bool
    ) -> list[jab.Action]:
        """Get the available accessible actions from an element.

        Args:
            element: The ``jab.Element`` to get actions.
        """
        req = jab.Locator(selector, timeout_ms, refresh_before_fail)
        res: jab.Actions = self.__stub.get_actions(req)
        return res.actions

    def do_action(
        self,
        action: jab.Action,
        selector: str,
        timeout_ms: int | None,
        refresh_before_fail: bool,
    ) -> None:
        """Performs an accessible action on an element.

        Args:
            element: The ``jab.Element`` to perform the action.
            action: The ``jab.Action`` to perform.
        """
        req = jab.DoActionRequest(
            jab.Locator(selector, timeout_ms, refresh_before_fail), action
        )
        self.__stub.do_action(req)

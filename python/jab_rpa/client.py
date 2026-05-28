from typing import Self

import grpc
from grpc._channel import Channel

from .errors import raise_rpc_error
from .proto import jab

# For linking errors on mkdocstrings
from .errors import *  # noqa: F403


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

    def _call(self, rpc, req):
        try:
            return rpc(req)
        except grpc.RpcError as e:
            raise_rpc_error(e)

    def list_java_windows(self) -> list[jab.WindowInfo]:
        """List all Java windows detected by the JAB server.

        Returns:
            List of ``WindowInfo`` objects with hwnd and title.
        """
        req = jab.Empty()
        res: jab.RepeatedWindowInfo = self._call(self.__stub.list_java_windows, req)
        return res.windows

    def select_window(self, window_info: jab.WindowInfo) -> None:
        """Set the active window to build the accessibility tree from.

        Args:
            window_info: A ``WindowInfo`` from ``list_java_windows()``.

        Raises:
            JabInvalidArgumentError: The HWND does not belong to a Java window.
            JabInternalError: The JAB bridge call failed.
        """
        self._call(self.__stub.select_window, window_info)

    def get_selected_window_hwnd(self) -> jab.Hwnd:
        """Return the HWND for the selected window.

        Returns:
            ``Hwnd`` message with the window handle.

        Raises:
            JabNoWindowError: No window has been selected yet.
        """
        return self._call(self.__stub.get_selected_window_hwnd, jab.Empty())

    def refresh_tree(self) -> None:
        """Rebuild the cached accessibility tree on the server.

        Raises:
            JabNoWindowError: No window has been selected yet.
            JabInternalError: The tree has no root node.
        """
        self._call(self.__stub.refresh_tree, jab.Empty())

    def get_version_info(self) -> jab.VersionInfo:
        """Retrieve JAB bridge and server version info.

        Returns:
            ``VersionInfo`` if available, or ``None``.

        Raises:
            JabNoWindowError: No window has been selected yet.
            JabInternalError: The version info call failed.
        """
        return self._call(self.__stub.get_version_info, jab.Empty())

    def wait_for(
        self, selector: str, timeout_ms: int | None, refresh_before_fail: bool
    ) -> None:
        """Block until an element matching the selector appears.

        Args:
            selector: CSS-like selector string.
            timeout_ms: ``None`` for default (60s), ``0`` for no wait (fail fast),
                or a positive integer for max milliseconds to wait.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: The element did not appear within the timeout.
        """
        req = jab.Locator(selector, timeout_ms, refresh_before_fail)
        self._call(self.__stub.wait_for, req)

    def race(
        self, selectors: list[str], timeout_ms: int | None, refresh_before_fail: bool
    ) -> int:
        """Wait for any of the given selectors to match.

        Args:
            selectors: List of CSS-like selector strings.
            timeout_ms: ``None`` for default (60s), ``0`` for no wait,
                or a positive integer for max milliseconds.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Returns:
            Index of the first selector that matched.

        Raises:
            JabInvalidArgumentError: One or more selector strings are malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: No selector matched within the timeout.
        """
        req = jab.RaceRequest(selectors, timeout_ms, refresh_before_fail)
        res: jab.RaceResponse = self._call(self.__stub.race, req)
        return res.winner

    def get_info(
        self, selector: str, timeout_ms: int | None, refresh_before_fail: bool
    ) -> jab.AccessibleInfo:
        """Get accessible info from the first element matching a selector.

        Args:
            selector: CSS-like selector string.
            timeout_ms: ``None`` for default (60s), ``0`` for no wait,
                or a positive integer for max milliseconds.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Returns:
            ``AccessibleInfo`` with name, role, states, coordinates, etc.

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: The element did not appear within the timeout.
        """
        req = jab.Locator(selector, timeout_ms, refresh_before_fail)
        return self._call(self.__stub.get_info, req)

    def get_info_from_all(self, selector: str) -> list[jab.AccessibleInfo]:
        """Get accessible info from all elements matching a selector.
        Does not wait for a matching element, returns immediately.

        Args:
            selector: CSS-like selector string.

        Returns:
            List of ``AccessibleInfo`` for every matching element.
            Empty list if no element matches.

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
        """
        req = jab.Locator(selector, None, False)
        res: jab.RepeatedAccessibleInfo = self._call(self.__stub.get_info_from_all, req)
        return res.ac_infos

    def get_text(
        self, selector: str, timeout_ms: int | None, refresh_before_fail: bool
    ) -> str:
        """Get accessible text from the first element matching a selector.

        Args:
            selector: CSS-like selector string.
            timeout_ms: ``None`` for default (60s), ``0`` for no wait,
                or a positive integer for max milliseconds.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Returns:
            Accessible text content.

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: The element did not appear within the timeout.
        """
        req = jab.Locator(selector, timeout_ms, refresh_before_fail)
        res: jab.Text = self._call(self.__stub.get_text, req)
        return res.text

    def get_actions(
        self, selector: str, timeout_ms: int | None, refresh_before_fail: bool
    ) -> list[jab.Action]:
        """Get available accessible actions from the first element matching a selector.

        Args:
            selector: CSS-like selector string.
            timeout_ms: ``None`` for default (60s), ``0`` for no wait,
                or a positive integer for max milliseconds.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Returns:
            List of ``Action`` objects.

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: The element did not appear within the timeout.
        """
        req = jab.Locator(selector, timeout_ms, refresh_before_fail)
        res: jab.Actions = self._call(self.__stub.get_actions, req)
        return res.actions

    def do_action(
        self,
        action: jab.Action,
        selector: str,
        timeout_ms: int | None,
        refresh_before_fail: bool,
    ) -> None:
        """Perform an accessible action on the first element matching a selector.

        Args:
            action: The ``Action`` to perform.
            selector: CSS-like selector string.
            timeout_ms: ``None`` for default (60s), ``0`` for no wait,
                or a positive integer for max milliseconds.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: The element did not appear within the timeout.
            JabInternalError: The action could not be performed.
        """
        req = jab.DoActionRequest(
            jab.Locator(selector, timeout_ms, refresh_before_fail), action
        )
        self._call(self.__stub.do_action, req)

import re
from typing import Self
import time
from pathlib import Path

from win32gui import ShowWindow, SetForegroundWindow
from win32con import SW_SHOWMAXIMIZED

from .server import (
    JabRpaServer,
    _SERVER_PATH,
    _WAIT_FOR_SERVER_TIMEOUT,
    _INIT_SERVER_STEP,
)
from .client import JabRpaClient
from .types import VersionInfo, WindowInfo, AccessibleState
from .locator import Locator
from .errors import WindowNotFound

# For linking errors on mkdocstrings
from .errors import *  # noqa: F403

_WAIT_FOR_WINDOW_TIMEOUT: int = 60
_WAIT_FOR_WINDOW_STEP: int = 5


class JabDriver:
    """High-level driver that manages the JAB server and provides the main entry point.

    ``JabDriver`` handles the full lifecycle:

    1. Spawns the ``jab-rpa-server.exe`` subprocess
    2. Waits for a Java window matching the given title regex
    3. Brings the window to the foreground and maximizes it
    4. Selects the window on the server to build the accessibility tree

    Typical usage:

        with JabDriver("MyApp.*") as driver:
            loc = driver.locator("push_button[name='Clear']")
            loc.wait_for()
            loc.click()
    """

    def __init__(
        self,
        window_title: str | re.Pattern[str],
        *,
        server_path: Path = _SERVER_PATH,
        server_timeout: int = _WAIT_FOR_SERVER_TIMEOUT,
        server_step: int = _INIT_SERVER_STEP,
        window_timeout: int = _WAIT_FOR_WINDOW_TIMEOUT,
        window_step: int = _WAIT_FOR_WINDOW_STEP,
    ) -> None:
        """Configure the driver.

        Args:
            window_title: Regex pattern to match against Java window titles.
                The first window whose title matches via ``re.search()`` is
                selected. Can be a string or compiled pattern.
            server_path: Path to the ``jab-rpa-server.exe`` binary.
            server_timeout: Maximum seconds to wait for the server to start.
            server_step: Seconds between server readiness checks.
            window_timeout: Maximum seconds to wait for a matching window.
            window_step: Seconds between window discovery polls.
        """
        self._window_title: str | re.Pattern[str] = window_title
        self._server_path: Path = server_path
        self._server_timeout: int = server_timeout
        self._window_timeout: int = window_timeout
        self._server_step: int = server_step
        self._window_step: int = window_step
        self._server: JabRpaServer = JabRpaServer(
            server_path=self._server_path,
            server_timeout=self._server_timeout,
            step=self._server_step,
        )
        self._client: JabRpaClient = JabRpaClient()

    def start(self) -> None:
        """Start the server and connect to the target window.

        Spawns the JAB gRPC server, polls for Java windows whose title
        matches ``window_title``, brings the first match to the foreground,
        maximizes it, and selects it on the server.

        Raises:
            WindowNotFound: If no matching window appears within the timeout.
            ServerStoppedError: If the server process exits prematurely.
        """
        self._server.start()
        self._client.start()

        wait_start = time.monotonic()
        while time.monotonic() - wait_start <= self._window_timeout:
            windows = [
                window
                for window in self._client.list_java_windows()
                if re.search(self._window_title, window.title)
            ]
            if windows:
                window_info = windows[0]
                break
            time.sleep(self._window_step)
        else:
            raise WindowNotFound(
                f"Java window with title matching {self._window_title!r} not found within timeout {self._window_timeout!r}."
            )

        SetForegroundWindow(window_info.hwnd)
        ShowWindow(window_info.hwnd, SW_SHOWMAXIMIZED)
        self.window_info: WindowInfo = window_info
        self._client.select_window(window_info)

    def __enter__(self) -> Self:
        self.start()
        return self

    def stop(self) -> None:
        """Stop the client and terminate the server."""
        self._client.stop()
        self._server.stop()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()

    def list_java_windows(self) -> list[WindowInfo]:
        """List all Java windows currently detected by the server.

        Returns:
            List of ``WindowInfo`` with hwnd and title.
        """
        return self._client.list_java_windows()

    def switch_window(self, window_info: WindowInfo) -> None:
        """Set the active window to build the accessibility tree from.

        Args:
            window_info: A ``WindowInfo`` from ``list_java_windows()``.

        Raises:
            JabInvalidArgumentError: The HWND does not belong to a Java window.
            JabInternalError: The JAB bridge call failed.
        """
        self.window_info: WindowInfo = window_info
        self._client.select_window(window_info)

    def refresh_tree(self) -> None:
        """Rebuild the cached accessibility tree on the server.

        Call this after UI changes (e.g. a dialog opens) so subsequent
        locator queries see the updated tree.

        Raises:
            JabNoWindowError: No window has been selected yet.
            JabInternalError: The tree has no root node.
        """
        self._client.refresh_tree()

    def get_version_info(self) -> VersionInfo | None:
        """Get version info for the JAB bridge and server.

        Returns:
            ``VersionInfo`` if available, or ``None``.

        Raises:
            JabNoWindowError: No window has been selected yet.
            JabInternalError: The version info call failed.
        """
        return self._client.get_version_info()

    def locator(
        self,
        selector: str,
        require_states: set[AccessibleState] | None = None,
        exclude_states: set[AccessibleState] | None = None,
    ) -> Locator:
        """Build a locator to find elements in the accessibility tree.

        Args:
            selector: A CSS-selector-like query string (e.g.
                ``"push_button[name='Clear']"``).
            require_states: A set of states an element matching this locator must have
            exclude_states: A set of states an element matching this locator must not have

        Returns:
            A ``Locator`` bound to this driver.
        """
        return Locator(self, selector, require_states, exclude_states)

    def race(
        self,
        locators: list[Locator],
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> int:
        """Wait for any of the given locators to have a match.

        Args:
            locators: List of Locators.
            timeout_ms: ``None`` for default (60s), ``0`` for no wait,
                or a positive integer for max milliseconds.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Returns:
            Index of the first locator that matched.

        Raises:
            JabInvalidArgumentError: One or more locator selectors are malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: No locator matched within the timeout.
        """
        return self._client.race(
            [loc._selector for loc in locators], timeout_ms, refresh_before_fail
        )

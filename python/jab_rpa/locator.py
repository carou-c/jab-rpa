from __future__ import annotations

from typing import Any, TYPE_CHECKING

from grpc import RpcError
import pyautogui

from .types import Action
from .proto.jab import (
    Locator as _Locator,
)

if TYPE_CHECKING:
    from .driver import JabDriver


_WAIT_FOR_TIMEOUT = 60  # seconds
_WAIT_FOR_STEP = 5  # seconds


class LocatorNotFound(Exception):
    """Raised when a locator query returns no matching elements."""


class Locator:
    """Selector-based query for finding elements in the JAB accessibility tree.

    Uses a CSS-selector-like syntax to describe element attributes — role,
    states, pseudo-classes, and attribute comparisons — all in a single string.

    Usually created via ``JabDriver.locator()``. Call ``.locator()`` again
    on the result to extend the query with a descendant combinator (space):

        btn = driver.locator("push_button[name='Clear']").wait_for()
        btn.click()

        # Chaining adds a descendant combinator:
        btn = driver.locator("dialog").locator("push_button").wait_for()
        # Equivalent to: driver.locator("dialog push_button")
    """

    def __init__(
        self,
        driver: JabDriver,
        selector: str,
    ):
        """Wrap a driver and a selector string.

        Args:
            driver: The ``JabDriver`` instance.
            selector: A CSS-selector-like query string (e.g.
                ``"push_button[name='Clear']"``).
        """
        self._driver: JabDriver = driver
        self._selector: str = selector

    def locator(
        self,
        selector: str,
    ) -> "Locator":
        return Locator(self._driver, self._selector + " " + selector)

    def to_dict(self) -> dict[str, Any]:
        """All locator criteria as a dictionary."""
        return _Locator(self._selector).to_dict()

    def __str__(self) -> str:
        return f"Locator {self.to_dict()}"

    def wait_for(
        self,
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> None:
        """Poll the accessibility tree until a matching element appears.

        Args:
            timeout_ms: Maximum milliseconds to wait.

        Raises:
            ``LocatorNotFound``: If no element matches within the timeout.
        """
        try:
            self._driver._client.wait_for(
                self._selector, timeout_ms, refresh_before_fail
            )

        except RpcError as e:
            raise LocatorNotFound() from e

    def do_accessible_action(
        self,
        action: Action,
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> None:
        """Performs an accessible action on a matching element."""
        self._driver._client.do_action(
            action, self._selector, timeout_ms, refresh_before_fail
        )

    def accessible_click(
        self,
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> None:
        """Click a matching element using the JAB accessibility API (not pyautogui).
        This does not move the mouse."""
        self.do_accessible_action(Action("click"), timeout_ms, refresh_before_fail)

    def click(
        self,
        clicks: int = 1,
        interval: int | float = 0.0,
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> None:
        """Click at a matching element's center using pyautogui.

        Moves the mouse to the center of a matching element's bounding box
        and performs a pyautogui click.

        Args:
            clicks: Number of clicks (default 1).
            interval: Seconds between clicks (default 0).
        """
        info = self._driver._client.get_info(
            self._selector, timeout_ms, refresh_before_fail
        )
        center_x = info.x + (info.width / 2)
        center_y = info.y + (info.height / 2)
        pyautogui.click(center_x, center_y, clicks, interval)

    def click_and_type(
        self,
        text: str,
        clicks: int = 1,
        interval_text: int | float = 0.0,
        interval_clicks: int | float = 0.0,
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> None:
        """Click then type text into a matching element.

        Moves the mouse to a matching element's center, clicks, then types
        the given text using pyautogui.

        Args:
            text: Text to type after clicking.
            clicks: Number of clicks before typing.
            interval_text: Seconds between keystrokes.
            interval_clicks: Seconds between clicks.
        """
        info = self._driver._client.get_info(
            self._selector, timeout_ms, refresh_before_fail
        )
        center_x = info.x + (info.width / 2)
        center_y = info.y + (info.height / 2)
        pyautogui.click(center_x, center_y, clicks, interval_clicks)
        pyautogui.write(text, interval_text)

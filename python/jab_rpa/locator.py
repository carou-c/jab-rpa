from __future__ import annotations

import time
from typing import Any, TYPE_CHECKING

from .element import Element
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

    def matching(self) -> list[Element]:
        """Find all elements matching this locator.

        Returns:
            List of matching ``Element`` objects (may be empty).
        """
        return [
            Element(el, self._driver)
            for el in self._driver._client.find_elements(_Locator(self._selector))
        ]

    def first_matching(self) -> Element:
        """Find the first element matching this locator.

        Returns:
            The first matching ``Element``.

        Raises:
            LocatorNotFound: If no element matches.
        """
        matching = self.matching()
        if not matching:
            raise LocatorNotFound(
                f"Element with locator = {_Locator(self._selector)!r} not found"
            )
        return matching[0]

    def exists(self) -> bool:
        """Check if an element matching this locator exists.

        Returns:
            ``True`` if an element mathcing this locator exists,
            ``False`` else
        """
        return not not self.matching()

    def wait_for(
        self,
        timeout: int | float = _WAIT_FOR_TIMEOUT,
        sleep_step: int | float = _WAIT_FOR_STEP,
    ) -> Element:
        """Poll the accessibility tree until a matching element appears.

        Repeatedly calls ``matching()`` and refreshes the tree between
        attempts. Useful for waiting for dialogs or dynamic content.

        Args:
            timeout: Maximum seconds to wait.
            sleep_step: Seconds between polling attempts.

        Returns:
            The first matching ``Element``.

        Raises:
            LocatorNotFound: If no element matches within the timeout.
        """
        start = time.monotonic()
        while time.monotonic() - start <= timeout:
            matching = self.matching()
            if matching:
                return matching[0]
            time.sleep(sleep_step)
            self._driver.refresh_tree()
        raise LocatorNotFound(
            f"Element with locator = {_Locator(self._selector)!r} not found within timeout {timeout!r}"
        )

    def do_accessible_action(self, action: Action) -> None:
        """Performs an accessible action on the first matching element."""
        self.first_matching().do_accessible_action(action)

    def accessible_click(self) -> None:
        """Click using the JAB accessibility API (not pyautogui).
        This does not move the mouse."""
        self.first_matching().accessible_click()

    def click(self, clicks: int = 1, interval: int | float = 0.0) -> None:
        """Click at the first matching element's center using pyautogui.

        Moves the mouse to the center of the first matching element's bounding box
        and performs a pyautogui click.

        Args:
            clicks: Number of clicks (default 1).
            interval: Seconds between clicks (default 0).
        """
        self.first_matching().click(clicks, interval)

    def click_and_type(
        self,
        text: str,
        clicks: int = 1,
        interval_text: int | float = 0.0,
        interval_clicks: int | float = 0.0,
    ) -> None:
        """Click then type text into the first matching element.

        Moves the mouse to the first matching element's center, clicks, then types
        the given text using pyautogui.

        Args:
            text: Text to type after clicking.
            clicks: Number of clicks before typing.
            interval_text: Seconds between keystrokes.
            interval_clicks: Seconds between clicks.
        """
        self.first_matching().click_and_type(text, clicks, interval_text, interval_clicks)

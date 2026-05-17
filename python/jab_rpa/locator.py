from __future__ import annotations

import time
from typing import Any, TYPE_CHECKING

from .element import Element
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
    """Structured locator query for finding elements in the JAB accessibility tree.

    Builds a query by specifying element attributes. All criteria are
    keyword-only and combine as AND conditions. String criteria supports
    regex matching.

    Usually created via ``JabDriver.locator()``, then chained with ``.child()``
    or ``.descendant()`` for hierarchical navigation:

        btn = (driver
               .locator(role="dialog")
               .child(role="push button", name="Confirm")
               .wait_for())
    """

    def __init__(
        self,
        driver: JabDriver,
        selector: str,
    ):
        """ """
        self._driver: JabDriver = driver
        self._locator: _Locator = _Locator(selector)

    def locator(
        self,
        selector: str,
    ) -> "Locator":
        return Locator(self._driver, self._locator.selector + " " + selector)

    def to_dict(self) -> dict[str, Any]:
        """All locator criteria as a dictionary."""
        return self._locator.to_dict()

    def __str__(self) -> str:
        return f"Locator {self.to_dict()}"

    def matching(self) -> list[Element]:
        """Find all elements matching this locator.

        Returns:
            List of matching ``Element`` objects (may be empty).
        """
        return self._driver.matching(self)

    def first_matching(self) -> Element:
        """Find the first element matching this locator.

        Returns:
            The first matching ``Element``.

        Raises:
            LocatorNotFound: If no element matches.
        """
        matching = self.matching()
        if not matching:
            raise LocatorNotFound(f"Element with locator = {self._locator!r} not found")
        return matching[0]

    def accessible_click(self) -> None:
        """Click the first matching element using the JAB accessibility API
        (not pyautogui). This does not move the mouse.

        Shortcut for ``self.first_matching().accessible_click()``.

        Raises:
            LocatorNotFound: If no element matches.
        """
        self.first_matching().accessible_click()

    def click(self, clicks: int = 1, interval: int | float | None = None) -> None:
        """Click the first matching element's center using pyautogui.

        Moves the mouse to the center of the element's bounding box
        and performs a pyautogui click.

        Shortcut for ``self.first_matching().click(clicks, interval)``.

        Args:
            clicks: Number of clicks.
            interval: Seconds between clicks.

        Raises:
            LocatorNotFound: If no element matches.
        """
        self.first_matching().click(clicks, interval)

    def click_and_type(
        self,
        text: str,
        clicks: int = 1,
        interval_text: int | float | None = None,
        interval_clicks: int | float | None = None,
    ) -> None:
        """Click then type into the first matching element.

        Moves the mouse to the element's center, clicks, then types
        the given text using pyautogui.

        Shortcut for ``self.first_matching().click_and_type(...)``.

        Args:
            text: Text to type.
            clicks: Number of clicks before typing.
            interval_text: Seconds between keystrokes.
            interval_clicks: Seconds between clicks.

        Raises:
            LocatorNotFound: If no element matches.
        """
        self.first_matching().click_and_type(
            text, clicks, interval_text, interval_clicks
        )

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
            f"Element with locator = {self._locator!r} not found within timeout {timeout!r}"
        )

from __future__ import annotations

from typing import Any, TYPE_CHECKING

import pyautogui

from .proto.jab import (
    Locator as _Locator,
)

from .types import Action, AccessibleState, AccessibleInfo

# For linking errors on mkdocstrings
from .errors import *  # noqa: F403

if TYPE_CHECKING:
    from .driver import JabDriver


_WAIT_FOR_TIMEOUT = 60  # seconds
_WAIT_FOR_STEP = 5  # seconds


class Locator:
    """Selector-based query for finding elements in the JAB accessibility tree.

    Uses a CSS-selector-like syntax to describe element attributes — role,
    states, pseudo-classes, and attribute comparisons — all in a single string.

    Usually created via ``JabDriver.locator()``. Call ``.locator()`` again
    on the result to extend the query with a descendant combinator (space):

        loc = driver.locator("push_button[name='Clear']")
        loc.wait_for()
        loc.click()

        # Chaining adds a descendant combinator:
        loc = driver.locator("dialog").locator("push_button")
        loc.wait_for()
        # Equivalent to: driver.locator("dialog push_button")
    """

    def __init__(
        self,
        driver: JabDriver,
        selector: str,
        has: list["Locator"] | None = None,
        require_states: set[AccessibleState] | None = None,
        exclude_states: set[AccessibleState] | None = None,
    ):
        """Wrap a driver and a selector string.

        Args:
            driver: The ``JabDriver`` instance.
            selector: A CSS-selector-like query string (e.g.
                ``"push_button[name='Clear']"``).
            has: A list of locators; each matching element must contain a descendant
                that satisfies every locator in the list. Appends ``:has()`` pseudo-classes
                to the selector (AND semantics).
            require_states: A set of states an element matching this locator must have
            exclude_states: A set of states an element matching this locator must not have
        """
        self._driver: JabDriver = driver
        has = has or []
        require_states = require_states or set()
        exclude_states = exclude_states or set()
        self._selector: str = (
            selector.rstrip()
            + "".join(":has(" + loc._selector + ")" for loc in has)
            + "".join(":require-state(" + state + ")" for state in require_states)
            + "".join(":exclude-state(" + state + ")" for state in exclude_states)
        )

    def locator(
        self,
        selector: str,
        has: list["Locator"] | None = None,
        require_states: set[AccessibleState] | None = None,
        exclude_states: set[AccessibleState] | None = None,
    ) -> "Locator":
        """Extend the query with a descendant combinator.

        Concatenates the new selector with a space, equivalent to
        a descendant combinator in CSS.

        Args:
            selector: Additional CSS-like selector to append.
            has: A list of locators; each matching element must contain a descendant
                that satisfies every locator in the list. Appends ``:has()`` pseudo-classes
                to the selector (AND semantics).
            require_states: A set of states an element matching this locator must have
            exclude_states: A set of states an element matching this locator must not have

        Returns:
            A new ``Locator`` with the combined selector.
        """
        return Locator(
            self._driver,
            f"{self._selector} {selector}",
            has,
            require_states,
            exclude_states,
        )

    def filter(
        self,
        has: list["Locator"] | None = None,
        require_states: set[AccessibleState] | None = None,
        exclude_states: set[AccessibleState] | None = None,
    ) -> "Locator":
        """Extend the query by requiring more states/sub-locators.

        Args:
            has: A list of locators; each matching element must contain a descendant
                that satisfies every locator in the list. Appends ``:has()`` pseudo-classes
                to the selector (AND semantics).
            require_states: A set of states an element matching this locator must have
            exclude_states: A set of states an element matching this locator must not have

        Returns:
            A new ``Locator`` with the added filtering.
        """
        return Locator(
            self._driver, self._selector, has, require_states, exclude_states
        )

    def get_info(
        self,
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> AccessibleInfo:
        """Get accessible info from the first matching element.

        Args:
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
        return self._driver._client.get_info(
            self._selector, timeout_ms, refresh_before_fail
        )

    def get_info_from_all(
        self,
    ) -> list[AccessibleInfo]:
        """Get accessible info from all matching elements.
        Does not wait for a matching element, returns immediately.

        Returns:
            List of ``AccessibleInfo`` for every matching element.
            Empty list if no element matches.

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
        """
        return self._driver._client.get_info_from_all(self._selector)

    def exists(
        self,
    ) -> bool:
        """
        Returns:
            ``True`` if a matching element exists,
            ``False`` otherwise

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
        """
        return not not self.get_info_from_all()

    def get_text(
        self,
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> str:
        """Get accessible text from the first matching element.

        Args:
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
        return self._driver._client.get_text(
            self._selector, timeout_ms, refresh_before_fail
        )

    def get_actions(
        self,
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> list[Action]:
        """Get available accessible actions from the first matching element.

        Args:
            timeout_ms: ``None`` for default (60s), ``0`` for no wait,
                or a positive integer for max milliseconds.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Returns:
            List of ``Action`` objects (each has a ``name`` field).

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: The element did not appear within the timeout.
        """
        return self._driver._client.get_actions(
            self._selector, timeout_ms, refresh_before_fail
        )

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
        """Block until a matching element appears in the accessibility tree.

        Args:
            timeout_ms: ``None`` for default (60s), ``0`` for no wait (fail fast),
                or a positive integer for max milliseconds to wait.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: The element did not appear within the timeout.
        """
        self._driver._client.wait_for(self._selector, timeout_ms, refresh_before_fail)

    def do_accessible_action(
        self,
        action: Action,
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> None:
        """Perform an accessible action on a matching element.

        Args:
            action: The ``Action`` to perform (e.g. ``Action("click")``).
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
        self._driver._client.do_action(
            action, self._selector, timeout_ms, refresh_before_fail
        )

    def accessible_click(
        self,
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> None:
        """Click a matching element via the JAB accessibility API.

        This does **not** move the mouse — the click is performed
        programmatically through the accessibility bridge.

        Args:
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
            interval: Seconds between clicks (default 0.0).
            timeout_ms: ``None`` for default (60s), ``0`` for no wait,
                or a positive integer for max milliseconds.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: The element did not appear within the timeout.
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
        """Click then type text into a matching element using pyautogui.

        Moves the mouse to a matching element's center, clicks, then types
        the given text.

        Args:
            text: Text to type after clicking.
            clicks: Number of clicks before typing (default 1).
            interval_text: Seconds between keystrokes (default 0.0).
            interval_clicks: Seconds between clicks (default 0.0).
            timeout_ms: ``None`` for default (60s), ``0`` for no wait,
                or a positive integer for max milliseconds.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Raises:
            JabInvalidArgumentError: The selector string is malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: The element did not appear within the timeout.
        """
        info = self._driver._client.get_info(
            self._selector, timeout_ms, refresh_before_fail
        )
        center_x = info.x + (info.width / 2)
        center_y = info.y + (info.height / 2)
        pyautogui.click(center_x, center_y, clicks, interval_clicks)
        pyautogui.write(text, interval_text)

    def race(
        self,
        others: list["Locator"],
        timeout_ms: int | None = None,
        refresh_before_fail: bool = True,
    ) -> int:
        """Wait for any of the given locators to have a match.

        Args:
            others: List of Locators.
            timeout_ms: ``None`` for default (60s), ``0`` for no wait,
                or a positive integer for max milliseconds.
            refresh_before_fail: If true, refresh the tree after timeout
                before the final check.

        Returns:
            Index of the first locator that matched (self -> 0, others -> 1..)

        Raises:
            JabInvalidArgumentError: One or more locator selectors are malformed.
            JabNoWindowError: No window has been selected yet.
            JabTimeoutError: No locator matched within the timeout.
        """
        return self._driver.race([self] + others, timeout_ms, refresh_before_fail)

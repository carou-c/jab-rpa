from __future__ import annotations

import time
from typing import Any, TYPE_CHECKING

from .element import Element
from .proto.jab import (
    Locator as _Locator,
    StringLocator,
    IndexLocator,
    AscendantLocator,
    DescendantLocator,
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
        *,
        name: str | None = None,
        role: str | None = None,
        description: str | None = None,
        text: str | None = None,
        has_state: list[str] | None = None,
        not_has_state: list[str] | None = None,
        index_in_parent: int | None = None,
        has_children: list["Locator"] | None = None,
        has_descendants: list["Locator"] | None = None,
        name_regex: bool = True,
        role_regex: bool = False,
        description_regex: bool = True,
        text_regex: bool = True,
        ascendant: AscendantLocator | None = None,
    ):
        """All parameters are keyword-only.

        Args:
            driver: The ``JabDriver`` instance to query against.
            name: Element's accessible name (regex by default).
            role: Element's accessible role (regex by default).
            description: Element's accessible description (regex by default).
            text: Element's text content (regex by default).
            has_state: Required states (e.g. ``["enabled"]``).
            not_has_state: Forbidden states (e.g. ``["disabled"]``).
            index_in_parent: Exact index in parent.
            has_children: Locators that must match at least one child each.
            has_descendants: Locators that must match at least one descendant each.
            name_regex: If False, match ``name`` exactly.
            role_regex: If False, match ``role`` exactly.
            description_regex: If False, match ``description`` exactly.
            text_regex: If False, match ``text`` exactly.
            ascendant: Internal — used by ``child()`` and ``descendant()``.
        """
        self._driver: JabDriver = driver

        has_state: list[str] = has_state or []
        not_has_state: list[str] = not_has_state or []
        has_children: list[Locator] = has_children or []
        has_descendants: list[Locator] = has_descendants or []

        self._locator: _Locator = _Locator(
            name=StringLocator(name, name_regex) if name is not None else None,
            role=StringLocator(role, role_regex) if role is not None else None,
            description=StringLocator(description, description_regex)
            if description is not None
            else None,
            text=StringLocator(text, text_regex) if text is not None else None,
            has_state=has_state,
            not_has_state=not_has_state,
            index_in_parent=IndexLocator(index_in_parent)
            if index_in_parent is not None
            else None,
            ascendant=ascendant,
            descendants=[
                DescendantLocator(locator._locator, True) for locator in has_children
            ]
            + [
                DescendantLocator(locator._locator, False)
                for locator in has_descendants
            ],
        )

    def child(
        self,
        *,
        name: str | None = None,
        role: str | None = None,
        description: str | None = None,
        text: str | None = None,
        has_state: list[str] | None = None,
        not_has_state: list[str] | None = None,
        index_in_parent: int | None = None,
        has_children: list["Locator"] | None = None,
        has_descendants: list["Locator"] | None = None,
        name_regex: bool = True,
        role_regex: bool = False,
        description_regex: bool = True,
        text_regex: bool = True,
    ) -> "Locator":
        """Narrow the search to direct children of the current locator.

        Creates a new ``Locator`` that matches elements whose direct
        parent matches the current locator. Accepts the same keyword
        arguments as ``Locator.__init__``.

        Returns:
            A new ``Locator`` with the parent constraint applied.
        """
        return Locator(
            self._driver,
            name=name,
            role=role,
            description=description,
            text=text,
            has_state=has_state,
            not_has_state=not_has_state,
            index_in_parent=index_in_parent,
            has_children=has_children,
            has_descendants=has_descendants,
            name_regex=name_regex,
            role_regex=role_regex,
            description_regex=description_regex,
            text_regex=text_regex,
            ascendant=AscendantLocator(self._locator, True),
        )

    def descendant(
        self,
        *,
        name: str | None = None,
        role: str | None = None,
        description: str | None = None,
        text: str | None = None,
        has_state: list[str] | None = None,
        not_has_state: list[str] | None = None,
        index_in_parent: int | None = None,
        has_children: list["Locator"] | None = None,
        has_descendants: list["Locator"] | None = None,
        name_regex: bool = True,
        role_regex: bool = False,
        description_regex: bool = True,
        text_regex: bool = True,
    ) -> "Locator":
        """Narrow the search to descendants of the current locator.

        Creates a new ``Locator`` that matches elements with any ancestor
        (not just the direct parent) matching the current locator. Accepts
        the same keyword arguments as ``Locator.__init__``.

        Returns:
            A new ``Locator`` with the ancestor constraint applied.
        """
        return Locator(
            self._driver,
            name=name,
            role=role,
            description=description,
            text=text,
            has_state=has_state,
            not_has_state=not_has_state,
            index_in_parent=index_in_parent,
            has_children=has_children,
            has_descendants=has_descendants,
            name_regex=name_regex,
            role_regex=role_regex,
            description_regex=description_regex,
            text_regex=text_regex,
            ascendant=AscendantLocator(self._locator, False),
        )

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

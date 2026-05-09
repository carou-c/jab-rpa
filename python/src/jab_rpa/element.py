from __future__ import annotations

from typing import Any, TYPE_CHECKING
import pyautogui

from .proto.jab import Element as _Element

if TYPE_CHECKING:
    from .driver import JabDriver


class Element:
    """Wraps a JAB accessible element with readable properties and interaction methods.

    Provides direct access to all element attributes exposed by the
    Java Access Bridge, plus convenience methods for clicking and typing.

    Instances are created by ``Locator.matching()``, ``Locator.first_matching()``,
    ``Locator.wait_for()``, ``Element.children()``, and ``Element.parent()``.
    """

    def __init__(self, element: _Element, driver: JabDriver):
        self._element: _Element = element
        self._driver: JabDriver = driver

    @property
    def name(self) -> str:
        """Accessible name of the element."""
        return self._element.name

    @property
    def role(self) -> str:
        """Accessible role (e.g. ``"push button"``, ``"checkbox"``)."""
        return self._element.role

    @property
    def states(self) -> list[str]:
        """Current states as localized strings (e.g. ``["enabled", "focusable"]``)."""
        return self._element.states

    @property
    def states_en_us(self) -> list[str]:
        """Current states as en-US strings (e.g. ``["enabled", "focusable"]``)."""
        return self._element.states_en_us

    @property
    def description(self) -> str:
        """Accessible description."""
        return self._element.description

    @property
    def text(self) -> str:
        """Text content of the element."""
        return self._element.text

    @property
    def x(self) -> int:
        """Screen X coordinate of the element's bounding box origin."""
        return self._element.x

    @property
    def y(self) -> int:
        """Screen Y coordinate of the element's bounding box origin."""
        return self._element.y

    @property
    def width(self) -> int:
        """Width of the element's bounding box in pixels."""
        return self._element.width

    @property
    def height(self) -> int:
        """Height of the element's bounding box in pixels."""
        return self._element.height

    @property
    def accessible_action(self) -> bool:
        """Whether the element supports accessible actions (e.g. clicking via JAB)."""
        return self._element.accessible_action

    @property
    def accessible_text(self) -> bool:
        """Whether the element supports accessible text operations."""
        return self._element.accessible_text

    @property
    def accessible_selection(self) -> bool:
        """Whether the element supports accessible selection operations."""
        return self._element.accessible_selection

    @property
    def children_count(self) -> int:
        """Number of direct children in the accessibility tree."""
        return self._element.children_count

    @property
    def index_in_parent(self) -> int:
        """Index of this element within its parent's children list."""
        return self._element.index_in_parent

    def children(self) -> list["Element"]:
        """Get the direct children of this element.

        Returns:
            List of child ``Element`` objects.
        """
        return self._driver.get_children(self)

    def parent(self) -> "Element | None":
        """Get the parent of this element.

        Returns:
            The parent ``Element`` or ``None`` if this is the root.
        """
        return self._driver.get_parent(self)

    def accessible_click(self) -> None:
        """Click using the JAB accessibility API (not pyautogui).
        This does not move the mouse."""
        self._driver.accessible_click(self)

    def click(self, clicks: int = 1, interval: int | float | None = None) -> None:
        """Click at the element's center using pyautogui.

        Moves the mouse to the center of the element's bounding box
        and performs a pyautogui click.

        Args:
            clicks: Number of clicks (default 1).
            interval: Seconds between clicks (default 0).
        """
        interval = interval or 0.0
        center_x = self.x + (self.width / 2)
        center_y = self.y + (self.height / 2)
        pyautogui.click(center_x, center_y, clicks, interval)

    def click_and_type(
        self,
        text: str,
        clicks: int = 1,
        interval_text: int | float | None = None,
        interval_clicks: int | float | None = None,
    ) -> None:
        """Click then type text into the element.

        Moves the mouse to the element's center, clicks, then types
        the given text using pyautogui.

        Args:
            text: Text to type after clicking.
            clicks: Number of clicks before typing.
            interval_text: Seconds between keystrokes.
            interval_clicks: Seconds between clicks.
        """
        interval_text = interval_text or 0.0
        interval_clicks = interval_clicks or 0.0
        center_x = self.x + (self.width / 2)
        center_y = self.y + (self.height / 2)
        pyautogui.click(center_x, center_y, clicks, interval_clicks)
        pyautogui.write(text, interval_text)

    def to_dict(self) -> dict[str, Any]:
        """All element properties as a dictionary."""
        return self._element.to_dict()

    def __str__(self) -> str:
        return f"Element {self.to_dict()}"

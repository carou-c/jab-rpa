from __future__ import annotations

from typing import Any, TYPE_CHECKING
import pyautogui

from .proto.jab import Element as _Element

if TYPE_CHECKING:
    from .driver import JabDriver


class Element:
    def __init__(self, element: _Element, driver: JabDriver):
        self._element: _Element = element
        self._driver: JabDriver = driver

    @property
    def name(self) -> str:
        return self._element.name

    @property
    def role(self) -> str:
        return self._element.role

    @property
    def states(self) -> list[str]:
        return self._element.states

    @property
    def states_en_us(self) -> list[str]:
        return self._element.states_en_us

    @property
    def description(self) -> str:
        return self._element.description

    @property
    def text(self) -> str:
        return self._element.text

    @property
    def x(self) -> int:
        return self._element.x

    @property
    def y(self) -> int:
        return self._element.y

    @property
    def width(self) -> int:
        return self._element.width

    @property
    def height(self) -> int:
        return self._element.height

    @property
    def accessible_action(self) -> bool:
        return self._element.accessible_action

    @property
    def accessible_text(self) -> bool:
        return self._element.accessible_text

    @property
    def accessible_selection(self) -> bool:
        return self._element.accessible_selection

    @property
    def children_count(self) -> int:
        return self._element.children_count

    @property
    def index_in_parent(self) -> int:
        return self._element.index_in_parent

    def children(self) -> list["Element"]:
        children: list[Element] = []
        for handle in self._element.children_handles:
            child = self._driver._client.get_element_from_handle(handle)
            if child is not None:
                children.append(Element(child, self._driver))
        return children

    def parent(self) -> "Element | None":
        handle = self._element.parent_handle
        parent = None
        if handle is not None:
            parent = self._driver._client.get_element_from_handle(handle)
        if parent is not None:
            return Element(parent, self._driver)

    def accessible_click(self) -> None:
        self._driver._client.click_element(self._element)

    def click(self, clicks: int = 1, interval: int | float | None = None) -> None:
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
        interval_text = interval_text or 0.0
        interval_clicks = interval_clicks or 0.0
        center_x = self.x + (self.width / 2)
        center_y = self.y + (self.height / 2)
        pyautogui.click(center_x, center_y, clicks, interval_clicks)
        pyautogui.write(text, interval_text)

    def to_dict(self) -> dict[str, Any]:
        return self._element.to_dict()

    def __str__(self) -> str:
        return f"Element {self.to_dict()}"

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
    """Raised when a locator is not found in the context tree"""


class Locator:
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
        role_regex: bool = True,
        description_regex: bool = True,
        text_regex: bool = True,
        ascendant: AscendantLocator | None = None,
    ):
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
        role_regex: bool = True,
        description_regex: bool = True,
        text_regex: bool = True,
    ) -> "Locator":
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
        role_regex: bool = True,
        description_regex: bool = True,
        text_regex: bool = True,
    ) -> "Locator":
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
        return self._locator.to_dict()

    def __str__(self) -> str:
        return f"Locator {self.to_dict()}"

    def matching(self) -> list[Element]:
        return self._driver.matching(self)

    def first_matching(self) -> Element:
        matching = self.matching()
        if not matching:
            raise LocatorNotFound(f"Element with locator = {self._locator!r} not found")
        return matching[0]

    def accessible_click(self) -> None:
        self.first_matching().accessible_click()

    def click(self, clicks: int = 1, interval: int | float | None = None) -> None:
        self.first_matching().click(clicks, interval)

    def click_and_type(
        self,
        text: str,
        clicks: int = 1,
        interval_text: int | float | None = None,
        interval_clicks: int | float | None = None,
    ) -> None:
        self.first_matching().click_and_type(
            text, clicks, interval_text, interval_clicks
        )

    def wait_for(
        self,
        timeout: int | float = _WAIT_FOR_TIMEOUT,
        sleep_step: int | float = _WAIT_FOR_STEP,
    ) -> Element:
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

from typing import Any
from .jab import (
    Locator as _Locator,
    StringLocator,
    IndexLocator,
    AscendantLocator,
    DescendantLocator,
)


class Locator:
    def __init__(
        self,
        *,
        name: str | None = None,
        role: str | None = None,
        description: str | None = None,
        text: str | None = None,
        index_in_parent: int | None = None,
        has_children: list[Locator] | None = None,
        has_descendants: list[Locator] | None = None,
        name_regex: bool = True,
        role_regex: bool = True,
        description_regex: bool = True,
        text_regex: bool = True,
        ascendant: AscendantLocator | None = None,
    ):
        has_children: list[Locator] = has_children or []
        has_descendants: list[Locator] = has_descendants or []

        self._locator: _Locator = _Locator(
            name=StringLocator(name, name_regex) if name is not None else None,
            role=StringLocator(role, role_regex) if role is not None else None,
            description=StringLocator(description, description_regex)
            if description is not None
            else None,
            text=StringLocator(text, text_regex) if text is not None else None,
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
        index_in_parent: int | None = None,
        has_children: list[Locator] | None = None,
        has_descendants: list[Locator] | None = None,
        name_regex: bool = True,
        role_regex: bool = True,
        description_regex: bool = True,
        text_regex: bool = True,
    ) -> Locator:
        return Locator(
            name=name,
            role=role,
            description=description,
            text=text,
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
        index_in_parent: int | None = None,
        has_children: list[Locator] | None = None,
        has_descendants: list[Locator] | None = None,
        name_regex: bool = True,
        role_regex: bool = True,
        description_regex: bool = True,
        text_regex: bool = True,
    ) -> Locator:
        return Locator(
            name=name,
            role=role,
            description=description,
            text=text,
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

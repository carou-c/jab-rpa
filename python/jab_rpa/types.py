"""Convenience re-exports of key types.

Provides a single import target for types used across the package:

- **WindowInfo** — info about a detected Java window (hwnd and title)
- **VersionInfo** — JAB and server version information
- **Action** — an accessible action that can be performed on an element
- **Locator** — selector-based element query builder
- **AccessibleState** — Literal with JAB Accessible States
- **AccessibleInfo** — Accessible information from a JAB element
"""

from typing import Literal

from .proto.jab import Action, WindowInfo, VersionInfo, AccessibleInfo

from .locator import Locator

type AccessibleState = Literal[
    "active",
    "armed",
    "busy",
    "checked",
    "collapsed",
    "editable",
    "enabled",
    "expandable",
    "expanded",
    "focusable",
    "focused",
    "horizontal",
    "iconified",
    "indeterminate",
    "manages_descendants",
    "modal",
    "multi_line",
    "multiselectable",
    "opaque",
    "pressed",
    "resizable",
    "selectable",
    "selected",
    "showing",
    "single_line",
    "transient",
    "truncated",
    "vertical",
    "visible",
]


__all__ = [
    "Action",
    "WindowInfo",
    "VersionInfo",
    "Locator",
    "AccessibleState",
    "AccessibleInfo",
]

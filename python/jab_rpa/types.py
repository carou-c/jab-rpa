"""Convenience re-exports of key types.

Provides a single import target for types used across the package:

- **WindowInfo** — info about a detected Java window (hwnd and title)
- **VersionInfo** — JAB and server version information
- **Element** — wrapper around a JAB accessible element
- **Locator** — structured locator query builder
"""

from .proto.jab import Action, WindowInfo, VersionInfo

from .element import Element
from .locator import Locator

__all__ = [
    "Action",
    "WindowInfo",
    "VersionInfo",
    "Element",
    "Locator",
]

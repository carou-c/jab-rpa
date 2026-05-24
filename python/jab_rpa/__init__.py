"""Python client for jab-rpa — automate 32-bit Java applications.

The public API is composed of two classes:

- **JabDriver** — manages server lifecycle and window selection
- **Locator** — builds queries to find and interact with elements

Typical usage:

    with JabDriver("MyApp.*") as driver:
        loc = driver.locator("push_button[name='Clear']")
        loc.wait_for()
        loc.click()
"""

from .driver import JabDriver
from .locator import Locator

__all__ = ["JabDriver", "Locator"]

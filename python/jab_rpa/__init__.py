"""Python client for jab-rpa — automate 32-bit Java applications.

The public API is composed of three classes:

- **JabDriver** — manages server lifecycle and window selection
- **Locator** — builds queries to find elements in the accessibility tree
- **Element** — wraps a JAB element with properties and interaction methods

Typical usage:

    with JabDriver("MyApp.*") as driver:
        btn = driver.locator("push_button[name='Clear']").wait_for()
        btn.click()
"""

from .driver import JabDriver
from .locator import Locator

__all__ = ["JabDriver", "Locator", "Element"]

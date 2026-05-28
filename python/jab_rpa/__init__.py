"""Python client for jab-rpa — automate 32-bit Java applications.

The public API is composed of one class:

- **JabDriver** — manages server lifecycle and window selection

Typical usage:

    with JabDriver("MyApp.*") as driver:
        loc = driver.locator("push_button[name='Clear']")
        loc.wait_for()
        loc.click()
"""

from .driver import JabDriver

__all__ = [
    "JabDriver",
]

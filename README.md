# jab-rpa

This project is a solution to a problem I had:

1. Automate processes that use a 32-bit Java desktop app
2. Do that, necessarily, from a 64-bit Python runtime

(1) means the existing tooling (like robocorp's excellent
[rpaframework.RPA.JavaAccessBridge](https://rpaframework.org/libraries/javaaccessbridge/python.html))
does not work properly.

(2) means, whatever the solution, it will involve some kind of IPC. My tool of
choice for this was gRPC.

This project exposes a fairly ergonomic Python API for developing RPA
workflows against Java desktop applications. The server includes multi-JVM
support (32-bit and 64-bit, Java 8 through 25) and a selector engine with
CSS-like syntax.

## Quickstart

1. Install from PyPI with the Java version extra:

   ```bash
   pip install jab-rpa[java8]      # Java 8 (proven in production)
   pip install jab-rpa[java17]     # Java 17
   # etc.
   ```

2. Use it:

```python
from jab_rpa import JabDriver

with JabDriver("My Java Application.*") as driver:
    button = driver.locator("push_button[name='Clear']")
    button.click()
```

See the [full documentation](https://carou-c.github.io/jab-rpa/) for details.

## Disclaimer

This package runs **only** on Windows, and supports both 32-bit and 64-bit JVMs.
If you exclusively target 64-bit JVMs,
[rpaframework.RPA.JavaAccessBridge](https://rpaframework.org/libraries/javaaccessbridge/python.html)
is a more mature option.

**Note:** Only Java 8 has been proven to work reliably in production. Java 11,
17, 21 and 25 are supported experimentally.

Bug reports, issues, discussions and contributions are always welcome.

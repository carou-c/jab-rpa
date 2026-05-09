# jab-rpa

This project is a "quick and dirty" solution to a problem I had:

1. Automate processes that use a 32-bit Java desktop app
2. Do that, necessarily, from a 64-bit Python runtime

(1) means the existing tooling (like robocorp's excellent
[rpaframework.RPA.JavaAccessBridge](https://rpaframework.org/libraries/javaaccessbridge/python.html))
does not work properly.

(2) means, whatever the solution, it will involve some kind of IPC. My tool of
choice for this was gRPC.

This project contains what I consider the "bare minimum" to develop RPA. It
exposes a fairly ergonomic Python API, but lacks many performance optimizations
and probably has a few bugs here and there.

## Quickstart

1. Download the latest `.whl` from the
   [releases page](https://github.com/carou-c/jab-rpa/releases).
2. Install it: `python -m pip install jab_rpa-x.y.z-py3-none-any.whl`
3. Use it:

```python
from jab_rpa import JabDriver

with JabDriver("My Java Application.*") as driver:
    button = driver.locator(role="push button", name="Clear").wait_for()
    button.click()
```

See the [full documentation](docs/index.md) for details.

## Disclaimer

This package is meant **ONLY** for windows, and **ONLY** targeting 32-bit JVMs.

If you are targeting a 64-bit JVM, use
[rpaframework.RPA.JavaAccessBridge](https://rpaframework.org/libraries/javaaccessbridge/python.html)
instead - it's a much more robust tool.

Bug reports, issues, discussions and contributions are always welcome.

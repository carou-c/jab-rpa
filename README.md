# jab-rpa

This project is a "quick and dirty" solution to a problem I had:

1. Automate processes that use a 32-bit Java desktop app
2. Do that, necessarily, from a 64-bit Python runtime

(1) means the existing tooling does not work properly. If, in your use-case, you can use
a 64-bit JVM, I highly recommend using robocorp's excellent [rpaframework.RPA.JavaAccessBridge](https://rpaframework.org/libraries/javaaccessbridge/python.html)
module. It is a much more robust tool.

(2) means, whatever the solution, it will involve some kind of IPC. My tool of choice for this
was gRPC.

This project contains what I consider the "bare minimum" to develop RPA. It exposes a fairly
ergonomic Python API, but lacks many performance optimizations and probably has a few bugs
here and there.

## Quickstart

1. Download the latest release `.whl` file.
2. Install it with your package manager of choice. For example, for pip, this would be done
via `python -m pip install jab_rpa-x.y.z-py3-none-any.whl`.
3. Have fun :)

## Disclaimer

This package is meant **ONLY** for windows, and **ONLY** targeting 32-bit JVMs.
Again, if you are targeting a 64-bit JVM, check out [rpaframework.RPA.JavaAccessBridge](https://rpaframework.org/libraries/javaaccessbridge/python.html).

Bug reports, issues, discussions and contributions are always welcome.

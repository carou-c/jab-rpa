# jab-rpa

Automate 32-bit Java desktop applications from Python.

## Problem

Standard RPA tools like
[robocorp's RPA.JavaAccessBridge](https://rpaframework.org/libraries/javaaccessbridge/python.html)
only work with 64-bit JVMs. If your target application runs on a 32-bit JVM
(e.g. JRE 1.8), those tools won't work.

## Solution

`jab-rpa` bridges this gap with a two-component architecture:

```text
┌─────────────────────────────────────┐
│  64-bit Python RPA Client           │
│  (jab_rpa package — this library)   │
└──────────────┬──────────────────────┘
               │ gRPC (localhost:port)
               ▼
┌─────────────────────────────────────┐
│  jab-rpa-server.exe (32-bit)        │
│  ┌──────────┐  ┌──────────────────┐ │
│  │ gRPC     │  │ Java Access      │ │
│  │ Service  │◄─┤ Bridge Wrapper   │ │
│  └──────────┘  └──────────────────┘ │
└──────────────────┬──────────────────┘
                   ▼
     WindowsAccessBridge-32.dll
                   │
     ┌─────────────────────────────┐
     │  32-bit Java App (JRE 1.8)  │
     └─────────────────────────────┘
```

1. **`jab-rpa-server.exe`** — a 32-bit Rust gRPC server that loads
   `WindowsAccessBridge-32.dll` and exposes the Java Accessibility Bridge over
   gRPC.
2. **`jab_rpa`** — the Python client library (this package) that spawns the
   server and provides an ergonomic API.

## Requirements

- **Windows only** (32-bit and 64-bit both work for the Python side)
- **32-bit or 64-bit Java application** (JRE 1.8+ with Java Access Bridge enabled)
- **Python 3.12+**

If you exclusively target 64-bit JVMs,
[robocorp's RPA.JavaAccessBridge](https://rpaframework.org/libraries/javaaccessbridge/python.html)
is a more mature option.

**Note:** Only Java 8 has been proven to work reliably in production. Java 11,
17, 21, and 25 are supported experimentally.

## Status

The Python API is fairly ergonomic and supports both 32-bit and 64-bit JVMs
across Java 8 through 25. The server includes a CSS-like selector engine with
attribute matching, combinators, and pseudo-classes.

Bug reports, issues, discussions, and contributions are always welcome.

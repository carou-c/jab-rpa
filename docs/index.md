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
               │ gRPC (localhost:50051)
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
- **32-bit Java application** (JRE 1.8+ with Java Access Bridge enabled)
- **Python 3.12+**

If you're targeting a 64-bit JVM, use
[robocorp's RPA.JavaAccessBridge](https://rpaframework.org/libraries/javaaccessbridge/python.html)
instead — it's a much more robust tool.

## Status

This project contains the "bare minimum" to develop RPA. The Python API is
fairly ergonomic, but the server lacks many performance optimizations and likely
has bugs.

Bug reports, issues, discussions, and contributions are always welcome.

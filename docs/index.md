# jab-rpa

Automate Java desktop applications from Python.

## Problem

Standard RPA tools like
[robocorp's RPA.JavaAccessBridge](https://rpaframework.org/libraries/javaaccessbridge/python.html)
only work with 64-bit JVMs. If your target application runs on a 32-bit JVM
(e.g. JRE 1.8), those tools won't work.

## Solution

`jab-rpa` bridges this gap with a two-component architecture:

```text
┌─────────────────────────────────────┐
│  Python RPA Client                  │
│  (jab_rpa — core package)           │
└──────────────┬──────────────────────┘
               │ gRPC (localhost:port)
               ▼
┌─────────────────────────────────────┐
│  jab-rpa-server.exe (32/64-bit)     │
│  ┌──────────┐  ┌──────────────────┐ │
│  │ gRPC     │  │ Java Access      │ │
│  │ Service  │◄─┤ Bridge Wrapper   │ │
│  └──────────┘  └──────────────────┘ │
└──────────────────┬──────────────────┘
                   ▼
     WindowsAccessBridge-{32,64}.dll
                   │
     ┌─────────────────────────────┐
     │  Java App (JRE 1.8+)        │
     └─────────────────────────────┘
```

The project is split into multiple packages:

- **`jab-rpa`** — the core Python client library (no binaries).
- **`jab-rpa-bin-java{8,11,17,21,25}`** — binary packages, each containing
  32-bit and 64-bit builds of `jab-rpa-server.exe` for a specific Java version.

Install the core package with a binary extra:

```bash
pip install jab-rpa[java8]     # Java 8 (proven in production)
pip install jab-rpa[java17]    # Java 17
```

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

The Python API supports both 32-bit and 64-bit JVMs across Java 8 through 25.
The server includes a CSS-like selector engine with attribute matching,
combinators, and pseudo-classes.

Bug reports, issues, discussions, and contributions are always welcome.

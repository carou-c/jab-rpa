# Quickstart

## Installation

Download the latest `.whl` from the
[releases page](https://github.com/carou-c/jab-rpa/releases) and install it:

```bash
python -m pip install jab_rpa-x.y.z-py3-none-any.whl
```

## Prerequisites

On your Windows machine, enable Java Access Bridge:

```bash
jabswitch -enable
```

## Basic usage

```python
from jab_rpa import JabDriver

with JabDriver("My Java Application.*") as driver:
    locator = driver.locator("push_button[name='Clear']")
    locator.click()
```

### What's happening

1. `JabDriver("My Java Application.*")` — spawns `jab-rpa-server.exe`, waits for
   a Java window whose title matches the regex `"My Java Application.*"`, then
   selects it and maximizes it.
2. `driver.locator("push_button[name='Clear']")` — builds a locator query using
   CSS-selector-like syntax (see [Selector syntax](selectors.md)).
3. `.click()` — polls until a matching element appears in the accessibility
   tree, moves the mouse to the matching element's center and clicks via
   `pyautogui`.

## Waiting for elements

```python
# Wait up to 30 seconds (default: 60s)
locator.wait_for(timeout_ms=30_000)

# Wait with no timeout (fail fast if element doesn't exist yet)
locator.wait_for(timeout_ms=0)
```

## Interacting with elements

```python
# Accessible click (uses JAB API directly — no mouse movement)
locator.accessible_click()

# Coordinate click (uses pyautogui)
locator.click()
locator.click(clicks=2, interval=0.5)

# Click and type text
locator.click_and_type("hello world")
```

### Accessible actions

Elements may expose additional actions beyond clicking (e.g. "toggle", "expand",
"select"):

```python
# List available actions
actions: list[Action] = locator.get_actions()
for action in actions:
    print(action.name)

# Perform a specific action by name
locator.do_accessible_action(action)
```

## Getting element info

```python
info = locator.get_info()

print(info.name)                # Accessible name
print(info.role)                # e.g. "push_button"
print(info.states)              # Localized states
print(info.states_en_us)        # en-US states
print(info.description)         # Accessible description
print(info.x, info.y)           # Screen coordinates
print(info.width, info.height)
print(info.index_in_parent)
print(info.children_count)
print(info.to_dict())           # All properties as dict
```

The `Locator` also supports querying across all matching nodes or retrieving
specific fields:

```python
# Get info from all matching elements
all_info = locator.get_info_from_all()

# Get accessible text directly
text = locator.get_text()

# Get available actions
actions = locator.get_actions()
```

## Driver utilities

```python
# List all detected Java windows
for w in driver.list_java_windows():
    print(w.hwnd, w.title)

# Switch to a different window
driver.switch_window(some_window_info)

# Rebuild the accessibility tree (call after UI changes)
driver.refresh_tree()

# Get JAB bridge and server version info
info = driver.get_version_info()
```

## Error handling

The library defines two custom exceptions:

| Exception            | Raised when                                        |
| -------------------- | -------------------------------------------------- |
| `WindowNotFound`     | No Java window matches the given title pattern     |
| `ServerStoppedError` | The `jab-rpa-server.exe` process exits prematurely |

```python
from jab_rpa import JabDriver
from jab_rpa.driver import WindowNotFound

try:
    with JabDriver("MyApp.*") as driver:
        loc = driver.locator("push_button[name='Clear']")
        loc.wait_for()
except WindowNotFound:
    print("Application not running")
```

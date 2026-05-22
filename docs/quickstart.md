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
    button = driver.locator("push_button[name='Clear']").wait_for()
    button.click()
```

### What's happening

1. `JabDriver("My Java Application.*")` — spawns `jab-rpa-server.exe`, waits for
   a Java window whose title matches the regex `"My Java Application.*"`, then
   selects it and maximizes it.
2. `driver.locator("push_button[name='Clear']")` — builds a locator query using
   CSS-selector-like syntax (see [Selector syntax](selectors.md)).
3. `.wait_for()` — polls until a matching element appears in the accessibility
   tree.
4. `.click()` — moves the mouse to the element's center and clicks via
   `pyautogui`.

## Interacting with elements

```python
# Accessible click (uses JAB API directly — no mouse movement)
element.accessible_click()

# Coordinate click (uses pyautogui)
element.click()
element.click(clicks=2, interval=0.5)

# Click and type text
element.click_and_type("hello world")
```

### Accessible actions

Elements may expose additional actions beyond clicking (e.g. "toggle", "expand", "select"):

```python
actions = element.get_accessible_actions()
for action in actions:
    print(action.name)

# Perform a specific action by name
element.do_accessible_action(action)
```

You can also use these directly on a `Locator`:

```python
locator.accessible_click()
locator.do_accessible_action(action)
```

## Getting element info

```python
print(element.name)                # Accessible name
print(element.role)                # e.g. "push_button"
print(element.states)              # Localized states
print(element.states_en_us)        # en-US states
print(element.get_accessible_text())  # Accessible text content
print(element.description)         # Accessible description
print(element.x, element.y)        # Screen coordinates
print(element.width, element.height)
print(element.index_in_parent)
print(element.children_count)
print(element.to_dict())           # All properties as dict
```

## Finding elements

```python
# Find the first matching element (raises LocatorNotFound if none match)
element = locator.find()

# Find all matching elements (returns empty list if none match)
elements = locator.find_all()

# Check if an element exists without fetching it
if locator.exists():
    element = locator.find()
```

## Waiting for elements

```python
# Wait up to 30 seconds (default: 60s)
element = locator.wait_for(timeout=30)

# Custom polling interval
element = locator.wait_for(timeout=60, sleep_step=2)
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

## Traversing the tree

```python
children = element.children()
parent = element.parent()
```

## Error handling

The library defines three custom exceptions:

| Exception            | Raised when                                          |
| -------------------- | ---------------------------------------------------- |
| `WindowNotFound`     | No Java window matches the given title pattern       |
| `LocatorNotFound`    | No element matches the locator query                 |
| `ServerStoppedError` | The `jab-rpa-server.exe` process exits prematurely   |

```python
from jab_rpa import JabDriver
from jab_rpa.driver import WindowNotFound
from jab_rpa.locator import LocatorNotFound

try:
    with JabDriver("MyApp.*") as driver:
        btn = driver.locator("push_button[name='Clear']").find()
except WindowNotFound:
    print("Application not running")
except LocatorNotFound:
    print("Button not found")
```

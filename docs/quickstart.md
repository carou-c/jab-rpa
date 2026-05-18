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

## Waiting for elements

```python
# Wait up to 30 seconds (default: 60s)
element = locator.wait_for(timeout=30)

# Custom polling interval
element = locator.wait_for(timeout=60, sleep_step=2)
```

## Traversing the tree

```python
children = element.children()
parent = element.parent()
```

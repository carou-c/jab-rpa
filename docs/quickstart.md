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
    button = driver.locator(role="push button", name="Clear")\
                   .wait_for()
    button.click()
```

### What's happening

1. `JabDriver("My Java Application.*")` — spawns `jab-rpa-server.exe`, waits for
   a Java window whose title matches the regex `"My Java Application.*"`, then
   selects it and maximizes it.
2. `driver.locator(role="push button", name="Clear")` — builds a structured
   locator query (no string parsing).
3. `.wait_for()` — polls until an element matching the locator appears in the
   accessibility tree.
4. `.click()` — moves the mouse to the element's center and clicks via
   `pyautogui`.

## Finding elements

Locators support various criteria:

```python
# By role and name (default: regex matching on both)
btn = driver.locator(role="push button", name="Clear")

# Exact matching (disable regex)
btn = driver.locator(role="push button", name="Clear",
                     role_regex=False, name_regex=False)

# By state
btn = driver.locator(role="push button", has_state=["enabled"])

# By absence of state
btn = driver.locator(role="push button", not_has_state=["disabled"])

# By index in parent
third_tab = driver.locator(role="page tab", index_in_parent=2)

# By description or text
field = driver.locator(role="text", description="Username input")
```

## Traversing the tree

```python
# Get children or parent of an element
children = element.children()
parent = element.parent()
```

## Chaining locators

Navigate the hierarchy by chaining children or descendants:

```python
# Direct child
btn = (driver.locator(role="dialog")
       .child(role="push button", name="Confirm"))

# Any descendant
btn = (driver.locator(role="dialog")
       .descendant(role="push button", name="Confirm"))
```

## Interacting with elements

```python
# Accessible click (uses JAB API directly)
element.accessible_click()

# Coordinate click (uses pyautogui)
element.click()
element.click(clicks=2, interval=0.5)

# Click and type text
element.click_and_type("hello world")
element.click_and_type("hello", clicks=2, interval_text=0.1)
```

## Waiting for elements

```python
# Wait up to 30 seconds (default: 60s)
element = locator.wait_for(timeout=30)

# Custom polling interval
element = locator.wait_for(timeout=60, sleep_step=2)
```

## Getting element info

```python
print(element.name)           # Accessible name
print(element.role)           # Accessible role (e.g. "push button")
print(element.states)         # States as localized strings
print(element.states_en_us)   # States as en-US strings
print(element.text)           # Text content
print(element.description)    # Accessible description
print(element.x, element.y)   # Screen coordinates
print(element.width, element.height)
print(element.index_in_parent)
print(element.children_count)
print(element.to_dict())      # All properties as dict
```

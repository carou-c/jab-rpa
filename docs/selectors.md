# Selector syntax

Locators in jab-rpa use a CSS-selector-like syntax to describe elements in the
Java accessibility tree. The selector is a single string passed to
`JabDriver.locator()`.

## Quick examples

```python
# By role
driver.locator("push_button")

# By role and attribute
driver.locator("push_button[name='Clear']")

# By state class
driver.locator(".enabled.focusable")

# Chaining (descendant combinator)
driver.locator("dialog").locator("push_button")
```

---

## Role matching

The role is written as the first identifier in the selector. Spaces in the JAB
role name are replaced with underscores and the comparison is case-insensitive.

| JAB role       | Selector       |
| -------------- | -------------- |
| `push button`  | `push_button`  |
| `check box`    | `check_box`    |
| `radio button` | `radio_button` |
| `page tab`     | `page_tab`     |

A selector with no role matches elements of any role.

## State classes

Prefix a state name with `.` to require that the element has that state.
Multiple states are AND-ed together.

```python
driver.locator(".enabled")           # any enabled element
driver.locator(".enabled.focusable") # enabled AND focusable
```

States are lowercased and spaces replaced with underscores, matching the same
transformation applied to roles.

## Attribute selectors

### String attributes

```python
[ name  =  "value" ]    # exact match
[ name ~= "word"   ]    # contains word (whitespace-separated)
[ name ^= "prefix" ]    # starts with
[ name $= "suffix" ]    # ends with
[ name *= "substr" ]    # contains substring
```

Supported string attribute names: `name`, `description`, `states`,
`states_en_us`, `text`, `actions`.

**Flags** — add before the closing `]`:

| Flag | Meaning                  |
| ---- | ------------------------ |
| `i`  | Case-insensitive         |
| `r`  | Interpret value as regex |

```python
driver.locator("[name='clear' i]")          # case-insensitive
driver.locator("[name='button[0-9]+' r]")   # regex
```

### Integer attributes

```python
[ x == 100 ]   # equal
[ x != 100 ]   # not equal
[ x < 100]     # less than
[ x <= 100]    # less or equal
[ x > 100]     # greater than
[ x >= 100]    # greater or equal
```

Supported integer attribute names: `x`, `y`, `width`, `height`,
`children_count`, `depth`.

### Boolean attributes

```python
[accessible_action]    # element has accessible actions
[accessible_text]      # element supports AccessibleText
[accessible_selection] # element supports accessible selection
```

## Pseudo-classes

| Selector                 | Description                                          |
| ------------------------ | ---------------------------------------------------- |
| `:scope`                 | Matches the reference element (used inside `:has()`) |
| `:not(`_selector_`)`     | Inverse — elements not matching the inner selector   |
| `:has(`_selector_`)`     | Elements that contain a matching descendant          |
| `:nth-child(`_n_`)`      | Match by 1-indexed position in parent                |
| `:nth-last-child(`_n_`)` | Match by 1-indexed position from end of parent       |

```python
driver.locator("push_button:not([name='Cancel'])")
driver.locator("dialog:has(push_button[name='Confirm'])")
driver.locator("page_tab:nth-child(1)")  # first tab
```

## Combinators

Separate compound selectors with a combinator to navigate the tree hierarchy.

| Combinator         | Symbol  | Meaning                       |
| ------------------ | ------- | ----------------------------- |
| Child              | `>`     | Direct child                  |
| Descendant         | (space) | Any descendant                |
| Next sibling       | `+`     | Immediately following sibling |
| Subsequent sibling | `~`     | Any following sibling         |

```python
driver.locator("dialog > push_button")         # direct child
driver.locator("dialog push_button")           # any descendant
driver.locator("label + text_field")           # next sibling
driver.locator("label ~ text_field")           # any subsequent sibling
```

## Alternatives

Separate multiple selectors with `,` — an element matching **any** of the
alternatives is returned.

```python
driver.locator("push_button, check_box, radio_button")
```

## Chaining via `.locator()`

Calling `.locator()` on an existing `Locator` concatenates the selectors with a
space (the descendant combinator):

```python
# These two are equivalent:
driver.locator("dialog").locator("push_button")
driver.locator("dialog push_button")
```

This is useful for building queries step by step:

```python
container = driver.locator("dialog")
btn = container.locator("push_button[name='Confirm']")
```

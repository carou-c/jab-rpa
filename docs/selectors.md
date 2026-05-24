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

# By state pseudo-class
driver.locator(":require-state(enabled):require-state(focusable)")

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

Write `*` to explicitly match any role (equivalent to omitting the role). A
selector with no role (or with `*`) matches elements of any role.

## State pseudo-classes

Use `:require-state()` to require that the element has a given state, and
`:exclude-state()` to exclude elements with a given state. Multiple states
are AND-ed together.

```python
driver.locator(":require-state(enabled)")                   # any enabled element
driver.locator(":require-state(enabled):require-state(focusable)") # enabled AND focusable
driver.locator(":exclude-state(disabled)")                  # not disabled
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

| Selector                 | Description                                                                                                                           |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------- |
| `:scope`                 | Matches the reference element (used inside `:has()`)                                                                                  |
| `:not(`_selector_`)`     | Inverse — elements not matching the inner selector (relative selectors not allowed)                                                   |
| `:has(`_selector_`)`     | Elements containing a matching descendant — supports relative selectors (`>`, `+`, `~`) and bare `:scope` for flexible tree traversal |
| `:nth-child(`_n_`)`      | Match by 1-indexed position in parent                                                                                                 |
| `:nth-last-child(`_n_`)` | Match by 1-indexed position from end of parent                        |
| `:require-state(`_s_`)`  | Element has the given state name                                      |
| `:exclude-state(`_s_`)`  | Element does not have the given state name                            |

```python
driver.locator("push_button:not([name='Cancel'])")
driver.locator("dialog:has(push_button[name='Confirm'])")
driver.locator("page_tab:nth-child(1)")  # first tab
driver.locator("push_button:require-state(enabled)") # enabled push button
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

## Relative selectors

A combinator at the start of a selector (before any compound) makes it a
**relative selector**. Relative selectors are **only** valid inside `:has()` —
they are rejected at the top level.

```python
driver.locator("dialog:has(> push_button)")        # immediate child
driver.locator("dialog:has(+ label)")              # adjacent sibling
driver.locator("dialog:has(~ text_field)")         # any following sibling
driver.locator("dialog:has( push_button)")         # any descendant (space)
```

You can also use `:scope` explicitly inside `:has()` to refer to the context
element:

```python
driver.locator("dialog:has(:scope > push_button)")  # same as > push_button
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
# Chaining (descendant combinator)
driver.locator("dialog").locator("push_button")

# Wildcard (matches any role)
driver.locator("*")
driver.locator("*[name='Clear']")

# Relative selectors inside :has()
driver.locator("dialog:has(> push_button)")  # immediate child
driver.locator("dialog:has(+ label)")        # adjacent sibling
```

This is useful for building queries step by step:

```python
container = driver.locator("dialog")
btn = container.locator("push_button[name='Confirm']")
```

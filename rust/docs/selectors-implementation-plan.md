# Selector Query Engine — Implementation Plan

## Overview

Implement a CSS-selector-inspired query engine for `ContextTree`. The selector syntax maps accessibility tree concepts onto CSS concepts:

| Our concept | CSS analogy |
|---|---|
| `role` | tag name |
| `states` | classes (`.focused`) |
| name, description, text, action | string attributes |
| x, y, width, height, children_count | int attributes |
| accessible_action, text, selection | bool attributes (presence) |

## Phases

### Phase 1: Dependencies & Module Structure

- Add `nom = "7"` to `Cargo.toml`
- Create `src/context_tree/selector/` with sub-modules
- Wire into existing module tree

**Files**: `Cargo.toml`, `src/context_tree.rs`, `src/context_tree/selector/mod.rs`

### Phase 2: AST Types (`selector/ast.rs`)

Define all types for the parsed selector:

- `Selector` — alternation list (comma-separated)
- `ComplexSelector` — linked compounds with combinators + optional leading combinator
- `Combinator` — Child(`>`), Descendant(` `), NextSibling(`+`), SubsequentSibling(`~`)
- `CompoundSelector` — role, states (classes), attributes, pseudo-classes
- `AttributeSelector` — `StringAttr` | `IntAttr` | `BoolAttr`
- `StringOp`, `IntOp` — operator enums
- `AttrFlags` — case-insensitive + regex flags
- `PseudoClassSelector` — Has, Not, NthChild, NthLastChild, NthOfType, NthLastOfType
- `NthFormula` — An+B representation

### Phase 3: Parser (`selector/parser.rs`)

nom-based parser for the full grammar:

```
selector           → complex ("," complex)*
complex            → leading_combinator? compound (combinator compound)*
leading_combinator → ">" | "+" | "~"
combinator         → ">" | "+" | "~" | " " (descendant)
compound           → type_selector? (class | attr | pseudo)*
type_selector      → "*" | role_ident
class              → "." ident
attr               → "[" name op value flags? "]"
pseudo             → ":" ("has" | "not" | "nth-child" | "nth-last-child"
                           | "nth-of-type" | "nth-last-of-type")
                       "(" inner ")"
string_op          → "=" | "~=" | "|=" | "^=" | "$=" | "*="
int_op             → "==" | "!=" | "<=" | ">=" | "<" | ">"
flags              → ident (e.g. "i", "r", "ir")
nth_formula        → "odd" | "even" | signed_int? "n" signed_int?
```

**Post-parse validation**:
- String attrs (`name`, `description`, `text`, `action`) → must use string ops
- Int attrs (`x`, `y`, `width`, `height`, `children_count`) → must use int ops
- Bool attrs (`accessible_action`, `accessible_text`, `accessible_selection`) → no op/value
- Leading combinator requires `relative_to` context at match time

### Phase 4: Matcher (`selector/matcher.rs`)

Left-to-right tree walking:

```
match_complex(tree, complex, relative_to):
    scope = if leading_combinator:
        apply_combinator(tree, relative_to, leading_combinator)
    else if relative_to:
        [relative_to] + descendants(relative_to)
    else:
        all_nodes(tree)

    current = filter(scope, first_compound)
    for (combinator, compound) in tail:
        next = flat_map(current, |n| apply_combinator(tree, n, combinator))
        current = filter(next, compound)
    return current
```

**Attribute resolution**:
- String attrs: name/description → stored fields; text/action → lazy JAB call via `OnceLock`
- Int attrs: direct field access
- Bool attrs: direct field access

**Pseudo-class evaluation**:
- `:has(sel)` → recursively call `select_nodes` with `relative_to = current_node`
- `:not(sel)` → evaluate sel from tree root, check if current_node is in results
- `:nth-child(An+B)` → 1-indexed, evaluate against parent's children
- `:nth-last-child(An+B)` → same, counting from end
- `:nth-of-type(An+B)` → filter siblings by same role first
- `:nth-last-of-type(An+B)` → same, from end

### Phase 5: ContextNode/ContextTree Changes (`base.rs`)

Add to `ContextNode`:
```rust
pub text_cache: OnceLock<String>,
pub action_names_cache: OnceLock<String>,
```

Add to `ContextTree`:
```rust
pub jab: Arc<JabWrapper>,
```

Add lazy resolver methods on `ContextTree`:
```rust
pub(crate) fn resolve_text(&self, node: &ContextNode) -> &str
pub(crate) fn resolve_action_names(&self, node: &ContextNode) -> &str
```

### Phase 6: Public API (`search.rs`, `context_tree.rs`)

- Rewrite `get_nodes` → returns `Result<Vec<&ContextNode>, GetNodesError>`
- `Locator` struct stays, add `Locator::new(selector: &str) -> Locator`
- New error enum: `GetNodesError::Parse(SelectorParseError)` + `GetNodesError::NoRelativeContext(String)`
- Re-export new types from `context_tree.rs`

### Phase 7: Error Types (`selector/error.rs`)

```rust
pub struct SelectorParseError {
    pub message: String,
    pub input: String,
    pub position: usize,
}
```

### Phase 8: Action Names Helper

Add a function `fetch_and_join_action_names` to fetch `AccessibleActions` via FFI and join names with space.

## Files Changed/Created

| File | Action |
|---|---|
| `Cargo.toml` | Add `nom = "7"` |
| `src/context_tree.rs` | Add `mod selector;`, re-export new types |
| `src/context_tree/base.rs` | Add `text_cache`, `action_names_cache`, `jab` field; resolver methods |
| `src/context_tree/search.rs` | Rewrite `get_nodes` → `Result` |
| `src/context_tree/selector/mod.rs` | **New** — re-exports |
| `src/context_tree/selector/ast.rs` | **New** — AST types |
| `src/context_tree/selector/parser.rs` | **New** — nom parser |
| `src/context_tree/selector/matcher.rs` | **New** — tree walker |
| `src/context_tree/selector/error.rs` | **New** — error types |
| `src/wrapper.rs` | Add `fetch_and_join_action_names` helper |

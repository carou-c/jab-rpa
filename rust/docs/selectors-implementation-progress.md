# Selector Query Engine — Implementation Progress

Legend: ✅ Done | 🔄 In Progress | ⬜ Pending

## Overall Status: ✅ Complete

---

- [✅] **Phase 1**: Dependencies & Module Structure
- [✅] **Phase 2**: AST Types (`selector/ast.rs`)
- [✅] **Phase 3**: Parser (`selector/parser.rs`)
- [✅] **Phase 4**: Matcher (`selector/matcher.rs`)
- [✅] **Phase 5**: ContextNode/ContextTree changes (`base.rs`)
- [✅] **Phase 6**: Public API (`search.rs`, `context_tree.rs`)
- [✅] **Phase 7**: Error Types (`selector/error.rs`)
- [✅] **Phase 8**: Action Names Helper (`wrapper.rs`)

## Detailed Progress

### Phase 1: Dependencies & Module Structure
- [✅] Add `nom = "7"` to `Cargo.toml`
- [✅] Create `selector/mod.rs`
- [✅] Wire `mod selector;` into `context_tree.rs`

### Phase 2: AST Types
- [✅] Define `Selector`, `ComplexSelector`, `Combinator`
- [✅] Define `CompoundSelector`
- [✅] Define `AttributeSelector`, `StringOp`, `IntOp`, `AttrFlags`
- [✅] Define `PseudoClassSelector`, `NthFormula`

### Phase 3: Parser
- [✅] Implement selector/selector_list parser
- [✅] Implement complex_selector with combinators (descendant via `multispace1`)
- [✅] Implement compound_selector (type, class, attr, pseudo)
- [✅] Implement attribute operators (int op `==` tried before string op `=`)
- [✅] Implement pseudo-class parsers (`:has`, `:not`, `:nth-*`)
- [✅] Implement nth formula parsing (An+B, odd, even)
- [✅] Implement post-parse validation (operator/attribute type checking)

### Phase 4: Matcher
- [✅] Implement `select_nodes` entry point
- [✅] Implement `match_complex` (left-to-right walking)
- [✅] Implement `match_compound` (role, states, attrs, pseudo)
- [✅] Implement combinator application (child, descendant, siblings)
- [✅] Implement string attribute matching (all ops + flags)
- [✅] Implement int attribute matching (all ops)
- [✅] Implement `:has()` evaluation (recursive from node subtree)
- [✅] Implement `:not()` evaluation (from tree root, membership check)
- [✅] Implement `:nth-*` evaluation (An+B formula)

### Phase 5: ContextNode/ContextTree changes
- [✅] Add `text_cache`, `action_names_cache` to `ContextNode`
- [✅] Add `jab` field to `ContextTree`
- [✅] Add lazy resolver methods (`resolve_text`, `resolve_action_names`)

### Phase 6: Public API
- [✅] Rewrite `get_nodes` in `search.rs` (returns `Result`)
- [✅] Update re-exports in `context_tree.rs`
- [✅] Add `GetNodesError` enum

### Phase 7: Error Types
- [✅] Define `SelectorParseError`
- [✅] Map nom errors to `SelectorParseError`

### Phase 8: Action Names Helper
- [✅] Add `fetch_and_join_action_names` to `wrapper.rs`

## Notes

- Tests compile but cannot execute natively: the crate targets `i686-pc-windows-gnu` (Java Access Bridge is Windows-only). Cross-compiled test binaries can't run on this dev machine.
- Parser tests cover: type selectors, class selectors, attribute selectors (string/int/bool), combinators (child, descendant, sibling), pseudo-classes (`:has`, `:not`, `:nth-child` etc.), nth-formula (odd, even, An+B), comma-separated lists, leading combinators, and error cases.

# Bugfix-2: Rewrite Locator Engine

## Bug Description
The original locator engine was buggy and unnecessarily complicated:
- Used string-based locator parsing (`"role:push button and name:Clear"` with `>` for child traversal)
- `parse_locator()` in `context_tree.rs` parsed strings into `Vec<Vec<SearchElement>>`
- `get_by_attrs()` had complex multi-level traversal with broken matching logic

## Fix Plan
Rewrite the locator engine using a structured protobuf-based approach with the following changes:

### 1. Updated Proto Definitions (`proto/jab.proto`)
- **`Element` message**: Now matches `ContextNode` struct (minus `vm_id` and `context` fields)
  - Added: `text`, `visible_children_count`, `index_in_parent`, `children` (recursive)
- **New `Locator` message**: Structured locator with optional fields:
  - `name`, `role`, `description`, `text`: `optional StringLocator`
  - `index_in_parent`: `optional IndexLocator`
  - `ascendant`: `optional AscendantLocator`
  - `descendants`: `repeated DescendantLocator`
- **New `StringLocator` message**: Fields `find: str`, `regex: bool`
- **New `IndexLocator` message**: Field `index: int`
- **New `AscendantLocator` message**: Fields `locator: Locator`, `is_parent: bool`
- **New `DescendantLocator` message**: Fields `locator: Locator`, `is_child: bool`
- **`GetElementsRequest`**: Changed `locator` field from `string` to `Locator`

### 2. Updated `src/context_tree.rs`
- Removed `SearchElement` struct and `parse_locator()` function
- Added `get_elements(&self, locator: &proto::Locator) -> Vec<&ContextNode>` method
- Implemented matching logic:
  - **String fields** (name, role, description, text): If field present, match using exact or regex (based on `StringLocator.regex`)
  - **`index_in_parent`**: If present, exact integer match
  - **`ascendant`**: Traverse ancestors (passed as path during tree traversal); if `is_parent=true`, only check direct parent
  - **`descendants`**: For each `DescendantLocator`, search subtree; if `is_child=true`, only check direct children
- Added helper functions: `matches_string_field()`, `matches_node_simple()`, `matches_node_simple_opt()`

### 3. Updated `src/jab_service.rs`
- Updated `get_elements()` to use new proto `Locator` instead of parsing strings
- Updated `element_from_context_node()` to include new fields and recursively convert `children`
- Fixed imports to properly use proto-generated types

### 4. Updated `src/lib.rs`
- Added `pub mod proto` with `tonic::include_proto!("jab")`

### 5. Updated `Cargo.toml`
- Added `regex = "1"` dependency for regex matching in `StringLocator`

## Progress
- [x] Step 1: Update `proto/jab.proto` with new Element and Locator messages
- [x] Step 2: Update `src/context_tree.rs` - remove old parsing, add new `get_elements` method
- [x] Step 3: Update `src/jab_service.rs` - use new Locator proto
- [x] Step 4: Update `src/lib.rs` - add proto module
- [x] Step 5: Update `Cargo.toml` - add regex dependency
- [x] Step 6: Build and verify compilation

## Summary
Rewrote the locator engine to use a structured protobuf-based approach instead of string parsing. The new implementation:
- Uses `Locator` proto message for structured queries
- Supports regex matching via `StringLocator.regex` field
- Supports ancestor/descendant relationships with `is_parent` and `is_child` flags
- Simplified the matching logic by traversing all nodes and checking against the locator criteria

### Changes Made
1. **`proto/jab.proto`**: Added new messages (`Locator`, `StringLocator`, `IndexLocator`, `AscendantLocator`, `DescendantLocator`) and updated `Element` message
2. **`src/context_tree.rs`**: Removed old `SearchElement`/`parse_locator()`/`get_by_attrs()`, added new `get_elements()` with proper matching logic
3. **`src/jab_service.rs`**: Updated to use new proto types, fixed imports
4. **`src/lib.rs`**: Added `proto` module
5. **`src/bin/jab-rpa-server.rs`**: Fixed import path for `JabServiceServer`
6. **`Cargo.toml`**: Added `regex` dependency

### Files Modified
- `proto/jab.proto`
- `src/context_tree.rs`
- `src/jab_service.rs`
- `src/lib.rs`
- `src/bin/jab-rpa-server.rs`
- `Cargo.toml`
- `docs/Bugfix-2.md` (this file)

## Build Status
✅ Compiles successfully for `i686-pc-windows-gnu` target

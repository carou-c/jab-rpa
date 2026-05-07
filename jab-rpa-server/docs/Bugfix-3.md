# Bugfix-3: Java Object Memory Leak Fix

## Problem
Java objects obtained from JAB (via `GetAccessibleContextFromHWND`, `GetAccessibleChildFromContext`, etc.) are never released with `ReleaseJavaObject`. The `release_element()` method exists but is never called, causing memory leaks.

## Root Causes
1. No cleanup on context tree replacement in `jab_service.rs`
2. No `Drop` implementation for `ContextTree`/`ContextNode`
3. `JabWrapper::Drop` doesn't clean up remaining elements
4. `list_java_windows()` doesn't release contexts after reading window info
5. `set_root_context()` doesn't release previous context
6. Callback objects may be leaking

## Implementation Plan

### Step 1: Add `Weak<JabWrapper>` to `ContextTree` ✅
- Add weak reference to `ContextTree` struct
- Update `ContextTree::from_root()` to accept `&Arc<JabWrapper>`

### Step 2: Implement `Drop` for `ContextTree` ✅
- Add recursive cleanup when tree is dropped
- Release all registered handles through the wrapper
- Added print statements for verification

### Step 3: Update `jab_service.rs` to Pass `Arc<JabWrapper>` ✅
- Updated `select_window_by_title()` to pass wrapper reference

### Step 4: Release Old Context in `set_root_context()` ✅
- Release previous root context before setting new one

### Step 5: Release Contexts in `list_java_windows()` ✅
- Release context after getting info

### Step 6: Clean Up Remaining Elements in `JabWrapper::Drop` ✅
- Release all remaining elements on shutdown
- Release root_context if set

### Step 7: Investigate Callback Object Cleanup ⏳
- Determine if callback objects need release
- Add release if necessary

## Progress Tracking

| Step | Status | Notes |
|------|--------|-------|
| 1    | ✅ Done | Added `wrapper: Option<Weak<JabWrapper>>` to ContextTree |
| 2    | ✅ Done | Implemented `Drop` for ContextTree with recursive cleanup |
| 3    | ✅ Done | Updated jab_service.rs to pass Arc to ContextTree::from_root() |
| 4    | ✅ Done | Updated set_root_context() to release old context |
| 5    | ✅ Done | Updated list_java_windows() to release contexts |
| 6    | ✅ Done | Updated JabWrapper::Drop to clean up all elements |
| 7    | ⏳ Pending | Need to investigate callback semantics |

## Testing
- [x] Build: `cargo build --target i686-pc-windows-gnu` - ✅ Success (no warnings)
- [ ] Test window selection and reselection
- [ ] Test list windows multiple times
- [ ] Test shutdown cleanup
- [ ] Long-running memory test

## Summary of Changes

### 1. `src/context_tree.rs`
- Added `wrapper: Option<Weak<JabWrapper>>` field to `ContextTree`
- Updated `from_root()` to accept `&Arc<JabWrapper>` and store weak reference
- Implemented `Drop` for `ContextTree` to release all registered elements
- Added `release_all_elements()` and `release_node_recursive()` helper methods
- Added `Clone` implementation (since `Drop` prevents derive)
- Added eprintln! statements for verification

### 2. `src/jab_wrapper.rs`
- Updated `set_root_context()` to release previous context via `ReleaseJavaObject`
- Updated `list_java_windows()` to release contexts after retrieving window info
- Updated `Drop` implementation to:
  - Release root_context if set
  - Drain and release all remaining elements in the elements map
- Added eprintln! statements in `release_element()` for verification

### 3. `src/jab_service.rs`
- Updated `select_window_by_title()` to pass `&wrapper` (Arc reference) to `ContextTree::from_root()`

## How It Works

1. **When a new window is selected**: The old `ContextTree` is dropped when replaced at `jab_service.rs:89`. The `Drop` implementation recursively calls `release_element()` for each node, which calls `ReleaseJavaObject` and removes the handle from the map.

2. **When the server shuts down**: `JabWrapper::Drop` now releases the root_context and all remaining elements before calling `shutdownAccessBridge()`.

3. **When listing windows**: Contexts obtained from `GetAccessibleContextFromHWND` are now released after retrieving window info.

## Next Steps

- Step 7: Investigate callback object cleanup (determine if `source`, `old_child`, `new_child` parameters in callbacks need to be released)
- Test the fixes on Windows with a real Java application
- Monitor memory usage during long-running sessions

## Files Modified
- `src/context_tree.rs` - Added Weak<JabWrapper>, implemented Drop, added print statements
- `src/jab_wrapper.rs` - Updated set_root_context(), list_java_windows(), Drop impl, added print statements
- `src/jab_service.rs` - Updated to pass Arc to ContextTree::from_root()

## Notes
- Added eprintln! statements to verify cleanup is happening
- ContextTree now requires `&Arc<JabWrapper>` instead of `&JabWrapper`
- Drop order: When tree is dropped, it releases all registered elements via the wrapper

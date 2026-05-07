# Bugfix-4: Clean Up Global Statics and Implement Drop for ContextNode

## Problem
1. `EVENT_TX` and `JOBJECT_TO_HANDLE` are global statics with unsafe raw pointers
2. `ContextNode` doesn't implement `Drop`, relying on `ContextTree::drop()` for cleanup

## Goals
1. Move `EVENT_TX` and `JOBJECT_TO_HANDLE` into `JabWrapper` struct
2. Implement `Drop` for `ContextNode` to release its handle when dropped

## Implementation Plan

### Step1: Restructure JabWrapper to Hold Event Sender and Handle Map ✅
- Add `event_tx: Mutex<Option<mpsc::Sender<crate::JabCallbackEvent>>>` to `JabWrapper`
- Add `jobject_to_handle: Mutex<HashMap<JObject, u64>>` to `JabWrapper`
- Replace separate globals with a single global `Weak<JabWrapper>`

### Step 2: Update All References to Use New Fields ✅
- Update `set_event_sender()` to store in struct
- Update `register_element()` to use struct field
- Update `release_element()` to use struct field
- Update `send_callback_event()` to access via global weak reference
- Update `JabWrapper::new()` to initialize new fields
- Update `JabWrapper::drop()` to clean up global weak reference

### Step 3: Implement Drop for ContextNode ✅
- Add `wrapper: Option<Weak<JabWrapper>>` field to `ContextNode`
- Implement `Drop` for `ContextNode` to release its handle
- Handle `Clone` carefully to avoid double-release (set wrapper to None when cloning)
- Update `build_node()` to pass wrapper reference to nodes

## Progress Tracking

| Step | Status | Notes |
|------|--------|-------|
| 1    | ✅ Done | Added event_tx and jobject_to_handle fields to JabWrapper |
| 2    | ✅ Done | Updated all methods to use struct fields, replaced globals |
| 3    | ✅ Done | Implemented Drop for ContextNode |

## Files Modified
- `src/jab_wrapper.rs` - Added fields, updated methods, replaced global statics with single `JAB_WRAPPER` weak reference
- `src/context_tree.rs` - Added wrapper to ContextNode, implemented Drop, updated Clone to avoid double-release

## Summary of Changes

### 1. `src/jab_wrapper.rs`
- Replaced `EVENT_TX` and `JOBJECT_TO_HANDLE` globals with `event_tx` and `jobject_to_handle` fields
- Added single global `static mut JAB_WRAPPER: *mut Weak<JabWrapper>`
- Updated `new()` to initialize new fields and set global weak reference
- Updated `register_element()` to use `jobject_to_handle` field
- Updated `release_element()` to use `jobject_to_handle` field
- Updated `set_event_sender()` to use `event_tx` field
- Updated `send_callback_event()` to access wrapper via global weak reference
- Updated `Drop` to clean up global weak reference

### 2. `src/context_tree.rs`
- Added `wrapper: Option<Weak<JabWrapper>>` field to `ContextNode`
- Implemented `Drop` for `ContextNode` to release its handle via wrapper
- Updated `Clone` for `ContextNode` to set `wrapper` to `None` (avoid double-release)
- Updated `build_node()` to pass wrapper reference to nodes

## How It Works

1. **Global access**: A single global `Weak<JabWrapper>` (`JAB_WRAPPER`) allows callback functions to access the wrapper
2. **ContextNode cleanup**: When a `ContextNode` is dropped, it calls `wrapper.release_element(node.handle)` to release the Java object
3. **Clone safety**: When `ContextNode` is cloned, the clone has `wrapper: None`, so only the original releases the handle
4. **ContextTree cleanup**: No longer needs explicit cleanup - nodes release themselves

## Testing
- [x] Build: `cargo build --target i686-pc-windows-gnu` - ✅ Success
- [ ] Test ContextNode drop on tree replacement
- [ ] Test callback event sending
- [ ] Verify no double-release of handles
- [ ] Long-running memory test

## Notes
- Eliminated unsafe raw pointer globals (except the single `JAB_WRAPPER` weak reference)
- Each `ContextNode` now owns its handle and releases it on drop
- Clone creates a "view" that doesn't own the handle

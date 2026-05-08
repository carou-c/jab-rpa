# Bugfix-001: Windows Message Pump Thread Alignment

## Bug Description
The Java Access Bridge (JAB) requires the Windows message pump to run on the same thread that called `initializeAccessBridge()`. Currently, `initializeAccessBridge()` is invoked on the main thread, while the message pump runs in a separate thread, resulting in JAB callbacks not being delivered.

## Fix Plan
1. **Restructure Initialization**: Move `initializeAccessBridge()` execution into the message pump thread to ensure both operations share the same thread context.
2. **Synchronization Channel**: Use `std::sync::mpsc::channel` to signal initialization completion and propagate success/failure to the main thread.
3. **Modify `JabWrapper::new()`**:
   - Create an initialization result channel
   - Spawn the message pump thread that initializes JAB, reports result, then runs the message loop
   - Block main thread until initialization completes
   - Proceed with callback registration and wrapper setup
4. **Clean Shutdown Handling**:
   - Implement `Drop` for `JabWrapper` to call `shutdownAccessBridge()`
   - Post `WM_QUIT` to the message pump thread to exit the loop cleanly
   - Wait for the message pump thread to join

## Progress
- [x] Step 1: Restructure Initialization - Moved `initializeAccessBridge()` into message pump thread
- [x] Step 2: Use Synchronization Channel - Added `std::sync::mpsc::channel` for init synchronization
- [x] Step 3: Modify `JabWrapper::new()` - Updated to spawn thread that initializes JAB then runs pump
- [x] Step 4: Clean Up on Shutdown - Implemented `Drop` trait with `WM_QUIT` and `shutdownAccessBridge()`
- [x] Step 5: Verify compilation - Build succeeds for `i686-pc-windows-gnu` target

## Summary
Fixed the bug where `initializeAccessBridge()` was called on a different thread than the Windows message pump. Now both operations occur on the same thread (the message pump thread), ensuring JAB callbacks are properly delivered.

### Changes Made
1. **`src/jab_wrapper.rs`**:
   - Added `message_pump_thread_id` field to track the message pump thread
   - Modified `JabWrapper::new()` to use a channel (`std::sync::mpsc`) for synchronization
   - The message pump thread now calls `initializeAccessBridge()` before running the message loop
   - Main thread waits for initialization result before proceeding
   - Implemented `Drop` trait to cleanly shut down the message pump via `WM_QUIT` and call `shutdownAccessBridge()`

### Files Modified
- `src/jab_wrapper.rs`
- `docs/Bugfix-001.md` (this file)

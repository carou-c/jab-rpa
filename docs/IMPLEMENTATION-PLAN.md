# JAB-RPA Server Implementation Plan

## Purpose

This project solves a specific interoperability problem: automating a **32-bit Java Desktop Application** (JRE 1.8.0_481) using a **64-bit Python RPA runtime**.

Standard tools like Robocorp's `RPA.JavaAccessBridge` only support 64-bit Java applications. The solution is a **32-bit Rust gRPC server** that:

1. Loads `WindowsAccessBridge-32.dll` (required for 32-bit Java apps)
2. Exposes a gRPC API mirroring Robocorp's `RPA.JavaAccessBridge`
3. Allows 64-bit Python clients to automate the Java app via gRPC

## Architecture

```text
┌─────────────────────────────────────┐
│      64-bit Python RPA Client       │
│         (uses grpcio, NOT jab)      │
└──────────────┬──────────────────────┘
               │ gRPC (localhost:50051)
               ▼
┌────────────────────────────────────────┐
│    jab-rpa-server.exe (32-bit)         │
│  ┌──────────────┐     ┌──────────────┐ │
│  │ gRPC Service │     │  JAB Wrapper │ │
│  │  (tonic)     │◄────│  (bindgen)   │ │
│  └──────┬───────┘     └──────┬───────┘ │
│         │                    │         │
│         ▼                    ▼         │
│  ┌────────────────┐   ┌──────────────┐ │
│  │ proto/jab.proto│   │ Message Pump │ │
│  │  (generated)   │   │ (Windows)    │ │
│  └────────────────┘   └──────────────┘ │
└──────────────┼─────────────────────────┘
               ▼
    WindowsAccessBridge-32.dll (loaded at runtime)
               │
    ┌─────────────────────────────┐
    │  32-bit Java App (jre1.8)   │
    └─────────────────────────────┘
```

## Key Technical Findings

### JAB Initialization Flow

**Critical Understanding** (verified from `native/AccessBridgeCalls.c`):

1. `initializeAccessBridge()` is a C wrapper function that:
   - Loads `WindowsAccessBridge-32.dll` via `LoadLibrary`
   - Resolves all function pointers into `theAccessBridge` struct using `GetProcAddress`
   - Calls `theAccessBridge.Windows_run()` (sets up the DLL, does NOT block)
   - Returns TRUE on success

2. **`Windows_run()` does NOT block** - contrary to initial assumptions

3. **Manual Message Pump Required**: For JAB callbacks to be delivered, you must run a Windows message loop:

   ```c
   MSG msg;
   while (GetMessage(&msg, NULL, 0, 0)) {
       TranslateMessage(&msg);
       DispatchMessage(&msg);
   }
   ```

### JAB Data Types

From `AccessBridgePackages.h`:

- `JOBJECT64` = `jlong` (i64) when `ACCESSBRIDGE_ARCH_32` is defined (line 47)
- This is because we're using the "new bridge" 32-bit mode, not legacy

### Existing Code Status

| Component                   | Status       | Notes                                              |
| --------------------------- | ------------ | -------------------------------------------------- |
| `Cargo.toml`                | ❌ Broken    | `edition = "2024"` is invalid (should be `"2021"`) |
| `build.rs`                  | ✅ Working   | Compiles C code, generates bindings                |
| `src/lib.rs`                | ✅ Minimal   | Just exposes `bindings` module                     |
| `src/bin/jab-rpa-server.rs` | ⚠️ Empty     | Just imports bindings                              |
| Bindings                    | ✅ Generated | `target/.../out/bindings.rs`                       |

## References

### Robocorp RPA.JavaAccessBridge API

- **RPAFramework RPA.JavaAccessBridge Python API Docs**: <https://rpaframework.org/libraries/javaaccessbridge/python.html>
- **RPAFramework RPA.JavaAccessBridge (high-level RPA API) Source Code**: <https://raw.githubusercontent.com/robocorp/rpaframework/refs/heads/master/packages/main/src/RPA/JavaAccessBridge.py>
- **java-access-bridge-wrapper (low-level JAB wrapper) Source Code**: <https://github.com/robocorp/java-access-bridge-wrapper/tree/master/src/JABWrapper>

### Oracle JAB API

- **Official API Docs**: <https://docs.oracle.com/javase/8/docs/technotes/guides/access/jab/api.html>
- **Header Files** (in `native/` directory):
  - `AccessBridgeCalls.h` / `AccessBridgeCalls.c` - Gateway functions + DLL loading
  - `AccessBridgePackages.h` - Data structures (`AccessibleContextInfo`, `AccessBridgeVersionInfo`, etc.)
  - `AccessBridgeCallbacks.h` - Callback function pointer types

### Key Implementation Patterns (from Robocorp)

1. **Locator Syntax**: `"role:push button and name:Clear"` with `>` for child traversal
2. **Context Tree**: Build full tree after window selection, cache for fast queries
3. **Element Handles**: Use `ContextNode` objects with `get_by_attrs()` for searching
4. **Window Management**: `EnumWindows` + `isJavaWindow()` + `GetAccessibleContextFromHWND`
5. **Message Pump**: Robocorp runs it in a dedicated thread (`_pump_background`)

## Implementation Plan

### Phase 1: Fix Foundation

**1.1 Verify Bindings**
Check `target/i686-pc-windows-gnu/debug/build/jab-rpa-*/out/bindings.rs` for:

- `initializeAccessBridge() -> BOOL`
- `shutdownAccessBridge() -> BOOL`
- `Windows_run` function pointer type
- `SetFocusGained`, `SetPropertyChange`, etc. (callback registration)
- `JOBJECT64` should be `i64`
- `AccessBridgeVersionInfo`, `AccessibleContextInfo` structs

**1.2 Update `src/lib.rs`**

```rust
pub mod bindings;
pub mod jab_wrapper;
pub mod jab_service;
pub mod context_tree;
// proto module generated by tonic-build
```

---

### Phase 2: Protocol Definition

**New File**: `proto/jab.proto`

Key service methods (mirroring Robocorp's API):

- `ListJavaWindows()` → List all Java windows
- `SelectWindowByTitle()` / `SelectWindowByPid()` → Set active window
- `GetElements()` → Find elements by locator string
- `ClickElement()` → Click using JAB actions only
- `TypeText()` → Type text into element
- `ReadTable()` → Return full table (rows of cells)
- `WaitUntilElementExists()` → Poll with timeout
- `GetVersionInfo()` → JAB version info
- `SubscribeCallbacks()` → Server-streaming for JAB events

Full proto definition in `proto/jab.proto` (see Phase 3 for complete file).

---

### Phase 3: JAB Wrapper (`src/jab_wrapper.rs`)

**State Management**:

```rust
pub struct JabWrapper {
    initialized: AtomicBool,
    vm_id: Mutex<Option<i64>>,
    root_context: Mutex<Option<i64>>,  // AccessibleContext as i64 (JOBJECT64)
    elements: Mutex<HashMap<u64, (i64, i64)>>,  // handle → (vmID, context)
    next_handle: AtomicU64,
    context_tree: Mutex<Option<ContextTree>>,
    event_tx: Mutex<Option<mpsc::Sender<CallbackEvent>>>,
    message_pump_handle: Option<std::thread::JoinHandle<()>>,
}
```

**Initialization** (corrected):

```rust
impl JabWrapper {
    pub fn new() -> Arc<Self> {
        let wrapper = Arc::new(Self { /* init fields */ });

        // 1. Call initializeAccessBridge (non-blocking)
        let init_result = unsafe { bindings::initializeAccessBridge() };
        if init_result == 0 {
            panic!("Failed to initialize JAB");
        }

        // 2. Start Windows message pump in dedicated thread
        let wrapper_clone = wrapper.clone();
        let pump_handle = std::thread::spawn(move || {
            wrapper_clone.run_message_pump();
        });

        // 3. Register callbacks (always, per user requirement)
        wrapper.register_callbacks();

        wrapper
    }

    fn run_message_pump(&self) {
        unsafe {
            let mut msg: winapi::um::winuser::MSG = std::mem::zeroed();
            loop {
                let result = winapi::um::winuser::GetMessageW(
                    &mut msg,
                    std::ptr::null_mut(),  // HWND NULL = all windows
                    0,  // wMsgFilterMin
                    0   // wMsgFilterMax
                );

                if result == 0 {
                    // WM_QUIT received
                    break;
                } else if result == -1 {
                    eprintln!("Message pump error");
                    break;
                } else {
                    winapi::um::winuser::TranslateMessage(&msg);
                    winapi::um::winuser::DispatchMessageW(&msg);
                }
            }
        }
    }
}
```

**Callback Registration**:

```rust
impl JabWrapper {
    fn register_callbacks(&self) {
        unsafe {
            // Define Rust extern "C" callback functions (see module level below)
            bindings::SetFocusGained(Some(rust_focus_gained_cb));
            bindings::SetFocusLost(Some(rust_focus_lost_cb));
            bindings::SetCaretUpdate(Some(rust_caret_update_cb));
            // ... register ALL callbacks
        }
    }
}

// Module-level callback functions
extern "C" fn rust_focus_gained_cb(vm_id: i64, event: i64, source: i64) {
    // Send event via channel if subscribed
    // IMPORTANT: Call bindings::ReleaseJavaObject if needed
}
```

**Element Management**:

- `register_element(vm_id, context) -> u64` - Store element, return handle
- `get_element(handle) -> Option<(i64, i64)>` - Retrieve (vmID, context)
- `release_element(handle)` - Remove from registry, call `ReleaseJavaObject`

---

### Phase 4: Context Tree (`src/context_tree.rs`)

Port Robocorp's `ContextTree`/`ContextNode` to Rust:

```rust
pub struct ContextNode {
    pub vm_id: i64,
    pub context: i64,  // JOBJECT64 as i64
    pub handle: u64,
    pub info: AccessibleContextInfoMsg,
    pub children: Vec<ContextNode>,
    pub text: String,
    pub visible_children_count: i32,
    pub ancestry: i32,
}

pub struct ContextTree {
    pub root: Option<ContextNode>,
    pub max_depth: Option<i32>,
}

impl ContextTree {
    pub fn from_root(vm_id: i64, root_context: i64, max_depth: Option<i32>, jab: &JabWrapper) -> Self {
        // Build tree recursively
        // Call bindings::GetAccessibleContextInfo for each node
        // Register elements with jab.register_element()
    }

    pub fn get_by_attrs(&self, searches: &[SearchElement]) -> Vec<&ContextNode> {
        // Traverse tree, match attributes
        // Support name, role, description matching
    }
}
```

**Locator Parsing** (port Robocorp's `_parse_locator`):

```rust
pub struct SearchElement {
    pub key: String,  // "name", "role", "description", etc.
    pub value: String,
    pub strict: bool,
}

pub fn parse_locator(locator: &str, strict_default: bool) -> Vec<Vec<SearchElement>> {
    // Split on ">" for child levels
    // Split each level on " and "
    // Parse "key:value" or bare value (default to "name")
    // Support "strict:True" to set strict mode
    // Integer types: x, y, width, height, indexInParent
}
```

---

### Phase 5: gRPC Service (`src/jab_service.rs`)

```rust
pub struct JabService {
    wrapper: Arc<JabWrapper>,
}

#[tonic::async_trait]
impl JabService for JabService {
    async fn list_java_windows(... ) -> ... {
        tokio::task::spawn_blocking(move || {
            wrapper.list_java_windows()
        }).await.unwrap()
    }

    async fn select_window_by_title(... ) -> ... {
        // Call wrapper.select_window_by_title
        // Build context tree
    }

    async fn get_elements(... ) -> ... {
        // Parse locator
        // Search context tree
        // Return matching elements with handles
    }

    async fn click_element(... ) -> ... {
        let (vm_id, context) = wrapper.get_element(handle).ok_or("Element not found")?;
        // Call bindings::getAccessibleActions to get actions
        // Call bindings::doAccessibleActions with "click" action
        // Return success/error
    }

    async fn read_table(... ) -> ... {
        // Find table element via locator
        // Call bindings::getAccessibleTableInfo
        // For each cell, call bindings::getAccessibleTableCellInfo
        // Build and return full table
    }

    async fn subscribe_callbacks(... ) -> ... {
        // Return stream of CallbackEvent from wrapper.event_rx
    }
}
```

---

### Phase 6: Build Configuration

**Update `build.rs`** to add proto compilation:

```rust
use std::env;
use std::path::PathBuf;

const JAVA_HOME: &str = "/usr/lib/jvm/java-8-openjdk";
const MINGW_SYSROOT: &str = "/usr/i686-w64-mingw32";

fn main() {
    // 1. Compile C code (existing)
    cc::Build::new()
        .cpp(true)
        .prefer_clang_cl_over_msvc(true)
        .warnings(false)
        .include(format!("{}/include", JAVA_HOME))
        .include(format!("{}/include/linux", JAVA_HOME))
        .include("native")
        .file("native/AccessBridgeCalls.c")
        .file("native/AccessBridgeDebug.cpp")
        .compile("accessbridge");

    println!("cargo:rustc-link-search=native={}/lib", MINGW_SYSROOT);
    println!("cargo:rustc-link-lib=static=accessbridge");
    println!("cargo:rustc-link-lib=static=stdc++");

    // 2. Generate bindings (existing)
    let bindings = bindgen::Builder::default()
        .header("native/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_arg("-Inative")
        .clang_arg(format!("-I{}/include", JAVA_HOME))
        .clang_arg(format!("-I{}/include/linux", JAVA_HOME))
        .clang_arg("-Wno-everything")
        .blocklist_type("_LONGDOUBLE")
        .allowlist_file("native/AccessBridgePackages.h")
        .allowlist_file("native/AccessBridgeCallbacks.h")
        .allowlist_file("native/AccessBridgeCalls.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).unwrap();

    // 3. Compile proto (NEW)
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(&["proto/jab.proto"], &["proto/"])
        .unwrap();
}
```

---

### Phase 7: Main Entry Point

**Update `src/bin/jab-rpa-server.rs`**:

```rust
use jab_rpa::jab_wrapper::JabWrapper;
use jab_rpa::jab_service::JabService;
use tonic::transport::Server;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wrapper = JabWrapper::new();

    // Wait for JAB init
    std::thread::sleep(std::time::Duration::from_secs(2));

    let service = JabService { wrapper: Arc::new(wrapper) };

    let addr = "127.0.0.1:50051".parse()?;
    println!("JAB gRPC Server listening on {}", addr);

    Server::builder()
        .add_service(JabServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
```

---

## Key Design Decisions

| Decision                               | Rationale                                                                                       |
| -------------------------------------- | ----------------------------------------------------------------------------------------------- |
| Keep `AccessBridgeCalls.c` compilation | Provides `initializeAccessBridge()` wrapper that loads DLL and sets up function pointers        |
| Manual message pump                    | `Windows_run()` doesn't block; pump required for JAB callbacks                                  |
| Always register callbacks              | Per user requirement; enables event-driven automation                                           |
| `u64` element handles                  | Raw `(vmID, AccessibleContext)` can't cross to 64-bit client; server maps handles → JAB objects |
| Full ContextTree                       | Built after window selection; enables fast re-querying like Robocorp                            |
| JAB actions only for ClickElement      | Per user requirement; no coordinate fallback                                                    |
| Return full table in single response   | Per user requirement; simpler client implementation                                             |
| `tonic` gRPC                           | Industry standard; supports streaming for callbacks                                             |

## Build & Deploy

```bash
# Cross-compile from Linux
cargo build --target i686-pc-windows-gnu

# On Windows target machine:
# 1. Enable Java Access Bridge: jabswitch -enable
# 2. Run server: jab-rpa-server.exe
# 3. Server listens on localhost:50051
```

## References for Implementation

- **RPAFramework RPA.JavaAccessBridge Python API Docs**: <https://rpaframework.org/libraries/javaaccessbridge/python.html>
- **RPAFramework RPA.JavaAccessBridge (high-level RPA API) Source Code**: <https://raw.githubusercontent.com/robocorp/rpaframework/refs/heads/master/packages/main/src/RPA/JavaAccessBridge.py>
- **java-access-bridge-wrapper (low-level JAB wrapper) Source Code**: <https://github.com/robocorp/java-access-bridge-wrapper/tree/master/src/JABWrapper>
- **Oracle JAB API**: <https://docs.oracle.com/javase/8/docs/technotes/guides/access/jab/api.html>
- **JAB Header Files**: `native/` directory
- **Bindings Output**: `target/i686-pc-windows-gnu/debug/build/jab-rpa-*/out/bindings.rs`

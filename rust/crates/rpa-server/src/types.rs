use windows::Win32::Foundation::HWND;

use jab_wrapper::context_tree::ContextNode;
use jab_wrapper::selector::Locator;
use jab_wrapper::types::{AccessBridgeVersionInfo, WindowInfo};
use jab_wrapper::utils::utf16_to_string;

use crate::proto;

impl From<WindowInfo> for proto::WindowInfo {
    fn from(w: WindowInfo) -> Self {
        Self {
            hwnd: w.hwnd.0 as _,
            title: w.title,
        }
    }
}

impl From<HWND> for proto::Hwnd {
    fn from(hwnd: HWND) -> Self {
        Self {
            hwnd: hwnd.0 as _
        }
    }
}

impl From<AccessBridgeVersionInfo> for proto::VersionInfo {
    fn from(vi: AccessBridgeVersionInfo) -> Self {
        Self {
            vm_version: utf16_to_string(&vi.VMversion),
            bridge_java_class_version: utf16_to_string(&vi.bridgeJavaClassVersion),
            bridge_java_dll_version: utf16_to_string(&vi.bridgeJavaDLLVersion),
            bridge_win_dll_version: utf16_to_string(&vi.bridgeWinDLLVersion),
        }
    }
}

impl From<proto::Locator> for Locator {
    fn from(loc: proto::Locator) -> Self {
        Self {
            selector: loc.selector,
        }
    }
}

impl From<&ContextNode> for proto::Element {
    fn from(node: &ContextNode) -> Self {
        Self {
            handle: node.handle,
            name: node.name.clone(),
            role: node.role.clone(),
            states: node.states.clone(),
            states_en_us: node.states_en_us.clone(),
            description: node.description.clone(),
            children_handles: node.children.clone(),
            x: node.x,
            y: node.y,
            width: node.width,
            height: node.height,
            accessible_action: node.accessible_action,
            accessible_text: node.accessible_text,
            accessible_selection: node.accessible_selection,
            children_count: node.children_count,
            index_in_parent: node.index_in_parent,
            parent_handle: node.parent,
            depth: node.depth,
        }
    }
}

impl From<&[String]> for proto::Actions {
    fn from(slice: &[String]) -> Self {
        Self {
            actions: slice
                .iter()
                .map(|s| proto::Action { name: s.clone() })
                .collect(),
        }
    }
}

pub mod proto {
    tonic::include_proto!("jab");
}

use std::sync::Mutex;
use tonic::{Request, Response, Status};

use jab_wrapper::context_tree::Locator;
use jab_wrapper::context_tree::{ContextNode, ContextTree};
use jab_wrapper::types::{AccessBridgeVersionInfo, WindowInfo};
use jab_wrapper::utils::utf16_to_string;
use jab_wrapper::wrapper::JabWrapper;

// Import all proto types; tonic generates them in the proto module
// We alias our local types to avoid confusion
use crate::proto::jab_service_server::JabService as JabServiceTrait;
use crate::proto::{
    ClickElementRequest, ClickElementResponse, Element, FindElementsRequest, FindElementsResponse,
    GetElementFromHandleRequest, GetElementFromHandleResponse, GetVersionInfoRequest,
    GetVersionInfoResponse, ListJavaWindowsRequest, ListJavaWindowsResponse, RefreshTreeRequest,
    RefreshTreeResponse, SelectWindowRequest, SelectWindowResponse,
    VersionInfo as ProtoVersionInfo, WindowInfo as ProtoWindowInfo,
};

impl From<WindowInfo> for ProtoWindowInfo {
    fn from(w: WindowInfo) -> Self {
        Self {
            hwnd: w.get_hwnd(),
            title: w.title,
        }
    }
}

impl From<ProtoWindowInfo> for WindowInfo {
    fn from(w: ProtoWindowInfo) -> Self {
        unsafe { Self::new(w.hwnd, w.title) }
    }
}

impl From<AccessBridgeVersionInfo> for ProtoVersionInfo {
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

pub struct JabService {
    wrapper: JabWrapper,
    ctx_tree: Mutex<Option<ContextTree>>,
}

impl JabService {
    pub fn new(wrapper: JabWrapper) -> Self {
        Self {
            wrapper,
            ctx_tree: Mutex::new(None),
        }
    }
}

#[tonic::async_trait]
impl JabServiceTrait for JabService {
    async fn list_java_windows(
        &self,
        _request: Request<ListJavaWindowsRequest>,
    ) -> Result<Response<ListJavaWindowsResponse>, Status> {
        let windows = self.wrapper.list_java_windows();
        let proto_windows: Vec<ProtoWindowInfo> = windows.into_iter().map(|w| w.into()).collect();

        Ok(Response::new(ListJavaWindowsResponse {
            windows: proto_windows,
        }))
    }

    async fn select_window(
        &self,
        request: Request<SelectWindowRequest>,
    ) -> Result<Response<SelectWindowResponse>, Status> {
        let req = request.into_inner();
        let Some(w) = req.window_info else {
            return Ok(Response::new(SelectWindowResponse {
                success: false,
                error_message: "window_info not present in request".to_string(),
            }));
        };

        let result = self.wrapper.get_root_obj_from_window(w.into());

        match result {
            Ok(root) => {
                let tree = ContextTree::from_root(root, None);

                let mut tree_lock = self.ctx_tree.lock().unwrap();
                *tree_lock = Some(tree);

                Ok(Response::new(SelectWindowResponse {
                    success: true,
                    error_message: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(SelectWindowResponse {
                success: false,
                error_message: e,
            })),
        }
    }

    async fn refresh_tree(
        &self,
        _request: Request<RefreshTreeRequest>,
    ) -> Result<Response<RefreshTreeResponse>, Status> {
        let mut tree_lock = self.ctx_tree.lock().unwrap();
        let root_obj = match tree_lock.take() {
            Some(tree) => tree.into_root(),
            None => {
                return Ok(Response::new(RefreshTreeResponse {
                    success: false,
                    error_message: "No window selected. Call SelectWindow first.".to_string(),
                }));
            }
        };

        let tree = ContextTree::from_root(root_obj, None);
        *tree_lock = Some(tree);

        Ok(Response::new(RefreshTreeResponse {
            success: true,
            error_message: String::new(),
        }))
    }

    async fn get_version_info(
        &self,
        _request: Request<GetVersionInfoRequest>,
    ) -> Result<Response<GetVersionInfoResponse>, Status> {
        let tree_lock = self.ctx_tree.lock().unwrap();

        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => {
                return Ok(Response::new(GetVersionInfoResponse {
                    version_info: None,
                    error_message: "No window selected. Call SelectWindow first.".to_string(),
                }));
            }
        };

        let root = tree.root();

        match root.obj.get_version_info() {
            Ok(version_info) => Ok(Response::new(GetVersionInfoResponse {
                version_info: Some(version_info.into()),
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(GetVersionInfoResponse {
                version_info: None,
                error_message: e,
            })),
        }
    }

    async fn find_elements(
        &self,
        request: Request<FindElementsRequest>,
    ) -> Result<Response<FindElementsResponse>, Status> {
        let req = request.into_inner();

        let tree_lock = self.ctx_tree.lock().unwrap();
        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => {
                return Ok(Response::new(FindElementsResponse {
                    elements: Vec::new(),
                    error_message: "No window selected. Call SelectWindow first.".to_string(),
                }));
            }
        };

        let default_locator = proto::Locator {
            selector: String::new(),
        };
        let locator = req.locator.as_ref().unwrap_or(&default_locator);
        let nodes = match tree.get_nodes(&locator.clone().into()) {
            Ok(nodes) => nodes,
            Err(e) => {
                return Ok(Response::new(FindElementsResponse {
                    elements: Vec::new(),
                    error_message: e.to_string(),
                }));
            }
        };

        let elements = nodes.into_iter().map(Into::into).collect();

        Ok(Response::new(FindElementsResponse {
            elements,
            error_message: String::new(),
        }))
    }

    async fn get_element_from_handle(
        &self,
        request: Request<GetElementFromHandleRequest>,
    ) -> Result<Response<GetElementFromHandleResponse>, Status> {
        let req = request.into_inner();

        let tree_lock = self.ctx_tree.lock().unwrap();
        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => {
                return Ok(Response::new(GetElementFromHandleResponse {
                    element: None,
                    error_message: "No window selected. Call SelectWindow first.".to_string(),
                }));
            }
        };

        let node = match tree.nodes.get(&req.handle) {
            Some(node) => node,
            None => {
                return Ok(Response::new(GetElementFromHandleResponse {
                    element: None,
                    error_message: format!("No node with handle={}", req.handle),
                }));
            }
        };

        Ok(Response::new(GetElementFromHandleResponse {
            element: Some(node.into()),
            error_message: String::new(),
        }))
    }

    async fn click_element(
        &self,
        request: Request<ClickElementRequest>,
    ) -> Result<Response<ClickElementResponse>, Status> {
        let req = request.into_inner();
        let tree_lock = self.ctx_tree.lock().unwrap();

        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => {
                return Ok(Response::new(ClickElementResponse {
                    success: false,
                    error_message: "No window selected. Call SelectWindow first.".to_string(),
                }));
            }
        };

        let node = match tree.nodes.get(&req.handle) {
            Some(node) => node,
            None => {
                return Ok(Response::new(ClickElementResponse {
                    success: false,
                    error_message: format!("No element with handle={}", req.handle),
                }));
            }
        };

        match node.obj.click_element() {
            Ok(()) => Ok(Response::new(ClickElementResponse {
                success: true,
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(ClickElementResponse {
                success: false,
                error_message: e,
            })),
        }
    }
}

impl From<&ContextNode> for Element {
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

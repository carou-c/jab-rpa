pub mod proto {
    tonic::include_proto!("jab");
}

use std::sync::{Mutex, MutexGuard};
use tonic::{Request, Response, Status};

use jab_wrapper::context_tree::Locator;
use jab_wrapper::context_tree::{ContextNode, ContextTree};
use jab_wrapper::types::{AccessBridgeVersionInfo, WindowInfo};
use jab_wrapper::utils::utf16_to_string;
use jab_wrapper::wrapper::JabWrapper;

use crate::proto::jab_service_server::JabService as JabServiceTrait;

macro_rules! no_window_selected {
    () => {
        Err(Status::failed_precondition(
            "No window selected. Call SelectWindow first.",
        ))
    };
}

macro_rules! no_element_with_handle {
    ($handle:expr) => {
        Err(Status::invalid_argument(format!(
            "No node with handle={}",
            $handle
        )))
    };
}

impl From<WindowInfo> for proto::WindowInfo {
    fn from(w: WindowInfo) -> Self {
        Self {
            hwnd: w.get_hwnd(),
            title: w.title,
        }
    }
}

impl From<proto::WindowInfo> for WindowInfo {
    fn from(w: proto::WindowInfo) -> Self {
        unsafe { Self::new(w.hwnd, w.title) }
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

impl JabService {
    #[inline]
    fn get_tree_lock(&self) -> Result<MutexGuard<'_, Option<ContextTree>>, Status> {
        self.ctx_tree.lock().map_err(|e| {
            Status::internal(format!(
                "The service's context tree mutex was poisoned: {}",
                e
            ))
        })
    }
}

#[tonic::async_trait]
impl JabServiceTrait for JabService {
    async fn list_java_windows(
        &self,
        _request: Request<proto::Empty>,
    ) -> Result<Response<proto::RepeatedWindowInfo>, Status> {
        let windows = self.wrapper.list_java_windows();
        let proto_windows: Vec<proto::WindowInfo> = windows.into_iter().map(|w| w.into()).collect();

        Ok(Response::new(proto::RepeatedWindowInfo {
            windows: proto_windows,
        }))
    }

    async fn select_window(
        &self,
        request: Request<proto::WindowInfo>,
    ) -> Result<Response<proto::Empty>, Status> {
        let req = request.into_inner();
        let result = self.wrapper.get_root_obj_from_window(req.into());

        match result {
            Ok(root) => {
                let tree = ContextTree::from_root(root, None);
                let mut tree_lock = self.get_tree_lock()?;
                *tree_lock = Some(tree);

                Ok(Response::new(proto::Empty {}))
            }
            Err(e) => Err(Status::unknown(e)),
        }
    }

    async fn refresh_tree(
        &self,
        _request: Request<proto::Empty>,
    ) -> Result<Response<proto::Empty>, Status> {
        let mut tree_lock = self.get_tree_lock()?;
        let root_obj = match tree_lock.take() {
            Some(tree) => tree.into_root(),
            None => return no_window_selected!(),
        };

        let tree = ContextTree::from_root(root_obj, None);
        *tree_lock = Some(tree);

        Ok(Response::new(proto::Empty {}))
    }

    async fn get_version_info(
        &self,
        _request: Request<proto::Empty>,
    ) -> Result<Response<proto::VersionInfo>, Status> {
        let tree_lock = self.get_tree_lock()?;

        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let root = tree.root();

        match root.obj.get_version_info() {
            Ok(version_info) => Ok(Response::new(version_info.into())),
            Err(e) => Err(Status::unknown(e)),
        }
    }

    async fn find_elements(
        &self,
        request: Request<proto::Locator>,
    ) -> Result<Response<proto::RepeatedElement>, Status> {
        let req = request.into_inner();

        let tree_lock = self.get_tree_lock()?;
        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let nodes = match tree.get_nodes(&req.clone().into()) {
            Ok(nodes) => nodes,
            Err(e) => return Err(Status::invalid_argument(e.to_string())),
        };

        let elements = nodes.into_iter().map(Into::into).collect();

        Ok(Response::new(proto::RepeatedElement { elements }))
    }

    async fn get_element_from_handle(
        &self,
        request: Request<proto::ElementHandle>,
    ) -> Result<Response<proto::Element>, Status> {
        let req = request.into_inner();

        let tree_lock = self.get_tree_lock()?;
        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let node = match tree.nodes.get(&req.handle) {
            Some(node) => node,
            None => return no_element_with_handle!(req.handle),
        };

        Ok(Response::new(node.into()))
    }

    async fn click_element(
        &self,
        request: Request<proto::Element>,
    ) -> Result<Response<proto::Empty>, Status> {
        let req = request.into_inner();
        let tree_lock = self.get_tree_lock()?;

        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let node = match tree.nodes.get(&req.handle) {
            Some(node) => node,
            None => return no_element_with_handle!(req.handle),
        };

        match node.obj.click_element() {
            Ok(()) => Ok(Response::new(proto::Empty {})),
            Err(e) => Err(Status::unknown(e)),
        }
    }

    async fn get_element_text(
        &self,
        request: Request<proto::Element>,
    ) -> Result<Response<proto::Text>, Status> {
        let req = request.into_inner();
        let tree_lock = self.get_tree_lock()?;

        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let node = match tree.nodes.get(&req.handle) {
            Some(node) => node,
            None => return no_element_with_handle!(req.handle),
        };

        Ok(Response::new(proto::Text {
            text: node.resolve_text().to_string(),
        }))
    }

    async fn get_element_actions(
        &self,
        request: Request<proto::Element>,
    ) -> Result<Response<proto::Actions>, Status> {
        let req = request.into_inner();
        let tree_lock = self.get_tree_lock()?;

        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let node = match tree.nodes.get(&req.handle) {
            Some(node) => node,
            None => return no_element_with_handle!(req.handle),
        };

        Ok(Response::new(node.resolve_actions().into()))
    }

    async fn do_action_on_element(
        &self,
        request: Request<proto::DoActionRequest>,
    ) -> Result<Response<proto::Empty>, Status> {
        let req = request.into_inner();
        let handle = match req.element {
            Some(el) => el.handle,
            None => return Err(Status::invalid_argument("Missing element")),
        };
        let action = match req.action {
            Some(act) => act.name,
            None => return Err(Status::invalid_argument("Missing action")),
        };

        let tree_lock = self.get_tree_lock()?;

        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let node = match tree.nodes.get(&handle) {
            Some(node) => node,
            None => return no_element_with_handle!(handle),
        };

        match node.obj.do_action(action) {
            Ok(()) => Ok(Response::new(proto::Empty {})),
            Err(e) => Err(Status::unknown(e)),
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

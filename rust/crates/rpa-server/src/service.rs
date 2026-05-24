use std::sync::Arc;
use std::thread;

use parking_lot::{RwLock, RwLockReadGuard};
use tonic::{Request, Response, Status};

use windows::Win32::Foundation::HWND;

use jab_wrapper::context_tree::ContextTree;
use jab_wrapper::selector::{Locator, Selector};
use jab_wrapper::wrapper::JabWrapper;

use crate::proto;

pub struct JabService {
    wrapper: JabWrapper,
    ctx_tree: Arc<RwLock<Option<ContextTree>>>,
    _updater: thread::JoinHandle<()>,
}

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

impl JabService {
    pub fn new(wrapper: JabWrapper) -> Self {
        let ctx_tree = Arc::new(RwLock::new(None));
        let updater = wrapper.spawn_tree_updater(&ctx_tree);
        Self {
            wrapper,
            ctx_tree,
            _updater: updater,
        }
    }

    fn find_elements(
        &self,
        locator: Locator,
    ) -> Result<(RwLockReadGuard, Vec<&ContextNode>), Status> {
        let tree_lock = self.ctx_tree.read();
        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let locator: Locator = req.into();
        let selector: Selector = match locator.parse() {
            Ok(selector) => selector,
            Err(e) => return Err(Status::invalid_argument(e.to_string())),
        };

        let nodes = tree.get_nodes(&selector);

        let elements = nodes.into_iter().map(Into::into).collect();

        Ok(Response::new(proto::RepeatedElement { elements }))
    }

    fn find_element(
        &self,
        request: Request<proto::Locator>,
    ) -> Result<Response<proto::Element>, Status> {
        let req = request.into_inner();

        let tree_lock = self.ctx_tree.read();
        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let locator: Locator = req.into();
        let selector: Selector = match locator.parse() {
            Ok(selector) => selector,
            Err(e) => return Err(Status::invalid_argument(e.to_string())),
        };

        let node = match tree.get_node(&selector) {
            Some(node) => node,
            None => {
                return Err(Status::not_found(format!(
                    "No element matches selector {}",
                    selector
                )));
            }
        };

        Ok(Response::new(node.into()))
    }

    fn get_element_from_handle(
        &self,
        request: Request<proto::ElementHandle>,
    ) -> Result<Response<proto::Element>, Status> {
        let req = request.into_inner();

        let tree_lock = self.ctx_tree.read();
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
}

#[tonic::async_trait]
impl proto::jab_service_server::JabService for JabService {
    async fn wait_for(&self) {
        todo!()
    }

    async fn race(&self) {
        todo!()
    }

    async fn get_info(&self) {
        todo!()
    }

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

        let hwnd = HWND(req.hwnd as _);

        if !self.wrapper.is_java_window(hwnd) {
            return Err(Status::invalid_argument(format!(
                "{:?} does not contain a hwnd to a valid Java window",
                req
            )));
        }

        let result = unsafe { self.wrapper.get_root_obj_from_hwnd(hwnd) };

        match result {
            Ok(root) => {
                // Clean up possible old tree
                let mut tree_lock = self.ctx_tree.write();
                if let Some(tree) = tree_lock.take() {
                    drop(tree)
                };

                // Build tree
                let tree = ContextTree::from_root(root, None);
                *tree_lock = Some(tree);

                Ok(Response::new(proto::Empty {}))
            }
            Err(e) => Err(Status::unknown(e)),
        }
    }

    async fn get_selected_window_hwnd(
        &self,
        _request: Request<proto::Empty>,
    ) -> Result<Response<proto::Hwnd>, Status> {
        let tree_lock = self.ctx_tree.read();

        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let root = tree.root();
        Ok(Response::new(
            self.wrapper.get_hwnd_from_obj(&root.obj).into(),
        ))
    }

    async fn refresh_tree(
        &self,
        _request: Request<proto::Empty>,
    ) -> Result<Response<proto::Empty>, Status> {
        let mut tree_lock = self.ctx_tree.write();
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
        let tree_lock = self.ctx_tree.read();

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

    async fn accessible_click(
        &self,
        request: Request<proto::Element>,
    ) -> Result<Response<proto::Empty>, Status> {
        let req = request.into_inner();
        let tree_lock = self.ctx_tree.read();

        let tree = match tree_lock.as_ref() {
            Some(tree) => tree,
            None => return no_window_selected!(),
        };

        let node = match tree.nodes.get(&req.handle) {
            Some(node) => node,
            None => return no_element_with_handle!(req.handle),
        };

        match node.obj.accessible_click() {
            Ok(()) => Ok(Response::new(proto::Empty {})),
            Err(e) => Err(Status::unknown(e)),
        }
    }

    async fn get_text(
        &self,
        request: Request<proto::Element>,
    ) -> Result<Response<proto::Text>, Status> {
        let req = request.into_inner();
        let tree_lock = self.ctx_tree.read();

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

    async fn get_actions(
        &self,
        request: Request<proto::Element>,
    ) -> Result<Response<proto::Actions>, Status> {
        let req = request.into_inner();
        let tree_lock = self.ctx_tree.read();

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

    async fn do_action(
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

        let tree_lock = self.ctx_tree.read();

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

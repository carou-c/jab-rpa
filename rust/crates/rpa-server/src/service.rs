use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::MutexGuard;
use tonic::{Request, Response, Status};

use windows::Win32::Foundation::HWND;

use jab_wrapper::context_tree::ContextTree;
use jab_wrapper::selector::{Locator, Selector};
use jab_wrapper::wrapper::{JabWrapper, SharedCtxTree};

use crate::DEFAULT_TIMEOUT;
use crate::proto;
use crate::utils::{find_node, find_nodes, get_root, parse_locator, refresh_tree};

pub struct JabService {
    wrapper: JabWrapper,
    ctx_tree: Arc<SharedCtxTree>,
    _updater: thread::JoinHandle<()>,
}

impl JabService {
    pub fn new(wrapper: JabWrapper) -> Self {
        let ctx_tree = Arc::new(SharedCtxTree::new());
        let updater = wrapper.spawn_tree_updater(&ctx_tree);
        Self {
            wrapper,
            ctx_tree,
            _updater: updater,
        }
    }

    fn wait_for_selector(
        &self,
        selector: &Selector,
        timeout: Option<Duration>,
        refresh_before_fail: bool,
    ) -> Result<MutexGuard<'_, Option<ContextTree>>, Status> {
        let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
        let condition = |tree: &mut _| find_node(tree, selector).is_err();

        let mut tree_lock = self.ctx_tree.lock();
        let wait_result = self
            .ctx_tree
            .wait_change_while_for(&mut tree_lock, condition, timeout);

        if !wait_result.timed_out() {
            return Ok(tree_lock);
        }

        if refresh_before_fail {
            refresh_tree(&mut tree_lock)?;
        }

        match find_node(&tree_lock, selector) {
            Ok(_) => Ok(tree_lock),
            Err(_) => Err(Status::deadline_exceeded(format!(
                "No element matched selector {} within timeout of {}ms",
                selector,
                timeout.as_millis()
            ))),
        }
    }
}

#[tonic::async_trait]
impl proto::jab_service_server::JabService for JabService {
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
                let mut tree_lock = self.ctx_tree.lock();
                tree_lock.take();

                // Build tree
                let tree = ContextTree::from_root(root, None);
                *tree_lock = Some(tree);

                Ok(Response::new(proto::Empty {}))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_selected_window_hwnd(
        &self,
        _request: Request<proto::Empty>,
    ) -> Result<Response<proto::Hwnd>, Status> {
        let tree_lock = self.ctx_tree.lock();
        let root = get_root(&tree_lock)?;

        Ok(Response::new(
            self.wrapper.get_hwnd_from_obj(&root.obj).into(),
        ))
    }

    async fn refresh_tree(
        &self,
        _request: Request<proto::Empty>,
    ) -> Result<Response<proto::Empty>, Status> {
        let mut tree_lock = self.ctx_tree.lock();
        refresh_tree(&mut tree_lock)?;

        Ok(Response::new(proto::Empty {}))
    }

    async fn get_version_info(
        &self,
        _request: Request<proto::Empty>,
    ) -> Result<Response<proto::VersionInfo>, Status> {
        let tree_lock = self.ctx_tree.lock();
        let root = get_root(&tree_lock)?;

        match root.obj.get_version_info() {
            Ok(version_info) => Ok(Response::new(version_info.into())),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn wait_for(
        &self,
        request: Request<proto::Locator>,
    ) -> Result<Response<proto::Empty>, Status> {
        let req = request.into_inner();

        let selector = parse_locator(&Locator {
            selector: req.selector,
        })?;

        let _guard = self.wait_for_selector(
            &selector,
            req.timeout_ms.map(Duration::from_millis),
            req.refresh_before_fail,
        )?;

        Ok(Response::new(proto::Empty {}))
    }

    async fn race(
        &self,
        request: Request<proto::RaceRequest>,
    ) -> Result<Response<proto::RaceResponse>, Status> {
        let req = request.into_inner();

        let selectors: Vec<_> = req
            .selectors
            .into_iter()
            .map(|selector| parse_locator(&Locator { selector }))
            .collect::<Result<_, _>>()?;

        let timeout = req
            .timeout_ms
            .map(Duration::from_millis)
            .unwrap_or(DEFAULT_TIMEOUT);

        let mut winner: Option<usize> = None;
        let condition = |tree: &mut _| {
            for (idx, selector) in selectors.iter().enumerate() {
                if find_node(tree, selector).is_ok() {
                    winner = Some(idx);
                    return false;
                }
            }
            true
        };

        let mut tree_lock = self.ctx_tree.lock();
        let wait_result = self
            .ctx_tree
            .wait_change_while_for(&mut tree_lock, condition, timeout);

        if !wait_result.timed_out()
            && let Some(winner) = winner
        {
            return Ok(Response::new(proto::RaceResponse {
                winner: winner as _,
            }));
        }

        if req.refresh_before_fail {
            refresh_tree(&mut tree_lock)?;
        }

        for (idx, selector) in selectors.iter().enumerate() {
            if find_node(&tree_lock, selector).is_ok() {
                return Ok(Response::new(proto::RaceResponse { winner: idx as _ }));
            }
        }

        Err(Status::deadline_exceeded(format!(
            "No element matched any of selectors {:?} within timeout of {}ms",
            selectors,
            timeout.as_millis()
        )))
    }

    async fn get_info(
        &self,
        request: Request<proto::Locator>,
    ) -> Result<Response<proto::AccessibleInfo>, Status> {
        let req = request.into_inner();

        let selector = parse_locator(&Locator {
            selector: req.selector,
        })?;

        let tree_lock = self.wait_for_selector(
            &selector,
            req.timeout_ms.map(Duration::from_millis),
            req.refresh_before_fail,
        )?;

        let node = find_node(&tree_lock, &selector)?;
        Ok(Response::new(node.into()))
    }

    async fn get_info_from_all(
        &self,
        request: Request<proto::Locator>,
    ) -> Result<Response<proto::RepeatedAccessibleInfo>, Status> {
        let req = request.into_inner();

        let selector = parse_locator(&Locator {
            selector: req.selector,
        })?;

        let tree_lock = self.ctx_tree.lock();
        let nodes = find_nodes(&tree_lock, &selector)?;

        Ok(Response::new(proto::RepeatedAccessibleInfo {
            ac_infos: nodes.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_text(
        &self,
        request: Request<proto::Locator>,
    ) -> Result<Response<proto::Text>, Status> {
        let req = request.into_inner();

        let selector = parse_locator(&Locator {
            selector: req.selector,
        })?;

        let tree_lock = self.wait_for_selector(
            &selector,
            req.timeout_ms.map(Duration::from_millis),
            req.refresh_before_fail,
        )?;

        let node = find_node(&tree_lock, &selector)?;

        Ok(Response::new(proto::Text {
            text: node.resolve_text().to_string(),
        }))
    }

    async fn get_actions(
        &self,
        request: Request<proto::Locator>,
    ) -> Result<Response<proto::Actions>, Status> {
        let req = request.into_inner();

        let selector = parse_locator(&Locator {
            selector: req.selector,
        })?;

        let tree_lock = self.wait_for_selector(
            &selector,
            req.timeout_ms.map(Duration::from_millis),
            req.refresh_before_fail,
        )?;

        let node = find_node(&tree_lock, &selector)?;

        Ok(Response::new(node.resolve_actions().into()))
    }

    async fn do_action(
        &self,
        request: Request<proto::DoActionRequest>,
    ) -> Result<Response<proto::Empty>, Status> {
        let req = request.into_inner();
        let Some(locator) = req.locator else {
            return Err(Status::invalid_argument(
                "locator argument must be specified",
            ));
        };
        let Some(action) = req.action else {
            return Err(Status::invalid_argument(
                "action argument must be specified",
            ));
        };

        let selector = parse_locator(&Locator {
            selector: locator.selector,
        })?;

        let tree_lock = self.wait_for_selector(
            &selector,
            locator.timeout_ms.map(Duration::from_millis),
            locator.refresh_before_fail,
        )?;

        let node = find_node(&tree_lock, &selector)?;

        match node.obj.do_action(action.name) {
            Ok(()) => Ok(Response::new(proto::Empty {})),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

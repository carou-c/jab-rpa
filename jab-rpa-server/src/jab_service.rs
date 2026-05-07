use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::jab_wrapper::JabWrapper;
// Import all proto types; tonic generates them in the proto module
// We alias our local types to avoid confusion
use crate::proto::{
    CallbackEvent as ProtoCallbackEvent, GetElementsResponse, ListJavaWindowsResponse,
    SelectWindowByTitleResponse, SelectWindowByPidResponse, ClickElementResponse,
    TypeTextResponse, ReadTableResponse, WaitUntilElementExistsResponse,
    GetVersionInfoResponse, VersionInfo, SubscribeCallbacksRequest,
    WindowInfo as ProtoWindowInfo, Element, ListJavaWindowsRequest,
    SelectWindowByTitleRequest, SelectWindowByPidRequest, GetElementsRequest,
    ClickElementRequest, TypeTextRequest, ReadTableRequest,
    WaitUntilElementExistsRequest, GetVersionInfoRequest, Locator,
};
use crate::proto::jab_service_server::JabService as JabServiceTrait;

#[derive(Debug, Clone)]
pub struct CallbackEvent {
    pub event_type: String,
    pub vm_id: i32,
    pub context_handle: u64,
    pub message: String,
    pub event_time: i64,
}

pub struct JabService {
    wrapper: Arc<JabWrapper>,
}

impl JabService {
    pub fn new(wrapper: Arc<JabWrapper>) -> Self {
        Self { wrapper }
    }
}

#[tonic::async_trait]
impl JabServiceTrait for JabService {
    async fn list_java_windows(
        &self,
        _request: Request<ListJavaWindowsRequest>,
    ) -> Result<Response<ListJavaWindowsResponse>, Status> {
        let wrapper = self.wrapper.clone();
        let windows = tokio::task::spawn_blocking(move || wrapper.list_java_windows())
            .await
            .map_err(|e| Status::internal(format!("Task failed: {}", e)))?;

        let proto_windows: Vec<ProtoWindowInfo> = windows
            .into_iter()
            .map(|w| window_info_to_proto(&w))
            .collect();

        Ok(Response::new(ListJavaWindowsResponse {
            windows: proto_windows,
        }))
    }

    async fn select_window_by_title(
        &self,
        request: Request<SelectWindowByTitleRequest>,
    ) -> Result<Response<SelectWindowByTitleResponse>, Status> {
        let req = request.into_inner();
        let wrapper = self.wrapper.clone();

        let result = tokio::task::spawn_blocking({
            let wrapper = wrapper.clone();
            move || wrapper.select_window_by_title(&req.title, req.partial_match)
        })
        .await
        .map_err(|e| Status::internal(format!("Task failed: {}", e)))?;

        match result {
            Ok(()) => {
                let vm_id = wrapper.get_vm_id().unwrap_or(0);
                let root_context = wrapper.get_root_context().unwrap_or(0);

                let tree = crate::context_tree::ContextTree::from_root(
                    vm_id,
                    root_context,
                    None,
                    &wrapper,
                );

                {
                    let mut tree_lock = wrapper.context_tree.lock().unwrap();
                    *tree_lock = Some(tree);
                }

                Ok(Response::new(SelectWindowByTitleResponse {
                    success: true,
                    error_message: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(SelectWindowByTitleResponse {
                success: false,
                error_message: e,
            })),
        }
    }

    async fn select_window_by_pid(
        &self,
        _request: Request<SelectWindowByPidRequest>,
    ) -> Result<Response<SelectWindowByPidResponse>, Status> {
        Ok(Response::new(SelectWindowByPidResponse {
            success: false,
            error_message: "select_window_by_pid not yet implemented".to_string(),
        }))
    }

    async fn get_elements(
        &self,
        request: Request<GetElementsRequest>,
    ) -> Result<Response<GetElementsResponse>, Status> {
        let req = request.into_inner();

        let tree_lock = self.wrapper.context_tree.lock().unwrap();
        if let Some(ref tree) = *tree_lock {
            let default_locator = Locator {
                name: None,
                role: None,
                description: None,
                text: None,
                index_in_parent: None,
                ascendant: None,
                descendants: Vec::new(),
            };
            let locator = req.locator.as_ref().unwrap_or(&default_locator);

            let nodes = tree.get_elements(locator);

            let elements = nodes.into_iter().map(element_from_context_node).collect();

            Ok(Response::new(GetElementsResponse {
                elements,
                error_message: String::new(),
            }))
        } else {
            Ok(Response::new(GetElementsResponse {
                elements: Vec::new(),
                error_message: "No window selected. Call SelectWindowByTitle first.".to_string(),
            }))
        }
    }

    async fn click_element(
        &self,
        request: Request<ClickElementRequest>,
    ) -> Result<Response<ClickElementResponse>, Status> {
        let req = request.into_inner();
        let wrapper = self.wrapper.clone();

        let result = tokio::task::spawn_blocking(move || wrapper.click_element(req.handle))
            .await
            .map_err(|e| Status::internal(format!("Task failed: {}", e)))?;

        match result {
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

    async fn type_text(
        &self,
        request: Request<TypeTextRequest>,
    ) -> Result<Response<TypeTextResponse>, Status> {
        let req = request.into_inner();
        let wrapper = self.wrapper.clone();
        let text = req.text.clone();

        let result = tokio::task::spawn_blocking(move || wrapper.type_text(req.handle, &text))
            .await
            .map_err(|e| Status::internal(format!("Task failed: {}", e)))?;

        match result {
            Ok(()) => Ok(Response::new(TypeTextResponse {
                success: true,
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(TypeTextResponse {
                success: false,
                error_message: e,
            })),
        }
    }

    async fn read_table(
        &self,
        _request: Request<ReadTableRequest>,
    ) -> Result<Response<ReadTableResponse>, Status> {
        Ok(Response::new(ReadTableResponse {
            table: None,
            error_message: "read_table not yet implemented".to_string(),
        }))
    }

    async fn wait_until_element_exists(
        &self,
        _request: Request<WaitUntilElementExistsRequest>,
    ) -> Result<Response<WaitUntilElementExistsResponse>, Status> {
        Ok(Response::new(WaitUntilElementExistsResponse {
            exists: false,
            error_message: "wait_until_element_exists not yet implemented".to_string(),
        }))
    }

    async fn get_version_info(
        &self,
        _request: Request<GetVersionInfoRequest>,
    ) -> Result<Response<GetVersionInfoResponse>, Status> {
        let wrapper = self.wrapper.clone();

        let result = tokio::task::spawn_blocking(move || wrapper.get_version_info())
            .await
            .map_err(|e| Status::internal(format!("Task failed: {}", e)))?;

        match result {
            Ok(info) => {
                let version_info = VersionInfo {
                    vm_version: String::from_utf16_lossy(&info.VMversion)
                        .trim_end_matches('\0')
                        .to_string(),
                    bridge_java_class_version: String::from_utf16_lossy(
                        &info.bridgeJavaClassVersion,
                    )
                    .trim_end_matches('\0')
                    .to_string(),
                    bridge_java_dll_version: String::from_utf16_lossy(&info.bridgeJavaDLLVersion)
                        .trim_end_matches('\0')
                        .to_string(),
                    bridge_win_dll_version: String::from_utf16_lossy(&info.bridgeWinDLLVersion)
                        .trim_end_matches('\0')
                        .to_string(),
                };

                Ok(Response::new(GetVersionInfoResponse {
                    version_info: Some(version_info),
                    error_message: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(GetVersionInfoResponse {
                version_info: None,
                error_message: e,
            })),
        }
    }

    type SubscribeCallbacksStream = ReceiverStream<Result<ProtoCallbackEvent, Status>>;

    async fn subscribe_callbacks(
        &self,
        _request: Request<SubscribeCallbacksRequest>,
    ) -> Result<Response<Self::SubscribeCallbacksStream>, Status> {
        let (tx, mut rx) = mpsc::channel::<crate::JabCallbackEvent>(100);

        let (proto_tx, proto_rx) = mpsc::channel(100);

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let proto_event = ProtoCallbackEvent {
                    event_type: event.event_type,
                    vm_id: event.vm_id as i64,
                    context_handle: event.context_handle,
                    message: event.message,
                    event_time: event.event_time,
                };
                if proto_tx.send(Ok(proto_event)).await.is_err() {
                    break;
                }
            }
        });

        self.wrapper.set_event_sender(tx);

        Ok(Response::new(ReceiverStream::new(proto_rx)))
    }
}

fn window_info_to_proto(w: &WindowInfo) -> ProtoWindowInfo {
    ProtoWindowInfo {
        vm_id: w.vm_id as i64,
        hwnd: w.hwnd,
        title: w.title.clone(),
        role: w.role.clone(),
    }
}

fn element_from_context_node(node: &crate::context_tree::ContextNode) -> Element {
    Element {
        handle: node.handle,
        name: node.name.clone(),
        role: node.role.clone(),
        description: node.description.clone(),
        text: node.text.clone(),
        x: node.x,
        y: node.y,
        width: node.width,
        height: node.height,
        accessible_action: node.accessible_action,
        accessible_text: node.accessible_text,
        accessible_selection: node.accessible_selection,
        visible_children_count: node.visible_children_count,
        index_in_parent: node.index_in_parent,
        children: node.children.iter().map(element_from_context_node).collect(),
    }
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub vm_id: i32,
    pub hwnd: u64,
    pub title: String,
    pub role: String,
}

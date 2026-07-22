use tonic::Status;

use jab_wrapper::{
    context_tree::{ContextNode, ContextTree},
    selector::{Locator, Selector},
};

pub(crate) fn no_window_selected<T>() -> Result<T, Status> {
    Err(Status::failed_precondition(
        "No window selected. Call SelectWindow first.",
    ))
}

pub(crate) fn parse_locator(locator: &Locator) -> Result<Selector, Status> {
    locator
        .parse()
        .map_err(|e| Status::invalid_argument(e.to_string()))
}

pub(crate) fn find_node<'a>(
    tree: &'a Option<ContextTree>,
    selector: &Selector,
) -> Result<&'a ContextNode, Status> {
    let tree = match tree.as_ref() {
        Some(tree) => tree,
        None => return no_window_selected(),
    };

    match tree.get_node(selector) {
        Some(node) => Ok(node),
        None => Err(Status::not_found(format!(
            "No element matches selector {}",
            selector
        ))),
    }
}

pub(crate) fn find_nodes<'a>(
    tree: &'a Option<ContextTree>,
    selector: &Selector,
) -> Result<Vec<&'a ContextNode>, Status> {
    let tree = match tree.as_ref() {
        Some(tree) => tree,
        None => return no_window_selected(),
    };

    Ok(tree.get_nodes(selector))
}

pub(crate) fn refresh_tree(tree: &mut Option<ContextTree>) -> Result<(), Status> {
    let root_obj = match tree.take() {
        Some(tree) => tree
            .into_root()
            .map_err(|e| Status::internal(e.to_string()))?,
        None => return no_window_selected(),
    };

    let new_tree = ContextTree::from_root(root_obj, None);
    *tree = Some(new_tree);
    Ok(())
}

pub(crate) fn get_root(tree: &Option<ContextTree>) -> Result<&ContextNode, Status> {
    match tree.as_ref() {
        Some(tree) => tree.root().map_err(|e| Status::internal(e.to_string())),
        None => no_window_selected(),
    }
}

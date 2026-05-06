#[derive(Debug, Clone)]
pub struct SearchElement {
    pub key: String,
    pub value: String,
    pub strict: bool,
}

#[derive(Debug, Clone)]
pub struct ContextNode {
    pub vm_id: i32,
    pub context: i64,
    pub handle: u64,
    pub name: String,
    pub role: String,
    pub description: String,
    pub children: Vec<ContextNode>,
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub accessible_action: bool,
    pub accessible_text: bool,
    pub accessible_selection: bool,
    pub visible_children_count: i32,
    pub index_in_parent: i32,
}

#[derive(Debug, Clone)]
pub struct ContextTree {
    pub root: Option<ContextNode>,
    pub max_depth: Option<i32>,
}

impl ContextTree {
    pub fn from_root(
        vm_id: i32,
        root_context: i64,
        max_depth: Option<i32>,
        jab: &super::jab_wrapper::JabWrapper,
    ) -> Self {
        let mut tree = ContextTree {
            root: None,
            max_depth,
        };

        if root_context == 0 {
            return tree;
        }

        tree.root = Some(Self::build_node(vm_id, root_context, 0, max_depth, jab));

        tree
    }

    fn build_node(
        vm_id: i32,
        context: i64,
        depth: i32,
        max_depth: Option<i32>,
        jab: &super::jab_wrapper::JabWrapper,
    ) -> ContextNode {
        let handle = jab.register_element(vm_id, context);

        let mut node = ContextNode {
            vm_id,
            context,
            handle,
            name: String::new(),
            role: String::new(),
            description: String::new(),
            children: Vec::new(),
            text: String::new(),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            accessible_action: false,
            accessible_text: false,
            accessible_selection: false,
            visible_children_count: 0,
            index_in_parent: 0,
        };

        if let Some(info) = jab.get_accessible_context_info(vm_id, context) {
            node.name = String::from_utf16_lossy(&info.name)
                .trim_end_matches('\0')
                .to_string();
            node.role = String::from_utf16_lossy(&info.role)
                .trim_end_matches('\0')
                .to_string();
            node.description = String::from_utf16_lossy(&info.description)
                .trim_end_matches('\0')
                .to_string();
            node.x = info.x;
            node.y = info.y;
            node.width = info.width;
            node.height = info.height;
            node.accessible_action = info.accessibleAction != 0;
            node.accessible_text = info.accessibleText != 0;
            node.accessible_selection = info.accessibleSelection != 0;
            node.visible_children_count = info.childrenCount;
            node.index_in_parent = info.indexInParent;
        }

        if let Some(max) = max_depth
            && depth >= max
        {
            return node;
        }

        unsafe {
            let child_count = if let Some(info) = jab.get_accessible_context_info(vm_id, context) {
                info.childrenCount
            } else {
                0
            };

            for i in 0..child_count {
                let child_context =
                    super::bindings::GetAccessibleChildFromContext(vm_id as _, context, i);
                if child_context != 0 {
                    node.children.push(Self::build_node(
                        vm_id,
                        child_context,
                        depth + 1,
                        max_depth,
                        jab,
                    ));
                }
            }
        }

        node
    }

    pub fn get_by_attrs(&self, searches: &[Vec<SearchElement>]) -> Vec<&ContextNode> {
        let mut results = Vec::new();

        if let Some(ref root) = self.root {
            for search_path in searches {
                self.search_node(root, search_path, 0, &mut results);
            }
        }

        results
    }

    fn search_node<'a>(
        &self,
        node: &'a ContextNode,
        search_path: &[SearchElement],
        depth: usize,
        results: &mut Vec<&'a ContextNode>,
    ) {
        if depth >= search_path.len() {
            results.push(node);
            return;
        }

        let search = &search_path[depth];
        let matches = self.node_matches(node, search);

        if matches && depth == search_path.len() - 1 {
            results.push(node);
        }

        if matches || depth == 0 {
            for child in &node.children {
                self.search_node(child, search_path, depth + 1, results);
            }
        }
    }

    fn node_matches(&self, node: &ContextNode, search: &SearchElement) -> bool {
        let value_lower = search.value.to_lowercase();

        match search.key.as_str() {
            "name" => node.name.to_lowercase().contains(&value_lower),
            "role" => node.role.to_lowercase().contains(&value_lower),
            "description" => node.description.to_lowercase().contains(&value_lower),
            "text" => node.text.to_lowercase().contains(&value_lower),
            _ => false,
        }
    }
}

pub fn parse_locator(locator: &str, strict_default: bool) -> Vec<Vec<SearchElement>> {
    let mut result = Vec::new();

    if locator.is_empty() {
        return result;
    }

    let levels: Vec<&str> = locator.split('>').map(|s| s.trim()).collect();
    let mut search_path = Vec::new();

    for level in levels {
        let parts: Vec<&str> = level.split(" and ").map(|s| s.trim()).collect();

        for part in parts {
            if part.starts_with("strict:") {
                continue;
            }

            let (key, value) = if let Some(pos) = part.find(':') {
                let key = part[..pos].trim();
                let value = part[pos + 1..].trim();
                (key.to_string(), value.to_string())
            } else {
                ("name".to_string(), part.to_string())
            };

            search_path.push(SearchElement {
                key,
                value,
                strict: strict_default,
            });
        }
    }

    result.push(search_path);
    result
}

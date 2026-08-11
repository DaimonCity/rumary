use crate::domain::perms::PermissionNode;
use crate::dto::api::response::group::GroupPermissionResponse;

impl From<&PermissionNode> for GroupPermissionResponse {
    fn from(node: &PermissionNode) -> Self {
        Self {
            key: node.key().as_str().to_string(),
            allow: node.value().is_allow(),
            context: node
                .context()
                .iter()
                .map(|(k, v)| (k.as_str().to_string(), v.as_str().to_string()))
                .collect(),
            source_priority: node.source_priority().get(),
            expires_at: node.expires_at().map(|e| e.get()),
        }
    }
}
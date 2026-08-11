use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::dto::api::request::share_target::ShareTargetRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewInstanceRequest {
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub loader: String,
    pub loader_version: Option<String>,
    /// Сделать ресурс видимым всем, кто прошёл RBAC-гейт `instance.get`,
    /// независимо от ACL. Отдельный, более сильный переключатель — если
    /// `true`, `share_with` ниже фактически не имеет значения при проверке
    /// доступа (см. `require_resource_access`: `is_public` даёт ранний return).
    #[serde(default)]
    pub is_public: bool,
    /// Кому, кроме автора, дать доступ на чтение сразу при создании.
    /// Пусто по умолчанию — только автор.
    #[serde(default)]
    pub share_with: Vec<ShareTargetRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInstanceRequest {
    pub icon: Option<String>,
    pub dir_name: Option<String>,
    pub display_name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetInstanceRequest {
    pub instance_id: Uuid,
}

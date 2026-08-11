use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::dto::api::request::share_target::ShareTargetRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigurationsRequest {
    pub instance_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewConfigurationRequest {
    pub icon: String,
    pub dir_name: String,
    pub display_name: String,
    pub instance_id: Uuid,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigurationRequest {
    pub icon: Option<String>,
    pub dir_name: Option<String>,
    pub display_name: Option<String>,
    pub instance_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetConfigurationRequest {
    pub configuration_id: Uuid,
}

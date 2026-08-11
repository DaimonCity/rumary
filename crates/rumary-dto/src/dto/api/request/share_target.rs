use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Сериализуемая версия `ShareTarget` для HTTP-слоя. Хендлер конвертирует
/// её в доменный `ShareTarget` перед вызовом `register_created_resource` —
/// сюда НЕ должны попадать доменные value-object (`GroupName`, `GroupWeight`)
/// напрямую, чтобы невалидный JSON с фронта падал с понятной 400-кой на
/// этапе конвертации, а не глубже в бизнес-логике.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShareTargetRequest {
    /// "Мой ранг и выше" — вес считается от ролей автора в момент создания.
    Peers,
    /// Явный порог ранга, не привязанный к автору.
    MinRank { weight: i32 },
    /// Конкретная группа по имени.
    Group { name: String },
    /// Конкретные пользователи по ID.
    Users { user_ids: Vec<Uuid> },
}

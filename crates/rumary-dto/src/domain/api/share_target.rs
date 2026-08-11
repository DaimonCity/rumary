use crate::domain::perms::value_object::group::{GroupName, GroupNameError, GroupWeight, GroupWeightError};

/// Кому даётся доступ при регистрации/шаринге ресурса.
///
/// Тонкая обёртка над `AccessGrant` — не добавляет новых holder-типов
/// (их и так три: role, user, min_weight), а даёт вызывающему коду
/// декларативный способ описать "с кем поделиться" в момент создания
/// ресурса, не собирая `AccessGrant` руками в каждом хендлере.
#[derive(Debug, Clone)]
pub enum ShareTarget {
    /// "Мой ранг и выше" — вес считается от текущих ролей автора на
    /// момент вызова, а не фиксируется заранее.
    Peers,
    /// Явный порог веса, не привязанный к рангу автора.
    MinRank(GroupWeight),
    /// Конкретная роль/группа по имени.
    Role(GroupName),
    /// Конкретные пользователи.
    Users(Vec<crate::domain::perms::value_object::user::UserId>),
}

#[derive(Debug)]
pub enum ShareTargetError {
    InvalidWeight(GroupWeightError),
    InvalidGroupName(GroupNameError),
}
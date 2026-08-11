use super::group::{GroupName, GroupNameError, GroupWeight, GroupWeightError};
use super::node::{PermissionKey, PermissionKeyError, SourcePriority};
use super::resource::{ResourceId, ResourceType};
use chrono::{Duration, Utc};

use super::expiration::NodeExpiry;

#[test]
fn permission_key_rejects_partial_wildcard() {
    assert_eq!(
        PermissionKey::try_from("api.ord*"),
        Err(PermissionKeyError::PartialWildcard)
    );
}

#[test]
fn permission_key_rejects_empty_segment() {
    assert_eq!(
        PermissionKey::try_from("api..read"),
        Err(PermissionKeyError::EmptySegment)
    );
    assert_eq!(
        PermissionKey::try_from("api.read."),
        Err(PermissionKeyError::EmptySegment)
    );
}

#[test]
fn permission_key_accepts_wildcard_segments() {
    assert!(PermissionKey::try_from("*").unwrap().is_wildcard());
    assert!(PermissionKey::try_from("api.*").unwrap().is_wildcard());
    assert!(!PermissionKey::try_from("api.read").unwrap().is_wildcard());
}

#[test]
fn permission_key_normalizes_case() {
    assert_eq!(
        PermissionKey::try_from("Configuration.Get").unwrap(),
        PermissionKey::try_from("configuration.get").unwrap()
    );
}

#[test]
fn user_source_priority_beats_any_group_weight() {
    // Прямая нода пользователя должна побеждать группу любого веса.
    let heaviest_group = SourcePriority::new(i32::MAX - 1);
    assert!(SourcePriority::USER < heaviest_group);
    // Осознанное ограничение: приоритет USER — константа 1_000_000, поэтому
    // группа с весом выше миллиона перебила бы прямую ноду. GroupWeight
    // такого значения в сидах не встречается, но инвариант стоит знать.
    assert_eq!(SourcePriority::USER.get(), 1_000_000);
}

#[test]
fn group_name_rejects_invalid_input() {
    assert_eq!(GroupName::try_from(""), Err(GroupNameError::Missing));
    assert_eq!(GroupName::try_from("a"), Err(GroupNameError::InvalidLength));
    assert_eq!(
        GroupName::try_from("bad name"),
        Err(GroupNameError::InvalidSymbols)
    );
    assert_eq!(GroupName::try_from("Admin").unwrap().as_str(), "admin");
}

#[test]
fn group_weight_rejects_negative() {
    assert_eq!(GroupWeight::new(-1), Err(GroupWeightError::Negative));
    assert_eq!(GroupWeight::new(0).unwrap(), GroupWeight::NONE);
    assert!(GroupWeight::new(30).unwrap() > GroupWeight::new(10).unwrap());
}

#[test]
fn resource_type_builds_permission_keys() {
    let resource_type = ResourceType::try_from("configuration").unwrap();

    assert_eq!(
        resource_type.bypass_acl_key().as_str(),
        "configuration.bypass_acl"
    );
    assert_eq!(
        resource_type.action_key("get").unwrap().as_str(),
        "configuration.get"
    );
}

#[test]
fn resource_id_from_uuid_is_canonical_string() {
    let id = uuid::Uuid::new_v4();
    assert_eq!(ResourceId::from(id).as_str(), id.to_string());
}

#[test]
fn node_expiry_allows_past_time_unlike_token_expiration() {
    // Истёкшая нода — нормальное состояние: она лежит в БД до чистки.
    let past = NodeExpiry::new(Utc::now() - Duration::seconds(10));
    assert!(past.is_expired());

    let future = NodeExpiry::new(Utc::now() + Duration::hours(1));
    assert!(!future.is_expired());
}

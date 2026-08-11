use crate::domain::perms::context::ContextSet;
use crate::domain::perms::value_object::expiration::NodeExpiry;
use crate::domain::perms::value_object::node::{PermissionKey, SourcePriority};
use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};

/// Одна нода прав: `configuration.get` -> true/false, с контекстом и
/// опциональным сроком действия.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionNode {
    key: PermissionKey,
    value: NodeValue,
    context: ContextSet,
    /// None = бессрочно
    expires_at: Option<NodeExpiry>,
    /// приоритет источника: прямая нода пользователя > группа (weight группы)
    source_priority: SourcePriority,
}

impl PermissionNode {
    pub fn new(
        key: PermissionKey,
        value: NodeValue,
        context: ContextSet,
        expires_at: Option<NodeExpiry>,
        source_priority: SourcePriority,
    ) -> Self {
        Self {
            key,
            value,
            context,
            expires_at,
            source_priority,
        }
    }

    /// Бессрочная нода без контекста — самый частый случай в сидах и тестах.
    pub fn permanent(key: PermissionKey, value: NodeValue, source_priority: SourcePriority) -> Self {
        Self::new(key, value, ContextSet::empty(), None, source_priority)
    }

    pub fn key(&self) -> &PermissionKey {
        &self.key
    }

    pub fn value(&self) -> NodeValue {
        self.value
    }

    pub fn context(&self) -> &ContextSet {
        &self.context
    }

    pub fn expires_at(&self) -> Option<NodeExpiry> {
        self.expires_at
    }

    pub fn source_priority(&self) -> SourcePriority {
        self.source_priority
    }

    /// Действует ли нода сейчас: не истекла и её контекст удовлетворён
    /// контекстом запроса.
    pub fn is_active(&self, request_ctx: &ContextSet) -> bool {
        self.is_active_at(request_ctx, Utc::now())
    }

    /// То же самое с явным "сейчас" — так проверку истечения можно
    /// тестировать без ожидания реального времени.
    pub fn is_active_at(&self, request_ctx: &ContextSet, now: DateTime<Utc>) -> bool {
        let not_expired = self
            .expires_at
            .is_none_or(|expiry| !expiry.is_expired_at(now));

        not_expired && self.context.is_satisfied_by(request_ctx)
    }
}

/// Значение ноды: явное разрешение или явный запрет. Именованный тип вместо
/// `bool` — на месте вызова `set_group_permission(.., NodeValue::Deny, ..)`
/// читается однозначно, в отличие от `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeValue {
    Allow,
    Deny,
}

impl NodeValue {
    pub fn is_allow(self) -> bool {
        matches!(self, Self::Allow)
    }
}

impl From<bool> for NodeValue {
    fn from(value: bool) -> Self {
        if value { Self::Allow } else { Self::Deny }
    }
}

impl From<NodeValue> for bool {
    fn from(value: NodeValue) -> Self {
        value.is_allow()
    }
}

impl Display for NodeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => f.write_str("allow"),
            Self::Deny => f.write_str("deny"),
        }
    }
}

/// Результат проверки права. Undefined ("ничего не сказано") — это не то же
/// самое, что явный Deny; политику "Undefined => запретить" применяет
/// вызывающий код.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tristate {
    Allow,
    Deny,
    Undefined,
}

impl Tristate {
    pub fn as_bool(self, default_if_undefined: bool) -> bool {
        match self {
            Self::Allow => true,
            Self::Deny => false,
            Self::Undefined => default_if_undefined,
        }
    }

    pub fn is_undefined(self) -> bool {
        matches!(self, Self::Undefined)
    }
}

impl From<NodeValue> for Tristate {
    fn from(value: NodeValue) -> Self {
        match value {
            NodeValue::Allow => Self::Allow,
            NodeValue::Deny => Self::Deny,
        }
    }
}

impl From<Option<NodeValue>> for Tristate {
    fn from(value: Option<NodeValue>) -> Self {
        value.map_or(Self::Undefined, Self::from)
    }
}

impl Display for Tristate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => f.write_str("allow"),
            Self::Deny => f.write_str("deny"),
            Self::Undefined => f.write_str("undefined"),
        }
    }
}

use crate::domain::perms::value_object::context::{
    ContextKey, ContextKeyError, ContextValue, ContextValueError,
};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};

/// Набор пар key=value, описывающий условия, в которых проверяется право
/// (например tenant=acme, env=prod). Пустой контекст = "везде".
///
/// Внутри `BTreeMap`, а не `Vec<(String, String)>`: ключи уникальны по
/// определению (два разных значения одного ключа в одном наборе — это
/// противоречие, а не два условия), порядок стабилен без сортировки на каждый
/// `cache_key`, а поиск по ключу — логарифмический, а не линейный.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextSet(BTreeMap<ContextKey, ContextValue>);

impl ContextSet {
    pub fn empty() -> Self {
        Self(BTreeMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Собрать набор из уже провалидированных пар.
    pub fn new(pairs: impl IntoIterator<Item = (ContextKey, ContextValue)>) -> Self {
        Self(pairs.into_iter().collect())
    }

    /// Добавить условие; возвращает предыдущее значение этого ключа, если было.
    pub fn insert(&mut self, key: ContextKey, value: ContextValue) -> Option<ContextValue> {
        self.0.insert(key, value)
    }

    pub fn get(&self, key: &ContextKey) -> Option<&ContextValue> {
        self.0.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ContextKey, &ContextValue)> {
        self.0.iter()
    }

    /// true, если все условия из `self` присутствуют в контексте запроса.
    /// Нода с контекстом {tenant=acme} активна только при запросе с tenant=acme;
    /// нода без контекста активна всегда.
    pub fn is_satisfied_by(&self, request_ctx: &Self) -> bool {
        self.0
            .iter()
            .all(|(key, value)| request_ctx.0.get(key) == Some(value))
    }

    /// Стабильный хэш для использования в качестве части кэш-ключа.
    /// `BTreeMap` уже упорядочен, так что сортировать перед хэшированием не нужно.
    pub fn cache_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}

impl FromIterator<(ContextKey, ContextValue)> for ContextSet {
    fn from_iter<T: IntoIterator<Item = (ContextKey, ContextValue)>>(iter: T) -> Self {
        Self::new(iter)
    }
}

impl Display for ContextSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for (key, value) in &self.0 {
            if !first {
                f.write_str(", ")?;
            }
            write!(f, "{key}={value}")?;
            first = false;
        }
        Ok(())
    }
}

/// Сборка контекста из сырых строк — то, что приходит из HTTP-запроса или из
/// JSONB-колонки. Валидация каждой пары происходит здесь, поэтому дальше по
/// коду невалидного контекста уже не бывает.
impl ContextSet {
    pub fn try_from_pairs<K, V>(
        pairs: impl IntoIterator<Item = (K, V)>,
    ) -> Result<Self, ContextSetError>
    where
        K: Into<String>,
        V: Into<String>,
    {
        pairs
            .into_iter()
            .map(|(key, value)| {
                let key = ContextKey::try_from(key.into())?;
                let value = ContextValue::try_from(value.into())?;
                Ok((key, value))
            })
            .collect::<Result<BTreeMap<_, _>, ContextSetError>>()
            .map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextSetError {
    Key(ContextKeyError),
    Value(ContextValueError),
}

impl Display for ContextSetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(err) => err.fmt(f),
            Self::Value(err) => err.fmt(f),
        }
    }
}

impl From<ContextKeyError> for ContextSetError {
    fn from(value: ContextKeyError) -> Self {
        Self::Key(value)
    }
}

impl From<ContextValueError> for ContextSetError {
    fn from(value: ContextValueError) -> Self {
        Self::Value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pairs: &[(&str, &str)]) -> ContextSet {
        ContextSet::try_from_pairs(pairs.iter().copied()).expect("valid context")
    }

    #[test]
    fn empty_context_always_satisfied() {
        assert!(ContextSet::empty().is_satisfied_by(&ctx(&[("tenant", "acme")])));
    }

    #[test]
    fn matching_context_is_satisfied() {
        let node_ctx = ctx(&[("tenant", "acme")]);
        assert!(node_ctx.is_satisfied_by(&ctx(&[("tenant", "acme"), ("env", "prod")])));
    }

    #[test]
    fn mismatched_context_is_not_satisfied() {
        let node_ctx = ctx(&[("tenant", "acme")]);
        assert!(!node_ctx.is_satisfied_by(&ctx(&[("tenant", "other")])));
    }

    #[test]
    fn cache_key_is_order_independent() {
        let a = ctx(&[("tenant", "acme"), ("env", "prod")]);
        let b = ctx(&[("env", "prod"), ("tenant", "acme")]);
        assert_eq!(a.cache_key(), b.cache_key());
    }

    #[test]
    fn key_case_is_normalized() {
        assert_eq!(ctx(&[("Tenant", "acme")]), ctx(&[("tenant", "acme")]));
    }

    #[test]
    fn invalid_key_is_rejected() {
        assert!(ContextSet::try_from_pairs([("ten ant", "acme")]).is_err());
    }
}

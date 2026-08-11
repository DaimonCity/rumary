use chrono::{DateTime, Utc};
use rumary_dto::domain::perms::value_object::node::PermissionKey;
use rumary_dto::domain::perms::{ContextSet, PermissionNode, Tristate};

/// Специфичность совпадения pattern (нода) -> checked (запрашиваемое право).
///
/// `api.orders.*` совпадает с `api.orders.read`, но менее специфично, чем
/// точная нода `api.orders.read`. Возвращает None, если pattern не подходит.
///
/// Каждый точный сегмент даёт +10, wildcard закрывает хвост и добавляет 1 —
/// иначе `api.*` и `*` получили бы одинаковый счёт для `api.orders.read`, и
/// более узкий грант проигрывал бы более широкому по случайности порядка нод.
fn match_specificity(pattern: &PermissionKey, checked: &PermissionKey) -> Option<u32> {
    let checked_segments: Vec<&str> = checked.segments().collect();

    let mut score = 0u32;
    let mut pattern_len = 0usize;

    for (index, segment) in pattern.segments().enumerate() {
        pattern_len += 1;

        /// Каждый точный сегмент даёт +10, wild
        if segment == "*" {
            // Wildcard закрывает весь хвост: `a.*` покрывает и `a.b`, и `a.b.c`.
            return Some(score + 1);
        }

        match checked_segments.get(index) {
            Some(checked_segment) if *checked_segment == segment => score += 10,
            _ => return None,
        }
    }

    // Pattern без wildcard совпадает только при равной длине:
    // `api.orders` не даёт права на `api.orders.delete`.
    (pattern_len == checked_segments.len()).then_some(score)
}

/// Резолвит право `checked` по уже собранному списку нод пользователя и групп.
///
/// Порядок разрешения конфликтов: специфичность -> приоритет источника ->
/// явный Deny. Последний тайбрейк важен: при полностью равных специфичности и
/// приоритете (две группы одного веса, одна разрешает, другая запрещает)
/// результат должен быть детерминированным и fail-closed, а не зависеть от
/// порядка строк, который вернул Postgres.
pub fn resolve(
    checked: &PermissionKey,
    request_ctx: &ContextSet,
    nodes: &[PermissionNode],
) -> Tristate {
    resolve_at(checked, request_ctx, nodes, Utc::now())
}

/// Тот же резолвинг с явным "сейчас" — для тестов истечения нод.
pub fn resolve_at(
    checked: &PermissionKey,
    request_ctx: &ContextSet,
    nodes: &[PermissionNode],
    now: DateTime<Utc>,
) -> Tristate {
    nodes
        .iter()
        .filter(|node| node.is_active_at(request_ctx, now))
        .filter_map(|node| {
            match_specificity(node.key(), checked)
                .map(|specificity| (specificity, node.source_priority(), node.value()))
        })
        // Deny сортируется ниже Allow, поэтому при равенстве первых двух ключей
        // max_by_key выбрал бы Allow — инвертируем, чтобы победил Deny.
        .max_by_key(|(specificity, priority, value)| (*specificity, *priority, !value.is_allow()))
        .map(|(_, _, value)| value)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use rumary_dto::domain::perms::NodeValue;
    use rumary_dto::domain::perms::value_object::expiration::NodeExpiry;
    use rumary_dto::domain::perms::value_object::node::SourcePriority;

    fn key(value: &str) -> PermissionKey {
        PermissionKey::try_from(value).expect("valid permission key")
    }

    fn node(pattern: &str, value: bool, priority: i32) -> PermissionNode {
        PermissionNode::permanent(
            key(pattern),
            NodeValue::from(value),
            SourcePriority::new(priority),
        )
    }

    fn ctx(pairs: &[(&str, &str)]) -> ContextSet {
        ContextSet::try_from_pairs(pairs.iter().copied()).expect("valid context")
    }

    #[test]
    fn exact_node_wins_over_wildcard() {
        let nodes = vec![
            node("api.orders.*", true, 0),
            node("api.orders.delete", false, 0),
        ];
        assert_eq!(
            resolve(&key("api.orders.delete"), &ContextSet::empty(), &nodes),
            Tristate::Deny
        );
        assert_eq!(
            resolve(&key("api.orders.read"), &ContextSet::empty(), &nodes),
            Tristate::Allow
        );
    }

    #[test]
    fn narrower_wildcard_wins_over_root_wildcard() {
        // owner-нода "*" разрешает всё, но точечный запрет на поддерево должен
        // перебить её: он специфичнее.
        let nodes = vec![node("*", true, 0), node("api.billing.*", false, 0)];
        assert_eq!(
            resolve(&key("api.billing.read"), &ContextSet::empty(), &nodes),
            Tristate::Deny
        );
        assert_eq!(
            resolve(&key("api.orders.read"), &ContextSet::empty(), &nodes),
            Tristate::Allow
        );
    }

    #[test]
    fn higher_priority_source_wins_at_equal_specificity() {
        // группа (priority 0) разрешает, прямая нода пользователя запрещает
        let nodes = vec![
            node("api.billing.read", true, 0),
            node("api.billing.read", false, 100),
        ];
        assert_eq!(
            resolve(&key("api.billing.read"), &ContextSet::empty(), &nodes),
            Tristate::Deny
        );
    }

    #[test]
    fn deny_wins_on_full_tie() {
        let nodes = vec![
            node("api.orders.read", true, 10),
            node("api.orders.read", false, 10),
        ];
        assert_eq!(
            resolve(&key("api.orders.read"), &ContextSet::empty(), &nodes),
            Tristate::Deny
        );

        // и в обратном порядке — результат не зависит от порядка строк
        let reversed = vec![
            node("api.orders.read", false, 10),
            node("api.orders.read", true, 10),
        ];
        assert_eq!(
            resolve(&key("api.orders.read"), &ContextSet::empty(), &reversed),
            Tristate::Deny
        );
    }

    #[test]
    fn parent_key_does_not_grant_child() {
        let nodes = vec![node("api.orders", true, 0)];
        assert_eq!(
            resolve(&key("api.orders.read"), &ContextSet::empty(), &nodes),
            Tristate::Undefined
        );
    }

    #[test]
    fn no_match_is_undefined() {
        let nodes = vec![node("api.orders.read", true, 0)];
        assert_eq!(
            resolve(&key("api.billing.read"), &ContextSet::empty(), &nodes),
            Tristate::Undefined
        );
    }

    #[test]
    fn context_filters_inactive_nodes() {
        let nodes = vec![PermissionNode::new(
            key("api.orders.read"),
            NodeValue::Allow,
            ctx(&[("tenant", "acme")]),
            None,
            SourcePriority::ZERO,
        )];

        assert_eq!(
            resolve(&key("api.orders.read"), &ctx(&[("tenant", "other")]), &nodes),
            Tristate::Undefined
        );
        assert_eq!(
            resolve(&key("api.orders.read"), &ctx(&[("tenant", "acme")]), &nodes),
            Tristate::Allow
        );
    }

    #[test]
    fn expired_node_is_ignored() {
        let now = Utc::now();
        let nodes = vec![PermissionNode::new(
            key("api.orders.read"),
            NodeValue::Allow,
            ContextSet::empty(),
            Some(NodeExpiry::new(now - Duration::seconds(1))),
            SourcePriority::ZERO,
        )];

        assert_eq!(
            resolve_at(&key("api.orders.read"), &ContextSet::empty(), &nodes, now),
            Tristate::Undefined
        );
    }
}

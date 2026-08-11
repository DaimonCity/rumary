use crate::domain::perms::context::ContextSet;
use crate::domain::perms::value_object::expiration::NodeExpiry;
use crate::domain::perms::value_object::group::{GroupName, GroupWeight};

/// Группа (роль) — именованный набор прав с весом.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    name: GroupName,
    weight: GroupWeight,
}

impl Group {
    pub fn new(name: GroupName, weight: GroupWeight) -> Self {
        Self { name, weight }
    }

    pub fn name(&self) -> &GroupName {
        &self.name
    }

    pub fn weight(&self) -> GroupWeight {
        self.weight
    }
}

/// Членство пользователя в группе — с контекстом и опциональным сроком.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMembership {
    group: GroupName,
    context: ContextSet,
    expires_at: Option<NodeExpiry>,
}

impl GroupMembership {
    pub fn new(group: GroupName, context: ContextSet, expires_at: Option<NodeExpiry>) -> Self {
        Self {
            group,
            context,
            expires_at,
        }
    }

    pub fn permanent(group: GroupName) -> Self {
        Self::new(group, ContextSet::empty(), None)
    }

    pub fn group(&self) -> &GroupName {
        &self.group
    }

    pub fn context(&self) -> &ContextSet {
        &self.context
    }

    pub fn expires_at(&self) -> Option<NodeExpiry> {
        self.expires_at
    }
}

/// Роли пользователя и его ранг — то, что нужно ACL-проверке за один поход
/// в хранилище. Раньше эти два значения ходили по коду как `&[String]` и
/// `i32` рядом, и их приходилось передавать в правильном порядке вручную.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupSnapshot {
    groups: Vec<GroupName>,
    max_weight: GroupWeight,
}

impl GroupSnapshot {
    pub fn new(groups: Vec<GroupName>, max_weight: GroupWeight) -> Self {
        Self { groups, max_weight }
    }

    /// Снимок пользователя без ролей.
    pub fn empty() -> Self {
        Self::new(Vec::new(), GroupWeight::NONE)
    }

    /// Собрать снимок из списка групп, посчитав ранг как максимум их весов.
    pub fn from_groups(groups: Vec<Group>) -> Self {
        let max_weight = groups
            .iter()
            .map(Group::weight)
            .max()
            .unwrap_or(GroupWeight::NONE);

        Self::new(
            groups.into_iter().map(|group| group.name).collect(),
            max_weight,
        )
    }

    pub fn groups(&self) -> &[GroupName] {
        &self.groups
    }

    pub fn max_weight(&self) -> GroupWeight {
        self.max_weight
    }

    pub fn has_group(&self, group: &GroupName) -> bool {
        self.groups.contains(group)
    }

    /// Строковые имена ролей для биндинга в SQL (`= ANY($1)`).
    pub fn group_names(&self) -> Vec<String> {
        self.groups.iter().map(GroupName::to_string).collect()
    }

    /// Строго ли этот снимок выше по рангу, чем `other`.
    pub fn outranks(&self, other: &Self) -> bool {
        self.max_weight > other.max_weight
    }
}

/// Наследование: `group` получает права `parent` при выполнении `context`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupInheritance {
    group: GroupName,
    parent: GroupName,
    context: ContextSet,
}

impl GroupInheritance {
    pub fn new(group: GroupName, parent: GroupName, context: ContextSet) -> Self {
        Self {
            group,
            parent,
            context,
        }
    }

    pub fn group(&self) -> &GroupName {
        &self.group
    }

    pub fn parent(&self) -> &GroupName {
        &self.parent
    }

    pub fn context(&self) -> &ContextSet {
        &self.context
    }
}

/// Сводка по группе — то, что нужно показать в списке ролей админке,
/// без похода за полным набором прав/участников на каждую строку списка.
#[derive(Debug, Clone)]
pub struct GroupSummary {
    pub name: GroupName,
    pub weight: GroupWeight,
}

#[derive(Debug, Clone, Default)]
pub struct GroupListQuery {
    pub limit: Option<u32>,
    pub offset: u32,
}
#![allow(unused)]

use rumary_dto::domain::launcher::FileInfo;
use slint::platform::Key::H;
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

// type EntityId = usize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EntityId<T> {
    data: T,
}

impl<T> EntityId<T> {
    fn new(data: T) -> Self {
        Self { data }
    }

    fn into_inner(self) -> T {
        self.data
    }
}
type Archetype = u8;
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Entity(EntityId<usize>);

const NAME_BIT: u8 = 1 << 0;
const ICON_BIT: u8 = 1 << 1;
const HARD_CHECK_BIT: u8 = 1 << 2;
const SOFT_CHECK_BIT: u8 = 1 << 3;
const ID_BIT: u8 = 1 << 4;
#[derive(Debug, Clone, PartialEq, Eq)]
struct Name(String);
#[derive(Debug, Clone, PartialEq, Eq)]
struct Id(Uuid);
#[derive(Debug, Clone, PartialEq, Eq)]
struct Icon(String);
#[derive(Debug, Clone)]
struct HardCheck(Vec<FileInfo>);
#[derive(Debug, Clone)]
struct SoftCheck(Vec<FileInfo>);
#[derive(Debug, Clone, PartialEq, Eq)]
struct Column<T>(Vec<T>);

impl<T> Column<T> {
    pub fn new(c: usize) -> Self {
        Self(Vec::with_capacity(c))
    }

    pub fn push(&mut self, item: T) {
        self.0.push(item);
    }

    pub fn get(&self, c: usize) -> Option<&T> {
        self.0.get(c)
    }

    pub fn get_mut(&mut self, c: usize) -> Option<&mut T> {
        self.0.get_mut(c)
    }

    pub fn append(&mut self, other: &mut Column<T>) {
        self.0.append(&mut other.0)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn get_archetype(components: &[Components]) -> Archetype {
    let mut archetype = 0;
    for component in components {
        match component {
            Components::Id(_) => archetype |= ID_BIT,
            Components::Name(_) => archetype |= NAME_BIT,
            Components::Icon(_) => archetype |= ICON_BIT,
            Components::HardCheck(_) => archetype |= HARD_CHECK_BIT,
            Components::SoftCheck(_) => archetype |= SOFT_CHECK_BIT,
        }
    }

    archetype
}
#[derive(Debug)]
enum Components {
    Id(Id),
    Name(Name),
    Icon(Icon),
    HardCheck(HardCheck),
    SoftCheck(SoftCheck),
}

#[derive(Debug)]
struct Table {
    archetype: Archetype,
    entities: Vec<Entity>,

    ids: Option<Column<Id>>,
    names: Option<Column<Name>>,
    icons: Option<Column<Icon>>,
    hard_checks: Option<Column<HardCheck>>,
    soft_checks: Option<Column<SoftCheck>>,
}

impl Table {
    fn new(archetype: Archetype, n: usize) -> Self {
        let names = if archetype & NAME_BIT != 0 {
            Some(Column::new(n))
        } else {
            None
        };

        let ids = if archetype & ID_BIT != 0 {
            Some(Column::new(n))
        } else {
            None
        };

        let icons = if archetype & ICON_BIT != 0 {
            Some(Column::new(n))
        } else {
            None
        };

        let hard_checks = if archetype & HARD_CHECK_BIT != 0 {
            Some(Column::new(n))
        } else {
            None
        };

        let soft_checks = if archetype & SOFT_CHECK_BIT != 0 {
            Some(Column::new(n))
        } else {
            None
        };

        Self {
            ids,
            archetype,
            entities: Vec::with_capacity(n),
            names,
            icons,
            hard_checks,
            soft_checks,
        }
    }

    #[inline]
    fn add_entity(&mut self, entity: Entity, components: &[Components]) {
        self.entities.push(entity);

        for component in components {
            match component {
                Components::Id(id) => {
                    if let Some(ids) = &mut self.ids {
                        ids.push(id.clone());
                    }
                }
                Components::Name(name) => {
                    if let Some(names) = &mut self.names {
                        names.push(name.clone());
                    }
                }
                Components::Icon(icon) => {
                    if let Some(icons) = &mut self.icons {
                        icons.push(icon.clone());
                    }
                }
                Components::HardCheck(hard_check) => {
                    if let Some(hard_checks) = &mut self.hard_checks {
                        hard_checks.push(hard_check.clone());
                    }
                }
                Components::SoftCheck(soft_check) => {
                    if let Some(soft_checks) = &mut self.soft_checks {
                        soft_checks.push(soft_check.clone());
                    }
                }
            }
        }
    }
    #[inline]
    fn has_mask(&self, mask: Archetype) -> bool {
        self.archetype & mask != 0
    }
}
#[derive(Debug,  Default)]
struct World {
    entities_count: u32,
    // Box<dyn Any> позволяет нам прятать внутри HashMap любые Storage<T>
    storages: HashMap<std::any::TypeId, Box<dyn Any>>,
}

impl World {
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(EntityId {
            data: self.entities_count as usize,
        });
        self.entities_count += 1;
        entity
    }

    // Метод для добавления компонента любой сущности
    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        let type_id = std::any::TypeId::of::<T>();

        // Извлекаем существующее хранилище или создаем новое, если такого компонента еще не было
        let storage = self
            .storages
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStorage::<T>::new()));

        // Даункастим generic Any обратно в наш ComponentStorage<T>
        let storage_mut = storage.downcast_mut::<ComponentStorage<T>>().unwrap();

        storage_mut.insert(entity, component);
    }
}

// Хранилище для конкретного типа компонента T
struct ComponentStorage<T> {
    // Плотный массив: хранит сами данные
    dense_data: Vec<T>,
    // Параллельный ему массив: хранит ID сущностей, чтобы знать, кому принадлежит dense_data[i]
    dense_entities: Vec<Entity>,

    // Разреженный массив: индекс = Entity.0, значение = индекс в dense_data.
    // Используем usize::MAX как маркер того, что у сущности нет этого компонента
    sparse: Vec<usize>,
}

impl<T> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            dense_data: Vec::new(),
            dense_entities: Vec::new(),
            sparse: Vec::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, component: T) {
        let id = entity.0.data;

        // Расширяем разреженный массив, если ID сущности больше его текущей длины
        if id >= self.sparse.len() {
            self.sparse.resize(id + 1, usize::MAX);
        }

        // Если у сущности уже был этот компонент, просто обновляем его
        if self.sparse[id] != usize::MAX {
            let dense_idx = self.sparse[id];
            self.dense_data[dense_idx] = component;
            return;
        }

        // Если компонента не было: добавляем в конец плотного массива
        let dense_idx = self.dense_data.len();
        self.sparse[id] = dense_idx;

        self.dense_data.push(component);
        self.dense_entities.push(entity);
    }
}

#[cfg(test)]
mod test {
    use rumary_dto::dto::api::response::ProfileDto;
    use super::*;


    struct TestState {
       world: World,
    }

    impl TestState {
        fn new(world: World) -> Self {
             Self {
                 world
             }
        }

        fn create_profile_dto_entity(&mut self, profile_dto: ProfileDto) {
            let mut world = &mut self.world;

            let profile_entity= world.spawn();
            let id = profile_dto.id;
            let name = Name(profile_dto.name);
            // let hard_check = HardCheck(profile_dto.hard_check.into_iter().map(Into::into).collect());

        }
    }

    #[test]
    fn test() {

        let mut world = World::default();

        // Парсим первый JSON (например, UserDto)
        let user = world.spawn();
        world.add_component(user, Id(Uuid::new_v4()));
        world.add_component(user, Name("Алексей".to_string()));

        // Парсим второй JSON (например, AchievementDto)
        // У него совсем другой набор полей, но мы используем тот же world!
        let achievement = world.spawn();
        world.add_component(achievement, Id(Uuid::new_v4()));
        world.add_component(achievement, Name("Первая кровь".to_string()));
        world.add_component(achievement, Icon("⚔️".to_string()));

        println!("{:?}", world.storages);
        println!("{:?}", world.storages);
        println!("{:?}", world.storages);
        println!("{:?}", world.storages);
    }


}

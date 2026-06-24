# Rumary API

## Зачем оно надо?
Rumary - это экосистема для удобного менеджмента сборок майнкрафта под нужды вашего Сервера
Rumary API - это основная логика управления клиентами и профилями для лаунчера.

## Основные задачи, которое решает
### Создание клиентов (instance)
Тип запроса: POST

#### DTO Model
```json
{
  "icon": "<something>",
  "dir_name": "smp",
  "displayed_name": "SMP",
  "version": "1.21.1",
  "description": "It's just an yet another minecraft smp server",
  "loader": "vanilla",
  "loader_version": null
}
```
icon - не определенно
dir_name - название директории instance
displayed_name - имя instance в системе
version - версия майнкрафта
loader - загрузчик
loader_version - версия загрузчика (только у vanilla нет версии)

Соответственно, сервис по созданию профиля должен обработать эти поля.

#### Domain Model
```rust
pub struct NewInstance {
    icon: String,
    dir_name: String,
    displayed_name: String,
    version: String,
    description: String,
    loader: Loader
} 

#[derive(Copy, Clone, Debug, Default)]
pub enum Loader {
    Vanilla,
    Fabric(String),
    Forge(String),
    Neoforge(String)
}
```
Где String - версия загрузчика.

Обработка полей происходит следующим образом:
1. [DTO модель](#dto-model) преобразуется в [Domain Model](#domain-model)
2. Валидация полей: проверка существования нужных версий, загрузчиков и т.п.
3. Данные добавляются в БД через [функцию](#instancerepo-trait)

### Создание профиля (configuration)
Тип запроса: Mixed

URL: {uuid}

REST: Post

```json
{
  "displayed_name": "Low",
  "dir_name": "low"
}
```

### Загрузка модов

## Traits and Functions
### InstanceRepo trait
```rust
trait InstanceRepo { 
    fn create_instance(&self, new_instance: NewInstance) -> Result<Instance, Error>;
    fn update_instance(&self, update_instance: UpdateInstance) -> Result<Instance, Error>;
    fn find_instance(&self, uuid: Uuid) -> Result<Instance, Error>;
    fn delete_instance(&self, uuid: Uuid) -> Result<(), Error>;
}
```

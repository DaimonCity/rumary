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

#### DTO Model
```json
{
  "displayed_name": "Low",
  "client_uuid": "{uuid}",
  "dir_name": "low",
  "icon": "<url-to-icon>"
}
```

#### Domain Model
```rust
pub struct NewConfiguration {
    
}
```
Обработка полей происходит следующим образом:
1. [DTO модель](#dto-model-1) преобразуется в [Domain Model](#domain-model-1)
2. Валидация полей: проверка существования нужных версий, загрузчиков и т.п.
3. Данные добавляются в БД через [функцию](#instancerepo-trait)

### Загрузка файлов
Типа запроса: POST

#### DTO Model
##### Request 
```json
{
  "uuid": "<uuid>"
}
```

##### Response
```json
{
    "hard_dirs": ["mods"],
    "soft_dirs": ["config"],

    "files": {
        "mods/fabric-api.jar": {
            "sha1": "<hash>",
            "type": "required",
            "url": ""
        },
        "mods/map.jar": {
            "sha1": "<hash>",
            "type": "required",
            "url": ""
        },
        "config/test.json": {
            "sha1": "<hash>",
            "url": ""
        }
    }
} 
```

#### Процесс проверки
##### 1. Проходимся по хэшмапе files и разбиваем её элементы на две хэшмапы:
    - soft_files
    - hard_files

##### 2. Файлы из soft_files проверяем так же, как делаем с библиотеками.

##### 3. Чтобы проверить файлы из hard_files:
    **Сервис 1:**
        Проходится по директориям которые помечены как hard,
        для каждого файла смотрит, есть ли он в hard_files через ключи, и если нет - удаляет

    **Сервис 2:**
        Проходится по файлам из hard_files:
            1. Если файл required:
                Проверяет есть ли файл:
                - Нет: Скачивает
                - Есть: проверяет хэш и перескачивает при надобности 

            2. Если файл optional:
                Проверяет есть ли файл:
                - Нет: идёт дальше
                - Есть: проверяет хэш и перескачивает при надобности 


## Сценарии
1. Регистрация (есть прото-релиз)
2. Авторизация (есть прото-релиз)
3. Выйти из аккаунта
4. Выбор instance (клиент) - если есть несколько:
   - список доступных instances (get запрос)
5. Выбор конфигурацию (профиль) - если есть несколько
    - список доступных конфигураций (post запрос)
6. Нажатие кнопки ИГРАТЬ
    - [Загрузка файлов](#загрузка-файлов)
7. Выбор скина
    - Сохранение нового скина на сервере (post запрос)

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

### ConfigurationRepo trait
```rust
trait ConfigurationRepo { 
    fn create_config(&self, new_config: NewConfiguration) -> Result<Configuration, Error>;
    fn update_config(&self, update_instance: UpdateConfiguration) -> Result<Configuration, Error>;
    fn find_config(&self, uuid: Uuid) -> Result<Instance, Error>;
    fn delete_config(&self, uuid: Uuid) -> Result<(), Error>;
}
```
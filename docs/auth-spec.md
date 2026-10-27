# Модуль авторизації — контракт API

Це єдиний контракт. Кодер реалізує `src/auth.rs` точно за цим API.
Тестер пише `tests/auth.rs` (інтеграційні unit-тести) лише проти публічного API нижче.
Жодних зовнішніх залежностей (Cargo.toml лишається без `[dependencies]`).

## Публічний API `seed_10::auth`

```rust
use std::fmt;

/// Ролі користувачів (RBAC), впорядковані за рівнем привілеїв.
/// Guest < User < Admin (derive PartialOrd + Ord).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Guest,
    User,
    Admin,
}

/// Користувач.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub username: String,
    /// Демонстраційний (НЕ криптографічний) хеш пароля — FNV-1a 64-bit.
    pub password_hash: u64,
    pub role: Role,
}

/// Помилки авторизації.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthError {
    UnknownUser,   // користувача немає у сховищі
    WrongPassword, // пароль не збігається
    Forbidden,     // роль недостатня
}

impl fmt::Display for AuthError { /* людськочитані повідомлення */ }
impl std::error::Error for AuthError {}

pub type Result<T> = std::result::Result<T, AuthError>;

/// FNV-1a 64-bit хеш. Детермінований, демонстраційний, не для реальних паролів.
#[must_use]
pub fn hash_password(password: &str) -> u64;

/// Сховище користувачів (у памʼяті).
#[derive(Debug, Default, Clone)]
pub struct UserStore {
    // внутрішнє представлення на розсуд кодера (Vec/HashMap)
}

impl UserStore {
    #[must_use]
    pub fn new() -> Self;

    /// Додає користувача. Повертає `&mut Self` для ланцюжкового стилю.
    pub fn add(&mut self, user: User) -> &mut Self;

    /// Зручний конструктор: хешує пароль і додає користувача.
    pub fn register(&mut self, username: &str, password: &str, role: Role) -> &mut Self;

    /// Пошук за імʼям.
    #[must_use]
    pub fn get(&self, username: &str) -> Option<&User>;
}

/// Аутентифікація: перевіряє наявність користувача і пароль.
/// `UnknownUser`, якщо немає користувача; `WrongPassword`, якщо пароль не той.
pub fn authenticate(store: &UserStore, username: &str, password: &str) -> Result<&User>;

/// Авторизація: роль користувача має бути >= потрібної (`user.role >= required`).
/// Інакше `Forbidden`.
pub fn authorize(user: &User, required: Role) -> Result<()>;

/// Повний вхід: аутентифікація + авторизація.
pub fn login(store: &UserStore, username: &str, password: &str, required: Role) -> Result<&User>;
```

## Інтеграція в `src/lib.rs`

Додати `pub mod auth;` після наявного коду (і `pub use auth::...` не потрібно —
тести звертаються як `seed_10::auth::...`).

## Вимоги якості

- `#![forbid(unsafe_code)]` уже на рівні crate — жодного `unsafe`.
- Ніяких `unwrap()`/`panic!` у логіці (тільки `Result`/`Option`).
- clippy pedantic має проходити (`cargo clippy --all-targets`).
- Документація (`///`) на кожному публічному елементі.

## Приймальний критерій (для обох агентів)

`cargo test` та `cargo clippy --all-targets` мають завершуватися успішно.

//! Модуль авторизації: ролі, користувачі, хешування паролів і перевірки доступу.
//!
//! Хеш пароля тут **демонстраційний** (FNV-1a 64-bit) і не призначений
//! для реального захисту паролів.

use std::fmt;

/// Ролі користувачів (RBAC), впорядковані за рівнем привілеїв.
///
/// `Guest < User < Admin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// Гість: мінімальні привілеї.
    Guest,
    /// Звичайний користувач.
    User,
    /// Адміністратор: максимальні привілеї.
    Admin,
}

/// Користувач.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Унікальне імʼя користувача.
    pub username: String,
    /// Демонстраційний (НЕ криптографічний) хеш пароля — FNV-1a 64-bit.
    pub password_hash: u64,
    /// Роль користувача.
    pub role: Role,
}

/// Помилки авторизації.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthError {
    /// Користувача немає у сховищі.
    UnknownUser,
    /// Пароль не збігається.
    WrongPassword,
    /// Роль користувача недостатня для операції.
    Forbidden,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::UnknownUser => write!(f, "unknown user"),
            AuthError::WrongPassword => write!(f, "wrong password"),
            AuthError::Forbidden => write!(f, "insufficient privileges"),
        }
    }
}

impl std::error::Error for AuthError {}

/// Результат операцій авторизації.
pub type Result<T> = std::result::Result<T, AuthError>;

/// FNV-1a 64-bit хеш. Детермінований, демонстраційний, не для реальних паролів.
#[must_use]
pub fn hash_password(password: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    password.as_bytes().iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    })
}

/// Сховище користувачів (у памʼяті).
#[derive(Debug, Default, Clone)]
pub struct UserStore {
    users: Vec<User>,
}

impl UserStore {
    /// Створює порожнє сховище.
    #[must_use]
    pub fn new() -> Self {
        Self { users: Vec::new() }
    }

    /// Додає користувача. Повертає `&mut Self` для ланцюжкового стилю.
    pub fn add(&mut self, user: User) -> &mut Self {
        self.users.push(user);
        self
    }

    /// Зручний конструктор: хешує пароль і додає користувача.
    pub fn register(&mut self, username: &str, password: &str, role: Role) -> &mut Self {
        let user = User {
            username: username.to_string(),
            password_hash: hash_password(password),
            role,
        };
        self.add(user)
    }

    /// Пошук за імʼям.
    #[must_use]
    pub fn get(&self, username: &str) -> Option<&User> {
        self.users.iter().find(|user| user.username == username)
    }
}

/// Аутентифікація: перевіряє наявність користувача і пароль.
///
/// `UnknownUser`, якщо немає користувача; `WrongPassword`, якщо пароль не той.
pub fn authenticate<'a>(store: &'a UserStore, username: &str, password: &str) -> Result<&'a User> {
    let user = store.get(username).ok_or(AuthError::UnknownUser)?;
    if user.password_hash == hash_password(password) {
        Ok(user)
    } else {
        Err(AuthError::WrongPassword)
    }
}

/// Авторизація: роль користувача має бути >= потрібної (`user.role >= required`).
///
/// Інакше `Forbidden`.
pub fn authorize(user: &User, required: Role) -> Result<()> {
    if user.role >= required {
        Ok(())
    } else {
        Err(AuthError::Forbidden)
    }
}

/// Повний вхід: аутентифікація + авторизація.
pub fn login<'a>(
    store: &'a UserStore,
    username: &str,
    password: &str,
    required: Role,
) -> Result<&'a User> {
    let user = authenticate(store, username, password)?;
    authorize(user, required)?;
    Ok(user)
}

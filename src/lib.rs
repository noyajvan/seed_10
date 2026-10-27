//! `seed_10` — стартовий каркас проєкту.
//!
//! Тримаємо всю логіку в бібліотеці, а `main.rs` лишаємо тонким,
//! щоб код можна було тестувати на хості без бінарника.

use std::fmt;

/// Помилки рівня застосунку.
#[derive(Debug)]
pub enum Error {
    /// Некоректні аргументи командного рядка.
    Usage(String),
    /// Запит довідки (не помилка, окремий вихід).
    Help,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Usage(msg) => write!(f, "usage error: {msg}"),
            Error::Help => write!(f, "{}", usage()),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Розібрана конфігурація запуску.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Імʼя, яке буде привітане.
    pub name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "world".to_string(),
        }
    }
}

impl Config {
    /// Будує конфігурацію з аргументів без імені програми.
    pub fn from_args<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut args = args.into_iter().map(Into::into);
        let mut config = Config::default();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    return Err(Error::Help);
                }
                "--name" => {
                    let value = args
                        .next()
                        .ok_or_else(|| Error::Usage("--name потребує значення".to_string()))?;
                    config.name = value;
                }
                other if other.starts_with('-') => {
                    return Err(Error::Usage(format!("невідомий прапорець: {other}")));
                }
                positional => config.name = positional.to_string(),
            }
        }

        if config.name.trim().is_empty() {
            return Err(Error::Usage("імʼя не може бути порожнім".to_string()));
        }

        Ok(config)
    }
}

/// Текст довідки.
#[must_use]
pub fn usage() -> &'static str {
    "usage: seed_10 [NAME] [--name NAME]"
}

/// Основна логіка: повертає рядок привітання.
#[must_use]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name.trim())
}

/// Точка входу, придатна для тестів.
pub fn run<I, S>(args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let config = Config::from_args(args)?;
    Ok(greet(&config.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_world() {
        assert_eq!(run(Vec::<String>::new()).unwrap(), "Hello, world!");
    }

    #[test]
    fn takes_positional_name() {
        assert_eq!(run(["Ada"]).unwrap(), "Hello, Ada!");
    }

    #[test]
    fn takes_flag_name() {
        assert_eq!(run(["--name", "Grace"]).unwrap(), "Hello, Grace!");
    }

    #[test]
    fn rejects_unknown_flag() {
        assert!(run(["--nope"]).is_err());
    }

    #[test]
    fn rejects_empty_name() {
        assert!(run(["--name", "   "]).is_err());
    }

    #[test]
    fn help_returns_help_variant() {
        assert!(matches!(run(["--help"]), Err(Error::Help)));
        assert!(matches!(run(["-h"]), Err(Error::Help)));
    }

    #[test]
    fn greet_trims_whitespace() {
        assert_eq!(greet("  Ada  "), "Hello, Ada!");
    }
}

pub mod auth;

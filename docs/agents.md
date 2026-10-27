# Історія роботи (хто що робив)

Мультиагентна сесія створення модуля авторизації `seed_10::auth`.
Оркестратор — lead на `deepseek-v4-pro`; воркери — `deepseek-v4-flash`.

## Агенти та внесок

| Агент | Роль | Модель | Що зробив | Файли | Коміт |
|-------|------|--------|-----------|-------|-------|
| `fox` (координатор) | lead / архітектор | `deepseek-v4-pro` | контракт API, розподіл задач, верифікація | `docs/auth-spec.md` | — |
| `hamster` | coder | `deepseek-v4-flash` | логіка RBAC-модуля + підключення | `src/auth.rs`, `src/lib.rs` | `49b6afb` |
| `giraffe` | tester | `deepseek-v4-flash` | 30 unit-тестів проти публічного API | `tests/auth.rs` | `82a497c` |

## Деталі за агентом

### `fox` — координатор (lead)
- Написав єдиний контракт `docs/auth-spec.md` (Role, User, AuthError, FNV-1a,
  UserStore, authenticate/authorize/login, вимоги якості), щоб coder і tester
  працювали паралельно без конфліктів і вгадування сигнатур.
- Розподілив задачі через `swarm` (паралельний spawn обох воркерів).
- Адаптував запит: проєкт Rust, тому замість `auth.py`/`test_auth.py` —
  `src/auth.rs`/`tests/auth.rs`.
- Верифікував результат: `cargo test` (41 passed), `cargo clippy --all-targets`
  без попереджень, незалежно перерахував FNV-1a вектори.

### `hamster` — coder
- Реалізував `src/auth.rs` точно за контрактом: `Role` (Guest<User<Admin),
  `User`, `AuthError` (+ Display/std::error::Error), `hash_password` (FNV-1a 64),
  `UserStore::new/add/register/get`, `authenticate`, `authorize`
  (`user.role >= required`), `login` (authenticate → authorize).
- Додав `pub mod auth;` у `src/lib.rs`.
- Без залежностей, без `unsafe`, без `unwrap`/`panic!`, документація на всіх
  публічних елементах.
- Перевірив: `cargo build` і `cargo clippy --all-targets` — чисто.

### `giraffe` — tester
- Написав `tests/auth.rs` (30 тестів) проти публічного API:
  `hash_password` (детермінованість, канонічні FNV-1a вектори), порядок ролей,
  `UserStore` (new/add/register/get), `authenticate` (Ok/UnknownUser/WrongPassword/
  регістр імені), `authorize` (повна матриця 3×3), `login` (повний шлях +
  порядок «пароль перед роллю»), `AuthError` Display/Error/Copy.
- Лише публічний API, без внутрішніх деталей.
- Перевірив: `cargo test --test auth` — 30 passed.

## Хронологія

1. Спека (`docs/auth-spec.md`) — координатор.
2. Паралельно: `hamster` пише логіку, `giraffe` пише тести.
3. Верифікація: `fox` запускає повний `cargo test` + `clippy`, комітить тести
   та спека.

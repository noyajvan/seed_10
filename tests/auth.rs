//! Інтеграційні unit-тести модуля `seed_10::auth`.
//!
//! Тестують лише публічний контракт із `docs/auth-spec.md`.

use seed_10::auth::{AuthError, Role, User, UserStore, authenticate, authorize, hash_password, login};

/// Сховище з одним користувачем `alice` / `s3cret` / `User`.
fn store_with_alice() -> UserStore {
    let mut store = UserStore::new();
    store.register("alice", "s3cret", Role::User);
    store
}

/// Сховище з трьома ролями для перевірки границь authorize/login.
fn store_with_all_roles() -> UserStore {
    let mut store = UserStore::new();
    store
        .register("guest", "pw", Role::Guest)
        .register("user", "pw", Role::User)
        .register("admin", "pw", Role::Admin);
    store
}

// --- hash_password -------------------------------------------------------

#[test]
fn hash_password_is_deterministic() {
    assert_eq!(
        hash_password("correct horse battery staple"),
        hash_password("correct horse battery staple"),
        "одинаковий пароль мусить давати одинаковий хеш"
    );
}

#[test]
fn hash_password_differs_for_different_inputs() {
    let inputs = ["", "a", "b", "password", "Password", "passwor", "passwords"];
    for (i, left) in inputs.iter().enumerate() {
        for right in &inputs[i + 1..] {
            assert_ne!(
                hash_password(left),
                hash_password(right),
                "різні входи {left:?} і {right:?} не повинні давати однаковий хеш"
            );
        }
    }
}

#[test]
fn hash_password_matches_fnv1a_64_vectors() {
    // Стандартні контрольні значення FNV-1a 64-bit, на які посилається спека.
    assert_eq!(
        hash_password(""),
        0xcbf2_9ce4_8422_2325,
        "порожній рядок мусить дорівнювати offset basis FNV-1a 64"
    );
    assert_eq!(
        hash_password("a"),
        0xaf63_dc4c_8601_ec8c,
        "\"a\" мусить дорівнювати канонічному вектору FNV-1a 64"
    );
}

#[test]
fn hash_password_is_stable_across_calls_and_inputs() {
    let first = hash_password("hunter2");
    let second = hash_password("hunter2");
    assert_eq!(first, second);
    assert_ne!(first, hash_password("hunter3"));
}

// --- Role ----------------------------------------------------------------

#[test]
fn role_ordering_is_guest_user_admin() {
    assert!(Role::Guest < Role::User, "Guest нижче за User");
    assert!(Role::User < Role::Admin, "User нижче за Admin");
    assert!(Role::Guest < Role::Admin, "Guest нижче за Admin");
}

// --- UserStore::new / add / register / get -------------------------------

#[test]
fn new_store_is_empty() {
    let store = UserStore::new();
    assert!(
        store.get("alice").is_none(),
        "нове сховище не повинно містити користувачів"
    );
}

#[test]
fn default_store_is_empty() {
    let store = UserStore::default();
    assert!(
        store.get("anyone").is_none(),
        "сховище за замовчуванням має бути порожнім"
    );
}

#[test]
fn add_stores_user_verbatim_and_supports_chaining() {
    let mut store = UserStore::new();
    let user = User {
        username: "bob".to_string(),
        password_hash: hash_password("pw"),
        role: Role::Admin,
    };

    let returned = store.add(user.clone());
    assert!(
        returned.get("bob").is_some(),
        "add має повертати &mut Self для ланцюжка"
    );

    let stored = store.get("bob").expect("bob має бути у сховищі");
    assert_eq!(stored, &user, "add має зберегти користувача без змін");
    assert_eq!(stored.username, "bob");
    assert_eq!(stored.password_hash, hash_password("pw"));
    assert_eq!(stored.role, Role::Admin);
}

#[test]
fn add_chain_adds_all_users() {
    let mut store = UserStore::new();
    store
        .add(User {
            username: "u1".to_string(),
            password_hash: 1,
            role: Role::Guest,
        })
        .add(User {
            username: "u2".to_string(),
            password_hash: 2,
            role: Role::User,
        });

    assert!(store.get("u1").is_some(), "перший користувач має бути доданий");
    assert!(store.get("u2").is_some(), "другий користувач має бути доданий");
}

#[test]
fn register_hashes_password_and_sets_role() {
    let mut store = UserStore::new();
    store.register("carol", "top-secret", Role::Admin);

    let user = store.get("carol").expect("carol має бути у сховищі");
    assert_eq!(user.username, "carol");
    assert_eq!(
        user.password_hash,
        hash_password("top-secret"),
        "register мусить хешувати пароль через hash_password"
    );
    assert_ne!(
        user.password_hash,
        hash_password("wrong"),
        "хеш має відповідати саме переданому паролю"
    );
    assert_eq!(user.role, Role::Admin);
}

#[test]
fn register_supports_chaining() {
    let mut store = UserStore::new();
    store
        .register("a", "pw", Role::Guest)
        .register("b", "pw", Role::User)
        .register("c", "pw", Role::Admin);

    assert_eq!(store.get("a").map(|u| u.role), Some(Role::Guest));
    assert_eq!(store.get("b").map(|u| u.role), Some(Role::User));
    assert_eq!(store.get("c").map(|u| u.role), Some(Role::Admin));
}

#[test]
fn get_returns_none_for_missing_user() {
    let store = store_with_alice();
    assert!(
        store.get("nobody").is_none(),
        "get для невідомого імені має повертати None"
    );
}

#[test]
fn get_returns_stored_user() {
    let store = store_with_alice();
    let user = store.get("alice").expect("alice має бути у сховищі");
    assert_eq!(user.username, "alice");
    assert_eq!(user.role, Role::User);
    assert_eq!(user.password_hash, hash_password("s3cret"));
}

// --- authenticate --------------------------------------------------------

#[test]
fn authenticate_ok_for_valid_credentials() {
    let store = store_with_alice();
    let user = authenticate(&store, "alice", "s3cret")
        .expect("правильні облікові дані мають проходити");
    assert_eq!(user.username, "alice");
    assert_eq!(user.role, Role::User);
}

#[test]
fn authenticate_unknown_user() {
    let store = store_with_alice();
    assert_eq!(
        authenticate(&store, "mallory", "s3cret"),
        Err(AuthError::UnknownUser),
        "невідоме імʼя мусить давати UnknownUser"
    );
}

#[test]
fn authenticate_wrong_password() {
    let store = store_with_alice();
    assert_eq!(
        authenticate(&store, "alice", "wrong"),
        Err(AuthError::WrongPassword),
        "невірний пароль мусить давати WrongPassword"
    );
    assert_eq!(
        authenticate(&store, "alice", ""),
        Err(AuthError::WrongPassword),
        "порожній пароль не має збігатися"
    );
}

#[test]
fn authenticate_is_case_sensitive_on_username() {
    let store = store_with_alice();
    assert_eq!(
        authenticate(&store, "Alice", "s3cret"),
        Err(AuthError::UnknownUser),
        "імена користувачів мають бути чутливими до регістру"
    );
}

// --- authorize -----------------------------------------------------------

#[test]
fn authorize_allows_equal_role() {
    let store = store_with_all_roles();

    let guest = store.get("guest").expect("guest має бути у сховищі");
    assert_eq!(authorize(guest, Role::Guest), Ok(()));

    let user = store.get("user").expect("user має бути у сховищі");
    assert_eq!(authorize(user, Role::User), Ok(()));

    let admin = store.get("admin").expect("admin має бути у сховищі");
    assert_eq!(authorize(admin, Role::Admin), Ok(()));
}

#[test]
fn authorize_allows_higher_role() {
    let user = User {
        username: "admin".to_string(),
        password_hash: 0,
        role: Role::Admin,
    };
    assert_eq!(authorize(&user, Role::Guest), Ok(()));
    assert_eq!(authorize(&user, Role::User), Ok(()));
    assert_eq!(authorize(&user, Role::Admin), Ok(()));

    let regular = User {
        username: "user".to_string(),
        password_hash: 0,
        role: Role::User,
    };
    assert_eq!(authorize(&regular, Role::Guest), Ok(()));
    assert_eq!(authorize(&regular, Role::User), Ok(()));
}

#[test]
fn authorize_forbidden_for_insufficient_role() {
    let guest = User {
        username: "guest".to_string(),
        password_hash: 0,
        role: Role::Guest,
    };
    assert_eq!(
        authorize(&guest, Role::User),
        Err(AuthError::Forbidden),
        "Guest не має доступу до User"
    );
    assert_eq!(
        authorize(&guest, Role::Admin),
        Err(AuthError::Forbidden),
        "Guest не має доступу до Admin"
    );

    let user = User {
        username: "user".to_string(),
        password_hash: 0,
        role: Role::User,
    };
    assert_eq!(
        authorize(&user, Role::Admin),
        Err(AuthError::Forbidden),
        "User не має доступу до Admin"
    );
}

#[test]
fn authorize_respects_role_ordering_matrix() {
    let cases = [
        (Role::Guest, Role::Guest, true),
        (Role::Guest, Role::User, false),
        (Role::Guest, Role::Admin, false),
        (Role::User, Role::Guest, true),
        (Role::User, Role::User, true),
        (Role::User, Role::Admin, false),
        (Role::Admin, Role::Guest, true),
        (Role::Admin, Role::User, true),
        (Role::Admin, Role::Admin, true),
    ];

    for (actual, required, allowed) in cases {
        let user = User {
            username: "matrix".to_string(),
            password_hash: 0,
            role: actual,
        };
        let result = authorize(&user, required);
        if allowed {
            assert_eq!(
                result,
                Ok(()),
                "{actual:?} має мати доступ до {required:?}"
            );
        } else {
            assert_eq!(
                result,
                Err(AuthError::Forbidden),
                "{actual:?} не має мати доступ до {required:?}"
            );
        }
    }
}

// --- login (повний шлях) -------------------------------------------------

#[test]
fn login_ok_with_correct_password_and_sufficient_role() {
    let store = store_with_all_roles();

    let guest = login(&store, "guest", "pw", Role::Guest).expect("guest входить як Guest");
    assert_eq!(guest.username, "guest");
    assert_eq!(guest.role, Role::Guest);

    let user = login(&store, "user", "pw", Role::User).expect("user входить як User");
    assert_eq!(user.username, "user");
    assert_eq!(user.role, Role::User);

    let admin = login(&store, "admin", "pw", Role::Admin).expect("admin входить як Admin");
    assert_eq!(admin.username, "admin");
    assert_eq!(admin.role, Role::Admin);
}

#[test]
fn login_ok_when_role_is_higher_than_required() {
    let store = store_with_all_roles();
    let admin = login(&store, "admin", "pw", Role::Guest)
        .expect("Admin має проходити з нижчою вимогою");
    assert_eq!(admin.role, Role::Admin);
}

#[test]
fn login_unknown_user() {
    let store = store_with_alice();
    assert_eq!(
        login(&store, "ghost", "s3cret", Role::Guest),
        Err(AuthError::UnknownUser),
        "невідомий користувач не може увійти навіть як Guest"
    );
}

#[test]
fn login_wrong_password() {
    let store = store_with_alice();
    assert_eq!(
        login(&store, "alice", "nope", Role::User),
        Err(AuthError::WrongPassword),
        "невірний пароль мусить давати WrongPassword"
    );
    assert_eq!(
        login(&store, "alice", "nope", Role::Guest),
        Err(AuthError::WrongPassword),
        "перевірка пароля йде до перевірки ролі"
    );
}

#[test]
fn login_forbidden_for_insufficient_role() {
    let store = store_with_all_roles();
    assert_eq!(
        login(&store, "guest", "pw", Role::User),
        Err(AuthError::Forbidden),
        "Guest не може увійти як User"
    );
    assert_eq!(
        login(&store, "guest", "pw", Role::Admin),
        Err(AuthError::Forbidden),
        "Guest не може увійти як Admin"
    );
    assert_eq!(
        login(&store, "user", "pw", Role::Admin),
        Err(AuthError::Forbidden),
        "User не може увійти як Admin"
    );
}

#[test]
fn login_checks_password_before_role() {
    let store = store_with_all_roles();
    // Недостатня роль + невірний пароль -> саме WrongPassword, не Forbidden.
    assert_eq!(
        login(&store, "guest", "wrong", Role::Admin),
        Err(AuthError::WrongPassword),
        "спершу аутентифікація, потім авторизація"
    );
}

// --- AuthError: Display / Error -----------------------------------------

#[test]
fn auth_error_display_is_human_readable_and_distinct() {
    let unknown = AuthError::UnknownUser.to_string();
    let wrong = AuthError::WrongPassword.to_string();
    let forbidden = AuthError::Forbidden.to_string();

    for (name, message) in [
        ("UnknownUser", &unknown),
        ("WrongPassword", &wrong),
        ("Forbidden", &forbidden),
    ] {
        assert!(
            !message.trim().is_empty(),
            "Display для {name} не має бути порожнім"
        );
    }

    assert_ne!(unknown, wrong, "повідомлення для різних помилок мають різнитися");
    assert_ne!(unknown, forbidden);
    assert_ne!(wrong, forbidden);
}

#[test]
fn auth_error_implements_std_error() {
    let error: &dyn std::error::Error = &AuthError::Forbidden;
    assert!(
        !error.to_string().trim().is_empty(),
        "AuthError має поводитися як std::error::Error"
    );
}

#[test]
fn auth_error_is_copy_and_comparable() {
    let error = AuthError::UnknownUser;
    let copy = error;
    assert_eq!(error, copy, "AuthError має бути Copy + PartialEq");
    assert_ne!(error, AuthError::Forbidden);
}

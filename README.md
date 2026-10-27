# seed_10

Стартовий каркас Rust-проєкту: тонкий `main.rs` + уся логіка в бібліотеці,
щоб код тестувався на хості.

## Швидкий старт

```bash
cargo run                 # Hello, world!
cargo run -- Ada          # Hello, Ada!
cargo run -- --name Ada   # Hello, Ada!
cargo run -- --help       # довідка, код виходу 0
cargo test                # модульні + інтеграційні тести
cargo clippy --all-targets
cargo build --release
```

## Структура

```
src/lib.rs      логіка: Config, greet, run, Error
src/main.rs     розбір exit-code, вивід
tests/cli.rs    чорноскринькові тести бінарника
```

## Коди виходу

| Код | Значення |
|-----|----------|
| 0   | успіх (у т.ч. `--help`) |
| 2   | помилка використання |

## Ліцензія

MIT

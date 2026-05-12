# terminal-demos

Маленькие прикольные Rust-скетчи в терминале, без внешних зависимостей.

## Запуск

Пончик:

```powershell
cargo run --release
```

Огонь:

```powershell
cargo run --release --bin fire
```

Если хочешь собрать одним `rustc`, тоже можно:

```powershell
rustc .\src\main.rs -O -o terminal-donut.exe
.\terminal-donut.exe
```

```powershell
rustc .\src\bin\fire.rs -O -o fire.exe
.\fire.exe
```

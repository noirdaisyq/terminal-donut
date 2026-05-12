# terminal-donut

Маленький прикольный Rust-скетч: цветной вращающийся 3D donut в терминале, без внешних зависимостей.

## Запуск

```powershell
cargo run --release
```

Если хочешь собрать одним `rustc`, тоже можно:

```powershell
rustc .\src\main.rs -O -o terminal-donut.exe
.\terminal-donut.exe
```

В этой среде Rust пока не установлен, поэтому я не смог прогнать сборку локально.

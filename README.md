# Notarium

Editor de partituras em Rust (GPL-3.0-or-later). Esta é a etapa 0: janela com menu inicial renderizado em **DirectX 11**.

## Gerar o .exe (Windows)
1. Instale o Rust com toolchain MSVC: https://rustup.rs
2. Execute `build.bat`  ->  `dist\Notarium.exe`

Sem instalar nada: suba esta pasta para um repositório no GitHub, abra *Actions → Build Notarium (Windows) → Run workflow* e baixe o artefato `Notarium-windows-x64`.

## Observação
O código ainda não foi compilado. Se o `cargo build` reclamar de versões (`egui`, `egui-directx11`, `windows`), as correções ficam em `Cargo.toml` e `src/gfx.rs`. Cole o erro aqui e eu ajusto.

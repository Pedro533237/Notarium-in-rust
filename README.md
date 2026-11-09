# Notarium

Editor de partituras em Rust (GPL-3.0-or-later). Esta é a etapa 0: janela com menu inicial renderizado em **DirectX 11**.

## Gerar o .exe (Windows)
1. Instale o Rust com toolchain MSVC: https://rustup.rs
2. Execute `build.bat`  ->  `dist\Notarium.exe`

Sem instalar nada: suba esta pasta para um repositório no GitHub, abra *Actions → Build Notarium (Windows) → Run workflow* e baixe o artefato `Notarium-windows-x64`.

## Observação
O código ainda não foi compilado. Usa egui 0.28 + winit 0.29 porque o `egui-directx11 0.5` depende do egui 0.28.

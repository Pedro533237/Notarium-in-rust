# Notarium

Editor de partituras em Rust (GPL-3.0-or-later). Esta é a etapa 0: janela com menu inicial renderizado em **DirectX 11**.

## Gerar o .exe (Windows)
1. Instale o Rust com toolchain MSVC: https://rustup.rs
2. Execute `build.bat`  ->  `dist\Notarium.exe`

Sem instalar nada: suba esta pasta para um repositório no GitHub, abra *Actions → Build Notarium (Windows) → Run workflow* e baixe o artefato `Notarium-windows-x64`.

## Observação
O código ainda não foi compilado. Usa egui 0.28 + winit 0.29 porque o `egui-directx11 0.5` depende do egui 0.28.


## Compatibilidade gráfica

O dispositivo Direct3D 11 é criado tentando os níveis de recurso 11_0, 10_1 e 10_0, nessa ordem. Isso permite testar GPUs mais antigas que expõem a API D3D11 com nível de recurso 10.x. Se ocorrer um panic na inicialização, o aplicativo mostra uma caixa de diálogo com a mensagem em vez de fechar silenciosamente.

Para compilar e executar a partir do terminal no Windows:

```powershell
cargo run
```

Para gerar a versão otimizada:

```powershell
cargo build --release
```

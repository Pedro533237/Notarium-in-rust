@echo off
REM Requer Rust (https://rustup.rs) com o toolchain MSVC instalado.
cargo build --release
if errorlevel 1 exit /b 1
if not exist dist mkdir dist
copy /Y target\release\notarium.exe dist\Notarium.exe
echo.
echo Pronto: dist\Notarium.exe

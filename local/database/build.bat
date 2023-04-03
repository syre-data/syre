@echo off
setlocal enabledelayedexpansion

set dir=../../target/release

:: get target
for /f "tokens=* usebackq" %%o in (`rustc -Vv`) do (
  set v=%%o 
  if /i "!v:~0,5!"=="host:" set host=%%o
)

for /f "tokens=1,2 delims= " %%a in ("%host%") do (
  set target=%%b
)

:: build
if not exist "%dir%" mkdir "%dir%"
cargo build --release -F server
move %dir%/thot-local-database.exe %dir%/thot-local-database-%target%.exe
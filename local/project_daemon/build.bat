@echo off
setlocal enabledelayedexpansion
set PROGRAM_BASENAME="syre-project-daemon-server"
set BIN_NAME=%PROGRAM_BASENAME%-server
set ROOT=..\..
set release_dir=%ROOT%\target\release
set crate_release_dir=%ROOT%\target\release
set bin=%ROOT%\desktop\src-tauri\bin
set lang=%ROOT%\lang

:: get target
:: TODO: Use `rustc --print target-tuple`
for /f "tokens=* usebackq" %%o in (`rustc -Vv`) do (
  set v=%%o 
  if /i "!v:~0,5!"=="host:" set host=%%o
)

for /f "tokens=1,2 delims= " %%a in ("%host%") do (
  set target=%%b
)
set target_out=%release_dir%\%PROGRAM_BASENAME%-%target%.exe
set crate_target_out=%bin%\%BIN_NAME%-%target%.exe

:: build
if not exist "%bin%" md "%bin%"
cargo build --release -F server
copy %crate_release_dir%\%PROGRAM_BASENAME%.exe %crate_target_out%

:: copy to other directories
set python_path=%lang%\python\src\syre\bin\
set r_path=%lang%\r\inst\
if not exist "%python_path%" md "%python_path%"
if not exist "%r_path%" md "%r_path%"

copy "%target_out%" "%python_path%"
copy "%target_out%" "%r_path%"

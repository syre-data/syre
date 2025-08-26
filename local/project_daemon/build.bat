@echo off
setlocal enabledelayedexpansion
set PROGRAM_BASENAME="syre-project-watcher"
set ROOT=..\..
set release_dir=%ROOT%\target\release
set crate_release_dir=target\release
set lang=%ROOT%\lang

:: get target
for /f "tokens=* usebackq" %%o in (`rustc -Vv`) do (
  set v=%%o 
  if /i "!v:~0,5!"=="host:" set host=%%o
)

for /f "tokens=1,2 delims= " %%a in ("%host%") do (
  set target=%%b
)

set target_out=%release_dir%\%PROGRAM_BASENAME%-%target%.exe
set crate_target_out=%crate_release_dir%\%PROGRAM_BASENAME%-%target%.exe

:: build
if not exist "%release_dir%" md "%release_dir%"
cargo build --release -F server
move %crate_release_dir%\%PROGRAM_BASENAME%.exe %crate_target_out%

:: copy to other directories
set python_path=%lang%\python\src\syre\bin\
set r_path=%lang%\r\inst\
if not exist "%python_path%" md "%python_path%"
if not exist "%r_path%" md "%r_path%"

copy %crate_target_out% %target_out%
copy "%target_out%" "%python_path%"
copy "%target_out%" "%r_path%"
@echo off
setlocal enabledelayedexpansion
@REM set root=..\..
set root=.
set dir=%root%\target\release

:: get target
for /f "tokens=* usebackq" %%o in (`rustc -Vv`) do (
  set v=%%o 
  if /i "!v:~0,5!"=="host:" set host=%%o
)

for /f "tokens=1,2 delims= " %%a in ("%host%") do (
  set target=%%b
)

set target_out=%dir%\syre-local-database-%target%.exe

:: build
if not exist "%dir%" mkdir "%dir%"
cargo build --release -F server
move %dir%\syre-local-database.exe %target_out%

:: copy to other directories
set lang=%root%\lang
copy "%target_out%" "%lang%\python\src\syre\bin\"
copy "%target_out%" "%lang%\r\inst\"
@echo off
:start
echo Running Rust application...
cargo run
if errorlevel 1 goto error

echo Application exited normally. Waiting to restart...
timeout /t 5 >nul
goto start

:error
echo Application exited with an error. Restarting...
timeout /t 5 >nul
goto start

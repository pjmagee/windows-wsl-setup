@echo off
rem Launch the default Ubuntu 26.04 session in the Linux home.
rem Installed onto the user PATH by windows/bootstrap.ps1.
wsl.exe -d Ubuntu-26.04 --cd ~ %*

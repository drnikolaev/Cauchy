@echo off
setlocal
rem Initialize Intel Fortran, oneMKL, and the x64 MSVC tools for this command.
rem ONEAPI_ROOT can select an installation outside the default directory.
if "%SETVARS_COMPLETED%"=="1" goto check_tools
if not defined ONEAPI_ROOT set "ONEAPI_ROOT=%ProgramFiles(x86)%\Intel\oneAPI"
if not exist "%ONEAPI_ROOT%\setvars.bat" (
    echo Intel oneAPI setvars.bat was not found under "%ONEAPI_ROOT%". 1>&2
    echo Install Intel Fortran and oneMKL, or set ONEAPI_ROOT. 1>&2
    exit /b 1
)
call "%ONEAPI_ROOT%\setvars.bat" intel64
if errorlevel 1 exit /b %errorlevel%
:check_tools
if not defined FC set "FC=ifx"
if exist "%FC%" goto check_archiver
where "%FC%" >nul 2>&1
if errorlevel 1 (
    echo Intel Fortran "%FC%" was not found. Install the compiler or set FC. 1>&2
    exit /b 1
)
:check_archiver
where lib.exe >nul 2>&1
if errorlevel 1 (
    echo MSVC lib.exe was not found. Install Visual Studio C++ build tools. 1>&2
    exit /b 1
)
if not defined MKLROOT (
    echo oneMKL was not found. Install Intel oneMKL. 1>&2
    exit /b 1
)
if "%~1"=="" (
    cargo build
) else (
    cargo %*
)
exit /b %errorlevel%

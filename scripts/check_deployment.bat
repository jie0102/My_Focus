@echo off
echo ==============================================
echo     My Focus - Tesseract 便携式部署检查
echo     (命令行OCR版本 - 无需DLL依赖)
echo ==============================================
echo.

set "PROJECT_ROOT=%~dp0"
set "TESSERACT_DIR=%PROJECT_ROOT%resources\tesseract"
set "TESSDATA_DIR=%TESSERACT_DIR%\tessdata"

echo 📁 检查目录结构...
if not exist "%TESSERACT_DIR%" (
    echo 创建 resources\tesseract 目录
    mkdir "%TESSERACT_DIR%"
)

if not exist "%TESSDATA_DIR%" (
    echo 创建 tessdata 目录
    mkdir "%TESSDATA_DIR%"
)

echo.
echo 📥 检查语言包...
if exist "%TESSDATA_DIR%\eng.traineddata" (
    echo ✅ 英文语言包已存在
    for %%F in ("%TESSDATA_DIR%\eng.traineddata") do echo    大小: %%~zF 字节
) else (
    echo ❌ 英文语言包缺失
)

if exist "%TESSDATA_DIR%\chi_sim.traineddata" (
    echo ✅ 中文语言包已存在
    for %%F in ("%TESSDATA_DIR%\chi_sim.traineddata") do echo    大小: %%~zF 字节
) else (
    echo ❌ 中文语言包缺失
)

echo.
echo 🔧 检查Tesseract程序文件...
if exist "%TESSERACT_DIR%\tesseract.exe" (
    echo ✅ tesseract.exe 已存在
    for %%F in ("%TESSERACT_DIR%\tesseract.exe") do echo    大小: %%~zF 字节
    
    echo 📝 测试Tesseract版本...
    "%TESSERACT_DIR%\tesseract.exe" --version 2>nul
    if errorlevel 1 (
        echo ⚠️ Tesseract可执行文件可能损坏或缺少依赖
    ) else (
        echo ✅ Tesseract可执行文件正常
    )
) else (
    echo ❌ tesseract.exe 缺失
    echo.
    echo 请按以下步骤完成设置:
    echo 1. 下载 Tesseract: https://github.com/UB-Mannheim/tesseract/wiki
    echo 2. 安装到临时目录 (如: C:\temp\tesseract)
    echo 3. 复制 tesseract.exe 到 %TESSERACT_DIR%
    echo 4. 复制 tessdata 目录内容到 %TESSDATA_DIR%
    echo.
    echo 或者运行: copy_tesseract_files.bat
)

echo.
echo 📊 目录大小统计:
if exist "%TESSERACT_DIR%" (
    set "TOTAL_SIZE=0"
    for /r "%TESSERACT_DIR%" %%F in (*.*) do (
        set /a TOTAL_SIZE+=%%~zF
    )
    set /a TOTAL_MB=TOTAL_SIZE/1048576
    echo    总大小: !TOTAL_MB! MB
)

echo.
echo ==============================================
echo                 部署状态检查
echo ==============================================
set "READY=1"

if not exist "%TESSERACT_DIR%\tesseract.exe" set "READY=0"
if not exist "%TESSDATA_DIR%\eng.traineddata" set "READY=0"

if %READY% EQU 1 (
    echo ✅ 便携式部署准备完成!
    echo.
    echo 🚀 可以运行以下命令进行测试:
    echo    npm run tauri:dev
    echo    npm run tauri:build
    echo.
    echo 📋 部署优势:
    echo    - ✅ 无需复杂的DLL依赖
    echo    - ✅ 使用命令行OCR调用
    echo    - ✅ 支持便携式部署
    echo    - ✅ 中英文OCR支持
) else (
    echo ⚠️ 便携式部署未完成，请按上述提示完成配置
)

echo.
pause
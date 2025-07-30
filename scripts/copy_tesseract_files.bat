@echo off
echo ==============================================
echo     复制Tesseract便携式文件
echo     (命令行OCR版本 - 简化部署)
echo ==============================================
echo.

set "PROJECT_ROOT=%~dp0.."
set "TESSERACT_DIR=%PROJECT_ROOT%\resources\tesseract"
set "SYSTEM_TESSERACT=D:\Tesseract-OCR"

echo 🔍 检查系统Tesseract安装...
if not exist "%SYSTEM_TESSERACT%" (
    echo ❌ 系统未找到Tesseract安装路径: %SYSTEM_TESSERACT%
    echo.
    echo 请先安装Tesseract:
    echo 1. 访问: https://github.com/UB-Mannheim/tesseract/wiki
    echo 2. 下载并安装 tesseract-ocr-w64-setup-v5.3.3.20231005.exe
    echo 3. 确保安装到默认路径: C:\Program Files\Tesseract-OCR
    pause
    exit /b 1
)

echo ✅ 找到系统Tesseract: %SYSTEM_TESSERACT%

echo.
echo 📁 创建目标目录...
if not exist "%TESSERACT_DIR%" mkdir "%TESSERACT_DIR%"

echo.
echo 📋 复制Tesseract主程序...
copy "%SYSTEM_TESSERACT%\tesseract.exe" "%TESSERACT_DIR%\" /Y
if errorlevel 1 (
    echo ❌ 复制tesseract.exe失败
    pause
    exit /b 1
)
echo ✅ tesseract.exe 复制完成

echo.
echo 📋 复制tessdata目录...
xcopy "%SYSTEM_TESSERACT%\tessdata" "%TESSERACT_DIR%\tessdata\" /E /I /Y
if errorlevel 1 (
    echo ❌ 复制tessdata失败
    pause
    exit /b 1
)
echo ✅ tessdata 目录复制完成

echo.
echo 🔍 验证复制结果...
if exist "%TESSERACT_DIR%\tesseract.exe" (
    echo ✅ tesseract.exe: 已复制
    for %%F in ("%TESSERACT_DIR%\tesseract.exe") do echo    大小: %%~zF 字节
) else (
    echo ❌ tesseract.exe: 复制失败
)

if exist "%TESSERACT_DIR%\tessdata\eng.traineddata" (
    echo ✅ 英文语言包: 已复制
    for %%F in ("%TESSERACT_DIR%\tessdata\eng.traineddata") do echo    大小: %%~zF 字节
) else (
    echo ❌ 英文语言包: 复制失败
)

if exist "%TESSERACT_DIR%\tessdata\chi_sim.traineddata" (
    echo ✅ 中文语言包: 已复制
    for %%F in ("%TESSERACT_DIR%\tessdata\chi_sim.traineddata") do echo    大小: %%~zF 字节
) else (
    echo ⚠️ 中文语言包: 未找到 (可能需要单独下载)
)

echo.
echo 📊 统计复制的文件...
set "FILE_COUNT=0"
for %%f in ("%TESSERACT_DIR%\*.*") do set /a FILE_COUNT+=1

set "TESSDATA_COUNT=0"
for %%f in ("%TESSERACT_DIR%\tessdata\*.traineddata") do set /a TESSDATA_COUNT+=1

echo    程序文件: %FILE_COUNT% 个
echo    语言包: %TESSDATA_COUNT% 个

echo.
echo 📏 计算总大小...
set "TOTAL_SIZE=0"
for /r "%TESSERACT_DIR%" %%F in (*.*) do (
    set /a TOTAL_SIZE+=%%~zF
)
set /a TOTAL_MB=TOTAL_SIZE/1048576
echo    总大小: %TOTAL_MB% MB

echo.
echo ==============================================
echo              复制操作完成
echo ==============================================

if exist "%TESSERACT_DIR%\tesseract.exe" (
    echo ✅ Tesseract便携式部署准备完成!
    echo.
    echo 🎯 部署优势:
    echo    - ✅ 无需复杂的DLL依赖管理
    echo    - ✅ 使用稳定的命令行调用
    echo    - ✅ 支持中英文OCR识别
    echo    - ✅ 便携式自包含部署
    echo.
    echo 🚀 下一步操作:
    echo 1. 运行 scripts\check_deployment.bat 验证部署
    echo 2. 运行 npm run tauri:dev 测试应用
    echo 3. 运行 npm run tauri:build 构建发布版本
) else (
    echo ❌ 部署未完成，请检查错误信息
)

echo.
pause
# Tesseract便携版下载脚本
# 用法: PowerShell -ExecutionPolicy Bypass .\scripts\download_tesseract.ps1

$ErrorActionPreference = "Stop"

# 项目根目录
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$TesseractDir = Join-Path $ProjectRoot "resources\tesseract"
$TessdataDir = Join-Path $TesseractDir "tessdata"

Write-Host "🚀 开始下载Tesseract便携版..." -ForegroundColor Green

# 创建目录
Write-Host "📁 创建目录结构..."
New-Item -ItemType Directory -Force -Path $TesseractDir | Out-Null
New-Item -ItemType Directory -Force -Path $TessdataDir | Out-Null

# 定义下载URLs
$Downloads = @{
    "tesseract.exe" = "https://github.com/UB-Mannheim/tesseract/releases/download/v5.3.3.20231005/tesseract-ocr-w64-setup-v5.3.3.20231005.exe"
    "eng.traineddata" = "https://github.com/tesseract-ocr/tessdata/raw/main/eng.traineddata"
    "chi_sim.traineddata" = "https://github.com/tesseract-ocr/tessdata/raw/main/chi_sim.traineddata"
}

# 下载语言包
Write-Host "📥 下载语言包..."
foreach ($file in @("eng.traineddata", "chi_sim.traineddata")) {
    $url = $Downloads[$file]
    $output = Join-Path $TessdataDir $file
    
    Write-Host "   下载 $file..." -ForegroundColor Yellow
    try {
        Invoke-WebRequest -Uri $url -OutFile $output -UseBasicParsing
        Write-Host "   ✅ $file 下载完成" -ForegroundColor Green
    } catch {
        Write-Host "   ❌ $file 下载失败: $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "⚠️  注意：Tesseract可执行文件需要手动安装后复制" -ForegroundColor Yellow
Write-Host "请执行以下步骤："
Write-Host "1. 访问: https://github.com/UB-Mannheim/tesseract/wiki"
Write-Host "2. 下载并安装 tesseract-ocr-w64-setup-v5.3.3.20231005.exe"
Write-Host "3. 从安装目录复制以下文件到 $TesseractDir :"
Write-Host "   - tesseract.exe"
Write-Host "   - leptonica-*.dll"
Write-Host "   - tesseract*.dll"
Write-Host ""
Write-Host "或者运行: .\scripts\copy_tesseract_files.ps1" -ForegroundColor Cyan

# 创建复制脚本
$CopyScript = @"
# 从系统安装复制Tesseract文件
`$TesseractSystem = "C:\Program Files\Tesseract-OCR"
`$TesseractTarget = "$TesseractDir"

if (Test-Path `$TesseractSystem) {
    Write-Host "📋 从系统安装复制Tesseract文件..."
    
    # 复制主要文件
    Copy-Item "`$TesseractSystem\tesseract.exe" `$TesseractTarget -Force
    Copy-Item "`$TesseractSystem\*.dll" `$TesseractTarget -Force
    
    Write-Host "✅ Tesseract文件复制完成！"
} else {
    Write-Host "❌ 系统未找到Tesseract安装，请先安装Tesseract"
}
"@

$CopyScript | Out-File -FilePath (Join-Path $PSScriptRoot "copy_tesseract_files.ps1") -Encoding UTF8

Write-Host "✨ 便携版准备脚本已生成！" -ForegroundColor Green
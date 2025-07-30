# Tesseract便携式部署配置

## 目录结构
```
resources/
└── tesseract/
    ├── tessdata/
    │   ├── eng.traineddata      ✅ 已下载
    │   └── chi_sim.traineddata  ✅ 已下载
    ├── tesseract.exe           ⚠️ 需要手动复制
    ├── leptonica-*.dll         ⚠️ 需要手动复制
    └── tesseract*.dll          ⚠️ 需要手动复制
```

## 获取可执行文件和DLL

### 方法1: 从系统安装复制 (推荐)
如果你已经安装了Tesseract：
```bash
# 查找系统安装路径
where tesseract

# 通常在：C:\Program Files\Tesseract-OCR\
# 复制以下文件到 resources\tesseract\：
# - tesseract.exe
# - leptonica-*.dll 
# - tesseract*.dll
```

### 方法2: 下载便携版
1. 下载：https://github.com/UB-Mannheim/tesseract/releases
2. 安装到临时目录
3. 复制文件到项目

## 部署优势
- ✅ 用户无需安装Tesseract
- ✅ 版本控制，避免兼容性问题
- ✅ 支持离线使用
- ✅ 安装包自包含

## 文件大小估算
- tesseract.exe: ~2MB
- DLL文件: ~10MB
- 英文语言包: ~22MB
- 中文语言包: ~42MB
- **总计: ~76MB**
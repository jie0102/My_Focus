<p align="center">
  <a href="https://github.com/jie0102/My_Focus/releases">
    <img src="/assets/icon.png" alt="Product Logo" width="200">
  </a>
</p>

<p align="center"><b>My Focus - 专注度监控应用</b></p>
<p align="center">
  <a href="https://github.com/jie0102/My_Focus">English</a> | 中文 | <a href="https://github.com/jie0102/My_Focus/issues">反馈</a><br>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-developing-yellow.svg">
  <img src="https://img.shields.io/badge/Tauri-1.5-blue.svg">
  <img src="https://img.shields.io/badge/license-AGPLv3-green.svg">
</p>

---

## 🧐 项目介绍

**My Focus** 是一款基于 Tauri 框架开发的桌面专注度监控应用，旨在帮助用户提升专注力与工作效率。
应用通过智能监控系统与 AI 分析，实时评估用户专注状态，并提供个性化的提升建议。

<p align="center">
  <img src="/assets/screenshot1.png" width="45%">
  <img src="/assets/screenshot2.png" width="45%">
  <img src="/assets/screenshot3.png" width="45%">
  <img src="/assets/screenshot4.png" width="45%">
</p>

---

## ✨ 功能特色

### 已完成核心功能

- **智能监控系统**
  - 实时屏幕监控：定期截取屏幕快照，进行 AI 分析
  - 应用程序跟踪：监控活跃应用并分类
  - 专注状态评估：通过 AI 判断用户专注程度（专注/分心/严重分心）
  - 智能干预提醒：分心时自动提醒

- **灵活配置**
  - 多 AI 平台支持：OpenAI API、Ollama、Claude 等
  - 个性化设置：自定义监控间隔、干预方案、应用白名单/黑名单
  - 数据完全本地化，隐私有保障



### 开发中及未来规划
- **数据分析与报告**
  - 专注度统计与趋势分析
  - 历史数据可视化
  - 集成任务追踪
- AI 个性化专注建议
- 更多数据可视化
- 个性化专注学习算法
- 团队协作功能
- 多模态：如人脸表情识别等
- 移动端与设备同步
- 插件系统
- 新 AI 平台与本地模型支持

---

## 🔒 隐私保护

> 本项目高度重视用户隐私，所有数据与配置完全本地化，无任何云端上传或追踪。
> AI 服务调用仅在用户授权时发生，所有操作透明可控。

---

## 🤖 强烈推荐 Ollama 本地 AI 服务

**充分保护隐私，享受本地 AI 动力**
推荐下载并使用 [Ollama](https://ollama.ai) 在本地运行 推荐模型（如 qwen3）进行分析 —— 数据绝不出本机。

- 💡 离线运行，零成本，无需网络
- 🚀 性能优异，响应快速
- 🔐 数据安全，隐私无忧

```bash
# Ollama 拉取模型
ollama pull <模型名称>
```

---

## 🚀 快速开始

<details>
  <summary><b>系统要求</b>（点击展开）</summary>

  - Windows 10/11（主要适配）
  - 4GB+ RAM（推荐 8GB+）
  - Node.js 18+
  - Rust 环境（如需构建）
</details>

1. **下载**：前往 [Releases](../../../releases) 页面获取最新版本
2. **安装 Ollama（可选）**：[ Ollama 官网 ](https://ollama.ai) - 配置本地 AI
3. **初次配置**：在应用设置页面选择/配置 AI 服务
4. **开始专注**：启动监控功能，享受专注提升历程！

#### 开发构建

```bash
git clone <repository-url>
cd MyFocus
npm install
npm run tauri:dev   # 开发模式
npm run tauri:build # 构建发布
```

---

## 🏗️ 技术架构

| 层级      | 技术栈                       |
| --------- | --------------------------- |
| 前端      | HTML/CSS/JavaScript + Vite  |
| 后端      | Rust + Tauri Framework      |
| AI 集成   | OpenAI、Ollama、Claude      |
| 数据存储  | 本地 JSON 文件              |
| OCR 模块  | Tesseract                   |
| 跨平台    | 基于 Tauri 构建原生应用     |

---

## 🤝 参与贡献

欢迎您加入开源共建！

- Issues 反馈 Bug 或建议
- PR 贡献代码或文档
- 帮助多语言翻译
- 讨论新想法

---

## 📞 联系方式

- **问题反馈**：[GitHub Issues](../../../issues)
- **功能讨论**：[GitHub Discussions](../../../discussions)
- **电子邮件**：609568171@qq.com

---

## 📄 许可证

本项目采用 **GNU AGPLv3** 许可证 - 详见 [LICENSE](LICENSE)

---

## Star 趋势

[![Star History Chart](https://api.star-history.com/svg?repos=jie0102/My_Focus&type=Date)](https://star-history.com/#jie0102/My_Focus&Date)

---

<p align="center">
  <b>让专注成为习惯，让效率成为本能</b><br>
  <i>My Focus - 您的专注力管理专家，守护您的每一分专注时光。</i>
</p>

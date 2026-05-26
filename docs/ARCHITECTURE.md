# 技术架构

## 总体架构

```text
影核 AI Desktop
├── Tauri 桌面壳
├── React + TypeScript 前端
├── Rust 本地核心
├── SQLite 本地数据库
├── Python AI Worker
├── 图片处理模块
├── 视频处理模块
└── 非破坏式修图模块
```

## 阶段 2 架构

当前只实现桌面空壳：

```text
apps/desktop
├── React UI
├── Tauri shell
└── placeholder 状态
```

阶段 2 不接入：

- SQLite
- Rust 媒体扫描
- Python AI Worker
- FFmpeg
- 修图引擎

## 前端

技术：

- React
- TypeScript
- Vite

主要组件：

- Sidebar
- TopBar
- EmptyLibrary
- FeatureCard
- StatusBar

## Tauri

Tauri 用于创建跨平台桌面应用。

职责：

- 打开桌面窗口。
- 后续调用本地 Rust 命令。
- 后续处理本地文件权限和文件夹选择。

## Rust 核心，后续

后续 Rust 模块负责：

- 文件夹扫描。
- 文件 hash。
- 媒体类型识别。
- 缩略图任务调度。
- 与 SQLite 交互。

## SQLite，后续

后续数据库存储：

- 媒体文件路径。
- 元数据。
- 缩略图路径。
- 标签。
- 人脸信息。
- AI 分析状态。
- 修图参数。

## AI Worker，后续

Python AI Worker 负责：

- 图片 embedding。
- 自然语言搜图。
- OCR。
- 人脸检测。
- 图片质量评分。
- 场景标签。

## 视频模块，后续

视频处理后续使用 FFmpeg / ffprobe：

- 提取时长、分辨率、帧率、编码。
- 生成封面图。
- 抽取关键帧。
- 后续做视频语义标签。

## 非破坏式修图模块，后续

原则：

- 原图不改。
- 参数保存。
- 可重置。
- 可批量导出。

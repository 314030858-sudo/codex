# 影核 AI

影核 AI 是一个本地优先的超级智能照片、视频整理与 AI 修图软件。

最终目标：把本地智能相册、自动整理、自然语言找图、AI 修图、批量处理、视频素材管理和企业视觉资产库整合成一个面向个人与企业的视觉资产大脑。

## 当前阶段

已完成：

1. 阶段 1：项目文档与 AI Agent 工作规则。
2. 阶段 2：Tauri + React + TypeScript 桌面空壳。

阶段 2 只实现桌面应用外壳，不包含真实媒体扫描、SQLite、AI、修图或视频处理。

## 启动桌面应用

前置要求：Node.js、npm、Rust、Tauri 所需系统依赖。

```bash
cd apps/desktop
npm install
npm run tauri dev
```

如果只想看前端页面：

```bash
cd apps/desktop
npm install
npm run dev
```

## 项目结构

```text
.
├── AGENTS.md
├── README.md
├── docs/
└── apps/
    └── desktop/
```

## 阶段 2 可见内容

桌面窗口应显示：

- 左侧导航栏：全部照片、全部视频、智能相册、AI 修图、批量处理、设置。
- 顶部搜索框：搜索照片、视频、人物、地点或产品。
- 导入文件夹按钮，占位状态。
- 主内容空状态：还没有导入照片或视频。
- 能力卡片：本地优先、自动整理、AI 修图。
- 底部状态栏：准备就绪、0 个文件、AI 未开始。

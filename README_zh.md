# PortView - 网络连接监视器

## 项目概述

PortView 是一款使用 Tauri、Vue 3 和 TypeScript 构建的跨平台网络连接监控应用程序。该应用程序提供系统 TCP/UDP 连接的实时可视化，允许用户监控活动网络连接、关联进程，并执行诸如终止进程或打开进程目录等操作。

### 关键技术
- **前端**: Vue 3 配合 TypeScript，使用组合式 API
- **后端**: Rust 配合 Tauri 框架
- **UI 框架**: 通过 Tauri 实现的原生操作系统窗口
- **国际化**: Vue I18n 配合中文和英文支持
- **构建工具**: Vite

### 架构
该应用程序遵循客户端-服务器架构：
- **前端** (Vue 3) 提供用户界面并处理用户交互
- **后端** (Rust) 访问系统级网络信息和进程详情
- **Tauri** 桥接前端和后端，在 JavaScript 和 Rust 之间提供安全通信

## 功能特性

1. **实时网络监控**: 显示活动的 TCP/UDP 连接，包含协议、地址、端口和连接状态等详细信息
2. **进程信息**: 显示每个连接关联的进程名称、PID 和图标
3. **进程管理**: 允许终止进程或打开其所在的目录
4. **过滤和搜索**: 按协议、状态、进程名称或本地端口过滤连接
5. **自动刷新**: 定期自动更新连接数据
6. **排序**: 通过点击表头对表格列进行排序
7. **深色/浅色主题**: 支持系统主题检测和手动主题切换
8. **多语言**: 中文和英文本地化
9. **进程详情模态框**: 显示进程的详细信息，包括内存使用情况、CPU 使用率和命令行

## 构建和运行

### 前提条件
- Node.js (配合 pnpm 包管理器)
- Rust 和 Cargo
- Tauri CLI

### 开发环境设置
```bash
# 安装依赖
pnpm install

# 在开发模式下运行
pnpm tauri dev

# 或者分别运行:
pnpm dev  # 仅前端 (用于网页开发)
```

### 生产构建
```bash
# 构建应用程序
pnpm tauri build

# 或者: 先构建前端，然后构建 Tauri
pnpm build
pnpm tauri build
```

### 可用脚本
- `pnpm dev`: 启动开发服务器
- `pnpm build`: 构建前端用于生产环境
- `pnpm preview`: 预览生产构建
- `pnpm tauri`: 运行 Tauri CLI 命令

## 项目结构

```
portview/
├── src/                    # Vue 3 前端源码
│   ├── App.vue            # 主应用程序组件
│   ├── main.ts            # 应用程序入口点
│   ├── i18n.ts            # 国际化设置
│   ├── components/        # Vue 组件
│   └── locales/           # 翻译文件
├── src-tauri/             # Rust 后端源码
│   ├── src/
│   │   └── lib.rs         # 主 Rust 实现
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
├── public/                # 静态资源
├── index.html             # HTML 模板
├── package.json           # Node.js 依赖和脚本
├── vite.config.ts         # Vite 构建配置
└── tsconfig.json          # TypeScript 配置
```

## 开发规范

### 前端 (Vue/TypeScript)
- 使用 Vue 3 组合式 API 的组件化架构
- TypeScript 保证类型安全
- 使用 vue-i18n 进行国际化，配合 JSON 翻译文件
- 使用 CSS 变量响应式设计以支持主题
- 组件间的事件驱动通信

### 后端 (Rust)
- Tauri 命令用于向前端暴露 Rust 功能
- 使用 `netstat2` crate 进行跨平台网络监控
- 使用 `sysinfo` crate 获取进程信息
- 平台特定的进程图标实现
- 使用 Result 类型进行错误处理

### 国际化
- 翻译键定义在 `src/locales/zh-CN.json` 和 `src/locales/en-US.json` 中
- 语言首选项存储在 localStorage 中
- 自动系统语言检测

## 关键组件

### 前端组件
- `App.vue`: 主应用程序，包含连接管理和 UI 控制
- `ConnectionsTable`: 在可排序表格中显示网络连接
- `MenuBar`: 顶部菜单，包含过滤、搜索和主题控制
- `StatusBar`: 显示连接统计信息和刷新信息
- `ContextMenu`: 右键菜单用于进程操作
- `ProcessDetailsModal`: 显示进程详细信息的模态框
- `AboutDialog`: 关于应用程序对话框

### 后端命令
- `get_connections`: 检索所有活动网络连接
- `get_process_details`: 获取特定进程的详细信息
- `kill_process`: 通过 PID 终止进程
- `open_folder`: 打开包含进程可执行文件的目录

## 平台支持

该应用程序设计为跨平台工作：
- **Windows**: 完整支持，包含进程图标提取
- **Linux**: 网络监控和进程管理
- **macOS**: 网络监控和进程管理

平台特定功能通过 Rust 代码中的条件编译处理。

## 安全考虑

- Tauri 在前端和后端之间提供安全通信
- 对系统级信息的访问限制在后端 Rust 代码中
- 进程管理功能限于当前用户的权限
- 此配置中禁用了 CSP (内容安全策略)，以便于开发灵活性

## 测试

虽然在初始探索中未找到明确的测试文件，但典型的 Tauri + Vue 应用程序将包括：
- Vue 组件的单元测试
- Tauri 命令的集成测试
- 完整应用程序流程的端到端测试

## 部署

Tauri 构建过程为每个平台创建原生可执行文件：
- Windows: `.exe` 文件
- macOS: `.app` 捆绑包
- Linux: AppImage 或根据配置的 deb/rpm 包

该应用程序将前端资源和 Rust 后端捆绑成单个可执行文件。

---

## 致谢

特别感谢 [Qwen Code](https://github.com/QwenLM/qwen-code) 在创建和维护此项目文档方面的帮助。

---

[English README](./README.md)
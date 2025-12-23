# Batata RPA

<p align="center">
  <img src="public/tauri.svg" width="120" alt="Batata RPA Logo">
</p>

<p align="center">
  基于 Tauri 2、Vue 3 和 Rust 构建的跨平台桌面 RPA（机器人流程自动化）应用程序。
</p>

<p align="center">
  <a href="./README.md">English</a>
</p>

## 功能特性

### 流程设计器
- **可视化流程编辑** - 基于 Vue Flow 的拖拽式节点流程设计
- **丰富的节点库** - 预置常用自动化任务节点
- **实时预览** - 构建时即时查看流程结构

### 节点类型
| 分类 | 节点 |
|------|------|
| **流程控制** | 开始、结束、条件分支、循环、ForEach遍历、Try-Catch异常处理 |
| **网页自动化** | 打开网页、点击、输入、等待、截图 |
| **桌面自动化** | 点击、输入、获取文本（平台相关） |
| **数据处理** | 读取Excel、写入Excel、设置变量 |
| **工具** | 延迟、日志 |

### 执行引擎
- **断点调试** - 设置断点并单步执行流程
- **变量监视** - 执行期间监控变量值
- **执行日志** - 详细日志记录，支持过滤功能
- **错误处理** - Try-Catch节点实现健壮的错误管理

### 平台支持
- **Windows** - UI Automation API 原生应用控制
- **macOS** - Accessibility API 集成
- **Linux** - AT-SPI 和 X11 支持

### 浏览器自动化
- **Chrome DevTools Protocol** - 通过 CDP 直接控制浏览器
- **元素选择** - 网页元素可视化选择器
- **截图** - 页面或元素截图

### 数据操作
- **Excel支持** - 读写 Excel 文件（.xlsx）
- **变量系统** - 动态变量与表达式插值

## 技术栈

### 前端
- **Vue 3** - Composition API + TypeScript
- **Tailwind CSS 4** - 原子化 CSS 框架
- **Vue Flow** - 流程画布组件
- **Pinia** - 状态管理
- **Lucide Icons** - 图标库
- **Vue Router** - 客户端路由

### 后端
- **Tauri 2** - 轻量级桌面应用框架
- **Rust** - 系统编程语言
- **Tokio** - 异步运行时
- **SeaORM** - 数据库 ORM（支持 SQLite/MySQL/PostgreSQL）
- **chromiumoxide** - Chrome DevTools Protocol 客户端
- **mlua** - Lua 脚本插件支持

## 环境要求

- **Node.js** >= 18
- **pnpm** >= 8
- **Rust** >= 1.70
- **平台 SDK**：
  - Windows：Windows SDK
  - macOS：Xcode Command Line Tools
  - Linux：`libgtk-3-dev`、`libwebkit2gtk-4.1-dev`、`libayatana-appindicator3-dev`

## 安装

```bash
# 克隆仓库
git clone https://github.com/easynet-cn/batata-rpa.git
cd batata-rpa

# 安装依赖
pnpm install
```

## 开发

```bash
# 启动开发服务器（仅前端）
pnpm dev

# 启动完整 Tauri 应用（开发模式）
pnpm tauri dev

# 类型检查
pnpm build
```

## 构建

```bash
# 构建生产版本
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 项目结构

```
batata-rpa/
├── src/                          # Vue 3 前端
│   ├── views/                    # 页面组件
│   │   ├── Designer.vue          # 流程设计器
│   │   ├── ElementLibrary.vue    # 元素库管理
│   │   ├── Runner.vue            # 执行监控
│   │   └── Settings.vue          # 应用设置
│   ├── components/
│   │   ├── designer/             # 设计器组件
│   │   │   ├── NodePalette.vue   # 节点工具箱
│   │   │   ├── PropertyPanel.vue # 属性编辑器
│   │   │   ├── DebugPanel.vue    # 调试控制
│   │   │   └── nodes/            # 自定义节点组件
│   │   └── layout/               # 布局组件
│   ├── stores/                   # Pinia 状态仓库
│   ├── types/                    # TypeScript 类型
│   └── router/                   # Vue Router 配置
├── src-tauri/                    # Rust 后端
│   └── src/
│       ├── commands/             # Tauri IPC 命令
│       ├── automation/           # 平台自动化
│       │   ├── desktop/          # 桌面 UI 自动化
│       │   ├── web/              # 浏览器自动化
│       │   └── file/             # 文件操作
│       ├── engine/               # 执行引擎
│       └── element/              # 元素定义
├── plugins/                      # Lua 插件
└── public/                       # 静态资源
```

## 配置

### Tauri 配置
编辑 `src-tauri/tauri.conf.json` 进行应用配置：
- 窗口大小和标题
- 应用标识符
- 构建目标
- 权限设置

### 环境变量
创建 `.env` 文件配置环境相关设置。

## 插件系统

Batata RPA 支持 Lua 插件扩展功能。将插件文件放置于 `plugins/` 目录。

```lua
-- plugins/example_plugin.lua
function on_node_execute(node)
    print("正在执行节点: " .. node.id)
end
```

## 参与贡献

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m '添加新功能'`)
4. 推送分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 开源协议

本项目采用 MIT 协议开源 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- [Tauri](https://tauri.app/) - 桌面应用框架
- [Vue Flow](https://vueflow.dev/) - 流程编辑器
- [Tailwind CSS](https://tailwindcss.com/) - CSS 框架
- [Lucide](https://lucide.dev/) - 图标库

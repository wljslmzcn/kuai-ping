# 快Ping - 轻量高效的网络测试工具

基于 Tauri 2 + Vue 3 + TypeScript + Element Plus + ECharts 构建的跨平台桌面 Ping 工具。

## ✨ 功能特性

- 🎯 **单目标/批量Ping** - 支持IP和域名，自动去重
- 📊 **实时数据表格** - 延迟、丢包率、统计信息实时更新
- 📈 **延迟曲线图** - ECharts渲染，选中IP查看实时延迟走势
- ⚙️ **灵活配置** - 数据包大小、间隔、超时、模式可调
- 📥 **CSV导出** - 一键导出测试报告
- 🔍 **DNS解析** - 自动解析域名为IP地址
- 🚀 **跨平台** - 支持Windows、macOS、Linux

## 🛠️ 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 + TypeScript + Element Plus |
| 图表 | ECharts 5 |
| 后端 | Rust + Tokio (异步并发) |
| Ping实现 | 系统原生ping命令 (无需管理员权限) |

## 📁 项目结构

```
快ping/
├── src/                          # Vue前端源码
│   ├── components/
│   │   └── PingChart.vue         # ECharts图表组件
│   ├── App.vue                   # 主界面组件
│   ├── main.ts                   # Vue入口
│   └── style.css                 # 全局样式
├── src-tauri/                    # Rust后端源码
│   ├── src/
│   │   ├── main.rs               # 程序入口
│   │   ├── lib.rs                # Tauri命令定义
│   │   └── ping.rs               # Ping核心逻辑
│   ├── icons/                    # 应用图标
│   ├── Cargo.toml                # Rust依赖配置
│   ├── tauri.conf.json           # Tauri配置
│   └── build.rs                  # 构建脚本
├── package.json                  # 前端依赖配置
├── vite.config.ts                # Vite配置
├── tsconfig.json                 # TypeScript配置
└── index.html                    # HTML入口
```

## 🚀 快速开始

### 环境准备

1. **Node.js** >= 18
2. **Rust** >= 1.70
3. **系统依赖**:
   - Windows: 无额外依赖
   - macOS: `xcode-select --install`
   - Ubuntu/Debian: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`

### 安装依赖

```bash
# 安装前端依赖
npm install

# Rust依赖会自动在首次构建时下载
```

### 开发模式

```bash
npm run tauri dev
```

### 构建打包

```bash
# Windows (生成 exe 和 msi)
npm run tauri build

# macOS (生成 dmg)
npm run tauri build

# Linux (生成 AppImage 和 deb)
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 📖 使用说明

1. **输入目标**: 在左侧输入框每行输入一个IP或域名
2. **配置参数**: 调整数据包大小、发送间隔、超时时间等
3. **选择模式**:
   - 持续Ping: 持续测试直到手动停止
   - 指定次数: 达到设定次数后自动停止
4. **开始测试**: 点击"开始测试"按钮
5. **查看结果**: 右侧表格实时显示每个目标的测试数据
6. **查看图表**: 点击表格中某行，下方显示该IP的延迟曲线
7. **导出报告**: 点击"导出CSV"保存测试结果

## 🔧 跨平台注意事项

### Windows
- 使用系统自带的 `ping.exe`，无需管理员权限
- 命令格式: `ping -n 次数 -w 超时毫秒 -l 包大小 目标`

### macOS / Linux
- 使用系统 `ping` 命令
- 命令格式: `ping -c 次数 -W 超时秒 -s 包大小 目标`
- 部分Linux发行版可能需要安装 `iputils-ping`

### DNS解析
- Windows 使用 `nslookup` 命令
- macOS/Linux 使用 `host` 命令

## ⚠️ 已知限制

1. **权限**: 默认使用系统ping命令，无需管理员/root权限；如需使用原始套接字模式（更低延迟），需以管理员权限运行
2. **并发数**: 建议同时Ping的目标不超过50个，过多可能导致系统资源紧张
3. **图表性能**: 超过100个数据点时自动滚动，防止页面卡顿
4. **ICMP限制**: 某些网络环境可能禁止ICMP包，导致全部超时

## 📄 许可证

MIT License

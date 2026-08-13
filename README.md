# 快Ping

轻量高效的网络测试工具，基于 Tauri 2 + Vue 3 构建。

<p align="center">
  <img src="public/wechat.jpg" width="120" alt="公众号二维码" />
  <br/>
  <strong>关注公众号「网络技术联盟站」回复「加群」加入沟通群</strong>
</p>

## ✨ 功能特性

- 🚀 多目标并发 Ping 测试
- 📊 实时延迟曲线图
- 📋 测试结果导出 CSV
- 🎨 Mac 风格 UI，支持亮色/暗色模式
- 💻 Windows / macOS / Linux 全平台支持

## 📦 下载

前往 [Releases](https://github.com/wljslmzcn/kuai-ping/releases) 页面下载对应系统的版本。

| 平台 | 格式 |
|---|---|
| Windows | `.exe` 安装包 |
| macOS | `.dmg` 安装包 |
| Linux | `.AppImage` / `.deb` |

## 🛠️ 开发

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

## 📁 项目结构

```
kuai-ping/
├── src/                  # Vue 3 前端
│   ├── App.vue          # 主界面
│   ├── components/      # 组件
│   ├── style.css        # 全局样式
│   └── main.ts          # 入口
├── src-tauri/           # Tauri 后端
│   ├── src/
│   │   ├── lib.rs       # 应用逻辑
│   │   └── ping.rs      # Ping 核心
│   ├── icons/           # 应用图标
│   └── tauri.conf.json  # 配置
└── public/              # 静态资源
```

## 📄 License

MIT License

---

> 本软件由公众号「网络技术联盟站」开发
# 快Ping - 轻量高效的网络测试工具

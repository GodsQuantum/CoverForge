# CoverForge

**AI 负责生成主视觉，CoverForge 负责把它变成可发布的成品。**

一个基于 Rust 的确定性图片合成器，适合播客缩略图和社交媒体封面。

[English](README.md) · [Français](README.fr.md)

## 为什么

图像模型很擅长生成场景，但不擅长稳定复现字体、Logo 和品牌布局。CoverForge 把工作拆开：

1. 为目标比例生成 **原生 key art**；
2. 用确定性的渲染器加入字体、Logo 和颜色；
3. 一个节目只维护 **一个 JSON 风格文件**。

不使用 `16:9 → 模糊背景 → 中间塞一张图` 这种假适配。

## v0.3

- ⚡ Rust + Axum + resvg
- 🧩 每个节目一个 JSON，包含全部版式
- 🖥️ Svelte 编辑器
- 🔤 内置大量可再分发字体
- ⬆️ UI 上传自定义字体
- 📁 也可以直接把字体放进 Custom 目录
- 🐳 Docker Compose
- 🔌 API-first，并兼容旧自动化接口
- 💾 模板原子保存

## 输出格式

| key | size |
|---|---:|
| `youtube` | 3840×2160 |
| `square` | 1200×1200 |
| `feed` | 1080×1350 |
| `vertical` | 2160×3840 |
| `acast` | 3000×3000 |

推荐直接生成 16:9 / 1:1 / 4:5 / 9:16 的原生主视觉，再由 CoverForge 完成最终品牌化。

## 字体

内置自由字体包。自定义 `.ttf` / `.otf` / `.ttc` / `.otc` 可以通过 UI 上传，也可以直接放到持久化 Custom 目录。商业或私有字体不会进入公开仓库。

## 无废话原则

- 主视觉按最终比例生成；
- 字体和 Logo 交给合成器；
- 一个节目只维护一个风格文件；
- 私有素材不进 Git；
- 改模板不需要重新构建应用。

## Docker

```sh
docker compose up -d --build
curl -fsS http://127.0.0.1:3099/health
```

## License

MIT

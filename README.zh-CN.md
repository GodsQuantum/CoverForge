<p align="center">
  <img src="brand/coverforge-logo.svg" alt="CoverForge" width="520">
</p>

# CoverForge — 简体中文指南

**一张图，多种格式。自托管、API 优先的品牌自动化工作室。**

CoverForge 将一张原始图片和一套品牌规范转换成多平台视觉素材：YouTube、1:1 方图、4:5 Feed、9:16 竖图、1.91:1 横图，或 JSON 模板中定义的任意尺寸。

![CoverForge 工作流](docs/assets/coverforge-workflow.svg)

```text
图片 → 智能裁切 → 品牌工具包 → 动态图层 → 多格式 → ZIP / API
```

## 产品定位

CoverForge 是一个 **品牌自动化（Branding Automation）** 工具，适合：

- 内容创作者；
- 品牌团队；
- 设计与营销代理机构；
- 社交媒体团队；
- 电商团队；
- 自动化发布流程。

CoverForge 将以下职责分离：

1. 原始图片；
2. 品牌视觉规范；
3. 各种比例的独立布局；
4. 最终确定性渲染。

浏览器编辑器使用 Svelte 5 + Fabric.js。最终生产渲染由 Rust + resvg/tiny-skia 完成。

## 导入图片

在 **项目 / Projet** 页面中，可以拖拽或选择文件。

支持的位图格式：

- JPEG；
- PNG；
- WebP。

SVG 仅作为安全验证后的 Logo 使用。

替换原始图片不会删除：

- 标题；
- 副标题；
- Logo；
- 布局位置；
- Brand Kit 设置。

本次浏览器会话中导入的素材会显示在 **素材库 / Bibliothèque**，无需再次上传即可复用。

## Smart Reframe / 智能裁切

Smart Reframe 会在本地为每种目标比例计算：

- 焦点；
- 裁切区域；
- 归一化坐标。

算法会：

- 分析缩小后的图片；
- 优先考虑细节、边缘与高对比区域；
- 考虑颜色饱和度；
- 轻度使用三分法构图偏好；
- 保持完全确定性；
- 不依赖外部 AI API。

你可以：

- 使用自动焦点；
- 手动移动全局焦点；
- 为某个格式保留独立手动设置；
- 将某个格式重新切回自动模式。

## Brand Kit / 品牌工具包

Brand Kit 会保存在模板 JSON 中，可包含：

- 品牌名称；
- 视觉说明；
- 主 Logo；
- 次 Logo；
- 配色；
- 字体角色；
- 备注。

公共仓库不包含商业字体文件。CoverForge 可以使用：

- 系统字体；
- Docker 镜像中的自由字体；
- 你在实例中上传的自定义字体。

## 动态图层

支持三类图层：

- 图片；
- 文字；
- 矩形。

每个图层都有稳定的 API ID。

文字可使用变量：

```text
{{title}}
{{subtitle}}
{{badge}}
{{episode}}
```

编辑器支持：

- X / Y / 宽度 / 高度滑杆和精确数值；
- 不透明度；
- 旋转；
- 对齐；
- 颜色和描边；
- 字体家族；
- 真实字体变体；
- 自动适配文字；
- 最小字号；
- 最大行数；
- 图片焦点 X/Y；
- 图层排序；
- 显示/隐藏；
- 锁定；
- 复制；
- 删除。

## 多格式输出

内置通用模板 `starter-brand` 提供：

| 格式 | 尺寸 |
| --- | ---: |
| YouTube 16:9 | 1920×1080 |
| 方图 1:1 | 1080×1080 |
| Feed 4:5 | 1080×1350 |
| 竖图 9:16 | 1080×1920 |
| 横图 1.91:1 | 1200×628 |

每个格式拥有独立图层栈。因此修改竖图中的标题位置不会影响方图。

## ZIP 导出

在 **导出 / Exports** 页面中可以：

- 只渲染所选格式；
- 渲染全部格式；
- 单独下载每张图片；
- 下载一个 ZIP 包。

ZIP 包包含：

- 所有渲染图片；
- `manifest.json`。

Manifest 包含：

- CoverForge 版本；
- 生成时间；
- 模板 ID；
- 原始图片引用；
- 变量；
- 输出格式；
- 尺寸；
- 文件名。

## API 与 OpenAPI

动态 API 规范：

```text
GET /openapi.json
```

主要接口：

```text
POST /v1/assets
POST /v1/reframe
POST /v1/render
POST /v1/render/preview
POST /v1/render/batch
POST /v1/render/package
```

读取模板自动填充字段：

```text
GET /v1/templates/{id}/dataset
```

完整示例见：[docs/API.md](docs/API.md)。

## Docker 快速启动

```bash
docker build -t coverforge .
docker run --rm -p 3099:3099 \
  -e COVERFORGE_BIND=0.0.0.0:3099 \
  -e COVERFORGE_DEFAULT_TEMPLATE_DIR=/app/default-templates \
  coverforge
```

然后打开：

```text
http://localhost:3099
```

仓库中的 `compose.yaml` 针对现有 Cloud9 部署进行了定制。如果在其他机器部署，请修改 bind mount。

## 安全设计

CoverForge 包含以下保护：

- 上传文件大小限制；
- 解码像素上限；
- 实际图片格式检测；
- JPEG EXIF 方向归一化；
- 拒绝带脚本、事件处理器或外部引用的 SVG；
- 文件访问限制在允许目录中；
- ZIP 文件名防止路径穿越；
- Smart Reframe 与渲染均在本地完成。

v0.6 不包含应用层登录系统。如果公开暴露实例，请通过反向代理或访问控制系统进行保护。

## AutoPublisher 兼容性

旧接口仍然保留：

```text
POST /api/generate
```

例如 `DPAFM - YT` 这样的旧模板名称仍会映射到对应模板与格式。

历史模板 `cf`、`cp`、`dpafm`、`lcfp` 等继续兼容。它们现在只是通用品牌引擎的一种具体使用方式。

## JSON 模板

JSON 始终是唯一真实来源。

详细说明：[docs/SHOW_STYLES.md](docs/SHOW_STYLES.md)。

内置 `starter-brand` 位于 Docker 镜像默认模板目录中。如果用户目录存在同名模板，用户模板优先。

## 参与开发

提交前建议执行：

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings

cd web
npm ci
npm run check
npm test
npm run build
```

请勿提交：

- 私有素材；
- 商业字体文件；
- 密钥；
- 生产环境秘密。

## 许可证

MIT

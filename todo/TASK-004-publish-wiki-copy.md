---
id: TASK-004
status: done
depends_on: []
evidence: ["docs/vision.md", "docs/characters/agent.md", "docs/locations/shop.md", "docs/conventions.md"]
---

# Wiki 发布文案

## 交付内容

Wiki 以 N:SIDE 自身的世界、人物、玩法与制作原则直接表述，核心理念融入对应主题，开发任务只记录当前工作与实际结果

首发支持 PC，系统为 Linux 和 Windows，输入支持手柄和键鼠，图形接口采用 Vulkan；本次调整项目定位文案，未变更游戏渲染配置或声明跨平台运行验收完成

## 验证

- `bun run check:docs`：通过
- `bun run check:narrative`：EMPTY，当前没有正式叙事数据
- `bun run check:templates`：通过，1 份模板、2 个状态
- `git diff --check`：通过
- 相关项目约定按 prompt-skill-authoring 与 prompt-skill-review 检查，表述范围与正文、开发记录的职责一致

## 发布

`BUN_TMPDIR=/tmp BUN_INSTALL_CACHE_DIR=/tmp/nside-bun-cache bun run docs:build` 通过

使用 `sudo rsync -a --delete --chmod=D755,F644 docs/.vitepress/dist/ /srv/n-side-wiki/` 同步静态站点，10 个主要页面通过真实域名 IPv6 HTTPS 请求校验，与最终构建逐字节一致

访问入口：`https://ms.lzzz.ink:7777/`，发布说明统一使用现有 Caddy 入口

文案与构建验证完成，未执行浏览器交互或游戏跨平台运行验收，成果保留本地工作树且未提交

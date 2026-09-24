---
id: TASK-001
status: in_progress
depends_on: []
evidence: ["README.md", "AGENTS.md", "bun.lock", "game/README.md", "tools/tests/wiki.test.mjs"]
---

# 整理项目文档与迁移游戏工程

## 目标与验收

按交接包 `CODEX-PROMPT.md` 与 `migration.json` 合并 overlay，迁移现有游戏工程，生成真实 Bun 锁文件，完成文档、工具、Wiki 与游戏检查，成果保留在本地工作树供审阅

## 交付记录

2026-09-24：接入前工作树干净，原仓库只有 Cargo 工程、README、LICENSE 与 `docs/images/hero.png`，没有旧版设定正文、任务记录、资产、构建脚本、CI 或相对路径依赖需要合并

- `Cargo.toml`、`Cargo.lock`、`src/` 整体迁入 `game/`，内容与原 Git 版本一致，现有编译缓存迁入 `game/target/` 并继续忽略
- 接入 Wiki 正文、模板、导航、工具、三个项目 Skill 与开发任务，根 README 合并游戏启动入口和 MPL-2.0 许可证入口
- 使用 Bun 1.4.2 安装 VitePress 1.6.4，生成真实 `bun.lock`，所有 JavaScript 命令使用 Bun，VitePress CLI 显式使用 `bun --bun`，`node:` API 由 Bun 兼容层提供
- 三张文档图片使用 Pillow 12.3.0 的 `Image.save(..., 'WEBP', quality=80)` 转换，校验像素尺寸一致并更新引用，总大小从 6,507,434 bytes 降至 998,350 bytes，原 hero 图在根 README 中保留展示
- 两条新增规则写入根 `AGENTS.md`，项目约定同步说明；按 prompt-skill-review 检查相关指令，未发现高影响冲突
- 修复 VitePress 将行内 `{{...}}` 误解析为 Vue 表达式的问题，Markdown 行内代码统一加 `v-pre`，增加实际 Markdown 渲染回归测试

### 实际检查

下表命令从仓库根目录执行，Cargo 格式、检查、测试和启动命令在 `game/` 执行

| 命令或检查 | 实际结果 |
| --- | --- |
| `BUN_TMPDIR=/tmp BUN_INSTALL_CACHE_DIR=/tmp/nside-bun-cache bun install` | 通过，安装 127 个包并生成 `bun.lock` |
| `bun run check:docs` | 通过，51 份 Markdown、26 个 ID |
| `bun run check:narrative` | `EMPTY`，0 份正式叙事数据、0 个状态，不代表已有剧情通过验收 |
| `bun run check:templates` | 通过，1 份模板数据、2 个状态 |
| `bun run test:tools` | 通过，68 项 Python 测试、13 项 Bun JavaScript 测试 |
| `BUN_TMPDIR=/tmp BUN_INSTALL_CACHE_DIR=/tmp/nside-bun-cache bun run docs:build` | 通过，冻结锁文件安装及 Bun 运行时完整构建，产物在 `docs/.vitepress/dist/` |
| `bun run docs:preview` | Bun 预览启动成功，监听 `http://localhost:4173/`，检查后停止 |
| `python3 /tmp/nside-preview-check.py` | 本次临时 HTTP 检查通过：40 个 HTML 页面、四组导航及目标路由、`zh-CN` 与中文内容、字面占位符、3 份 JSON/CSV 源数据、3 张 WebP、6 个首页资源与 1 份搜索索引；数据及图片与源文件逐字节一致 |
| `cargo fmt --check` | 通过 |
| `cargo check --offline --locked` | 通过 |
| `cargo test --offline --locked` | 通过，当前工程 0 项测试 |
| `cargo build --offline --locked --manifest-path game/Cargo.toml` | 从仓库根目录执行，通过开发构建 |
| `timeout 10s cargo run --offline --locked` | 从 `game/` 正确启动二进制，退出码 101；当前环境未设置 `WAYLAND_DISPLAY`、`WAYLAND_SOCKET` 或 `DISPLAY`，无法建立窗口事件循环 |
| `git diff --check` 与新增文本检查 | 通过 |

### 问题与待处理

Bun 首次安装因沙箱临时目录只读和注册表 DNS 失败而未完成，指定临时目录并使用获准联网执行后成功；Bun 构建的 esbuild 子进程和预览监听在沙箱内受限，获准执行后通过，这些是本次执行环境限制

最初包内命令使用 Node 执行测试和 VitePress；收到 Bun 专用规则后已改为 Bun，并重新通过测试、完整构建及预览，最终命令以当前 `package.json` 为准

当前 Computer Use 返回无可用浏览器，创建预览标签也返回 `Browser is not available: iab`，因此浏览器中的实际搜索、导航点击、图片观感与模板显示尚待交互验收，HTTP 与构建检查不替代浏览器验收

游戏窗口与实际渲染尚待在具备显示服务的桌面执行 `cd game && cargo run --locked`，当前没有运行资产；release 构建、性能、手柄和玩法未验证

任务保留 `in_progress`，剩余事项为浏览器与游戏图形验收；本轮未提交、未推送、未部署 Cloudflare

### 本机 Caddy 交付

用户后续授权本机暴露 Wiki，确认 IPv6 放行端口为 `7777`，并要求关闭 `60000`

- 构建产物复制到 `/srv/n-side-wiki`，目录权限 755、文件权限 644，Caddy 直接提供静态页面，源代码与任务记录不对外发布
- 仓库保留配置 `tools/caddy/n-side-wiki.caddy`，安装到 `/etc/caddy/conf.d/n-side-wiki.caddy`，使用现有通配符证书和 `try_files` 支持 VitePress 无扩展名路由
- 原 `aercast.caddy` 移至 `/etc/caddy/disabled/aercast.caddy.before-n-side-20260924`，执行 `sudo systemctl reload caddy` 成功，服务保持 active
- `sudo ufw allow in proto tcp from ::/0 to any port 7777 comment 'N-SIDE Wiki IPv6'` 成功，仅新增 IPv6 TCP 7777 放行
- `sudo ufw --force delete allow in proto tcp from ::/0 to any port 60000` 与 UDP 对应命令成功，`ufw status numbered` 确认两条旧规则已删除
- `ss` 确认 Caddy 监听 7777，60000 无 TCP 或 UDP 监听
- `curl --noproxy '*' --resolve 'ms.lzzz.ink:7777:[::1]' https://ms.lzzz.ink:7777/` 在证书校验开启的情况下返回 200，真实域名 `curl --noproxy '*' -6 https://ms.lzzz.ink:7777/` 同样返回 200
- 真实域名 IPv6 HTTPS 的首页、四组导航、占位符模板、3 份 JSON/CSV 与 3 张 WebP 共 12 个响应与构建逐字节一致，证书校验通过
- DNS 查询显示的 AAAA 与本机枚举地址不同，但实际按域名 IPv6 HTTPS 请求成功；未变更 DNS，未从异地网络进行验收

本机访问入口为 `https://ms.lzzz.ink:7777/`，与 Cloudflare Pages 独立，浏览器交互与游戏窗口验收仍待完成

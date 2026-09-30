# Wiki 侧栏版本号检查

## 输入与交付

2026-09-30，输入内容 SHA-256 `a0b90d93e56972f7f222451d85eefb5efdfcdf9ce54dbad297f643640bf175bb`，按 package.json、docs/.vitepress/sidebar.ts、docs/dev/handbook/wiki.md 顺序拼接计算。版本为 v0.1.0，侧栏首页链接文字右侧显示，双站共用 package.json 的版本字段

## 验证

- PASS：`bun run check:types`
- PASS：`bun test tools/tests/wiki.test.ts`，14 项通过
- PASS：`bun run check:docs`
- PASS：`bun run docs:build`；最终标题辅助文本调整后再运行 `bun run docs:build:player` 与 `bun run docs:build:dev`，产物检查均通过
- PASS：浏览器 self-audit，桌面与 402×874 手机视口打开目录，v0.1.0 位于 N:SIDE 同行最右侧，无换行或遮挡；仅浏览器内观察截图，未落盘视觉产物，已恢复视口
- PASS：`git diff --check`
- NOT RUN：线上部署，本轮未提交或发布

## Cloudflare 报错排查

仓库中没有 cloudflareinsights/beacon 脚本或对应 integrity 注入。用户日志符合 Cloudflare 自动注入脚本形式；未能取得线上响应，不能确认网络拦截、缓存或响应内容异常的具体原因。当前没有可用的 Cloudflare 管理连接，未修改线上设置。官方说明见 https://developers.cloudflare.com/web-analytics/faq/ ，若不需要访客端统计，可在 Cloudflare Web Analytics 关闭该站自动注入；需要统计时继续核对实际脚本响应和浏览器拦截，不替换 integrity 为报错哈希

## 收尾

本轮没有生成需清理的截图、视频或探测程序，用户附件保留；构建日志保存在本目录，既有无关工作区改动保留

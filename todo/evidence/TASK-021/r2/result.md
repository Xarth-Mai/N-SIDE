# 实际站点接入与旧产物清理

## 原因与修复

本机 Caddy 的该站点仍指向迁移前 `docs/.vitepress/dist`，新工具链产物在 `output/wiki/player/dist`，所以线上 `/story/` 一直显示旧页面。此前把仓库生成路径直接当成当前线上路由的答复没有核对实际服务配置

已将 `/etc/caddy/conf.d/n-side-wiki.caddy` 的静态根切换到通过产物检查的玩家站目录，保留原站点域名、TLS 与 try_files 配置。补齐 Caddy 用户对父目录的遍历权限及公开 dist 的读取权限，设置 dist 子目录默认 ACL 以支持后续生成文件继承；未将开发源资料开放给服务用户

配置备份位于 `/tmp/nside-caddy-story-r2.backup`，权限快照位于 `/tmp/nside-caddy-story-r2.acl`。实际执行了 Caddy 配置校验及 systemd 平滑重载，服务保持 active

用户旧入口 `/story/` 现在提供保留锚点的兼容跳转，目标 `/player/encyclopedia/story/` 返回新时间轴。目录切换已在本机生效，不依赖 Git 推送

## 检查与清理

`python3 todo/evidence/TASK-021/r2/check-live.py https://ms.lzzz.ink:7777` 完成 [47 项真实 HTTPS 检查](live.json)：旧入口、十八张卡片、十八篇正文关系详情、25 个脚本／样式／字体资源，以及开发源资料返回 404。使用系统证书验证，未跳过 TLS 校验

旧 `docs/.vitepress/dist` 和 `docs/.vitepress/cache` 均未被 Git 跟踪，新配置不再引用它们；目录范围内 lsof 未发现使用进程。按用户要求删除两处旧生成目录，清理约 20.5 MiB 文件内容。源码、当前发布目录及历史任务证据保留；配置备份仅作记录，不再把已清理的旧产物作为可用回滚包

清理后重复实际 HTTPS 检查，确认新站点仍然可达。新增发布手册说明：构建成功不能证明静态服务已接入，必须检查根目录、运行用户权限、旧 URL 的实际目标和页面资源

## 状态

发布入口修复 PASS，旧产物清理 PASS。浏览器画面与交互仍为 NOT RUN；HTTP 内容和资源校验不替代客户端水合或手机布局观察，TASK-021 保留原验收缺口

文档与任务检查 PASS，开发站 144 页构建 PASS，玩家站现有产物检查 PASS，`git diff --check` PASS。ponytail-review：Lean already. Ship.

后续作者反馈确认原站点需要同时保留开发资料，本轮选择 player 构建不符合原站点范围；该发布选择与只验证玩家边界的结论由[第 3 轮纠正](../r3/result.md)替代，原始 HTTP 记录保留用于追溯

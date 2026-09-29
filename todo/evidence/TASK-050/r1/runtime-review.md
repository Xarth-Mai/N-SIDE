# 门前观察与大字恢复实机检查

输入故事基线为 `4801b81bc3c91b5b4eebe04ebd86aa1f245b52c3`，期间另一个叙事 Agent 提交 `1bfc1cf`。已核对该提交的序章与分场差异，第三项门前交接的公共步行／货运职责没有变化，当前运行片段继续有效；其余新故事发展不由这段片段代替

实际执行 `python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/prologue-handoff.json --output output/capture/task050-handoff-r1`：540 帧 / 18 秒，22 项检查 PASS，见 [原始运行](../capture-r1/run.json)；正式入口按实际移动接近店门，F 打开后取得且仅取得 `04`，暂停、断连、恢复与重复观察保持记录，回标题后清空

另执行 `python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/walk-observation-settings.json --output output/capture/task050-character-large-reentry`：780 帧 / 26 秒，21 项 PASS，见 [大字和重入](../large-reentry-r1/run.json)。125% 中文、键鼠／模拟手柄提示与返回按钮都在 1280×720 画布内；面板正文有滚动区域，按钮固定。root 实际看过 339、379、679 和首片段 190、191、199、299 等帧，哈希保留在对应 observed-images.json

大字 HUD 在附近名称和长操作提示处仍出现短词跨行，文字未丢失但排版可继续收敛；本轮没有用截图宣布全部 UI 完成。当前只观察一个地点，未触发三记录积累后的最终选择布局，仍需那条实际路线覆盖更多正文和按钮

状态机与实际输入路由 CPU 窄测覆盖乱序、缺项、台阶错误选择、修订、幂等与标题清理；完整 `04 → 29 → 28` GPU 路线仍为 NOT RUN。新的 [路线候选](../route-r1/analysis.md) 找到现有南侧铺面与服务前坪连接，应连续使用真实控制器验证，不能直接写目标或记录来补通过

任务维持 active；未完成搬运、NPC演出、完整序章、任务磁盘存档、作者或陌生玩家验收。视觉文件实际查看后清理，日志和状态保留，清理结果见本目录 cleanup.json

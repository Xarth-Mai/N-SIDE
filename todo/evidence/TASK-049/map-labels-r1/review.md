# 地图高程标签与正式名称核对

## 改动

基线、逐项旧值／新值及节点高程见 [changes.json](changes.json)，本轮只改 `district.json` 的 `sections.labels` 文本，编辑时将全部 labels 还原后做全对象等值检查，确认坐标、稳定 ID、拓扑、建筑高度、路线和其他文本均未变化

S9 有三处过期高程：台阶平台 `steps_mid` 原写 +32，实际 +39.636364；住宅路 `upper_homes` 原写 +39.2，实际 +61.6；住宅入口 `homes_entry` 原写 +42，实际 +72.315789

剖面消费者 `docs/.vitepress/components/DistrictPlan.vue` 已在每个名称下方直接显示 `sample.height`，因此删掉全体剖面标签中 19 个重复手填的高程后缀，保留名称与现有动态高程行；S1 的 `hillgate` 标签由“山脚”改为“林缘入口”，符合小店 +28 m 已是起坡点、原“山脚休息台”位于坡上 +170 m 林缘的既定基线

源数据说明补充 labels 只维护名称，现有地图测试增加标签必须引用本剖面真实节点且不得重复手填高程的检查

## 其他显式高程说明

扫描本文件全部带 `+数值`、`数值米`、`数值m` 的字符串并核对对象，除上述三处标签外，现行高程说明与所指节点或平台相符：镜厅地面 +25／屋顶 +37／电梯净升 12 m；校园南门 +26／北侧公共上街 +41／净升 15 m；桥面 +10／桥下通道 +6／高差 4 m；渡口 +2／浮桥 +1.5；小店 +28／林缘 +170／摘星台 +450。其余数字是通行净宽、设施深度或学位容量，不按高程处理

社区公共球场的入口说明指 `school_upper_street` +41 m，场地 `fw-e-existing-64.elevation` 也是 +41 m，因此保留该说明。另发现地点 `64.position[2]` 为 +57.730475，与场地高程不一致；本轮禁止改坐标，已交回主 Agent 按地点锚点用途另行核实，不把未修改的问题算作通过

抽查地图正式名称对应 Null Station、月台杂货、镜厅、星见学园、西环渡口、白页书店、BYTE BEAT、FRAME、PLAYROOM、FLASHBACK、四季温室、Null Site 服务中心、MONO、AFTER 9 和 LATE BITE；未引入新命名。地图文件不包含河流名称，白沙河正式设定及其他剧情正文均未修改

## 检查与范围

- PASS：`bun run check:map`，55 项通过、0 失败，见 [map-tests.log](map-tests.log)
- PASS：最后收窄高程后缀匹配后重跑 `bun test tools/tests/district-map.test.ts --test-name-pattern 'section labels'`，见 [labels-final.log](labels-final.log)
- PASS：`bun run check:types` 包含 TypeScript 与 Vue 检查，见 [typecheck.log](typecheck.log)；初版 `assert.ok(label?.trim())` 未将 `string | undefined` 收窄为 `string`，改用显式 `typeof label==='string'` 断言后通过，未改地图内容
- EXPECTED FAIL：修改源数据前新增标签检查实际失败，见 [before.log](before.log)
- Wiki 完整构建由主 Agent 在共享修改合流后执行，本记录不声称构建已通过
- 本轮无运行时几何变化，不运行 GPU capture；没有生成截图、视频或临时可执行文件

此修复只保证剖面说明读取真实高程，不证明人物通行、NPC 导航或整城视觉品质

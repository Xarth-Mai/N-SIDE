# 命名与基础数据检查

## 已执行

- `bun run check:docs`：PASS；初次发现一条历史文档链接指向旧街区页，修正链接后通过，历史正文未改写
- `bun run check:narrative`：PASS，1 份运行数据、22 个信息状态；只证明结构检查
- `bun run tasks:check`：PASS，17 张任务卡
- `bun test tools/tests/wiki.test.ts`：PASS，13 项；含改名页三代旧 URL 兼容和地图显示字段不改变几何投影
- `bun run check:types`：PASS
- `cargo test --manifest-path game/Cargo.toml --lib world::map::tests::real_map_and_axis_contract -- --exact`：PASS，1 项，实际读取当前地图源；12 项不属于这次窄测
- 基线地图与新地图递归比较：PASS，22 个展示字段改变，对象键、ID、数值、几何与拓扑不变，见 `map-diff.json`

## 边界

浏览器枚举结果见 `browser.json`：无可用浏览器，人工页面、地图、搜索和时间轴操作 NOT RUN。未运行新的游戏交互或 GPU 验收；游戏代码和运行资产未修改，地图变更为名称显示字段，已检查真实解析和绘制投影契约

第一批最终检查：`check:docs` PASS（257 Markdown、86 ID、31 Skills），`check:story-design` PASS（37 人、18 项既有故事、91 场所），`check:types` PASS；双站完整构建 PASS（玩家 91 页／531 文件，开发 148 页／754 文件），构建日志见 `build.log`

第一批仍保留原八支线数据，第二批扩为十二条；旧正式人名与废案地名在玩家正文扫描零残留，原 35 人的 ID、年龄、住处及既有场所关联保持。提交前简化审查：Lean already. Ship.

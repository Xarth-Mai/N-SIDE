# r9 独立代码复核

由独立子 Agent `/root/character_art_next/hair_review` 只读审查选择性更新和内容比较脚本，未执行 Blender 或替代视觉验收

- `--update-hair` 在全量删除场景之前读取当前 master，只替换指定两个对象的网格数据，保存、导出后退出
- 将检查器的 `startswith(changed)` 修为精确 `obj.name in changed`，避免同前缀其它对象被豁免
- 最初建议删去恢复 Head 顶点组的 fallback；复读 `update-first.log` 中真实 KeyError 以及成功日志后撤销此发现，因为替换 `obj.data` 会清空该组
- 复核 `head_top` 统一偏移两块发面，并在半径公式中消除同一偏移，以沿用现有 master 的头部手工修改
- 精确比较覆盖未改源网格、UV、权重、骨 rest／层级、源动作曲线、四动作导出采样、绑定、材质和三图；不声称逐项比较所有 Blender UI 或场景属性

最终代码与复杂性审查结论：Lean already. Ship.

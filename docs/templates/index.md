# 模板

[委托](quest.md) · [日常片段](daily-scene.md) · [角色](character.md) · [敌人](enemy.md) · [地点](location.md) · [开发稿](case-development.md) · [试玩](narrative-playtest.md) · [连续性检查](continuity-review.md) · [开发任务](task.md)

结构数据使用 [narrative.json](narrative.json)，正式对白使用 [dialogue.csv](dialogue.csv)。占位符替换为实际内容，辅助文件随制作阶段建立

## 资产关系

[asset-manifest.json](asset-manifest.json) 放在内容目录，`owner_id` 关联内容，`assets` 定义本对象资产，`uses` 引用共享资产 ID

每项资产记录 `id`、`kind`、`state`、`source_path`、`runtime_path`、`provenance`、`license_status`。路径相对仓库根，待制作时为 `null`；来源记录创作者与使用许可

制作状态为 `planned / working / integrated / retired`，许可状态为 `unverified / cleared`。同类资产的尺寸、材质、导出或音频规格集中记在资产包的制作说明中

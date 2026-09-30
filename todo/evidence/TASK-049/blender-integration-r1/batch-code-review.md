# 本批最终只读代码审查

2026-10-01，审查当前 `game/src/world/scene.rs` 与 `game/src/bin/map_viewer.rs` 未提交差异；对照现有加载器、碰撞来源规则、地图、appearance、资产清单和本轮相关测试，采用 ponytail 瘦身审查并单独检查正确性边界

代码未发现新的阻断问题或有效瘦身项，结论为 `Lean already. Ship.`；未修改代码，未重跑 Cargo、139 项测试或 GPU，未审查或改动用户的 `AGENTS.md`、`docs/dev/validation/runtime.md`

## 核对结果

- 两个 Blender 附件沿既有 `Appearance.models → props → WorldAsset` 路径各实例化一次，没有新 loader、schema、并行资产接口或碰撞机制；实际 GLB hash 与各自冻结资产一致，必需文件缺失会被预检拒绝
- V-15 仅跳过南面覆盖范围相交的完整窗框、截短两条楼层带并撤除 `cinema_entry` 原雨棚；原门、门框、中梃、北后勤门、东餐饮门和屋顶开放通路保持；V-A08 只撤除外圈旧压顶
- 新 GLB 不进入碰撞，但旧通用构件此前属于结构碰撞；接入记录明确列出净减少 372 个三角，未误报为整个世界碰撞完全不变；原 V-15 门面仍关闭，未声称影院室内可进入
- V-W10 砖面仅为西面首层三片浅饰面，保留 2.2×2.55m 的门框开口，世界米制 UV 连续；`brick_music` 使用 1.05m 周期，派生 source 不命中结构碰撞
- 橱窗物品采用现有材料和原生低分辨率网格，限定在 V-04 原窗开口、框唇和承托空间内；视觉 source 单独排除碰撞，原建筑和窗框继续负责阻挡；没有为单店样板扩张配置或资产服务
- 三段楼梯扶手复用源路段宽度、高程与实际 Ground，末端留出横向通路和休息平台；新增树复用现有植被避让与山顶视线检查，并针对实际 GLB 根部取样；相关测试读取实际源几何及胶囊查询，不只是重复常量
- 两个新增镜头分别明确为外观自由检查与公共门前 1.7m 视点，没有把自由机位冒称可玩出生点；正式 scene 与 appearance 中已无 reflectance 实验字段或赋值
- V-15 的线性纯色、贴图直接覆盖 Base Color、CC0 素材与 Noto 字形来源已分开说明；V-A08 记录 appearance sRGB 转线性，CHR-001 发型仍标 `needs_revision`，未把 DCC 检查夸大为作者接受

独立子审 `v15_contract` 只读复核了 V-W10 开口、两附件锚点及泛型外饰删除范围，未发现阻断项；相应测试覆盖真实 GLB 包络、五段源路线胶囊净空和逐盒体删除范围，其结论同为 `Lean already. Ship.`

## 非阻断的文案同步项

初查时 `source-assets/environment-kit/asset-manifest.json:879,913` 的 Bricks057 `purpose` 仍写「尚未绑定场景」，README 公共砖段也仍采用首次素材导入阶段的措辞；最终窄复查确认两者均已同步 V-W10 西面首层局部饰面绑定及作者验收边界，此项已解决

此项是组合后状态说明滞后，不是来源、CC0 许可或原件 hash 失败；原始公共素材导入证据中限定“本子项未改场景”的历史记录仍有效

## 证据边界

Parent 已报告完整库测试 `139 PASS / 1 ignored` 和两个二进制构建 PASS，本审查没有重跑或代签；`git diff --check` 与报告落盘后的 `bun run check:docs` 为 PASS

未观看当前新 GPU 结果，不为书杯外观、栏杆连续观感、树冠遮挡、砖面旧化、V-A08 接缝、V-15 大面留白及占位海报、CHR-001 形象作视觉放行；这些继续由 Root 实际查看运行画面，作者验收仍独立保留

| 审查对象 | SHA-256 |
| --- | --- |
| `game/src/world/scene.rs` | `1ed864e2f6cd00a35e4226b9a17fc0815330c700cac2cbb3bba67ac651f66a0c` |
| `game/src/bin/map_viewer.rs` | `bce520ee128a22ce85be5cb3785d05767981c1b59107fa6af70c7f82af2ae24a` |
| `source-assets/district-scene/appearance.json` | `3a3e72b6cbcc2d8330f9ab8f9e7554eb4e80d3ff8d248b781fa7c966b2eea436` |
| `v-a08-roof-eaves.glb` | `f52eb7559f4eb7e65f274f6b03dfaed9efbe0472e9b67a4348da9e4734c9b952` |
| `v15-mirror-hall-facade.glb` | `07e23026c08d5967f85adc825ff518a3a4e34fac453565c71545a2d22dddd2b2` |
| 本轮读取的两文件 diff 快照 | `38a1e97b4d72cbd216f160e0790fbada651c8ac50267bd60442a8e5d73dd85bc` |

唯一临时文件 `/tmp/nside-batch-code-review.diff` 在审查与哈希记录完成后清理；本审查没有生成图片、视频或探测二进制，未触碰 Root 正在使用的 GPU 产物

## walk-preview 修复后最终窄复查

原 650 帧脚本 reset 后沿 `north=255` 向 −X 行走，穿过 HEAD 早已开放的小店公共门；原失败记录的 frame389 已在「接待与陈列」，frame419 被室内后墙阻挡，与既有公共房间测试吻合，属于脚本路径过时而非本批外墙碰撞回归

已只读核对 `game/capture/walk-preview.json` 差异：保留 `240..241` Reset，新增 `241..288` 的 47 帧 S 输入，将后续事件、关键帧、wait 和断言一致平移 47 帧，总帧数为 697；保持撞墙位移 `11.0..11.8m`、持续顶墙 `≤0.01m`、退离 `≥3m` 等所有原阈值，两条墙断言另加 `inside_room:false`，未关闭合法入口或放宽验收

实际读取 `output/blender-integration-r1/walk-fixed/run.json` 与 `state.json`，确认 697 帧、run 的 17 项检查全部 PASS；frame316 起点 `[100,28.045700,-249.986313]`，frame436 在 `[88.745995,28.020384,-249.976639]` 被 `buildings[V-04]/derived-facade` 阻挡且 room=null，frame466 坐标保持不动且仍在室外；三段实际位移分别为 `11.254038m`、`0m`、`3.199997m`

脚本 SHA-256 为 `708e6ad524aedaed2ed021fb796ed1ef429e60fd00543deae1ff1dc7708c7538`；scene.rs 与 map_viewer.rs 的哈希均与上表一致，几何代码自上轮审查未变。脚本契约窄测 PASS 由 Parent 提供，本次未重复运行测试或 GPU、未新增视觉验收声明；Bricks057 文案同步项已关闭，没有剩余有效发现，最终结论为 `Lean already. Ship.`

# V-04 浅橱窗实体陈列 r1

2026-10-01，输入基线为 `f75387c19f98ad58aadb31f017ba80f35907aa22`。本资产子项将 V-04 已有可见橱窗的平面货品转为窗框内的浅实体陈列，保持地图、外墙、入口、原窗框和横档、碰撞选择及正式白天光照

本页记录首版工程结果，实际 GPU 对照后视觉 self-audit 为 FAIL；当前商品轮廓修订与检查见 [merchandise-review.md](merchandise-review.md)，首版数量与 hash 仅作历史证据

## 实现与实际范围

`scene.rs` 复用 `add_box` 和原五种材质，保留原金属横档作为上层搁板；下层木板分为左右两半，外端接侧窗框、内端接中梃。六件盒装文具各带一张纸标签，货品分置于中梃两侧，底面落在实际搁板顶面。新增源标识为 `buildings[V-04]/window-display`，按材质合成五个视觉批次，现有碰撞白名单不会选入这个标识

实际只生成 **1 个可见窗格、14 个盒体、168 个三角、5 个新增 mesh/entity 批次**，新增纹理数量、材质、GLB 均为 0。立面理论格距不能代替 `window_exposed` 和门位避让的实际结果；本轮保留这一个既有窗格，不扩开外墙

从实际模型几何核对：V-04 外墙为 `x=88.000 m`，背景为 `88.055 m`；原窗框后／前缘为 `88.085 / 88.365 m`。下层木板位于 `88.135–88.315 m`，货品位于 `88.145–88.265 m`，标签背面贴货品正面、前缘为 `88.271 m`。下层最大货品顶部距原横档底仍有 `75 mm`，中梃两侧保留物品净空；板端与支架的相接使用 `0.1 mm` 数值容差

`shop-display.svg` 保留原 `640×448` 尺寸、青绿背景和细框，只删除将被实体取代的货品、两道搁板、中心图形及假反光。运行 PNG 由已有导出器生成，从 `9,657` 减至 `1,015 bytes`；实际查看最终 PNG，确认它是保留原色的安静背板，不将这张单独背景图视为场景视觉验收

## 检查与失败修复

| 检查 | 结果与证据 |
| --- | --- |
| 源 SVG 导出及一致性 | PASS，`bun tools/export-district-scene.ts` 与 `--check`，分别见 `export.log`、`export-check.log`；15 张既有图中只有 `shop-display.png` 字节变化 |
| 初次最窄 Cargo 测试 | FAIL，`native-test.log`；编译成功，第一条把可见窗数误推为 3 的断言失败，实际返回 1 |
| 修正后的真实几何窄测 | PASS，`native-test-final.log`；检查实际墙面／窗框深度、窗内范围、中梃净空、板端支撑、货品落点、横档无穿插、既有材质与三角预算，实际 1 passed |
| 碰撞几何对照 | PASS，同一窄测对 `facades()` 的结果分别保留／移除新增视觉批次后构造 `CollisionWorld`，两者均为 `499076` 三角、`877` 来源，内存大小和 bounds 相同；这是立面部分的对照，不是全世界总数 |
| 原输入与已有源代码保持 | PASS，`source-contract.json`；`scene.rs` 原有每行字节按原顺序保留，仅新增代码，`geometry.rs`、`collision.rs`、地图、appearance 与 daylight 均逐字节等于输入提交 |
| 格式与差异 | PASS，单文件 `rustfmt --edition 2024 game/src/world/scene.rs` 与 `git diff --check` |
| 文档与引用 | PASS，`bun run check:docs`，见 `docs-check.log`；含源包说明与本轮证据链接 |
| GPU 场景与动态视觉 | NOT RUN，由主线程使用已保存的真实基线统一复验 |

并行只读审查发现首版下层木板未接窗框或中梃，已经扩到两端真实接触；窄测从生成后的 `trim` 盒体检查每块板至少有两个实际端部支撑，没有靠数组位置推定。窗数断言改为规模上限 `1..=3`，仍逐一验证当前实际窗格和货品，没有减少深度或碰撞断言

```fish
bun tools/export-district-scene.ts
bun tools/export-district-scene.ts --check
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib shop_window_display_has_depth_support_and_no_collision_change -- --nocapture
```

源文件 SHA-256、固定基线和保持项见 [source-contract.json](source-contract.json)。独立 `ponytail-review` 复审结果为 `Lean already. Ship.`；复审同时确认悬空板已修复，标签与货品面相接，货品未穿中梃或原横档

## 运行交接与清理

固定小店相机 A/B 沿用 `todo/evidence/TASK-045/daylight-r10/shop-fixed.json` 和正式 A 光照，再用 `game/capture/walk-shop-exterior.json` 检查近距离接近、横移和返回。重点查看搁板与货品的真实视差、阴影、侧向轮廓、重复贴纸及入口遮挡；原底图变为背景后，主线程须确认正式场景确实出现实体货品，不能仅凭背景 PNG 或数值 PASS 接受观感

本子项未生成截图、录屏、临时探测可执行文件或 DCC 备份，正式 PNG 作为运行资产保留，导出器已自动清理其临时目录。主线程保留的 `output/shop-display-r1/baseline/` 不在本子项清理范围，本轮未提交

# 自然陡坡岩土分区 r3

基线 `e6b398b`；接续 r2 的真实诊断，使用原生 `StandardMaterial` 对真实坡面分区。TASK-045 尚在进行中，45° 是本轮视觉试值，未当作永久地质规则

## 制作与不变量

`game/src/world/geometry.rs::MeshData::split_ground` 在原自然地面裁切及平滑法线完成之后，按每个三角的几何坡度分成两个批次

- 不高于 45°：保留 `terrain`、现有苔土地面图对和 `/terrain` 源关联
- 高于 45°：使用 `terrain_rock` 与 `/terrain/rock` 派生源关联，平地、人工台地和道路保留既有材质
- 完整复制每个三角的位置、顺序、平滑法线与 UV；不重采样地形、不移动顶点、不修改地图或建筑标高
- 两种自然地面都使用 r1 的主轴米制 UV；不引入 shader、额外覆盖层或地表几何，分区仅多一个原生材质批次
- 岩面使用源图本色及 0.92—1.0 的中性大尺度明度变化，不按绝对高程着色，不继承草地绿色调
- 碰撞层既有规则包含 `/terrain/*`，岩面继续参与同一 `CollisionWorld`，水面排除规则保持

源码消费者核对显示，几何全图测试原先只读 `/terrain` 的高度范围与 UV，需要同时覆盖新岩面批次，现已更新；app、route、observation 中其余 `/terrain` 用法是独立测试夹具，无需更改

## 岩土源资产

选择 ambientCG 的 [Rock043L](https://ambientcg.com/view?id=Rock043L)，由 Lennart Demes 的 ambientCG 发布，官方标为 photogrammetry、约 1.8×1.8m，下载资产按 [CC0](https://docs.ambientcg.com/license/) 发布

已实际查看选中 Color/NormalGL 原图，颜色为灰褐、裂隙和块面不规则，没有规则砖行或整幅平行层理。白色地衣斑仍可能在大面积铺设时重复，因此原图审查不代替运行验收

唯一源已移入 `source-assets/environment-kit/materials/Rock043L_1K-JPG_{Color,NormalGL}.jpg`，均为 1024² RGB，原始成员保持不改。`source-review.json` 保存公开下载 URL、许可、物理尺寸、ZIP/成员 hash、官方 SHA-1 与 CRC 检查；共享清单、运行导出与 appearance 接入由主 Agent 维护

## 检查

- `terrain_material_split_preserves_triangles_and_ignores_height` 检查 0°/40°/55° 与低处/500m 高处的分区、原三角属性逐值保持，以及新岩面源关联的真实碰撞射线覆盖
- 全地图几何测试继续检查有限坐标、源高程、完整可见高度范围、两种自然地面的 UV 面积下界和对应坡度
- 已运行 `bun todo/evidence/TASK-045/terrain-surface-r3/slope-probe.ts`，结果见 `slope-probe.json`：未扣道路/台地、含边界裙边的源地形共 20,386 三角，1,335 个超过 45°，约占 15.18% 源表面积；这不是最终裁切渲染网格的覆盖率
- 已运行 `git diff --check`，无空白问题
- 主 Agent 的首次库测试发现一个真实精度问题：裁切细长三角以 f32 做 cross 时进入草批次，而同一存储顶点的 f64 测量得到 `up=0.6871664657`，超过 45°；分类现改为先升 `DVec3` 再计算坡度，原 f32 顶点和阈值保持
- 首次修复后的链接尝试遇到中断遗留增量缓存的 `undefined hidden symbol`，主 Agent 定点清理本包测试缓存后复跑，未修改依赖或降低测试条件
- 主 Agent 实际执行库测试，`output/checks/art-r6/lib-final.log` 记录 126 PASS、0 FAIL、1 ignored，包含新分区/碰撞检查与全地图坡度、UV、高程检查
- 本子任务未独立运行 Cargo/GPU，主 Agent 串行运行 `output/capture/task045-cut-after-r3`：1280×720、30fps、90 帧，6 项检查全部通过，资源就绪、帧落盘、Transform 有限、机位保持及往返转头均满足脚本断言

## 实际画面自查

本子任务实际查看 `task045-cut-before-r3/keyframes/frame00044.png`，以及 `task045-cut-after-r3` 的 29、44、59 关键帧与 40—44 连续帧。它们是同一真实 `eye-descent-cut` 机位的脚本转头，不是重新制作的演示地形；视频编码通过，但本子任务使用逐帧查看，没有将其记录为完整视频或真实玩家体验验收

- 左侧陡坡由强方向性的绿色长条变为灰岩细节，材质与陡面职责更协调，是本轮实际改善；岩面仍有掠视方向性，但没有将其直接等同于退化 UV
- 40—44 的已看转头范围内未见材质突变、边界跳动或新孔洞；此结论只覆盖这些帧，完整几何保持及碰撞覆盖由上述 CPU 测试另行验证
- 大型平整楔面、近直线山脊和三角形草岩硬边仍然醒目，底部还有狭长绿色三角片；草岩分区忠实显示了现有面片，尚未形成自然坡体或可信的人工切坡结构
- 前景缓坡草面仍可见规则重复，岩土材质替换没有解决所有地面铺排；本轮结果不足以验收自然山体和整体材质品质

后续应一起处理山坡轮廓与转折、可解释的人工切坡边界、林缘和灌木体积，以及材质过渡。只改贴图周期、对比度或继续增加采样，不能消除这些结构问题；本批保持原地形和道路，未为画面隐藏问题而移动顶点

`runtime-review.json` 保存实际命令、二进制和脚本 hash、6 项机器结果、已看帧 hash 与上述自查范围。检查通过与画面自查分开记录；作者品质放行仍为 NOT RUN，TASK-045 保持进行中

## 产物清理

已确认本子任务的下载 ZIP 与临时图片在 CRC、hash 和原图审查后清理，唯一源 JPG 和公开来源文字保留。此次 capture 由主 Agent 统一产生和清理；本子任务已完成查看，可清理 PNG/视频，保留 `run.json`、`state.json`、脚本、日志和本记录

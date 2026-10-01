# 楼梯表面局部诊断

2026-10-01，范围为小店主梯 roads `58/60/62` 与镜厅后方上山长阶 roads `744..766`；本轮没有修改 scene、appearance、地图或核心几何，仅给 map_viewer 增加两个自由诊断机位

## CPU 结论

实际调用当前已构建的 Rust `geometry::generate()`，得到 `2466 parts / 106763 triangles`，再读取真实 Mesh 顶点与法线；15 段 steps 的 5490 个内点垂直射线全部先命中对应道路踏面，踏面上方的 `/terrain` 命中为 0，踏面水平/朝上、structure 垂直、叉积与法线一致性错误均为 0

| 源路段 | 级数 | 每级深度 | 每级高度 | 踏面三角数 | walk-fixed239 眼位朝向可见的踏面三角 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 58 | 12 | 0.914352m | 0.168460m | 24 | 24 |
| 60 | 26 | 0.380638m | 0.165221m | 52 | 20 |
| 62 | 32 | 0.504834m | 0.166222m | 64 | 0 |
| 744..766 偶数段，各段 | 45 | 0.296934m | 0.167836m | 90 | 0 |

`walk-fixed/state.json` 的 frame239 相机为世界 `[106.427109,31.657171,-268.974335]`；第三梯第一踏面已高于眼位，水平面的朝眼点积全部为负，从这里无法看到其水平踏面。前两梯朝眼踏面中心再经真实结构网格射线查询，分别有 24、20 个未被遮挡；第三梯剩余可见面为踢面或侧面，不能把连续深色外观直接解释为坡面盖板

当前 `eye-shop-uphill` 眼高为 `29.7m`，第二梯最低踏面约 `30.129m`、第三梯最低踏面约 `34.425m`，两者全部高于这个眼位。第二、三梯又比第一梯更密，远看时更容易只留下连续踢面轮廓

按现有 `sun_position=[650,780,-380]` 和水平/垂直几何法线计算，三段主梯下坡踢面 `N·L≈−0.534`，长阶为 `−0.525`，水平踏面为 `+0.719`；下坡踢面没有几何意义上的太阳直射。这是已验证的光照条件，未把它当作最终 GPU 黑像素归因；法线贴图、环境光、阴影与投影分辨率仍须由固定机位对照观察

## 已排除的代码机制

- `geometry.rs` 按完整道路 ribbon 切除地形，再将同一个 ribbon 分成恒高踏面；没有保留横贯踏步的连续地形坡面
- `MeshData::polygon` 在源 XY 中修正为 CCW，再由世界坐标叉积生成法线；实际生成 Mesh 的所有受检踏面均为 +Y，没有反向或倾斜顶面
- `paving_walls` 仅处理两侧接地边，跨宽踢面单独生成，受检 structure 三角法线的 Y 分量均为 0，没有斜坡盖板
- 长阶采样处未裁切 Ground 高程与踏面的差值为 `−0.092135..+0.042135m`，所有采样仍实际命中踏面；长阶依附地形且已有侧面，不支持凭远景细条错觉增加桥墩
- 子审基于源多边形的相交计算没有发现 58/60 或长阶被其他道路/平台/建筑面积覆盖；62 仅在末端与 road298 有 `0.065255m²` 同高连接，独立于本轮 Rust Mesh 取样

本轮未找到需要改核心几何的证据；保留地图高程、通路、踏步、碰撞和材质，先用近距离抬高侧视分离“真实踏面存在”与“低视点踢面观感”

## 固定机位对照

三个脚本均为 1280×720、30fps、60 帧、frame59 关键帧、无输入，使用相同 daylight 和 AA；先用既有默认 `msaa4`，不要同时改变曝光或太阳

| 脚本 | 机位 | 源坐标 eye → target | 目的 |
| --- | --- | --- | --- |
| `shop-baseline.json` | 既有 `eye-shop-uphill` | 原有机位 | 复现低视点连续深色面 |
| `shop-surfaces.json` | 新增 `inspect-shop-stair-surfaces` | `[103,279,40] → [114,295,34.8]` | 抬高侧看第二、三梯踏面与接地边 |
| `hill-surfaces.json` | 新增 `inspect-hill-stair-surfaces` | `[282,414,89] → [301,427,87]` | 近看长阶首两段、平台和侧墙 |

由 Root 统一构建和运行，命令为 `game/target/debug/map-viewer --project-root . --capture todo/evidence/TASK-049/stair-surface-r1/<script>.json --output output/stair-surface-r1/<name> --aa msaa4`；若原始复现采用 `taa-ssao`，才用同机位 `taa` 作单一 AO 对照，现阶段无需增加这一变量

## 证据和限制

CPU 复现命令为 `rustc --edition=2024 -O todo/evidence/TASK-049/stair-surface-r1/probe.rs --extern n_side=game/target/debug/deps/libn_side-bd339b1f3f0d39e4.rlib -L dependency=game/target/debug/deps -o /tmp/nside-stair-surface-probe`，然后从仓库根目录运行该程序；使用 `rustc 1.98.1 (48a229cea 2026-09-01)`，编译和执行均 exit 0，没有运行 Cargo、测试套件或 GPU

每级沿深度取 `0.1/0.5/0.9`，横向取宽度的 `−0.45/0/+0.45`，采样避开恰好重合的边线；法线检查覆盖全部目标三角。射线使用 `geometry::generate` 的全部结构网格，没有包含 scene 后添的栏杆、树木、GLB 或法线贴图，不证明最终着色、装饰遮挡或作者接受

输入源与既有 rlib 的哈希在 `input-sha256.txt`，完整原始输出在 `probe.log`；`probe.rs` SHA-256 为 `bba784a1d2413eb9247e0eac2ddeff2cc9784b530876c31a896ab622433f07af`，`probe.log` 为 `faf17304c7970bde0aa8ce1e112508e65b8676e47a58d2eb7e0821d00971590f`；临时探测二进制哈希为 `19e65c2887edd87473b0542f37d2bf20507ce110122654942b118f69f0a3102d`

此前 walk-fixed239/uphill PNG 已按项目规则清理，本诊断本阶段只读到了原始 state 与脚本，尚未实际观看新的 GPU 对照；视觉结论为 NOT RUN，不能代签为“仅角度/阴影”。临时探测二进制已在结果与哈希记录完成后删除，保留复现源、脚本、日志和参数，没有生成或删除 Root 的图片/录屏

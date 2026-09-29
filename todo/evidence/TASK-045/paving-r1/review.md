# 小店铺装内部侧片修复

## 范围与输入

本轮只修改 [geometry.rs](../../../../game/src/world/geometry.rs) 的道路／地面平台交接生成与对应回归，不改地图源、稳定 ID、道路宽度、节点高程、台阶计算、平台顶面或碰撞装配，店面陈列由并行工作另记。第二次修复删除平台内重复覆盖的道路顶面，该区域碰撞支撑由派生道路的 28.025 m 回到既有权威平台的 28 m，平台外道路高度保持原值

地图源 `source-assets/district-map/district.json` SHA-256 为 `a4ab861f756037206520258725a32232de40a96dc1a8da380d26a1de33c9e45a`，第一次修复 `geometry.rs` SHA-256 为 `ad78712ad7b571984dfe97ecf6a606ead6f938d157f9380f79cfcb28a4561c06`，第二次初版为 `4db4b7e1d04d999263f7860817d4263a088fdcea3f313fb7848129775e26d20d`，补齐公园语义后的当前版本为 `e1a38a20cd89e164432946de20ff66de573ae45eeed1a05fc14cb2a8eb2dc1b5`

## 修复前：FAIL

实际查看同一正式路线 `output/capture/task045-shop-r3-before/keyframes/frame00199.png` 与 `frame00330.png`，门前窄通道及向街道延伸的宽通道均存在细长暗线，两机位仍沿相同铺装边界出现，属于 self-audit，不是作者审美验收

两帧 SHA-256 分别为 `744cd1c82badb8bd4639d7637d1dcf21136c9876ccd42f9dae70928f8620f1c1` 与 `9490a0f8359502d9a569c63ef0d90cb3aaa1dffc687cd8452fc2a747b3bf9cd5`

初次检查发现 `/roads/135 (home -> shop_entry)/structure`、`/roads/190 (shop_entry -> shop_front_door)/structure` 及 `/surfaces/0 (shop-site)/structure` 的交叉边仍连接被裁掉的旧地形：平台顶面为 28 m，道路铺装顶面为 28.025 m。后续同机位复验说明这一真实几何问题没有完整解释画面中的黑细线，不能据 CPU 通过宣布视觉修复成立

用现有 `tools/district-map.ts` 的 `buildGround` 对同一地图控制点探测，窄路北缘 `[89,256]` 的旧地形为 28.144394 m，宽路北缘 `[92,258]` 为 28.433182 m，分别会在铺装上方留下约 0.119 m 与 0.408 m 的竖向侧片；平台东缘 `x=94` 与道路交叉处也受同一问题影响。[探测原值](ground-probe.json)保留 Delaunator 工具路径与数值，Rust 使用 Spade，其实际网格由新增回归验证，不将此探测当作 Rust 运行结果

## 第一次修改与 CPU 检查

复用既有道路相邻面裁切，将其命名为 `paving_walls` 并用于非悬空平台；道路侧面纳入地面平台实际高度，平台侧面纳入道路实际高度，只在邻面不存在处保留与原地形之间的挡墙。平台与道路沿用各自原高程，原有 25 mm 铺装边缘保留，不通过材质、阴影或相机掩盖侧片

`ribbon_height` 对常高面直接返回高程，使五边形 `shop-site` 和其他常高平台沿用同一裁切流程；斜坡插值、道路顶面、台阶踏面及碰撞源码保持原实现

新增 `shop_paving_walls_do_not_retain_earth_removed_by_neighboring_paving`，从真实地图生成网格，检查两条道路在平台内的侧面仅连接 28–28.025 m 成品面、平台穿路边不再高出路面、两条道路顶面仍为 28.025 m，且平台外露高差仍存在

以下 PASS 仅对应第一次修复，第二次源码已改变，当前验收见后文

| 项目 | 结果 | 范围 |
| --- | --- | --- |
| 地图地形探测 | PASS | 现有 Bun 工具直接读取实际地图，保留 JSON 原值 |
| `rustfmt --edition 2024 game/src/world/geometry.rs` | PASS | 仅格式化本轮负责文件 |
| `git diff --check -- game/src/world/geometry.rs` | PASS | 本文件差异无空白错误 |
| `bun run check:docs` | FAIL | 并行人物 README 引用的 `todo/evidence/TASK-047/model-r2/review.md` 尚未生成，未报铺装文件错误，收敛后由主 Agent 复跑 |
| 新增 Rust 网格回归 | PASS | 主 Agent 全库运行日志明确记录该测试通过，覆盖真实 Rust 网格 |
| 既有几何、碰撞与工程测试 | PASS | [全库日志](../integration-r3/lib-tests.log)：117 passed、0 failed、1 ignored，21.03 s；ignored 为供隔离进程调用的 `settings::tests::restart_worker`，其宿主恢复测试通过 |
| 第一次修复后正式人物路线、连续帧与状态 | 机器 PASS／视觉 FAIL | 主 Agent 运行同脚本 `game/capture/walk-shop-exterior.json` 获得 9 项机器 PASS，但实际查看帧 199、330 后黑细线仍存在 |
| 作者审美反馈与 G2 放行 | NOT RUN | 技术修复不改变任务验收计数 |

## CPU 后只读复核

常高多边形在 `ribbon_height` 的 `start == end` 分支直接返回高程，不读取四顶点切片，因此五边形 `shop-site` 及更多边的常高平台保持完整轮廓参与裁切；当前 76 个非悬空平台均为正向轮廓，不含建筑归属平台

`road_masks` 排除 `bridge`、`deck`、`lift`、`interior` 和建筑归属道路，平台不会选择这些悬空或室内面作为土体邻面；`ground_surfaces` 排除 `elevated`，桥面与架空道路也不追加地面平台邻面。当前地图没有以其他道路类型绑定架空平台的旁路，最近面选择仍在真实平面重叠区间内比较高程差，不改变桥栏、承重或平台支柱路径

台阶自身的每级踏面、端部立面和侧面 `steps` 分支保持原实现，全库日志中的切填面朝向、交叉地形切分、相邻道路高差与升降两向台阶回归均通过。小店 `shop-site` 不与 `steps` 道路相交，本轮修复不存在由台阶邻面造成的小店高程变化

另有既有精度限制：供相邻面选择的 `road_masks` 以整段台阶斜坡表示邻面，未拆成每级踏面，新的平台调用也沿用此近似。只读检查当前地图平台边与台阶的实际区间，仅发现 `stairs-rest` 和道路 64 的 0.124 m、0.021 m 两段重叠，其中点邻面与真实踏面高度分别差约 2.77 cm、0.33 cm；这不改变踏面或台阶数量，也未由本轮 GPU 路线验证，不将全库 PASS 表述为所有台阶交接处的视觉验收

## 第一次 GPU 复验：FAIL

实际重新查看 `task045-shop-r3-before/frame00199`、`task045-shop-r3-after/frame00199` 与 after 的 `frame00330`，长短两条黑细线在相同位置仍清楚可见，9 项机器 PASS 不能覆盖这一视觉失败。after 两张关键帧 SHA-256 分别为 `172a271e48b5c1258c8837c6747d6d9193afec57b07a049699b23d7e25f3e8ff` 与 `b5501fae10a1ca555ba0f6b90eece64ec79e90ffb44e4524de6785a7c8d2f602`

从 after 状态 JSON 的帧 199 读取真实相机位置、四元数与 55° FOV，用逆相机变换和透视投影反查线段：[投影原值](projection-probe.json)显示道路 135 南缘 `[90,252,28.025]` 投到 `(931.25,392.16)`，道路 190 南缘 `[88,254,28.025]` 投到 `(1016.83,353.86)`，与画面长线和短线起点吻合。剩余可见线对应道路南缘在平台之上额外生成的 25 mm 顶面凸边，第一次重点处理的北缘旧地形侧片只是另一个问题

## 第二次修复与待复验

同高平直道路进入已铺装平台时，由平台提供唯一顶面：在道路顶面掩膜和道路侧边裁切中加入同地图高程铺装平台，删除平台内部重复覆盖层与凸边，保留平台外道路、真实边界高差和台阶。限定非悬空、非 `steps` 道路且路段两端均与平台高程一致，平台沿用顶面材质判据 `kind != "park"`，不把斜坡或台阶压平

同一回归加强为平台内不得存在这两条道路的重复顶面或侧面，保留平台外道路 28.025 m、平台外露挡墙和交叉边高度检查，另用真实 `CollisionWorld` 在 `[89,254.2]`、`[92,252.2]`、`[93,256]` 三点检查 `shop-site` 按 28 m 提供连续支撑。原 CPU 断言未因画面失败而放宽，新增条件直接覆盖此次发现的重复铺装原因

独立审查发现第二次初版把 `kind=park` 的草地也当作铺装覆盖面，会裁掉地面公园 `surfaces[12]` 中道路 691、707–710 的花园步道；该版本未放行。修正后公园仍参与旧地形侧壁裁切，但不能删除道路铺装。新增 `park_ground_keeps_its_authored_paved_garden_paths`，用真实地图中上述五条道路检查铺装实体保留，以及各路段中点的碰撞支撑仍来自对应道路、保持 8.025 m

| 第二次修复检查 | 结果 | 范围 |
| --- | --- | --- |
| `rustfmt --edition 2024 game/src/world/geometry.rs` 与差异检查 | PASS | 仅本轮负责文件 |
| 加强后的网格／支撑、公园道路保留回归及工程测试 | NOT RUN | 等待主 Agent 统一 Cargo 运行 |
| 同机位正式人物路线与连续画面 | NOT RUN | 等待主 Agent 重新构建并捕获，保留第一次 GPU FAIL |

本子任务未生成截图、原始帧、录屏或探测可执行文件，已查看的基线画面归主 Agent 本轮 capture 所有，待完成统一前后对照记录后按 `AGENTS.md` 清理；本目录仅保留文字与 JSON

## 最终实机与回归

Root 全库最终118 PASS／0 FAIL／1 ignored，包含本轮重复铺装及公园道路保护回归，见[最终CPU日志](../integration-r3/lib-tests-final.log)。正式入口沿同一 `walk-shop-exterior.json` 完成450帧、9项检查PASS，root实际查看199、330帧及邻近帧，对照首次失败图确认门前两条25mm凸边黑线消失，平台外沿、陈列支腿及门口保持；机器报告与hash见[集成摘要](../integration-r3/runtime-summary.json)。这只关闭当前门前重叠铺装问题，不宣称全城所有高差接缝或最终美术通过

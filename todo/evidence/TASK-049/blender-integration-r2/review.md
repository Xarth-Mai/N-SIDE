# Blender 建筑与公共资产接入 r2

基线 `0efb89c61b432129cdbd292c05a3e1f90bb0b3bd`，本轮为 TASK-049 第 12 轮：用本机 Blender 完成 V-A08 整片住宅立面、镜厅南／西完整立面，装配并接入一件公共 CC0 长椅，同时把既有安可画像排成镜厅竖版海报。模型、正式游戏碰撞与实际画面已接入，保留地图、建筑和场所稳定 ID；本轮技术交付不等于作者审美或 G2 品质放行

## 本轮成果

| 成果 | 交付与来源 | 实际边界 |
| --- | --- | --- |
| V-A08 住宅 | [源文件与检查](../va08-facade-r1/README.md)：6060 三角、6 材质、27 组窗、四面覆层、雨棚、空调及落水管，`.blend` 可编辑并可重复导出 | 接管本栋旧泛型立面，保留原主体、三门节点与独立屋檐，没有增加可进入室内 |
| V-15 镜厅 | [模型自查](../v15-building-r2/self-audit.md)：12362 三角、10 材质，补完南／西立面并保留 r1 门头和东平台支承 | 北／东其他泛型立面仍在，不把局部完成写成整栋精修完成 |
| 公共长椅 | [来源与装配](../public-street-r2/review.md)：Poly Haven Modular Street Seating，Stuart Attenborrow、CC0，Blender 从原件装配，8906 三角 | 一件放在镜厅屋顶 `[312,237,37]`；保留原件、9 张 PBR 图、官方下载校验与来源，不新增坐下玩法 |
| 安可竖版海报 | [排版自查](../../TASK-048/cinema-poster-r1/review.md)：1024×1620，复用 AST-007 的 r2 画像及原字体许可 | 只替换镜厅第一块海报画芯并删除其四层旧抽象图形，其他海报与原横版保持；人物外观仍待作者反馈 |
| 静态碰撞接入 | [代码评审](code-review.md)：三件新模型的 27328 个实际静态三角按可视变换进入 `CollisionWorld::from_scene` | 只导入明确选定的住宅立面、镜厅立面与长椅；树、草、独立屋檐不顺带增加碰撞 |

视觉、碰撞都复用真实 GLB 和场景数据；没有另做演示副本。三模型的加载、变换、索引、有限值与三角退化诊断及新旧立面替换由实际解析和窄测覆盖，最终源码、二进制与资产 hash 见 [runtime-inputs.json](runtime-inputs.json)

## 实际验证与失败修复

| 检查 | 实际结果 | 证据范围 |
| --- | --- | --- |
| 完整库测试首跑 | FAIL：137 PASS、3 FAIL、1 ignored | [lib-tests.log](lib-tests.log)，三失败分别为共享来源后的实例匹配、被 Blender 接管住宅的玻璃期望及窗饰计数，不隐藏原始失败 |
| 修复后场景测试 | PASS：29 PASS、0 FAIL | [scene-tests-final.log](scene-tests-final.log)，实例按来源及 model 配对、移除失效住宅白名单项，保留其余九栋及实际 GLB 检查；没有再次运行完整库套件，不将其改写为全库通过 |
| 最终两个运行入口构建 | PASS | [build-final.log](build-final.log)，`n-side` 与 `map_viewer`，Bevy 及依赖版本保持 |
| Viewer 测试 | PASS：3 PASS、0 FAIL | [viewer-tests.log](viewer-tests.log)，实际镜头/地图关系检查 |
| 文档／Skills／资源导出 | PASS | [docs-check.log](docs-check.log)、[代码评审文档检查](code-review-docs-check.log)及[导出核对](code-review-export-check.log)，各资产自身另有来源及重复导出证据 |
| Wiki 初次检查 | FAIL 后复验 | [wiki-build.log](wiki-build.log)记录地形计时超限；空闲后 [wiki-build-isolated.log](wiki-build-isolated.log)地图检查通过，但系统 Node 缺 `libsimdjson.so.33`，不能视为项目构建通过 |
| 强制 Bun 承担 VitePress runtime | FAIL | [wiki-bun-runtime-probe.log](wiki-bun-runtime-probe.log)，esbuild service stopped；没有因此替换依赖或放宽检查 |
| Wiki 使用已有 bundled runtime | PASS | [wiki-build-bundled-runtime.log](wiki-build-bundled-runtime.log)，58 项地图测试、player 128 页／679 文件、dev 223 页／1046 文件均通过；Bun 仍是项目脚本与依赖入口 |
| 独立正确性与简化审查 | PASS | [code-review.md](code-review.md)，实际问题已修正，最终 `Lean already. Ship.`；评审不是运行或审美的替代 |

系统 Node 的动态库缺失是当前本机环境限制，bundled runtime 通过证明这份源码可以构建 Wiki；没有把本机系统依赖问题记作已经修复，开发站的大 chunk 提示仍保留在日志中

## 真实运行与实际看图

`output/blender-integration-r2/` 下 9 组 Viewer 成功运行共 630 帧、38 项机器检查：V-A08 东侧及东南两机位、镜厅总景重跑、海报近景、新旧立面接缝、屋顶长椅，以及三组台阶诊断。镜厅总景包含真实相机前进、平移与停止；其余固定机位不证明玩家移动。机器 `run.json` 的 `visual_review` 字段保持原值，实际观察另见 [independent-visual-review.md](independent-visual-review.md)

首次 `va08-east` 在受限环境未找到 GPU、exit 101，原失败保留；同参数宿主 `va08-east-host` 成功。首次 `cinema-facade` 中途停止、没有 `run.json`，frame113 损坏不可读取，Root 实看 frame112 后保留中断记录；该次不计 PASS，由新目录 `cinema-facade-retry` 完成整段重跑。失败和成功来源分开记录，没有把部分图片当成完整运行通过

独立看图确认窗深、空调与管线、镜厅入口和转角窗带、安可人像与文字、长椅装配及可见接地关系；[V-A08 地形复核](../va08-facade-r1/ground-review.md)另用真实 `Ground` 与导出 terrain 网格检查 405 点，最低窗下沿离地 0.90m，低眼位遮挡来自约 4.8m 外的前景坡，不能凭截图误判窗户埋土

正式游戏 `walk` 完成 697 帧、17 项机器检查 PASS，使用本轮最终 `n-side` 二进制；Root 实际查看 239／436／466／496／696 帧，记录人物上阶梯、贴墙保持阻挡、能离开墙面并返回标题。远处阶梯仍呈灰带，与 Viewer 观察一致；人物仍是未完成最终贴图的灰色造型模型。这是正式人物路径回归，不是玩家实际绕镜厅长椅、检验 V-A08 全部入口或整城碰撞的证明

## 未完成与下一批

- 远处阶梯仍会读成连续深灰带，近处则能辨认踏面；[台阶几何诊断](../stair-surface-r1/README.md)和本轮实际图片是定位证据，本轮没有修好该视觉问题，也没有据此把贴坡楼梯改成桥
- 镜厅前场和屋顶座椅周围仍是大片空草色空间，下一批优先做公共铺装、休息空间与邻接街景构图；住宅、镜厅的蓝灰窗面及实墙层次仍待制作
- README 预览图整体品质、作者人物造型、最终美术和真人手感均未验收；TASK-049 继续 active，TASK-048 继续 review，不因技术检查通过改为 done

## 复现与清理

仓库根目录可使用以下 fish 命令核对已冻结源资产及工程；完整运行脚本和 binary hash 记录在本轮 capture／输入证据中

```fish
python3 -B source-assets/buildings/V-A08/facade-check.py
python3 -B source-assets/buildings/V-15/verify.py
python3 -B todo/evidence/TASK-049/public-street-r2/check.py
bun source-assets/star-posters/export.mjs --check
bun tools/export-environment.ts --check
cargo test --manifest-path game/Cargo.toml --locked --lib world::scene::tests
cargo test --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bin n-side --bin map_viewer
fish -c 'set -px PATH /home/lzzz/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin; bun run docs:build'
```

最后一行对应本次成功的 `env PATH="/home/lzzz/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin:$PATH" bun run docs:build` 的 fish 形式，runtime 路径来自本机已有安装；其他机器使用其可运行的 Node 作为 VitePress 子进程宿主即可，不要求安装相同绝对路径

源资产生产阶段的临时图已有各自清理记录；本次 Bevy PNG、关键帧完成实际查看后，Root 在宿主以 `pgrep` 核对无 capture 进程，清理 `output/blender-integration-r2` 的 1482 个媒体文件，共 1,614,083,871 bytes，包含失败和中断产物。原运行状态、脚本和日志保存在 [captures.json](captures.json)及 `captures/`，完整媒体 hash 和清理结果见 [cleanup.json](cleanup.json)；正式 `.blend`、运行 GLB／PNG、源参考、复现脚本、版本参数及文字结论保留

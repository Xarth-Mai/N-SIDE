# Blender 建筑与公共材质 · 实际运行

2026-10-01，在 `main@f75387c` 上构建本批未提交代码与资产，输入 SHA-256 见 [runtime-inputs.json](runtime-inputs.json)，机器结果及已查看帧 hash 见 [runtime-summary.json](runtime-summary.json)。本页是实际画面的 self-audit，未代替作者接受或陌生玩家试玩

## 已执行

- `cargo test --manifest-path game/Cargo.toml --locked --lib -- --nocapture`：139 PASS、1 ignored，见 [lib-tests.log](lib-tests.log)
- `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bin n-side --bin map_viewer`：PASS，见 [build.log](build.log)
- `cargo test --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer`：3 PASS，见 [viewer-tests.log](viewer-tests.log)
- Viewer 五次真实运行：镜厅正面 150 帧、入口 60 帧、平台承托 60 帧、小店总景 60 帧、音乐街砖面 150 帧，均 PASS；正面及砖面包含真实 W/A/D 移动与释放检查，固定机位检查静止状态；每次保存脚本、run、state 和运行日志
- 正式游戏修订路线 697 帧 PASS，保留标题加载、楼梯抬脚、键鼠／脚本手柄切换、实墙阻挡、退出清理及真实 CHR-001 加载检查；另执行 240 帧 Idle 绕摄 PASS。输入通过既有设备事件进入系统，没有直接写入位置或成功结果
- 格式、文档与 Skills、任务卡及资产导出检查由本批对应日志承接；未测试 Windows、实体手柄及受控 GPU 性能前后差值

## 实际观察

镜厅正面查看第 59、149 帧：字牌、出挑深檐和四个海报框可辨，原南面窗口没有穿出新饰面。当前只有一半南墙具备特定建筑语言，其余两面仍是重复方窗的青灰盒体，大片白墙与抽象海报不代表参考图的完成度

入口第 59 帧采用真实门前节点加 1.7m 眼高：原双门与新厚门框、木檐底可见，没有看到门前新增横挡物。画面只证明外观和这处视点，原墙仍关闭，未新增影院内部进入能力

东侧承托第 59 帧：新斜撑连接贴墙板与既有平台板底，未见悬离；保留的旧柱和新梁共同承托。此低机位不能证明整段屋顶行走体验，五段路径胶囊净空由独立 CPU 检查说明

小店第 59 帧：书册与杯器形成不同轮廓，第一段主梯栏杆可见，上巷街树及其投影进入草坡。V-A08 新檐口在总景里占比很小，不能据此宣称完成檐下接缝全方向验收；近树也不在这一固定构图中，需由正式游戏转身画面补充。宽铺地、裸坡和连续盒体仍是下一批主要缺口

音乐街第 59、119、149 帧：V-W10 首层西面出现与上层灰泥有区别的砖行，门、窗和雨棚未被饰面吞没，横移后未见接缝跳位或明显闪烁。当前观察距离足以检查墙面分工，砖缝近景和不同侧光下的旧化强度仍未验收；没有把全城外墙统一替换成旧砖

正式路线查看第 316、436、466、496、646 帧：角色在门旁实体外墙停止，持续顶墙仍停在外侧，松开后可退离。紧贴橱窗时既有植物与窗框遮挡角色，未将其评价为已解决近墙镜头；步行腿部仍显僵硬，人物保持灰模候选。绕摄第 139、159、204 帧证实镜头旋转与实际场景关系，远树、上坡扶手、街景铺装可辨；近树位于镜头边缘，需要单独转向检查

## 失败、定位与修复

首次使用原 `game/capture/walk-preview.json`，650 帧运行中的两项外墙断言 FAIL；保留 [原运行](walk-before-fix/run.json)。该脚本从 `[100,28.046,-255]` 向西，进入已经开放的 V-04 公共门，389 帧 x=87.224 尚未阻挡，419 帧 x=84.32073 到达真实室内后墙。既有 `f7357e6` 已开放这扇门，CPU 外墙测试也已改为先向南侧移 5m，旧 capture 却未同步

本批没有改 `geometry.rs`、`player.rs` 或地图；V-04 新窗物件也不进入结构碰撞。独立只读核对确认不是本批撤除了门外阻挡，不通过关闭入口或放宽阈值修复。脚本在 Reset 后加 S 输入 `241..288`，向南约 5.0133m，与既有 CPU 外墙路线一致；后续事件和检查整体顺延 47 帧，原 11.0–11.8m 接近、≤0.01m 持续顶墙和 ≥3m 退离阈值保持，并补充 `inside_room:false`

脚本契约窄测 PASS，真实 [walk-fixed](walk-fixed/run.json) 697 帧复验 PASS；机器状态与实际外墙画面一致。首轮近树定向绕摄误用了转动符号，120 帧机器检查 PASS 只证明相机旋转，实际第 119 帧面向小店而非目标树，因此没有把它当作近树验收；保留其原脚本、状态与画面 hash，后续用正确方向重新运行

正确方向的 [shop-court-facing-tree](shop-court-facing-tree/run.json) 120 帧、6 项检查 PASS，第 119 帧实际可见近树完整树冠、树干、根部与投影，处在两条路之间的草地内，未覆盖店招或截断主梯。树冠仍较稀疏，单棵点植只是局部前后层次，不替代完整城市绿化

## 可复现命令

以下以镜厅为例，`--output` 必须使用尚不存在的目录；其他脚本位于同一证据目录，音乐街脚本在 `../public-urban-r1/music-wall.json`

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --project-root . --script todo/evidence/TASK-049/blender-integration-r1/cinema-facade.json --output output/recheck-cinema-facade --aa taa-ssao --no-video
python3 tools/capture.py --binary game/target/debug/n-side --project-root . --script game/capture/walk-preview.json --output output/recheck-building-walk --character-preview CHR-001 --no-video
```

固定时间步、seed 和输入限定本机复验条件，不声明跨 GPU 像素确定性；没有生成视频，实际查看关键帧与连续帧另记。可编辑 `.blend`、生成器、源素材、运行 GLB/DDS 保留，临时视觉产物在本轮全部查看与结论记录后清理

本批成功与失败运行均已完成检查和进程退出后清理：早期橱窗、招牌、头发及主梯共 2533 个视觉文件、2,411,973,755 bytes，最终集成共 2388 个、2,316,454,528 bytes；生成的 `.blend1` 备份另记录 hash 后清理。详见 [早期清理](cleanup-completed-runs.json)与[集成清理](cleanup-integration.json)，每个集成目录剩余视觉文件为零。留下源参考、可编辑工程、正式资源、复现脚本、版本和参数、日志、状态及已查看帧 hash

# 城市外观第 2 轮运行与画面自查

2026-09-30，本轮接入状态为站厅与音乐楼两类共 4 栋公共外皮、8 栋住宅细部和 5 处经营标识；本记录读取主 Agent 的真实 N站 capture，并独立查看原始关键帧与连续横移帧，是 self-audit，不是作者或陌生玩家验收

## 制作与检查范围

| 批次 | 实际交付与后续机器结果 |
| --- | --- |
| 公共外皮 | [V-01、V-32、V-78、V-79](../facades-r1/review.md)，站厅贴壁柱／檐带及音乐楼上部墙板／竖缝，复用源体量、门窗与已有材质 |
| 住宅 | [V-A08、V-A09、V-A13、V-A14、V-W08、V-13、V-A15、V-A16](../residential-r1/review.md)，窗台、窗檐、窗下裙板、护窗栏与带支架设备，护窗栏不是可进入阳台 |
| 经营标识 | [N站、BYTE BEAT、FRAME、PLAYROOM、AFTER 9](../signs-r1/review.md)原创 SVG 与 PNG，5 牌由[实际公共门与源外墙](../signs-r1/geometry-review.md)定位，不复用旧五店 8:1 牌面比例 |
| 集成测试 | [cpu-r2-0.log](../integration-r1/cpu-r2-0.log)为 13 项场景测试 PASS，覆盖住宅、公共外皮、植被等；[final-build-0.log](../integration-r1/final-build-0.log)为新增经营标识几何窄测 PASS，[final-build-2.log](../integration-r1/final-build-2.log)为 bins 构建成功 |

各制作子任务的 NOT RUN 是制作当时状态，后续主 Agent 日志补足上述编译与机器检查，没有改写历史记录。本次文档收束未再次运行 Cargo 或 GPU。[早期公共外皮两段 capture](../facades-r1/runtime-review.md)没有包含后来的住宅、街树、新天空和交接逻辑，不能用它覆盖本批全部视觉表现；[全城覆盖核对](../coverage-r1/review.md)中的数量与缺口对应制作前快照

## N站机器运行

主 Agent 实际执行 `python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/poster-station.json --output output/capture/task049-r2-station-sky`，本目录保留原始 [run.json](run.json)、[script.json](script.json)、[state.json](state.json)、[runtime.log](runtime.log) 与 [FFmpeg 日志](ffmpeg.log)，退出码 0、status PASS、视频编码成功

| 项目 | 实测 |
| --- | --- |
| 构建标识 | run 记录 HEAD `6e0a25e74092599e72bd06aaab52073ad6df98c8`，工作树非干净，二进制 SHA-256 `0c5ad4d39de8da934d9f7636f7dca913eeda0da784daac5a24e80e909fdcd1e2` |
| 脚本 | SHA-256 `1f533ab098aca9a26253048eb076a16fcabe98ced08ce3b54807c18d526df97c`，district / poster-station，1280 × 720、30 Hz、300 帧 |
| 设备 | runtime 记录 Linux、AMD Radeon RX 6650 XT、RADV / Vulkan、Mesa 26.2.3-arch3.1 |
| 机器检查 | 6/6 PASS：必需资源、300 帧保存、有限 Transform、实际相机前移／横移与释放后的稳定 |
| 实际运动 | 前移 2.886 m、横移 2.886 m，期望均为 1.5—4 m；释放输入后位移 0 m |

Viewer 复用真实世界渲染与 FreeCamera 输入，但没有人物碰撞、任务交互或手感验收。此 run 与小店 run 的 HEAD 和二进制不同，各自保留确切输入标识，不称为同一不可变构建或受控前后对比

## 实际画面自查

独立使用 view_image 查看关键帧 119、239 与连续帧 120—124，共 7 张；原 PNG 路径、字节与 SHA-256 见 [visual-review.json](visual-review.json)。没有播放完整视频，机器报告里的 `visual_review: NOT RUN` 保留为当时原始输出，本节补录后续实际观察

- 119／239：N站立柱、檐带、窗格和两侧墙面形成可辨的体量层次；东侧门楣标识能在斜视中看到，站名与屋顶安可海报承担不同识别用途，不能因此认为实际东门到达路线已经验收
- 120—124：横移可观察到视差，立柱、门楣和海报的外轮廓位置连续变化，抽查段内未见突然消失、跳位或明显贴面闪烁；未覆盖其他新招牌与八栋住宅的近景
- 天空在画面内呈很弱的上下蓝色层次，抽查范围没有明显立方图接缝；大面积仍接近平整色面，未包含高仰角天顶或完整太阳方向，不能宣称天空表现目标已完成
- 站前仍是大片浅色空地，生活物件、铺装分区与地面层次不足；右侧远景建筑重复窗格明显。现有画面与 README 概念预览的城市密度、经营内容及细节深度仍有差距

## 成本、结论与下一步

单次状态参考：观测 loading→ready 为 0.088846 s，ready 资源计数稳定为 4,983 entities、40 images、51 materials、3,386 meshes；世界 capture 帧间隔 299 项，p50 28.731 ms、p95 37.848 ms、最大 46.613 ms。指标包含截图回读与 PNG 工作，不是 GPU 帧时或原生 FPS；load 不含观察前准备，资源计数不是 GPU 字节。没有同条件旧版受控重跑，新增立面／招牌／植被的增量成本仍为 NOT RUN

PASS：所列源交付、后续构建与机器约束、N站相机路线和本节画面自查记录。NOT RUN：N站东门真实到达、兴趣街三店与 AFTER 9 门前的新标识专项路线、8 栋住宅近景、全城外观、作者审美、真人手感、Windows、物理手柄与夜景照明，任务保持 active

收束后实际执行 `bun run check:docs`：PASS，403 Markdown、102 IDs、31 Skills 与 25 recorded imports，见 [docs-check.log](docs-check.log)；限定四个卡片／README 路径的 `git diff --check` 与两个 runtime-r2 目录的 JSON 解析 PASS。任务看板由主 Agent 统一 sync／check，本次没有改写生成看板；没有修改 Wiki 页面，未单独重跑 Wiki 构建

下一步按真实入口补专项路线，再继续站前铺装和生活物件、住宅差异与相邻街段，修复后重跑受影响镜头；已完成的数量只描述本批制作，不增加 G2 已验收数量

本次未生成新截图或视频，未删除 `output/capture/task049-r2-station-sky/` 的原 PNG、关键帧和 MP4；主 Agent 统一清理前，本记录与 JSON 已保留查看帧 hash。清理完成后补清理记录，脚本、日志、状态和文字继续保留

root 已统一清理本次输出中的 PNG 连续帧、关键帧及录屏，日志与状态保留；数量和字节见 [cleanup.json](cleanup.json)

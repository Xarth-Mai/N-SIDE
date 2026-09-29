# 街树与住宅近景运行自查

本轮补充 TASK-045 新街树与 TASK-049 住宅外皮的真实 Viewer 近景，机器检查两路各 6 项 PASS；画面结论限于已查看的树木实例、V-A13 正面与短横移，任务均保持 active

## 运行输入与复现

运行 HEAD 为 `e92802ffd39daf5b7c990306ab23eeb1a78de821`，包含未提交的 Viewer 机位；实际二进制 SHA-256 为 `766ffe25fcc10de92604ffdc2dda1ff824618d7d977989fcc4bfcce4bc0c4d06`。两份 capture 记录的二进制一致，审核时文件仍匹配；脚本、源文件、地图、外观、光照、树模型、原始日志／状态和已查看帧的 hash 见 [runtime-summary.json](runtime-summary.json)。其中源码 hash 明确是运行后的审核快照，原始 `run.json` 保留运行时 HEAD、工作区与命令，不能把 HEAD 单独当作全部输入

运行设备为 Linux、AMD Radeon RX 6650 XT、Vulkan、Mesa 26.2.3-arch3.1；两路均为完整 district、1280×720、固定模拟 30 Hz、150 帧、FOV 55°、seed 0。当前场景没有消费该 seed 的随机逻辑，GPU 像素与墙钟时间不保证跨设备一致。采用既有白天配置：曝光 EV100 9.7、太阳 18,000 lux、MSAA4、TonyMcMapface、4096 阴影图，Bloom 与接触阴影关闭；没有为拍摄移动树木或住宅

```sh
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/street-tree-detail.json --output output/capture/task049-tree-detail-r1
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/exterior-details.json --output output/capture/task049-residential-detail-r1
```

构建成功记录见 [build.log](build.log)；Viewer 现有 3 项机位测试通过见 [view-tests.log](view-tests.log)。两个运行原始 `run.json`、`state.json`、`script.json`、`runtime.log` 保留在上述 output 目录，摘要只摘取参数、检查、统计和 hash，没有复制 300 条状态样本

| 机位 | 实际对象与原地图坐标 | 脚本 SHA-256 |
| --- | --- | --- |
| inspect-street-tree-detail | `trees[68]`，shop-steps-low 台地的 `tree_a`；eye `[127.75,325,46.827273]`，target `[136,321,48.127273]` | `1ce70ac8bd5f3058c4ae084a91d94ec1fab5dffc923d1240ccfb99281a6182e1` |
| inspect-residential-detail | `buildings[V-A13]`，从 east_mid_junction 观看西侧住宅立面；eye `[144,288,33.6]`，target `[161,282.5,38.536364]` | `c76a206c267e27068d9f37f47de9b9792e0fcac208a271a6fa8f62a42ef3379c` |

坐标使用地图的水平 x/y 与高程排列，Bevy 状态中的 y 为高度、z 为地图 y 的相反数；来源关系与路线见 [capture-plan.md](capture-plan.md)及 [capture-views.json](capture-views.json)，街树源资产检查见 [TASK-045 街树制作](../../TASK-045/vegetation-r1/review.md)

## 机器检查

两路均正常退出 0，必需资源就绪、150/150 帧保存、150 帧 Transform 有限各自 PASS。通过 M 启用既有自由相机，30–33 帧 W 接近、60–63 帧 A 横移、90–93 帧 D 回移，120 帧 Escape 退出控制；下表为真实相机系统输出，未直接改写位置

| 断言 | 街树 | 住宅 | 条件与结论 |
| --- | --- | --- | --- |
| real_camera_approach，29→49 | 2.086474 m | 2.086519 m | 均在 1–3 m 内，PASS |
| real_camera_strafe，59→79 | 2.086492 m | 2.086497 m | 均在 1–3 m 内，PASS |
| released_camera_stable，130→149 | 0 m、enabled=false | 0 m、enabled=false | 不超过 0.0001 m 且控制关闭，PASS |

## 实际画面 self-audit

实际使用 `view_image` 查看两路关键帧 59，以及完整连续帧 60、61、62、63；未播放视频，未进行隔离盲评或作者试玩。原始机器报告中的 visual_review 仍保留 NOT RUN，本文单独记录后续人工式看图结果

- 街树：主干、分叉和细枝连接可辨，叶簇间能看到天空，绿色和少量暖叶形成颜色变化；树根落在台地，树干与部分树冠投影可见。连续横移中枝干与叶簇随视差变化，未见整簇突然消失或整体跳位；此角度未遍历叶片背面，不能据此确认全方向双面表现或消除所有细边闪烁
- 住宅：V-A13 三层住宅窗檐、窗台、护窗栏与交替布置的窗下设备均可辨，设备下方有支架；小幅斜视能看出檐板与窗台厚度，59–63 帧未见这些构件整体跳位或明显互穿。首层门和门前小路保持可见；没有人物通过，故不证明碰撞、净空或可达性
- 品质仍需制作：街树近景叶片呈明显的稀疏多边形薄片，树皮缺纹理；住宅窗面仍是平整深色块，栏杆／设备重复明显，地面大片空白，天空层次偏弱。当前改善支持“新增细部已进入真实场景且可辨”，未达到 README 概念预览的完整品质

## 成本、边界与下一步

| 观测项 | 街树 | 住宅 |
| --- | --- | --- |
| 首次观察 loading→ready | 0.105731 s | 0.034132 s |
| capture 帧间隔 p50／p95 | 34.968／39.654 ms | 34.937／38.298 ms |
| ready 时资源峰值 | 4,983 entities／3,386 meshes／51 materials／40 images | 同左 |

上述为单次离屏录制的观测值，帧间隔含读回等待及 PNG 写入，加载计时不含首次观察之前的准备；资源数不是 GPU 字节或 draw calls。没有同设备同路线的改动前对照，因此原生 FPS、GPU 时间、显存与增量性能比较仍为 NOT RUN

本轮只覆盖一株新街树和一栋住宅立面；8 栋住宅逐栋近景、背面／上街入口、正式人物沿街路线、入口遮挡、全方向叶面、长期移动闪烁、Windows 与作者审美验收均未覆盖。下一步沿正式人物路径补齐住宅入口与街树投影，再继续立面、铺装和生活细节制作，不据本轮机器 PASS 放行 G2

本次证据与任务卡编辑检查：`python3 -B tools/validate_docs.py --root .` PASS，412 Markdown files／102 IDs；摘要 JSON 解析与受影响文件 `git diff --check` PASS。任务看板由根 Agent 统一同步并执行 tasks:check，本记录未编辑生成看板或重跑 GPU

视觉帧完成查看与hash登记后已清理，两路删除314个图片／视频，共301,574,066字节；实际逐目录数量与剩余占用以[cleanup.json](cleanup.json)为准，原始JSON、日志、源码、资产及复现脚本继续保留

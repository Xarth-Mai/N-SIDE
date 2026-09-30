# V-15 Blender 外立面制作 self-audit

日期 `2026-10-01`，范围为 `BLD-V15-FACADE-001` 的 Blender 源模型、派生 GLB、既有公共素材复用及 CPU 离线图；本代理未修改 `scene.rs`、`district.json`、任务卡或共享规格，未运行 Cargo、GPU 或提交

## 结论

源制作、重建、从保存主文件复导出、二进制几何检查和五个净空体积检查为 `PASS`；建筑美术完成度为 `needs_revision`，抽象海报仍明显是内容占位，大面积素墙在真实街景里的构图待观察；作者审美验收为 `NOT RUN`

主文件为 `source-assets/buildings/V-15/mirror-hall-facade.blend`，SHA-256 `237154280343e58fdd39a52f5cb69e058bddad05273114c4ac00444a4ad66e8e`；GLB 为 `game/assets/environment/buildings/v15-mirror-hall-facade.glb`，SHA-256 `07e23026c08d5967f85adc825ff518a3a4e34fac453565c71545a2d22dddd2b2`，字节 `5909108`；从主文件导出与完整重建得到相同 GLB 哈希

## 实际执行

```bash
ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-15/build.py -- --export-existing --render
ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-15/build.py -- --render
python3 source-assets/buildings/V-15/verify.py
```

两条最终 Blender 命令均退出 `0`，使用本机 `/usr/bin/blender` `5.2.2 LTS`，Cycles CPU、32 samples、1440×960、AgX；GLB 检查退出 `0`，`1 mesh / 8 primitives / 9552 vertices / 7154 triangles / 6 embedded images / 0 degenerate triangles`，没有 glTF 扩展、动画、骨骼、额外根变换或运行灯光

最终主文件另保存了可直接使用的南入口相机位置，再导出的 GLB 哈希保持不变；`bun run check:docs` 退出 `0`，Python 两脚本 AST 解析及清单文件哈希检查为 `PASS`

`verify.py` 读取实际二进制 POSITION、NORMAL、UV、indices 而非仅相信 metadata，核对有限数值、单位法线、索引、实际包络与声明一致；`component-bounds.json` 记录南门 3m 接近段、二层东门、北后勤门、东侧屋顶路径和公共平台接口五个空体积，无新增资产相交

本机 OpenAL 在受限音频服务上导致早期 Blender 完成文件写出后退出阻塞，已确认只是本轮进程且用 Ctrl-C 结束；最终以 `ALSOFT_DRIVERS=null` 正常完成，未更改系统音频设置；日志中不可写缩略图/扩展缓存及未启用 MeshOptimizer 的提示不参与 GLB 输出，最终静态检查和退出码已分别核对

## 实际看的图

首次模型渲染和修正后的以下三张最终 PNG 均通过图片工具逐张实际查看，后续没有以接触表或报告替代看图

| 最终文件 | SHA-256 | 观察 |
| --- | --- | --- |
| `output/buildings/V-15/r1/south-entry.png` | `5b04bc84f66c23adbc540615be81eea2d9366e6eae981b956e50775f09ba9564` | 深檐有上表面和折边体积，立面边柱与字牌可辨，入口外廊未落柱阻断；大面积素墙和几何海报仍显样板化 |
| `output/buildings/V-15/r1/entry-human-height.png` | `7303c34556bb548682f9d390d484e7fdef8ccda370f0d53113e9336a24d61404` | 人视能读出木底板、凹入灯具、两侧门框和海报背箱的厚度；原门玻璃仅为 REVIEW_ONLY 参照，没有导出新的假室内 |
| `output/buildings/V-15/r1/upper-connection.png` | `3451be6114903c36c1e687f1d1c2a1e96c3b4736fc2b868ae6b28a7a16cc0be5` | 按真实 0.40m 厚平台看见南斜撑接合板底，墙板贴墙；另一支架被真实源平台轮廓遮挡，不能仅用此视角断言所有构件可见，数值包络另外核对 |

Root 已实际查看三张最终图，认为字牌比例与深檐厚度可以进入 GPU 验证，并明确将抽象海报标为 `needs_revision`；这次协作自查不是作者最终验收

## 修正与独立审查

- 中文字牌初版过小，改为实际世界包络高度 1.8m；中途局部 dimensions 与旋转后的世界高度混淆被入口净空断言阻止，修正后重建、复导出和看图通过
- 字体轮廓三角化产生两个零面积三角，只在导出副本删除，保留主文件中的可编辑字体曲线
- 东側支架墙板原本距墙 2cm，改为低 x=339.98，嵌墙 2cm；南块墙板移至 north=240.27..240.73，按更宽窗框包络仍距既有候选窗约 0.119m
- DCC 平台参照先校正到真实板底 36.60m，再改为直接读取地图的 `cinema_upper_platform` 多边形，参考 collection 严格排除导出
- 独立子代理 `v15_contract` 只读审查地图、Bevy 接入契约、几何接合和 `build.py`/`verify.py`，最终未发现阻断项，结论为 `Lean already. Ship.`；未把静态审查写成运行验证

## 边界与后续

源坐标、公共门、餐饮门、后勤门、37m 屋顶接口和路线保持原地图；附加构件为视觉模型，未添加碰撞；原 V-15 主体墙没有实际影院门洞，外立面完成不代表室内可进入

Bevy 加载、GPU 同机位、角色入口/上街真实路径和作者审美验收在本资产制作范围均为 `NOT RUN`，由 Root 统一接入后继续；`Appearance.models` 与 `PropPlacement` 的参数、南侧通用窗/楼层带/原雨棚精确替换条件见资产 README

三张最终 PNG 已完成本代理与 Root 实际查看，按 Root 指示清理；保留源文件、导出、复现脚本、原始素材、哈希、日志、JSON 和本结论，未清理其他代理或 Root 的录制产物

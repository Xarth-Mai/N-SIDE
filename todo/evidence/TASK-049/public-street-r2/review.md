# 公共长椅：来源、装配与导出

本轮从已授权的免费公共素材中补一件可近看的街道座椅，沿用 AST-003 及既有环境导出器；没有新增资产平台或修改玩法。选用 [Poly Haven Modular Street Seating](https://polyhaven.com/a/modular_street_seating)，原作者 Stuart Attenborrow、CC0；两候选、官网元数据、11 份实际下载原件的 URL／官方 MD5／SHA-256 见 [source-review.json](source-review.json)

另一个候选 Painted Wooden Bench 为 1.2m、630 三角的红漆旧木凳，官网同为 CC0，但偏乡居室内，未下载。现有主模型清单没有细化公共座椅，原场景的 bench 是几何灰盒；镜厅 V-15 没有既有 bench fixture，不能把新模型说成已经替换该处原件

## 制作与实际检查

1. 下载单件所需的原始 glTF、共用 bin 与 9 张 1K JPEG，逐一匹配官方 MD5 和字节数；没有下载弧形连接件纹理或全套模型包
2. CPU 看源图发现上游是零件展示布局，座面、扶手和腿分开摆放。第一次探测只改 `scene.nodes`，Blender 仍导入孤立节点，报未下载的连接件贴图；记录保留在 [source-inspection.log](source-inspection.log)。正式装配同时限定实际节点，缺图问题消失
3. Blender 5.2.2 LTS 取原直座、靠背、支架、单端腿、扶手及横杆，组装两端对称支撑；原 UV、PBR 与米制尺寸保留，不对完整套件粗暴缩放。可编辑主文件和恢复脚本在 [street-furniture](../../../../source-assets/environment-kit/street-furniture/README.md)
4. 独立代理发现原扶手的一片共线三角在复制后成为两片零面积面；只在派生网格删除这两片，原件保留。最终 `8,906` 三角、4 材质、9 张内嵌原 JPEG，`5,925,368 bytes`
5. 实际复看最终正、背 CPU 图：靠背板与边支架相接、两端扶手与底横梁装配连贯，没有源展示布局中的悬浮部件。金属磨损较强，适合作为局部公共设施候选；这不是对整个城市动漫化效果的批准

最终 GLB SHA-256：`f5a48231b979242e5cdf8f354a39887248731b2b438f654620a0eb01a1753388`

| 检查 | 结果与范围 |
| --- | --- |
| 官方文件匹配 | PASS，11 份原件的实际大小、记录的官方 MD5 与 SHA-256 一致 |
| 环境导出 | PASS，[export.log](export.log) 与 [export-check.log](export-check.log)，36 个运行文件共 `90,443,346 bytes`；已有 35 个文件记录和运行字节保持不变 |
| 可编辑主文件恢复 | PASS，[primary-reexport.json](primary-reexport.json)及[日志](primary-reexport.log)，重新加载 `.blend` 独立导出与最终 GLB 逐字节一致；临时再导出文件已清理 |
| 模型实数值 | PASS，[check.py](check.py)／[check.json](check.json)读取实际 accessor，核对有限位置／UV、单位法线与切线、索引、三角面积、底面、包围盒和内嵌图像；源与运行 GLB 逐字节相同 |
| 纹理 | PASS，9 张内嵌 JPEG 均为 1024²，字节与原件完全相同；材质保留颜色、法线与粗糙度／金属度，原 ARM 红通道的 AO 没有绑定 `occlusionTexture` |
| 既有静态环境窄测 | PASS，[environment-tests.log](environment-tests.log)，1 项测试；本件完整检查另由上行的实际 GLB 检查承担 |
| 文档与引用 | PASS，[docs-check.log](docs-check.log)，561 Markdown、129 IDs、31 Skills／25 已登记导入 |
| CPU 视觉自查 | 已实际执行，[preview.py](preview.py)／[preview.log](preview.log)，Cycles CPU，32 samples，1100×750，正背两视角；制作代理和独立代理分别看图 |
| 独立审查 | 修复上述两个退化面及文档数值后复审，结论 `Lean already. Ship.`；没有将自查称为盲测 |
| Bevy 加载、落地、碰撞、游戏相机 | 本资产制作子任务 NOT RUN，由场景接入与对应实机记录承担 |
| 作者美术验收 | NOT RUN，本轮仍为局部候选 |

CPU 日志中的未安装 MeshOptimizer 不影响本件无压缩 GLB 导出；缓存缩略图写入失败与退出音频提示没有导致产物缺失。已通过实际 GLB 解析、字节比较及 CPU 重新载入验证，不将日志无错作为唯一判据

## 复现与交接

```fish
blender -b -t 4 -noaudio --python source-assets/environment-kit/street-furniture/assemble.py
bun tools/export-environment.ts
bun tools/export-environment.ts --check
python3 -B todo/evidence/TASK-049/public-street-r2/check.py
blender -b -t 4 -noaudio --python todo/evidence/TASK-049/public-street-r2/preview.py
```

`assemble.py` 从原件重建初始主文件，后续手工编辑 `.blend` 时采用资产 README 的直接导出命令。若更换 Blender 版本导致 GLB 字节改变，应先复核原件、几何及内嵌图像再更新清单，不能绕过哈希检查

运行引用为 `environment/street-furniture/street-bench.glb`、`Scene0`、缩放 1；坐者朝 +Z，映射到地图为南。完整包围盒与两柱足底见 [check.json](check.json)，宽 `1.920m`、深 `0.670394m`、高 `0.866914m`、座面约 `0.45m`；本次接入把单椅置于镜厅屋顶地图点 `[312,237,37]`，实际三角通过 `CollisionWorld::from_scene` 加入碰撞；两足支承与座前阻挡由 CPU 窄测核对，真实游戏足底接触、道路和镜头见 `todo/evidence/TASK-049/blender-integration-r2`，不把资产制作时的 NOT RUN 改成未观察的实机通过

本轮生成的源模型预览、最终正背 PNG 已在实际查看、记录后清理；保留源文件、正式资产、复现脚本、参数、日志及 JSON，清理明细见 [cleanup.json](cleanup.json)

### 进程清理纠正

首次清理的文件结果有效，但 `ps -C blender` 在沙箱中未看到宿主进程，将其记作“无进程”是错误判断；Root 随后在宿主发现 8 个已完成 R2 Blender 作业仍存活，日志均停在 OpenAL / PulseAudio 退出诊断，已保存产物及独立校验不受影响

已在宿主逐个核对本任务命令并发送 SIGTERM，8 个 R2 与当时已完成的 R3 作业均退出，PID 与复查见 [纠正记录](process-cleanup-correction.json)；后续无界面 Blender 使用 `env ALSOFT_DRIVERS=null timeout --signal=TERM --kill-after=5s 180s blender -b -noaudio`，逐次等待退出，并在宿主可见范围核实进程，不能以沙箱空列表代替

# V-15 南西立面 r2 资产检查

本轮把镜厅剩余南立面和整面西立面从泛型墙窗替换为 Blender 中的整层墙带、深窗框、公共大窗、餐饮三分窗、办公短窗加木格栅，并把 `Street programme 1 print` 换成已冻结的安可竖版海报；资产为可接入候选，未计作作者验收或镜厅最终美术完成

## 命令与实际结果

| 检查 | 命令／证据 | 结果 |
| --- | --- | --- |
| 增量编辑 | `ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-15/build.py -- --extend-r2 --render`，`build-render.log` | PASS；读取保存的 r1 主文件再添加 r2 分件 |
| 保存主文件复导出 | `ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-15/build.py -- --export-existing --render`，`final-render.log` | PASS；GLB SHA 与增量编辑首次导出相同 |
| 二进制读取 | `python3 source-assets/buildings/V-15/verify.py`，`binary-check.json` | PASS；实际 indices/POSITION/NORMAL/UV，零退化、有限值、合法索引与原点 |
| r1 保持 | `python3 todo/evidence/TASK-049/v15-building-r2/check-preservation.py`，`preservation-check.json` | PASS；53 旧件不变，1 print 有意换图，4 旧图形层有意删除，原 8 材质和 6 嵌图保持 |
| 空间 | `component-bounds.json`、`facade-checks.json` | PASS；原五净空体积和新东电梯转角体积无交叠，30 个评估窗面在实体墙外并位于框后 |
| 整体从零重建 | `--rebuild` | NOT RUN；本轮真实执行的是保存主文件增量编辑和复导出 |
| 真实 Bevy 街景／路线 | 主线程接入后运行 | NOT RUN in asset task |
| 作者视觉验收 | 独立于技术检查 | NOT RUN |
| 全仓文档检查 | `bun run check:docs`，`docs-check.log`、`docs-check-final.log` | 首次 FAIL 为并行 public-street-r2 的两处链接尚待落地；该文件落地后复测 PASS：561 Markdown、129 IDs、31 Skills |

当前为 21,482 顶点、12,362 三角、一个 mesh／10 primitives、7 张内嵌图、7,812,440 bytes，GLB SHA256 `a8d44ffe88ea53524ae1883715cb14680c750e02b965bd76033ffd236f32dc98`；相比 r1 增加 5,208 三角，16,000 三角上限保持，顶点上限因硬边和 UV 面角增至 30,000，新增玻璃和人物海报共两材质、一图已经主线程确认

GLB 内 Anke PNG 与正式 PNG 逐字节相同，SHA256 `0aa6fcef6414b0a229c80cd2c935b54f606fc628b5a8ddc2805d328be43eb1fe`；实际前表面顶点与 UV 证明图像横向未镜像、竖向没有倒置，所有材质的 glTF alphaMode 都为 OPAQUE

Blender 5.2.2 LTS 使用 Cycles CPU、固定 6 线程、32 samples、1280×720、AgX；未修改游戏曝光，未运行 GPU 或 Cargo；日志中的 `use_nodes` 弃用提醒、缩略图缓存权限、未安装可选 MeshOptimizer 库不影响主文件保存或未压缩标准 GLB 导出

## 实际看图 self-audit

五张最终 CPU 图均已实际打开，参数与 SHA 留在 `visuals.json`；主场景包含仅用于审查的灰色原建筑主体和平台，未把它们重复导出

- `building-southwest`：近似本轮实机镜头的西南方向，可同时看出完整西立面、入口中心、南面延伸；连续实柱、檐帽和不同窗高形成整栋层次，门头保留中心识别，原图中大片空盒侧墙与三行同尺寸小窗已转为大开间立面
- `annex-human-height`：以 1.75m 相机观察南面附楼，窗框侧面及上沿阴影存在真实进深，墙带、木格栅和暗玻璃彼此分离；近景构图未完整包含地坪，不能据此独立声称整条人行路线验收
- `west-human-height`：西面窗台、窗框、整层结构柱连续接合，办公窗与木格栅提供上层不同节奏；玻璃仍读作平整暗面，建筑仍偏规则，保留 needs_revision
- `upper-connection`：原东侧支承与平台底保持接合，主文件仍能检查梁与斜撑；原 53 件 hash 保持也覆盖这些构件
- `anke-programme`：追加近看整框镜头，脸、持麦手、「安可」与上下文字正向显示，框内无拉伸错向或旧抽象图形覆盖；深檐下较暗符合当前 CPU 光照，小字不作为远距离导航

本批显著收益是南、西两面整栋墙窗体量与楼层主次，未完成边界是另三张抽象海报、平整暗玻璃、规则开间，以及北／东其余泛型立面；离线看图不代替街景、碰撞、曝光一致性和作者审美判断

## 运行替换交接

只对 `V-15` 南边 `north=220` 与西边 `x=300`，整面跳过三层泛型窗的框／玻璃／窗台和29m／33m 楼层带；南面从 r1 的局部裁剪改为整面，北／东保持；继续只跳过原 `cinema_entry` 雨棚，保留门玻璃、门框和中梃

`V-15` 的 `floors` 已含 `RF`，原生成器不产生泛型 coping；独立审查纠正了仅凭包络推算存在旧 coping 的错误，因此未要求新增 coping 裁剪规则，也不虚报其三角移除数

GLB anchor 仍为 `Vec3(300,25,-220)`、identity rotation、scale 1、Scene0；本次运行集成已由 `CollisionWorld::from_scene` 加入模型实际三角碰撞，变换与视觉实例一致，保留原主体、屋顶、公用电梯、入口及路线，原泛型细件的碰撞移除数量由主线程按真实批次比较记录；本资产制作时未跑 GPU，接入后的游戏路径与画面见 `todo/evidence/TASK-049/blender-integration-r2`，不沿用早期“仅可视”假设

新增南面投影止 x=339.5，留原0.5m实体墙条给东电梯角；西面 north=220..250，最外投影 x=299.65；完整 GLB 包络为 `[-0.35,0,-30.08]` 至 `[43.150005,11.83,3.035]`

## 独立审查与瘦身

`street_evidence` 实际复读基线和当前二进制、分件包络并查看四张初版 CPU 图，确认旧结构保持、五净空与预算通过；发现 P2 必需海报文件不应可选，已改为必需输入断言并移除条件分支，防止缺图时静默产出错误版本；最终文档复核确认文件 hash、bytes、海报方向与预算一致，并发现、纠正 V-15 已因 RF 不生成泛型 coping 的交接前提错误

未增加运行 schema、引擎材质接口或共享几何框架；沿用现有 box、材质和合并导出流程，`--export-existing` 留作已有命令兼容；最终 ponytail 结论为 `Lean already. Ship.`

## 临时文件

PNG 为本轮 CPU 检查产物，完成实际查看和哈希记录后清理；`.blend1` 属于本轮保存产生的备份，确认无进程使用后记录哈希并清理，保留正式 master、GLB、海报、重现脚本、基线、日志与 JSON；最终状态见 `cleanup.json`

# R4 代码与来源复核

基线 `d7222a21786ad79c4c686748bb4fad13aaedd5c1`，检查 V-35 接管、碰撞、Viewer 机位、源模型导出与检查、曜 r11 生成器及清单差异；未修改源代码，用户的 `AGENTS.md` 与 `runtime.md` 不在审查范围

结论：没有发现需阻断本批的正确性或复杂度问题

- V-35 仅替换北面泛型窗、腰线、门、压顶和原 1.5 m 雨棚，原楼体、其他三面、南后勤门、招牌及稳定 ID 保留；测试比较替换前后实际构件，并将公共门交给真实导入三角形检查
- 0.55 m 停步距离依据实际 GLB 门体最大外凸 0.18 m、胶囊半径 0.30 m、skin 0.02 m，余量约 0.05 m；继续靠近必须命中闭门，未以删除碰撞或更改道路获得通过
- Viewer 两机位与捕获脚本使用当前地图坐标、真实 FreeCamera 输入和已有断言，声明范围为渲染及相机输入，不冒充人物通行证据
- 曜 r11 只调整既有连续夹克网格；519 顶点布局对应当前 231 个躯干顶点与两组 144 个袖部顶点，全量生成在焊接后调用同一函数，选择性更新保存唯一源，revision 标记避免重试时重复膨胀
- V-35 原创几何与脚本沿用项目 MPL-2.0，4 张既有 ambientCG 原图按 AST-003 的 CC0-1.0 记录保留，颜色原样复用、法线派生关系见下方增量复核，原 BYTE BEAT 招牌独立保留；曜未引入新外部素材

首次接入、法线修复前独立执行 `python3 source-assets/buildings/V-35/check.py`：PASS，3,612 三角面、7 材质、4 张嵌入图像、法线/绕序/度量 UV、源地图和 41 个保守接近采样通过；另以 Python `hashlib` 对照两份 manifest 的 5 项源/运行/候选文件 hash，全部一致，4 份相关 Python 文件 `ast.parse` 及范围内 `git diff --check` 通过

复读主任务 [lib-tests.log](lib-tests.log)：141 PASS、0 FAIL、1 ignored；复读曜 [promoted-contract.json](../../TASK-047/jacket-r11/promoted-contract.json) 与检查实现：20 项保存契约通过，含候选/正式 GLB 相同、非目标网格/权重/动画/贴图保持、一次性更新与原网格重放一致；这两项为已有实际日志复核，本审查未重复运行 Cargo 或 Blender，全量重新建模仅核对代码接线，未宣称本轮实际执行

本审查未执行 GPU 捕获或代签观感，画面自查与作者判断由本轮相应证据承接；未生成需清理的视觉或临时可执行产物

## 法线源修增量复核

本机 `game/Cargo.lock` 锁定 `bevy_gltf 0.19.1`；实际 registry 源码 `bevy_gltf-0.19.1/src/loader/mod.rs:1282–1290` 只提取 normal texture handle，并保留 `TODO: handle normal_texture.scale`，与修复依据一致

`bake_normals.py` 将原 RGB 解码到 `[-1,1]`，仅对 x/y 乘0.25，保持 z 后归一化并量化，符合目标切线空间扰动衰减；全量构建与保存后的主文件都使用 Non-Color 派生 PNG、Normal Map Strength 1，GLB scale 为1，避免正确支持 scale 的读取器再次衰减。原 JPG 与旧候选未覆盖，manifest 分别登记授权输入、派生 PNG、脚本及新导出 hash

独立执行 `python3 source-assets/buildings/V-35/bake_normals.py --check`、`python3 source-assets/buildings/V-35/check.py`、`python3 todo/evidence/TASK-049/blender-integration-r4/normal-scale-r1/compare_export.py` 均 PASS；两张衍生图逐像素匹配配方，实际导出除法线图与两项 scale 外，几何属性、索引、UV、节点、采样器、其他材质和颜色图保持。8 项 manifest hash 与现存文件一致，运行 GLB 为 `1879fad3909afeaf32e2aa75c58e553252f0fec8168fa3663855a217259d441a`，冻结候选仍为 `cca2fe302db5fce06565f67a03d12d8b3205f963988e4278c156a26a8288bd1b`

复读 [源保存对照](normal-scale-r1/source-preservation.json)及实现、[保存后源检查](normal-scale-r1/source-check.log)：源几何与其他材质摘要相同、两张 packed 图与 Strength 1 契约通过；本审查未重跑 Blender。本次技术结论不扩展到 mip 质量或最终画面，修复后 GPU 结果由主任务另行记录

Ponytail：沿用单一场景生成、碰撞、导出和捕获入口，没有新增平行系统或待用抽象

Lean already. Ship.

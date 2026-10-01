# V-35 法线强度源修

2026-10-01，Root 的同机位诊断确认移除 Plaster `normalTexture` 后密集黑点消失、原阴影保留，授权将原0.25强度烘进 V-35 两张法线。正式修复版 GPU 复验待 Root 执行，本记录不代签作者美术验收

## 实际变更

复用本目录 [bake_normal.py](bake_normal.py) 的 glTF 公式，在唯一资产包加入 [bake_normals.py](../../../../../source-assets/buildings/V-35/bake_normals.py) 和两张无损 RGB PNG：`x/y *= 0.25`，`z` 保持，归一化后量化，不做 sRGB 转换。Plaster 保持1024×1024，Concrete 保持1024×512，原 ambientCG JPG 与 AST-003 哈希不变；颜色 JPG 继续按原字节嵌入

[patch_source.py](patch_source.py) 打开现有 `facade.blend`，只替换 Plaster / Concrete 的 packed 法线图并将 Normal Map Strength 设为1，删除已无用户的旧法线图片数据块后保存；没有运行 `--rebuild`。随后调用现有 `build.py` 的导出函数，正式 GLB 两个 `normalTexture.scale` 均缺省为1。普通重建配方也改用衍生图和强度1，避免以后重新引入旧强度

原主文件备份位于 `output/assets/v35-normal-scale-r1/facade-before.blend`，SHA-256 `88549a9829efc68bef5f9935de57d679f8c0fef10f76ded4fa98a6bd91326fa3`；它仅供本轮回退，现有 `source-assets/buildings/V-35/facade.blend` 仍是唯一编辑源。冻结 `candidate.glb` 保持原字节

## 已完成检查

| 实际操作 | 结果与证据 |
| --- | --- |
| 2图烘焙与固定公式样例 | PASS，[source-bake.json](source-bake.json) 记录原图、衍生图、Pillow 12.3.0、尺寸、CC0许可与公式 |
| 常规烘焙复验入口 | PASS，`bake_normals.py --check`，[source-bake-check.json](source-bake-check.json)；该模式读取并比较像素，不改正式 PNG |
| 原位源修、保存及导出 | PASS，Blender 5.2.2 LTS、2线程，实际 exit 0，[source-export.log](source-export.log) |
| 源数据保留 | PASS，21组网格的坐标、角法线、面、材质索引、UV与对象变换，以及其余材质节点值和连线指纹前后一致，[source-preservation.json](source-preservation.json) |
| 正式 GLB 保留 | PASS，所有 accessor、索引、UV、法线、节点、场景、采样器、纹理绑定、其他材质值与两张颜色图字节逐项相同，[runtime-preservation.json](runtime-preservation.json)，检查脚本为 [compare_export.py](compare_export.py) |
| 正式 GLB 与衍生关系 | PASS，3,612三角、7,176顶点、7材质、4图、41个保守入口样本；两图逐像素符合烘焙公式，运行内嵌 PNG 与正式衍生文件 hash 相同，两个 scale 均为1，[runtime-check.json](runtime-check.json) |
| 保存后重新打开源工程 | PASS，Blender 2线程，实际 exit 0；21个网格、6台设备、8组窗、4张 packed 图及两节点图/强度契约，[source-check.log](source-check.log) |
| 独立只读审查 | PASS，`asset_audit` 使用另一份公式核对1,572,864像素，并复跑 GLB 对照与正式检查，确认原输入和颜色字节保留；结论 `Lean already. Ship.`，属于技术 self-audit |
| 修复版 GPU / 作者美术验收 | NOT RUN，本记录只覆盖源工程及文件检查；Root 接续同机位复验 |

保存日志中的宿主缩略图缓存不可写与未安装 MeshOptimizer 为非阻断提示：正式源和运行文件均写入，未启用压缩扩展，两个进程正常退出；上述文件检查依据真实落盘数据，不以日志结束语推定成功

```fish
python3 -B source-assets/buildings/V-35/bake_normals.py
python3 -B source-assets/buildings/V-35/bake_normals.py --check
timeout --signal=TERM --kill-after=5s 180s env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python todo/evidence/TASK-049/blender-integration-r4/normal-scale-r1/patch_source.py
python3 -B source-assets/buildings/V-35/check.py
python3 -B todo/evidence/TASK-049/blender-integration-r4/normal-scale-r1/compare_export.py
timeout --signal=TERM --kill-after=5s 90s env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/buildings/V-35/check.py -- --source
```

修复后主文件 SHA-256 为 `d8ba601a7f11594558d39fd16a275242b558f3af7078a5134d0474b75e0f3173`，运行 GLB 为 `1879fad3909afeaf32e2aa75c58e553252f0fec8168fa3663855a217259d441a`。两张 PNG 的来源与衍生哈希在资产 manifest 和 source-bake 中对应，原两张颜色与两张法线 JPG 保持已授权原件

PNG 仍只有一级 mip，强度修复不等于远距过滤或闪烁问题已解决；原0.25强度截图与源预览保留历史范围。没有新增截图或录屏，正式 PNG 不属临时媒体；既有 output 诊断媒体按 Root 指令保留，由主任务统一清理

# 共享法线修复独立审查

结论：PASS；当前范围没有遗留阻断项，最小性复审为 `Lean already. Ship.`。此结论覆盖共享烘焙、四包接入、许可追溯、实际二进制保持和证据准确性，不代替三个新 GLB 的 GPU 复验或作者外观验收

## 范围与方法

审查当前未提交的 [共享烘焙脚本](../../../../source-assets/environment-kit/materials/bake_normals.py)、V-15／V-A08 立面／V-55／V-35 的 build、check、manifest 与说明，以及 AST-003 清单的 `derived_normals` 和 README 对应尾段；检查本目录源修、复读、导出及保护记录。V-A08 使用 `facade-manifest.json`／`facade.md`，独立屋檐包不在修复范围

本审查只运行系统 Python 的只读检查，没有运行 Blender、Cargo 或 GPU，没有修改模型、实现或其他文档；唯一新增文件为本报告，没有生成视觉临时产物

## 独立复核结果

- PASS：实际执行 `python3 -B source-assets/environment-kit/materials/bake_normals.py --check`，exit0；7 张 RGB PNG 的尺寸、全部像素和既定公式一致，原 JPG 哈希保持，Pillow 为12.3.0
- PASS：重新在内存调用 [semantic_snapshot.py](semantic_snapshot.py) 的 `snapshot()`，经 JSON 标准化后逐项等于 [after-semantics.json](after-semantics.json)；四包 `protected` 全部等于 [before-semantics.json](before-semantics.json)，覆盖几何 accessor、UV、顶点法线、索引、非目标材质、非目标图片字节、节点／场景／采样器和纹理绑定；9 个目标法线槽的 glTF scale 均为1
- PASS：四包实际 source／runtime 哈希、V-15 已登记 build／verify 哈希、共享脚本哈希，以及各包和环境清单内原图／衍生图哈希与字节数逐项吻合；V-15 清单的地图构建快照按其明确的历史绑定语义读取，不冒充当前全图哈希
- PASS：两张 V-35 `scale025` PNG 与 `git show HEAD:source-assets/buildings/V-35/textures/<filename>` 的旧文件逐字节相同；V-35 整个运行 GLB 仍为 `1879fad3909afeaf32e2aa75c58e553252f0fec8168fa3663855a217259d441a`
- PASS：当前三个修复 GLB 分别为 V-15 `b3617d3c57d8e466ce4e6ca3e5fe4dd3f2b55d596f39c43a3cbdd5405d313bdd`、V-A08 `12e0fdef03f7a2ac2fcd22f2661302b0c29bc1932a0ac83d6ea7d8a02af1e09a`、V-55 `597bc0b96477295efd6a743e2ebf3884ad8b94305e9ba2e93d232e03b258cff4`，与源修后的清单和记录一致

四个现有检查器均验证原图在 AST-003 的许可／哈希登记、颜色原字节、内嵌共享 PNG 原字节、scale1，并在系统 Python 模式调用唯一烘焙函数核对像素；`--source` 模式明确将像素检查记为 NOT RUN，再核对实际 shader Normal 连线、Strength1、packed Non-Color 图及共享路径。四份已有检查 JSON 的 exit0 与 [commands.json](commands.json) 一致，本审查没有重复执行四包完整几何检查

## 准确性、许可与复现

原授权 JPG 保留；AST-003 登记的 CC0-1.0 原图与派生关系可追溯，脚本沿用项目 MPL-2.0，海报和字体仍按其既有来源管理。法线公式只缩放 x/y、保留 z 后归一化，没有引入 sRGB 转换；Blender Strength 与 glTF scale 同为1，避免再次衰减

已核对源修脚本的保存前保护、[复读脚本](reopen_source.py)、[诊断](reopen-diagnostic.json) 和 [最终复读结果](source-reopen.json)：例外仅允许 V-A08／V-55 各一个已验证零引用且无 fake user 的默认 `Material` 在保存时消失，未忽略在用材质或几何。V-15 首次 exit143 与首次复读失败均保留，后续宿主复读和三个普通导出 exit0 分开记录，没有把首次落盘当作成功进程

[导出脚本](export_saved.py) 使用现有普通导出入口，并保存本轮 V-15 报告后恢复历史证据；没有触发 `--rebuild`。保存前主文件／GLB 基线明确保留在 `output/assets/normal-scale-shared-r1/` 供复读和回退，复现依赖未被误作截图清理

首轮发现本目录 README 仍描述“修复准备、正式资产尚未修改”，制作方已更新；本次复读确认首页、四包说明与清单都将三个新 GLB 的 GPU 复验标为 NOT RUN，V-35 历史 R4 画面只用于当前原字节保持的 GLB。当前没有将离线检查、单级 PNG 或旧画面提升为任意距离采样稳定或作者验收

## 最小性复审

正式实现只维护一份 `scaled`／`baked_normal` 和7个已知组合，V-35 旧入口为短转发，保留既有命令入口有实际用途。准备阶段重复候选烘焙函数已移除，剩余旧路径只出现在迁移前状态记录；四个包没有新增独立烘焙函数、通用插件层或无使用者参数。各包保留自己已有的几何检查职责，保护与复读脚本承担本轮可核对的保存边界，不建议为本次修复再引入校验框架

`Lean already. Ship.`

# 公共室外空调装配证据收尾

基线 `8effe5d`，本次仅核对已完成资产与登记、迁移证据、同步空调安装文字；未启动 Blender、Cargo、GPU 或子代理，未修改任何模型、资产脚本、场景、appearance 或游戏清单。V55 与 CHR001 hood-r10 仅作只读核对，另经主线程明确授权修正 V55 的 runtime 状态一句

## 证据迁移

`output/assets/public-aircon-mount-r1` 中 10 个非空日志、JSON 与接点检视脚本按原字节迁入本目录，逐项路径／SHA256／字节见 [migration.json](migration.json)；三个零字节占位 `handoff.json`、`check.py.json`、`mount-check.py.json` 记录后移除，它们不是检查通过证据

初始单机的三份 DCC 记录另按原字节复制为 `candidate-build.log`、`candidate-preview.log`、`candidate-prepare.json`，原公共候选目录及其正式参考原件保持原状。历史日志内的 output 路径和脚本输出位置保留，以便追溯实际执行过程，不将迁移称为重新制作

## 真实 DCC 与本次只读检查

| 检查 | 结果与证据 |
| --- | --- |
| 原单机接点检视 | 已完成，`inspect-source.log` 记录打开真实候选主文件并生成阀口、背架、软管三张近图，结尾 Blender quit |
| 完整墙挂装配 | 已完成，`build.log` 记录打开候选、保存并重开 `wall-installation.blend`、导出5个 primitives，结尾 Blender quit |
| 装配 CPU 检视 | 已完成，`preview.log` 记录打开正式安装主文件和 mounted／connections／drain 三张图，结尾 Blender quit |
| 历史进程及看图范围 | [cleanup.json](cleanup.json) 记录两次会话正常退出、同步build exit0，制作者已查看五张最终保留图，root另查看安装／接点／收水口；首张 source-drain 曾被装配图覆盖，未虚构其哈希 |
| 本次候选检查 | `python3 -B source-assets/environment-kit/aircon/check.py`，exit0；[candidate-check-current.json](candidate-check-current.json) 验证9个原件与官方元数据、9493三角、两材质、六张图及有效格栅 MASK |
| 本次装配检查 | `python3 -B source-assets/environment-kit/aircon/mount-check.py`，exit0；[mount-check-current.json](mount-check-current.json) 验证10117三角、5材质、6图，保留单机位置加高／UV／材料／嵌图，法线导出最大差0.03884° |
| source／runtime与许可 | [asset-hashes.json](asset-hashes.json)，正式源导出与运行GLB逐字节相同；原单机及贴图CC0、新增624三角安装几何与脚本MPL分开登记 |
| 游戏安装观感、接触与通行 | GPU NOT RUN；主线程负责真实场景验证 |
| 作者验收 | NOT RUN |

真实制作使用 Blender 5.2.2 LTS、Cycles CPU、2线程；日志中的可选 MeshOptimizer 缺失和缓存缩略图权限提醒没有阻止导出。旧初始候选曾发生音频退出挂起，保留源README的边界；本轮装配三次正常退出的证据不倒推为旧进程正常退出

本次没有可重新打开的临时图，不声称重新完成视觉自查；复核的是制作者既有看图记录、真实 DCC 日志及当前二进制资产

## 已配置安装合同

实际代码以 `aircon_wall`、来源 `buildings[V-W10]/aircon-wall` 放在地图 `[567.79,95.6,12.666667]`，绕世界 Y 轴 `-90°`，scale1；世界 translation 为 `[567.79,12.666667,-95.6]`，风扇朝西。`CollisionWorld::from_scene` 已明确导入其实际 GLB 三角，包含低位收水口

原9,493三角单机只加高2.503333m，接回原包覆管口、软管末圈和背架；完整包络为局部 `[-0.399986,0,-0.235]` 至 `[0.399986,3.431230,0.187080]`，其中套管穿入墙体是已设计的连接。登记保持 exported，代码绑定已存在；实际 Ground 支承、墙／砖面接触、邻窗和灯具关系、玩家净空及格栅远近闪烁仍待主线程 GPU 与路线证据，不把代码配置称为已通过实机安装

## V55 与 hood-r10 只读核对

- V55：主文件与正式 GLB 哈希分别为 `b235d3837e29aad33153c81fa6d825b40894ba983b3e0cdabcd4c86ce991a8a5`、`8683357c862f708904de63bbe8a88a196f6e327b48ea5c638a1c318de8c6349b`，清单及最终 source-check／installed-geometry 一致；本次 Python check exit0，7362三角、13008顶点、6材质、4内嵌图，既有CC0图片和OFL字形来源齐全。初版 reexport-check 是6938三角旧版，handoff已明确仅证明初版复导出；最终版有真实源检查，不把旧复导出记录冒充最终证明。原 acceptance.runtime 的 parent integration required 已经主线程授权改为代码绑定存在、GPU待验，其他历史证据保持
- CHR001 hood-r10：主文件与正式 GLB 哈希分别为 `7f77c2380050a1d479e98f193c47ed2d4ea22fe62ee0989f54779b35c690a76d`、`ddb1cb5c0064e6e23994de4987458634ae24575e59d0e54b64f8739aa080719d`，登记与正式 promoted 记录一致；本次 `validate_character.py` exit0，32骨／四动作／三图和Root静止通过。真实 Blender 4.5.14 promoted 源检查和13项保持检查已存在，原模型MPL与参考图各自来源分开；`docs-check.log` 为零字节，不视作文档PASS。本次未修改角色目录或证据

## 清理与保留

迁移前未发现 Blender 进程，空调本轮输出目录、V55输出目录和hood-r10目录均无剩余临时PNG、视频或.blend1；空调历史五图已清理3,808,341 bytes，原件、候选主文件、完整装配主文件、正式GLB、日志与JSON保留。迁移后原 `public-aircon-mount-r1` 空目录已移除，未来脚本仍可重新创建；本次没有新生成媒体

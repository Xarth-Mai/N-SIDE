# 角色 GLB 验证器独立复验

审查工作区基线为 `4801b81bc3c91b5b4eebe04ebd86aa1f245b52c3`，实际被测文件 hash、Python/Pillow 版本与完整命令见 [checks.json](checks.json)。本轮没有修改源模型、导出脚本、场景、共享资产登记或故事

## 原问题复验

独立 [probe.py](probe.py) 直接解析和封装 GLB 字节，不导入验证器与已有测试辅助函数，经真实 CLI 验证三角形索引越界/有符号类型/数量不完整、保留 PNG 签名但破坏完整数据、仅首个根平移关键帧偏移 0.1 m、无关同名假根。六个坏输入均退出 1，并有对应的结构化诊断；未修改的真实 Blender 双动作夹具退出 0

已有测试另外覆盖截断 JPEG、所有根关键帧恒定偏移/偏转、选择 armature 时真实骨根移动、合法 JPEG 与 armature 根。原四类 false PASS 已关闭，修复前本轮结果见 [before.json](before.json)

## 本轮发现与修复

独立变异证实 VEC2 法线、零法线、UV 顶点数不足、SCALAR UV 四种错误数据仍返回 PASS。现在统一检查所有顶点属性数量与 POSITION 一致、NORMAL 为 float VEC3 单位向量、TEXCOORD 为 float 或 normalized unsigned VEC2；保留合法 normalized uint16 UV，通过正常数据回归避免将其误拒

只修改 [验证器](../../../../tools/validate_character.py) 与 [既有测试](../../../../tools/tests/test_validate_character.py)，复用原 accessor 数据，不添加依赖或新验证框架。顶点属性统一数量检查替代重复的蒙皮数量判断

## 实际检查

| 命令 | 结果 | 证据 |
| --- | --- | --- |
| `python3 -B -m unittest discover -s tools/tests -p test_validate_character.py` | PASS，9 项测试 | [日志](character-tests.log) |
| `python3 -B -m unittest discover -s tools/tests -p test_asset_environment.py` | PASS，静态环境数据消费者回归 | [日志](environment-tests.log) |
| `python3 -B todo/evidence/TASK-047/independent-r2/probe.py` | PASS，1 个正常输入退出 0，10 个变异均退出 1 | [逐例结果](independent-probe.log) |

以上命令可直接在仓库根目录的 fish 执行。原 false PASS 与新发现的属性检查只证明已覆盖的格式契约，不宣称完整 glTF 规范校验

## 边界与清理

当前入口限定 dense、embedded GLB、四权重、in-place、unit root TRS。高度和动作名由调用方给出；默认根位移 0.001 m、转角 0.1° 为已有导出检查容差，未新增人物身高或美术预算

本轮 Bevy 加载、骨骼变形、动作循环、接地和作者外观验收均 NOT RUN，正式角色由模型工作线继续制作与实机检查。数值检查通过不代表角色已完成

所有变异 GLB 使用 `TemporaryDirectory`，每次调用完成即清理；未生成截图、录屏、接触表或临时可执行文件。保留原正式回归夹具与复现脚本

## 复杂度审查

按 ponytail-review 检查验证器、共享 GLB 读取、受影响测试与本轮补丁；复用现有数据读取与 unittest，保留需要独立复现的脚本，未发现需要新增抽象或删除的有效复杂度项

Lean already. Ship.

这句只评价复杂度，正确性结论以本页实际输入、结果与未测范围为限

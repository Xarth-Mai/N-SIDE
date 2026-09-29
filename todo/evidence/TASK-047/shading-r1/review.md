# TASK-047 · 真实人物分段明暗样板 r1

基线 `9503dfa`，范围为显式 `--character-preview` 的 CHR-001 灰阶骨骼模型，沿用用户未批准配色状态。已实际查看镜厅预览左侧海报，取其成组的明暗和清楚轮廓；本轮不将材质样板视为形象批准，也没有从参考复制配色

## 接入与保持

`game/src/character.rs` 在角色预览启用时注册 `ExtendedMaterial<StandardMaterial, CharacterInk>`；原模型、图集、UV、骨骼、动画、控制器、阴影及深度路径保留。只转换角色实例中的 skinned mesh，环境和 UI 的材质不变，返回标题仍由原父子层级释放模型与材质句柄

`game/assets/shaders/character-ink.wgsl` 读取实际 StandardMaterial，并依据当前最强方向光将着色法线的 N·L 分为两个区间，再调用 Bevy 的完整 PBR 光照、雾与色调处理。原几何法线仍用于真实阴影采样，因此遮挡不会变成贴图上的假阴影。片元没有固定屏幕光源，没有设置 unlit，也未追加独立相机、屏幕贴图或材质框架

当前阈值 0.2、两档 cosine 0.2／0.82、最小过渡半宽 0.008 是可迭代的灰阶实验值，不是人物正式规格；边缘使用屏幕导数减轻锯齿，粗糙度 0.9 与 reflectance 0.15 用于压低塑料高光。项目现有一盏太阳光，未来多主光、脸部专用阴影图和正式描边需要按实际资产继续验证；没有方向光时继续原 PBR 光照

## 加载与检查

角色加载同时等待模型依赖与 WGSL 文件；shader 缺失走既有错误状态并保留胶囊，capture 读取同一 `CharacterStatus.error`。全部 skinned mesh 的 StandardMaterial 必须可读后才统一派发替换组件并隐藏胶囊，`CharacterStatus.shaded_meshes` 记录派发数量；它与 AssetServer loaded 都不是 GPU pipeline 成功证明

本地 Bevy 0.19.1 的 `extended_material.rs` 示例、MaterialExtension 源码、PBR 入口、光照函数与灯光绑定已逐项读取，路径和 SHA-256 见 [API 依据](api-source.json)。未升级 Bevy、未新增依赖，WGSL 为基于当前 API 的项目实现

小测试 `character::tests::ink_keeps_gltf_texture_and_geometry_contract` 覆盖图集句柄、灰阶基色、alpha、双面、forward 路径与原 vertex／shadow／prepass 合约；这不能证明 shader 编译或视觉成立

## 已执行的初次运行与失败复现

Root 统一构建后在 Linux／RADV RX 6650 XT 运行 `game/capture/character-orbit.json`，1280×720、30 fps、240 帧／8 秒，9 项断言通过；真实鼠标输入围绕静止角色，从正面经过侧面、背面回到侧前，Idle 时间推进至约 6.867 秒。原生退出 0，wrapper PASS，runtime 没有 Bevy ERROR。原日志、脚本及原始状态文件哈希见 [初次运行记录](initial-runtime.json)

本子任务实际查看第 59、89、139 帧，root 另看第 204 帧，独立 art_review 查看第 59、89、109、139 帧：人物可见，衣袖／裤腿／背部出现明确分区，脚下真实投影保留，侧背没有全黑或消失。角色形体仍偏技术样板，脸部占屏较小；这些是 self-audit，不证明人物风格已经达到海报目标，也不替代最终形体的同条件对照

Root 随后在 `output/assets/shader-failure-r1/project` 的隔离资源副本中故意损坏 WGSL，使用同一二进制完成 90 帧。该失败复现揭示：原生仍退出 0、state 5 项断言仍全部 PASS、`ready=true`／`shaded_meshes=1`，但渲染日志出现 `bevy_render::render_resource::pipeline_cache: failed to process shader error`。据此 root 在 `tools/capture.py` 加入实际运行 ERROR 检查，wrapper 正确输出 FAIL 并退出 1；这是着色器真实失败证据，不能写成原生状态系统已能发现编译失败

工具门禁沿用现有 capture 入口，去除 ANSI 后识别带 tracing 模块前缀的 ERROR，保留 WARN 和普通消息中的示例文字；root 增加对应 Python 窄测。没有新增通用管线监控框架，直接运行原生程序时仍须检查日志及画面

初次运行时人物形体仍在并行迭代，因此不将其作为最终 r4 造型对照。最终 GLB 冻结为 `0f16676224ab425aac27bcc6db05b068b0016dd3f862387674d26d9961fa42aa` 后，由 root 继续在相同模型、灯光、粗糙度、reflectance、机位和动作下比较「启用 N 重映射」与「只关闭 N 重映射」；这是明暗重映射单因素对照，不是与全部原生 glTF 材质参数的 PBR 对照

## 最终形体的同条件对照

`task047-ink-final-r1` 与 `task047-smooth-final-r1` 均实际运行 240 帧、9 项断言 PASS，原生退出 0、wrapper PASS，无 tracing ERROR。核对两次二进制与脚本 SHA 相同、两个资源根的最终 GLB 字节相同；隔离资源根仅把角色 WGSL 的 `if strongest > 0.0` 改为 `if false`，基色、图集、粗糙度 0.9、reflectance 0.15 与所有灯光路径保持。原始日志、脚本、状态文件哈希与所看图像哈希见 [最终对照](final-comparison.json)

本子任务对两版逐一实际查看第 59、88—90、138—140、204 帧，包含正面、转至侧面的连续帧、转至背面的连续帧与侧前。启用重映射后背部卫衣、帽沿和裤腿的受光面收为清楚色块，关闭时相同区域保留球面式渐变；人物侧面袖子和正面衣襟仍能区别，脚下投影、轮廓及贴地关系保持。在已看的连续帧中未观察到全黑、消失或明显的明暗整片跳变，不能据三个连续帧声称全部运动、灯光条件都无闪烁

收益集中在明暗组织，不能替代脸型、头发与服装材质制作。正面与侧前仍有简化的脸片、细线与灰阶样板感，脸部实际像素不足以判断最终表情和眼部层次；材质没有正式描边，也未测试夜间多光源、近距离脸部、Windows 与作者审美反馈。本轮允许保留此项真实运行能力，人物质量继续 `needs_revision`

各方完成实际查看、保存参数／哈希／日志后，root 已统一清理 GPU 视觉产物与两处隔离资源目录，见[清理记录](../../TASK-045/visual-r5/cleanup.json)。正式 WGSL、GLB、图集、复现脚本、运行日志与原始状态保留

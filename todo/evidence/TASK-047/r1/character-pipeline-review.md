# 角色导出工具技术复验

范围为角色 GLB 的 DCC 导出与数值预检，未制作或批准正式人物外观，未修改游戏控制器、动画系统或叙事

## PASS

- 使用实际 `output/tools/blender-4.5.14-linux-x64/blender` 4.5.14 LTS 背景执行 `tools/tests/create_character_fixture.py`，输出两个具名 Action、两骨、24 蒙皮顶点、4×4 嵌入 PNG 的方体夹具，导出日志为 `character-fixture-export.log`
- `export_animation_mode="ACTIONS"` 与逐 Action 的 `use_fake_user=True` 保留 Idle、Sway，`export_anim_slide_to_zero=True` 将每个 clip 起点归零，实际时长均为 1 秒；这是已执行的多动作导出接口，不是 API 推测
- `python3 -B tools/validate_character.py tools/tests/fixtures/character.glb --root-node Root --height 1.7 --clip Idle --clip Sway --require-texture` 退出 0，报告 `character-fixture-pass.json` 包含真实 accessor 高度、根位移、根转角与资产哈希
- `python3 -B -m unittest discover -s tools/tests -p test_validate_character.py` 最终 7 项 PASS，覆盖真实 Blender 导出、静态环境模型拒绝、尺度/clip/根名称、7 类坏导出、交错和归一整数 accessor、损坏输入的结构化失败，以及下列审查变异与合法 JPEG/armature 根
- `python3 -B -m unittest discover -s tools/tests -p test_asset_environment.py` 原有静态环境回归 PASS，GLB 读取已复用到 `tools/glb.py`，保留静态模型的原检查语义
- 第二次从同一脚本重新生成夹具，二进制哈希一致，见 `character-fixture-rebuild.json`
- 真实 `source-assets/environment-kit/nature/tree_oak.glb` 作为角色输入退出 1，诊断缺少 skin，命令与结果保存在 `character-static-rejected.json`

## 范围与后续

工具为当前 in-place 四权重角色路径预检，单位缩放的根层级由控制器负责世界平移和转向。每个人物高度、必需动作名取自制作规格；默认 0.001 m 根位移与 0.1° 根转角是导出检查容差。sparse、压缩 accessor、更多权重和 morph 动画明确拒绝，按实际需要另行验证；没有为尚未使用的格式预建支持

导出数据不能证明关节变形自然、动作循环无跳变、脚不滑动、Bevy 加载和动画时间推进。以上运行与视觉检查均为 NOT RUN，后续在真实角色和应用路径补齐。首轮图片仅检查签名与引用，经过独立审查已补全 Pillow `verify()` 和 `load()`，真实 Bevy 材质加载仍需验证

独立审查指出三角形索引越界、仅签名图片、根恒定错位和无关同名根可能被误报 PASS，现已补充数据检查与 9 个 CLI 变异：索引 999、带符号索引、不完整三角形数量、8 字节 PNG、截断 JPEG、全部根平移关键帧固定偏移 1 m、全部根旋转关键帧固定偏转 45°、独立假根，以及选择静止 armature 对象却让实际骨根移动。每项均退出 1 并输出明确 FAIL；原 Blender 夹具及真实嵌入 JPEG、合法 armature 根继续通过。根通道现在相对节点 rest TRS，而非相对首关键帧统计，指定根必须覆盖所有蒙皮关节并连同实际顶层骨根检查

文档只读检查在本次执行时因并行叙事稿引用尚未创建的 `docs/dev/design/quests/campaign-sequences.md` 失败，见 `character-docs-check.json`；新增角色工具引用无问题，最终全库检查由集成回合重跑

未生成截图、接触表、视频或临时探测可执行文件。保留的 `tools/tests/fixtures/character.glb` 是纳入工具回归的正式小型测试数据，主文件为生成脚本；它没有 CHR/AST 身份，不进入运行时资产目录。未留临时 `.blend` 或独立生成中间 GLB

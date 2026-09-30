# 玲模型 r2 · DCC 精修与复验

`CHR-002` / `AST-009` 保持 `needs_revision`，实际年龄 19、外观目标约 17，灰阶、身高缩放、32 骨及三个 in-place 动作契约保留；本记录属于制作自查，作者未批准最终造型或配色

## 输入与修改

基线为 `13397c5`，原源工程和 GLB 哈希见 [inputs.json](inputs.json)。本轮接续已完成的第一版颈肩与衣领修订：肩胸横向收窄 10.5%，衣领抬高约 23 mm 的未缩放距离，原骨架位置和动作不变；模型全体继续采用原 `0.9459975445842896` 缩放

先实际查看镜厅预览左侧角色海报、第一版面部与背发近景，确认需改善等宽叶片感、水平眉形和直菱形下颌。第二版把刘海改为七个不同宽度的侧分主次组，发束宽度从根部向末梢渐收，降低截面厚度；背发主组与次组的角宽、长度、转向分别调整。下颌过渡增加弧度，眉形改成连续弧线与两端收尖，上睑外段加厚、内段收细并加单个外眼角

未导入外部模型或动作，也未新增材质或改变色板。新增网格来自局部刘海和两侧外眼角，最终为 77 个可编辑分件，运行导出为一个材质、一个 primitive、22,058 顶点、41,936 三角面。完整高度约 1.6414 m，细微变化来自发束轮廓，未重定角色设定身高

## 命令与机器结果

以下命令在仓库根实际执行，均以零退出码完成；Blender 为 4.5.14 LTS，CPU，每进程 2 线程

```sh
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-002/model/build.py
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-002/model/check.py
python3 -B tools/validate_character.py game/assets/characters/CHR-002/ling-grey-study.glb --root-node Root --height 1.65 --clip Idle --clip Walk --clip Run --require-texture
python3 -B todo/evidence/TASK-047/ling-model-r1/measure.py --fps 120 --gait --check --output todo/evidence/TASK-047/ling-model-r2/exported-foot-tracks.json
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/ling-model-r2/render.py -- final --master source-assets/characters/CHR-002/model/ling-grey-study.blend
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/ling-model-r2/render.py -- final --motion --master source-assets/characters/CHR-002/model/ling-grey-study.blend
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-002/model/build.py -- --export-existing
```

[DCC 检查](dcc-check.json)覆盖三个动作的全部整数帧、骨矩阵有限性、根不位移、循环接缝、鞋底接地、五官贴合、鼻部连续曲面、背发和头顶覆盖与 UV；[GLB 检查](gltf-check.json)核对实际导出绑定、权重、尺寸、嵌入贴图、根通道和 clip。采样到的发壳最小头部间距约 6.77 mm，36 个后脑与头顶覆盖采样均通过

[120 FPS 导出足轨](exported-foot-tracks.json)中，支撑段最大位置漂移 Walk 约 0.268 mm、Run 约 0.607 mm，自然速度保持 `3.027192142669727` / `5.297586249672022 m/s`。另逐项比较基线与最终 GLB 的动画 sampler 输入和输出 accessor，三个动作数值完全一致，见 [动画契约](animation-contract.json)

主文件重新导出后 GLB 与重建导出物逐字节相同，见 [重导出结果](reexport.json)。最终源工程、运行文件、图集哈希与数量见 [delivery.json](delivery.json)，唯一资产登记已同步到 `source-assets/characters/CHR-002/asset-manifest.json`

## 实际看图

制作 Agent 实际查看最终正面、四分之三、侧面、背面、面部近景与后脑近景六图；参数与输入哈希见 [六视图记录](final-views.json)。颈肩衔接比基线紧凑，下颌线有弧度，眉弧、眼睑粗细和主次刘海可辨，后发末端形成不同高低与宽度；未观察到头皮裸露或五官浮离造成的新增明显缺口

实际查看 Walk 八个相位 `1/7/13/19/25/31/37/43`、Run 八个相位 `1/6/11/16/21/26/31/36` 的侧视接触表，以及 Run 正面 `1/7/13` 三图，完整参数见 [动作视图记录](final-motion-views.json)。所看帧中鞋底支撑与抬脚可区分，双臂摆动保留原范围，收窄肩部和抬高衣领没有出现新增明显断面或穿插；背发随 Head/Chest 权重运动，没有独立发骨或物理摆动

当前仍能看到发根接缝、较大的概括发片和侧视刘海悬空感；面部身份、简化嘴形、硬发夹与手部表现尚需迭代。年龄读感仍需作者判断，机器检查不能证明 17 岁的艺术表现；本轮改善没有变更最终造型验收状态

## 运行与清理

本制作 Agent 未启动 Cargo、GPU 或 Bevy；真实人物路径的加载、转身和走跑由主 Agent 在 r9 集成验证，本记录不以 DCC 结果代替实机验收

最终静态图与动作原帧已实际查看，按 [清理清单](cleanup.json)删除可复现的临时图；少量最终近景与接触表暂交主 Agent 查看，主 Agent 完成检查后清理。中断前的 before/first 图与基线冻结文件继续交主 Agent 按本轮归属统一清理

# V-55 首版交接

所属TASK-049；交付源目录 `source-assets/buildings/V-55/`，候选 `output/buildings/V-55/v55-workshop-facade.glb`，源／候选哈希和放置合同在源清单。未修改正式游戏资产、地图、appearance、Rust或任务卡

## 实际结果

- PASS：本机Blender5.2.2 LTS完成建模、打包4张既有CC0材质图及Noto字体、导出和源检查；单实例ALSOFT null／noaudio／2线程，带timeout，所有进程正常退出并已回收
- PASS：7362三角、13008顶点、6材质、4内嵌图；实际有限Transform、面面积／绕序／法线、PBR米制UV、原图哈希、地图外形／楼层／两门合同和82个接近采样
- PASS：初版从已保存主文件重导出的geometry JSON与首次结果逐字节一致；最后增厚字形／补转角后的源检查通过，最终候选hash见geometry.json
- PASS：关闭公門前0.45m会触碰中梃与下板，0.55m净空；另存clearance-comparison.json，范围是保守AABB，不冒充运行端胶囊体
- SELF-AUDIT：实际观察三张640×480／12samples CPU Cycles图，西南轮廓、三层工作间、两门主次、蓝紫首层图形和陈列层次可读；用途字由过细修为厚实，南西转角收口补齐。较规整的灰墙及重复上层窗仍需真实街景判断，不等于最终美术品质或作者批准
- NOT RUN：Bevy接入、真实Ground、碰撞、实际玩家行走和实机画面，由主任务安装后验证
- NOT RUN：新玩法、室内、角色动画，本静态建筑任务不包含这些内容

## 接入

源锚点 `[154,206,24]`，GLB Y-up／identity／scale1；model建议`v55_workshop_facade`。世界外包络X147.25…160、北向197.75…214、高程23.45…33。两门关闭，公门宽1.8m、服务门宽1.3m，二者高2.35m

跳过V-55西面x148与南面north198的旧泛型窗、腰线、被覆盖压顶；西面两门的泛型框／叶／小檐也替换。东／北面及源结构墙／屋顶／基础保留。新增低位几何按现有真实GLB碰撞路径集成，0.55m门外接近点不能被当作打开或进入建筑

源字体文件未修改，源曲线offset只是当前标签的造型调整；允许字体数据打包的OFL与原COPYRIGHT路径已记录。导出提示MeshOptimizer可选库缺失，实际GLB未使用压缩扩展；主文件缩略图写入用户cache被沙箱拒绝，`.blend`保存及重开检查均成功，不因此提升写权限

## 证据与清理

保留build.log、render.log、source-check.log、geometry.json、first-visual-hashes.json、reexport-check.json、clearance-comparison.json、visual-cleanup.json。最后三张PNG均经制作Agent与root实际查看，哈希、观察结论记录后已删除。源码目录的自生pycache已移除，无.blend1、临时预览图或后台Blender

Ponytail自查：复用既有GLB读取器、手工两立面建模方式、现成CC0材质和字体，没有新增生成框架、外部依赖或另一份素材总账；移除未用math导入。Lean already. Ship.

## 接入续记

root在R2提交 `8effe5d` 后授权正式接入；GLB原字节复制到 `game/assets/environment/buildings/v55-workshop-facade.glb`，源导出／检查入口及清单已更新为正式路径，原制作候选日志保留旧路径用于追溯

Rust场景新增固定实例、appearance模型绑定，西／南面的泛型窗／腰线／压顶及两门外观由新模型接管，仍运行原入口地形／相邻建筑及净口验证。实际GLB三角加入既有 `from_scene` 白名单，闭门保持；附加边界、两条路线、原北／东面几何不变与两个关闭门碰撞窄测

按主任务调度，本子任务未运行Cargo、GPU或再次启用Blender；本次窄测需root编译执行，不能把新增断言等同通过。源码Python语法、正式GLB与候选hash一致、导出实际几何检查以及git diff --check由本子任务运行

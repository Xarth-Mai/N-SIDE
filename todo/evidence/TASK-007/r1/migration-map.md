# R1 逐文件迁移索引

基线 `a0deb64e1447d84ab7161e494ee1748de279090c`；本索引分类实际文件后生成，尚未执行迁移。原始字节 hash、段落选择器、数据允许字段和消费者行号以同目录 `migration-map.json` 为准

## 保护与语义核对

- 文档不摘要化，删除重复全文前对比权威正文覆盖；保留原对象ID/显式锚点/主线章序/18故事/35角色/91场所/12街坊
- QST-002 narrative.json和数据模板JSON/CSV字节不变；catalog仅上述allowlist字段可变，其余递归语义相等
- approved→accepted仅设计状态词汇迁移；旧done及M0-01作者记录按实际范围保留，不赋予新游戏完成状态
- 玩家源码世界内Agent/剧情/结局不属于开发泄漏；任务状态、工具/内部规格和未验证按键不会进入玩家发布包
- 原始截图、inputs.sha256、提交版本、授权与许可声明完整保留；历史路径变更在本表追溯
- source-assets和game/assets不移动，地图/游戏入口行为不在文档任务中替换或伪造

## 文件去向

| 原文件 | 动作 | 具体去向 | 段落职责／依据 |
| --- | --- | --- | --- |
| `docs/.vitepress/components/DistrictArchitecture.vue` | update-consumer | `docs/.vitepress/components/DistrictArchitecture.vue` | 共享配置与组件，构建/导航/搜索/静态/数据统一按profile隔离；玩家组件只能读取允许投影 |
| `docs/.vitepress/components/DistrictMap.vue` | update-consumer | `docs/.vitepress/components/DistrictMap.vue` | 共享配置与组件，构建/导航/搜索/静态/数据统一按profile隔离；玩家组件只能读取允许投影 |
| `docs/.vitepress/components/DistrictPlaces.vue` | update-consumer | `docs/.vitepress/components/DistrictPlaces.vue` | 共享配置与组件，构建/导航/搜索/静态/数据统一按profile隔离；玩家组件只能读取允许投影 |
| `docs/.vitepress/components/DistrictPlan.vue` | update-consumer | `docs/.vitepress/components/DistrictPlan.vue` | 共享配置与组件，构建/导航/搜索/静态/数据统一按profile隔离；玩家组件只能读取允许投影 |
| `docs/.vitepress/config.mjs` | update-consumer | `docs/.vitepress/config.mjs` | 共享配置与组件，构建/导航/搜索/静态/数据统一按profile隔离；玩家组件只能读取允许投影 |
| `docs/.vitepress/sidebar.mjs` | update-consumer | `docs/.vitepress/sidebar.mjs` | 共享配置与组件，构建/导航/搜索/静态/数据统一按profile隔离；玩家组件只能读取允许投影 |
| `docs/characters/agent.md` | move | `docs/player/encyclopedia/characters/agent.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/brother.md` | move | `docs/player/encyclopedia/characters/brother.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/characters.json` | move-with-consumers | `docs/dev/design/catalogs/characters.json` | 35角色及其年龄、关系、场所与成长字段逐项保留 |
| `docs/characters/family-members/chen-jing-he.md` | move | `docs/player/encyclopedia/characters/family-members/chen-jing-he.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/family-members/index.md` | move | `docs/player/encyclopedia/characters/family-members/index.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/family-members/lin-qi-ming.md` | move | `docs/player/encyclopedia/characters/family-members/lin-qi-ming.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/family.md` | move | `docs/player/encyclopedia/characters/family.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/index.md` | move | `docs/player/encyclopedia/characters/index.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/chen-zhi-xia.md` | move | `docs/player/encyclopedia/characters/neighbors/chen-zhi-xia.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/cheng-jia.md` | move | `docs/player/encyclopedia/characters/neighbors/cheng-jia.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/chu-qing.md` | move | `docs/player/encyclopedia/characters/neighbors/chu-qing.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/gu-zhu.md` | move | `docs/player/encyclopedia/characters/neighbors/gu-zhu.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/he-qiu.md` | move | `docs/player/encyclopedia/characters/neighbors/he-qiu.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/he-xin.md` | move | `docs/player/encyclopedia/characters/neighbors/he-xin.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/he-yun.md` | move | `docs/player/encyclopedia/characters/neighbors/he-yun.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/index.md` | move | `docs/player/encyclopedia/characters/neighbors/index.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/jiang-tang.md` | move | `docs/player/encyclopedia/characters/neighbors/jiang-tang.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/liang-su.md` | move | `docs/player/encyclopedia/characters/neighbors/liang-su.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/lu-heng.md` | move | `docs/player/encyclopedia/characters/neighbors/lu-heng.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/mina.md` | move | `docs/player/encyclopedia/characters/neighbors/mina.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/ning-lang.md` | move | `docs/player/encyclopedia/characters/neighbors/ning-lang.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/qiao-yin.md` | move | `docs/player/encyclopedia/characters/neighbors/qiao-yin.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/qiu-ning.md` | move | `docs/player/encyclopedia/characters/neighbors/qiu-ning.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/shao-min.md` | move | `docs/player/encyclopedia/characters/neighbors/shao-min.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/shen-ling.md` | move | `docs/player/encyclopedia/characters/neighbors/shen-ling.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/song-lan.md` | move | `docs/player/encyclopedia/characters/neighbors/song-lan.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/su-he.md` | move | `docs/player/encyclopedia/characters/neighbors/su-he.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/tang-yu.md` | move | `docs/player/encyclopedia/characters/neighbors/tang-yu.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/wei-hang.md` | move | `docs/player/encyclopedia/characters/neighbors/wei-hang.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/wen-ran.md` | move | `docs/player/encyclopedia/characters/neighbors/wen-ran.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/yao-an.md` | move | `docs/player/encyclopedia/characters/neighbors/yao-an.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/ye-chu.md` | move | `docs/player/encyclopedia/characters/neighbors/ye-chu.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/zhao-yan.md` | move | `docs/player/encyclopedia/characters/neighbors/zhao-yan.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/zheng-bo.md` | move | `docs/player/encyclopedia/characters/neighbors/zheng-bo.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/zheng-che.md` | move | `docs/player/encyclopedia/characters/neighbors/zheng-che.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/zhong-fan.md` | move | `docs/player/encyclopedia/characters/neighbors/zhong-fan.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/neighbors/zhou-ran.md` | move | `docs/player/encyclopedia/characters/neighbors/zhou-ran.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/shared-dreams/index.md` | move | `docs/player/encyclopedia/characters/shared-dreams/index.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/shared-dreams/ji-wen.md` | move | `docs/player/encyclopedia/characters/shared-dreams/ji-wen.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/shared-dreams/xu-yao.md` | move | `docs/player/encyclopedia/characters/shared-dreams/xu-yao.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/characters/sister.md` | move | `docs/player/encyclopedia/characters/sister.md` | 逐页核对后现有正文为身份、关系、生活与表达；无须为每位人物创建空的制作副本 |
| `docs/conventions.md` | replace | `docs/dev/handbook/documentation.md` | 保留语言、源资产、稳定ID等约束，替换混合受众、单id与旧状态教学；新规范保存document_id/subject_id分工 |
| `docs/enemies/index.md` | move | `docs/player/encyclopedia/enemies/index.md` | 当前仅梦魇事实与索引，全部保留；无运行数值或行为树需要分拆 |
| `docs/gameplay/combat.md` | semantic-split | `docs/player/encyclopedia/gameplay/combat.md`<br>`docs/dev/design/systems/combat.md` | ## 角色与工具；## 处理异常；## 战败与重试末段“梦中的受击” → docs/player/encyclopedia/gameplay/combat.md；## 战败与重试首段“战败后可以从检查点重试” → docs/dev/design/systems/combat.md |
| `docs/gameplay/controls.md` | merge | `docs/dev/design/systems/input.md` | 两段均为目标动作与输入支持设计，当前无对应已玩证据，不将计划动作当玩家可用控制；已验证Viewer控制留开发手册 |
| `docs/gameplay/daily-life.md` | move | `docs/player/encyclopedia/gameplay/daily-life.md` | 日常/探索/潜梦的世界内概念与玩法设想作为百科保留，指南只解释实际版本 |
| `docs/gameplay/dream-diving.md` | semantic-split | `docs/player/encyclopedia/gameplay/dream-diving.md`<br>`docs/dev/design/systems/state-and-recovery.md` | 全文，除## 处理结果中“战败重试按”段 → docs/player/encyclopedia/gameplay/dream-diving.md；## 处理结果中“战败重试按”至“世界时间发生倒流” → docs/dev/design/systems/state-and-recovery.md |
| `docs/gameplay/levels.md` | move | `docs/player/encyclopedia/gameplay/levels.md` | 日常/探索/潜梦的世界内概念与玩法设想作为百科保留，指南只解释实际版本 |
| `docs/index.md` | semantic-split | `docs/index.md`<br>`docs/player/index.md`<br>`docs/dev/direction/vision.md`<br>`docs/dev/index.md` | ## Overview与## 游戏百科 → docs/player/index.md；### 游戏方式中PC/Linux/Windows/输入/Vulkan交付承诺 → docs/dev/direction/vision.md；## 开发资料 → docs/dev/index.md；源码知识入口 → docs/index.md |
| `docs/locations/n-district.md` | move-with-consumers | `docs/player/encyclopedia/locations/n-district.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格；DistrictMap必须使用玩家允许字段投影，移除完整地图源的客户端引用 |
| `docs/locations/place-network.md` | move | `docs/player/encyclopedia/locations/place-network.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b01.md` | move | `docs/player/encyclopedia/locations/places/b01.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b02.md` | move | `docs/player/encyclopedia/locations/places/b02.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b03.md` | move | `docs/player/encyclopedia/locations/places/b03.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b04.md` | move | `docs/player/encyclopedia/locations/places/b04.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b05.md` | move | `docs/player/encyclopedia/locations/places/b05.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b06.md` | move | `docs/player/encyclopedia/locations/places/b06.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b07.md` | move | `docs/player/encyclopedia/locations/places/b07.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b08.md` | move | `docs/player/encyclopedia/locations/places/b08.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b09.md` | move | `docs/player/encyclopedia/locations/places/b09.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b10.md` | move | `docs/player/encyclopedia/locations/places/b10.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b11.md` | move | `docs/player/encyclopedia/locations/places/b11.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/b12.md` | move | `docs/player/encyclopedia/locations/places/b12.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/places/index.md` | move | `docs/player/encyclopedia/locations/places/index.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/shop.md` | move | `docs/player/encyclopedia/locations/shop.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/locations/stargazing-terrace.md` | move | `docs/player/encyclopedia/locations/stargazing-terrace.md` | 生活、到达和访问边界属于地点事实，全文保留；工程尺寸已有独立district规格 |
| `docs/narrative/data.md` | adapt | `docs/dev/engineering/content-contracts.md` | 保留每个字段、信息状态、节点语义与对白CSV格式，明确仅静态设计检查并扩展运行加载/错误边界 |
| `docs/narrative/examples/last-toy.md` | merge | `docs/dev/production/narrative.md`<br>`docs/player/encyclopedia/story/main/last-toy.md` | 首段方法目的与## 可玩结构；## 信息与体验 → docs/dev/production/narrative.md；人物处境与“表面解释/深层机制”两段 → docs/player/encyclopedia/story/main/last-toy.md |
| `docs/narrative/playtest.md` | adapt | `docs/dev/validation/playtesting.md` | 保留待校准指标、分母、玩家原话和修订路径，明确自查/隔离评审/真人区别 |
| `docs/narrative/workflow.md` | adapt | `docs/dev/production/narrative.md` | 完整方法保留，与项目样板和输入/步骤/输出/失败处理合并 |
| `docs/production/art.md` | semantic-split | `docs/dev/production/art-direction.md`<br>`docs/dev/production/asset-pipeline.md` | ## 视觉方向至## N街区空间参考（除下面逐段例外） → docs/dev/production/art-direction.md；## 材质与构件中“运行资产的基线”段 → docs/dev/production/asset-pipeline.md；### 白天视觉样板 → docs/dev/production/art-direction.md；## 制作 → docs/dev/production/asset-pipeline.md |
| `docs/production/asset-tool-review.md` | semantic-split | `docs/dev/decisions/asset-tools.md`<br>`docs/dev/production/asset-pipeline.md` | ## 已采用 → docs/dev/production/asset-pipeline.md；## 暂缓或不采用 → docs/dev/decisions/asset-tools.md；## 来源与改编说明 → docs/dev/decisions/asset-tools.md |
| `docs/production/assets.md` | adapt | `docs/dev/production/asset-pipeline.md` | 保留唯一源、元数据与导出/验证；原版asset_report现为声明过的补丁；GLB详细绑定规范可引用blender-animation |
| `docs/production/audio.md` | expand | `docs/dev/production/sound.md` | 保留原声音层/交付/检查，补项目事件owner/start-stop/loop/并发/恢复方法，不杜撰已接入系统 |
| `docs/production/character-narrative.md` | move | `docs/dev/design/characters/narrative.md` | 出场节奏、知情范围与协作职责保留，人物事实指向百科和catalog |
| `docs/production/city-story-map.json` | move-with-consumers | `docs/dev/design/catalogs/city-story-map.json` | 91场所、12街坊与共楼/人物/任务/访问/后续字段完整保留，地图源路径不变 |
| `docs/production/demo-scope.md` | adapt | `docs/dev/direction/demo-scope.md`<br>`todo/tasks/TASK-012-last-toy-mechanism-review.md` | 完成体验、三支柱、既有约束、完整游戏与Demo范围 → docs/dev/direction/demo-scope.md；## 当前未知与后续工作：问题列 → docs/dev/direction/demo-scope.md；开头/末段当前轮次与M0工作包调度 → todo/tasks/TASK-012-last-toy-mechanism-review.md |
| `docs/production/district-architecture.md` | move | `docs/dev/design/locations/district-architecture.md` | 保留建筑尺寸、道路、高差、图纸组件、到达与验收；JSON仍source-assets唯一源 |
| `docs/production/district-places.md` | move | `docs/dev/design/locations/district-places.md` | 保留建筑尺寸、道路、高差、图纸组件、到达与验收；JSON仍source-assets唯一源 |
| `docs/production/district-plan.md` | move | `docs/dev/design/locations/district-plan.md` | 保留建筑尺寸、道路、高差、图纸组件、到达与验收；JSON仍source-assets唯一源 |
| `docs/production/district-space.md` | move | `docs/dev/design/locations/district-space.md` | 保留建筑尺寸、道路、高差、图纸组件、到达与验收；JSON仍source-assets唯一源 |
| `docs/production/district-station.md` | move | `docs/dev/design/locations/district-station.md` | 保留建筑尺寸、道路、高差、图纸组件、到达与验收；JSON仍source-assets唯一源 |
| `docs/production/district-waterfront.md` | move | `docs/dev/design/locations/district-waterfront.md` | 保留建筑尺寸、道路、高差、图纸组件、到达与验收；JSON仍source-assets唯一源 |
| `docs/production/engineering.md` | adapt | `docs/dev/engineering/index.md` | 现有技术/源码/就绪/错误/排错入口全文保留；数据详细字段引用content-contracts，运行方法引用validation |
| `docs/production/gameplay.md` | semantic-split | `docs/dev/design/systems/index.md`<br>`docs/dev/design/systems/input.md`<br>`docs/dev/design/locations/level-specification.md`<br>`docs/dev/design/systems/combat.md`<br>`docs/dev/design/systems/daily-life.md`<br>`docs/dev/design/systems/state-and-recovery.md`<br>`docs/dev/design/systems/enemies.md`<br>`docs/dev/design/quests/index.md` | 开头 → docs/dev/design/systems/index.md；## 操作（含交互约定/操作验证） → docs/dev/design/systems/input.md；## 关卡（含3节） → docs/dev/design/locations/level-specification.md；## 战斗（含4节） → docs/dev/design/systems/combat.md；## 日常与时间：生活内容/昼夜/区域状态 → docs/dev/design/systems/daily-life.md；## 日常与时间：任务实现 → docs/dev/design/systems/state-and-recovery.md；## 敌人 → docs/dev/design/systems/enemies.md；## 梦魇事件约束 → docs/dev/design/quests/index.md |
| `docs/production/place-catalog.json` | move-with-consumers | `docs/dev/design/catalogs/place-catalog.json` | 91场所、12街坊与共楼/人物/任务/访问/后续字段完整保留，地图源路径不变 |
| `docs/production/runtime-validation.md` | adapt | `docs/dev/validation/runtime.md` | 保留capture实际Viewer覆盖、命令、机器/看图边界、来源署名；具体旧运行结果仍在todo |
| `docs/production/solo-workflow.md` | semantic-split | `docs/dev/handbook/codex.md`<br>`docs/dev/handbook/tasks.md`<br>`todo/archive/legacy/solo-workflow.md` | 每次开始先知道在哪里/工作循环/作者与Codex分工/三个入口/行为验收/格式依据 → docs/dev/handbook/codex.md；三层粒度/进度计数/批次/状态与证据/命令 → docs/dev/handbook/tasks.md；原文完整旧工作流 → todo/archive/legacy/solo-workflow.md |
| `docs/production/wiki.md` | adapt | `docs/dev/handbook/wiki.md` | 改共享配置双profile，页面/搜索/JSON/public/dev server统一受众 |
| `docs/production/workflow.md` | adapt | `docs/dev/production/pipeline.md` | 阶段方法与内容制作保留，旧任务状态改链接新任务手册 |
| `docs/production/world-research.md` | move | `docs/dev/design/locations/world-research.md` | 14项研究及具体来源、观察/推断、适用方式逐项保留，作为空间规格依据 |
| `docs/public/images/hero.webp` | classify-public | `docs/public/images/hero.webp` | 只从已审查引用按受众复制；不默认复制public整个目录；当前无正文引用则不输出，保留源文件 |
| `docs/public/images/hillside-layout-concept.webp` | classify-public | `docs/public/images/hillside-layout-concept.webp` | 只从已审查引用按受众复制；不默认复制public整个目录 |
| `docs/public/images/hillside-reference.webp` | classify-public | `docs/public/images/hillside-reference.webp` | 只从已审查引用按受众复制；不默认复制public整个目录 |
| `docs/quests/QST-002/README.md` | semantic-split | `docs/dev/design/quests/QST-002/README.md`<br>`docs/player/encyclopedia/story/main/last-toy.md`<br>`todo/tasks/TASK-012-last-toy-mechanism-review.md` | 开头与## 制作入口 → docs/dev/design/quests/QST-002/README.md；## 故事与职责 → docs/player/encyclopedia/story/main/last-toy.md；## 当前验收 → todo/tasks/TASK-012-last-toy-mechanism-review.md |
| `docs/quests/QST-002/demo-scope.md` | semantic-split | `docs/dev/design/quests/QST-002/demo-scope.md`<br>`todo/tasks/TASK-012-last-toy-mechanism-review.md` | 全文范围与体验走查 → docs/dev/design/quests/QST-002/demo-scope.md；## 下一轮要验证的设计问题 → todo/tasks/TASK-012-last-toy-mechanism-review.md |
| `docs/quests/QST-002/development.md` | semantic-split | `docs/dev/design/quests/QST-002/development.md`<br>`todo/tasks/TASK-012-last-toy-mechanism-review.md` | ## Pitch；## Synopsis → docs/dev/design/quests/QST-002/development.md；## 本案的条件与规则至## 保存与恢复责任 → docs/dev/design/quests/QST-002/development.md；## 因果走查与下一步第一段 → docs/dev/design/quests/QST-002/development.md；## 因果走查与下一步第二段“下一步在M0-03” → todo/tasks/TASK-012-last-toy-mechanism-review.md |
| `docs/quests/QST-002/narrative.json` | move-with-consumers | `docs/dev/design/quests/QST-002/narrative.json` | 同一委托设计与数据唯一源，固定夹/供能/摆臂仍待原型验证，不声称运行已接入；JSON应逐字节不变 |
| `docs/quests/index.md` | adapt | `docs/dev/design/quests/index.md` | 制作目录与格式说明进入dev，现有百科故事通过链接引用 |
| `docs/quests/quests.json` | move-with-consumers | `docs/dev/design/catalogs/quests.json` | 18故事ID、BC、潜梦分工、角色和场所关联保持；不修改设计为已实现 |
| `docs/story/daily/after-the-late-show.md` | move | `docs/player/encyclopedia/story/daily/after-the-late-show.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/daily/collect-after-rain.md` | move | `docs/player/encyclopedia/story/daily/collect-after-rain.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/daily/courtside-chair.md` | move | `docs/player/encyclopedia/story/daily/courtside-chair.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/daily/different-dinners.md` | move | `docs/player/encyclopedia/story/daily/different-dinners.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/daily/index.md` | move | `docs/player/encyclopedia/story/daily/index.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/daily/new-pocket.md` | move | `docs/player/encyclopedia/story/daily/new-pocket.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/daily/photograph-today.md` | move | `docs/player/encyclopedia/story/daily/photograph-today.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/daily/unfinished-window.md` | move | `docs/player/encyclopedia/story/daily/unfinished-window.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/daily/ways-to-read.md` | move | `docs/player/encyclopedia/story/daily/ways-to-read.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/index.md` | move | `docs/player/encyclopedia/story/index.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/main/after-echoes.md` | move | `docs/player/encyclopedia/story/main/after-echoes.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/main/beyond-autumn.md` | move | `docs/player/encyclopedia/story/main/beyond-autumn.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/main/bring-today-back.md` | move | `docs/player/encyclopedia/story/main/bring-today-back.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/main/epilogue.md` | move | `docs/player/encyclopedia/story/main/epilogue.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/main/extra-applause.md` | move | `docs/player/encyclopedia/story/main/extra-applause.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/main/index.md` | semantic-split | `docs/player/encyclopedia/story/main/index.md`<br>`docs/dev/production/narrative.md` | 全文，除末段“重要事实至少有两种不同来源” → docs/player/encyclopedia/story/main/index.md；末段“重要事实至少有两种不同来源”至“主线唯一必需证据” → docs/dev/production/narrative.md |
| `docs/story/main/last-toy.md` | move | `docs/player/encyclopedia/story/main/last-toy.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/main/no-school-bell.md` | move | `docs/player/encyclopedia/story/main/no-school-bell.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/main/prologue.md` | move | `docs/player/encyclopedia/story/main/prologue.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/main/river-did-not-reverse.md` | move | `docs/player/encyclopedia/story/main/river-did-not-reverse.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/story/main/still-together.md` | move | `docs/player/encyclopedia/story/main/still-together.md` | 保留主线章序、完整因果、结局、显式qst锚点及角色选择；添加入口/章节剧透提示 |
| `docs/templates/asset-manifest.json` | move-with-consumers | `docs/dev/handbook/templates/asset-manifest.json` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/case-development.md` | adapt | `docs/dev/handbook/templates/case-development.md` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/character.md` | adapt | `docs/dev/handbook/templates/character.md` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/continuity-review.md` | adapt | `docs/dev/handbook/templates/continuity-review.md` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/daily-scene.md` | adapt | `docs/dev/handbook/templates/daily-scene.md` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/dialogue.csv` | move-with-consumers | `docs/dev/handbook/templates/dialogue.csv` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/enemy.md` | adapt | `docs/dev/handbook/templates/enemy.md` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/index.md` | adapt | `docs/dev/handbook/templates/index.md` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/location.md` | adapt | `docs/dev/handbook/templates/location.md` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/narrative-playtest.md` | adapt | `docs/dev/handbook/templates/narrative-playtest.md` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/narrative.json` | move-with-consumers | `docs/dev/handbook/templates/narrative.json` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/quest.md` | adapt | `docs/dev/handbook/templates/quest.md` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/templates/task.md` | adapt | `docs/dev/handbook/templates/task.md` | 保留数据模板原始语义，Markdown区分百科/规格/制作/任务职责；任务模板换新frontmatter |
| `docs/world/history.md` | move | `docs/player/encyclopedia/world/history.md` | 世界规则与历史全文保留；既有结局与知情边界不重写 |
| `docs/world/nightmares.md` | move | `docs/player/encyclopedia/world/nightmares.md` | 世界规则与历史全文保留；既有结局与知情边界不重写 |
| `docs/world/null-city.md` | move | `docs/player/encyclopedia/world/null-city.md` | 世界规则与历史全文保留；既有结局与知情边界不重写 |
| `todo/README.md` | generate-from-tasks | `todo/README.md` | 由任务卡生成，显示DOCS-PIPELINE与G0-G4分区，不以迁移计入游戏完成 |
| `todo/archive/daylight-visual.md` | archive | `todo/archive/legacy/daylight-visual.md` | 历史状态、作者授权、PASS/NOT RUN和未做项完整保留，仅更新迁移引用 |
| `todo/archive/godogen-adoption.md` | archive | `todo/archive/legacy/godogen-adoption.md` | 历史状态、作者授权、PASS/NOT RUN和未做项完整保留，仅更新迁移引用 |
| `todo/archive/map-viewer.md` | archive | `todo/archive/legacy/map-viewer.md` | 历史状态、作者授权、PASS/NOT RUN和未做项完整保留，仅更新迁移引用 |
| `todo/archive/n-district-closeout.md` | archive | `todo/archive/legacy/n-district-closeout.md` | 历史状态、作者授权、PASS/NOT RUN和未做项完整保留，仅更新迁移引用 |
| `todo/archive/n-district-modern-city.md` | archive | `todo/archive/legacy/n-district-modern-city.md` | 历史状态、作者授权、PASS/NOT RUN和未做项完整保留，仅更新迁移引用 |
| `todo/archive/n-district-space.md` | archive | `todo/archive/legacy/n-district-space.md` | 历史状态、作者授权、PASS/NOT RUN和未做项完整保留，仅更新迁移引用 |
| `todo/demo-progress.json` | archive-and-replan | `todo/archive/legacy/demo-progress.json` | 原始JSON冻结，61项逐项映射见legacy_work_items；不再作为活动状态源 |
| `todo/demo-progress.md` | archive | `todo/archive/legacy/demo-progress.md` | 旧生成看板冻结，仅用于历史，不与任务卡并行编辑 |
| `todo/demo-roadmap.md` | archive-and-replan | `todo/archive/legacy/demo-roadmap.md`<br>`todo/roadmap.md` | 完整原文 → todo/archive/legacy/demo-roadmap.md；M0→G0；M1/M2/M3→G1；M4→G2；M5→G3；M6/M7→G4 → todo/roadmap.md；城市与Viewer归档承接 → todo/roadmap.md |
| `todo/demo-worklog.md` | archive | `todo/archive/legacy/demo-worklog.md` | 保持显式锚点和全部真实反馈/测试范围；M0-01已验收事实保留，M0-02待审不被迁移验收 |
| `todo/evidence/city-closeout/before-shop.webp` | preserve-evidence | `todo/evidence/city-closeout/before-shop.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/city-closeout/before-street.webp` | preserve-evidence | `todo/evidence/city-closeout/before-street.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/city-closeout/bridge.webp` | preserve-evidence | `todo/evidence/city-closeout/bridge.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/city-closeout/campus.webp` | preserve-evidence | `todo/evidence/city-closeout/campus.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/city-closeout/cinema.webp` | preserve-evidence | `todo/evidence/city-closeout/cinema.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/city-closeout/hillside.webp` | preserve-evidence | `todo/evidence/city-closeout/hillside.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/city-closeout/inputs.sha256` | preserve-evidence | `todo/evidence/city-closeout/inputs.sha256` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/city-closeout/overview.webp` | preserve-evidence | `todo/evidence/city-closeout/overview.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/city-closeout/render.txt` | preserve-evidence | `todo/evidence/city-closeout/render.txt` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/city-closeout/shop.webp` | preserve-evidence | `todo/evidence/city-closeout/shop.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/city-closeout/street.webp` | preserve-evidence | `todo/evidence/city-closeout/street.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/bridge.webp` | preserve-evidence | `todo/evidence/daylight/bridge.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/campus.webp` | preserve-evidence | `todo/evidence/daylight/campus.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/cinema.webp` | preserve-evidence | `todo/evidence/daylight/cinema.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/contact-off.webp` | preserve-evidence | `todo/evidence/daylight/contact-off.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/contact-on.webp` | preserve-evidence | `todo/evidence/daylight/contact-on.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/eye-corner.webp` | preserve-evidence | `todo/evidence/daylight/eye-corner.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/eye-shade.webp` | preserve-evidence | `todo/evidence/daylight/eye-shade.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/eye-shop.webp` | preserve-evidence | `todo/evidence/daylight/eye-shop.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/hillside.webp` | preserve-evidence | `todo/evidence/daylight/hillside.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/inputs.sha256` | preserve-evidence | `todo/evidence/daylight/inputs.sha256` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/overview.webp` | preserve-evidence | `todo/evidence/daylight/overview.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/render.txt` | preserve-evidence | `todo/evidence/daylight/render.txt` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/shop.webp` | preserve-evidence | `todo/evidence/daylight/shop.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/street.webp` | preserve-evidence | `todo/evidence/daylight/street.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/taa-eye-shade.webp` | preserve-evidence | `todo/evidence/daylight/taa-eye-shade.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/taa-eye-shop.webp` | preserve-evidence | `todo/evidence/daylight/taa-eye-shop.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/taa-shop.webp` | preserve-evidence | `todo/evidence/daylight/taa-shop.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/taa-ssao-eye-shade.webp` | preserve-evidence | `todo/evidence/daylight/taa-ssao-eye-shade.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/taa-ssao-eye-shop.webp` | preserve-evidence | `todo/evidence/daylight/taa-ssao-eye-shop.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/daylight/taa-ssao-shop.webp` | preserve-evidence | `todo/evidence/daylight/taa-ssao-shop.webp` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/godogen/asset-validation.md` | preserve-evidence | `todo/evidence/godogen/asset-validation.md` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/godogen/runtime-validation.md` | preserve-evidence | `todo/evidence/godogen/runtime-validation.md` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/godogen/skill-forward-probe.md` | preserve-evidence | `todo/evidence/godogen/skill-forward-probe.md` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/godogen/skill-validation.md` | preserve-evidence | `todo/evidence/godogen/skill-validation.md` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/setting-v1/integration.md` | preserve-evidence | `todo/evidence/setting-v1/integration.md` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/setting-v1/source-continuity.md` | preserve-evidence | `todo/evidence/setting-v1/source-continuity.md` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/setting-v1/source-decisions.md` | preserve-evidence | `todo/evidence/setting-v1/source-decisions.md` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/setting-v1/source-manifest.json` | preserve-evidence | `todo/evidence/setting-v1/source-manifest.json` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/setting-v1/source-validation.json` | preserve-evidence | `todo/evidence/setting-v1/source-validation.json` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/evidence/workflow-integration.md` | preserve-evidence | `todo/evidence/workflow-integration.md` | 保留历史证据内容及图片/哈希路径；Markdown引用可按映射修复，inputs.sha256保持原输入哈希不重新计算 |
| `todo/map-viewer-art-reference.md` | archive | `todo/archive/legacy/map-viewer-art-reference.md` | 一次性官方图源观察与转化历史全保留；有效视觉选择已在art-direction，reference-analysis引用本实例，不重复生成新事实 |
| `todo/map-viewer-assets.md` | archive | `todo/archive/legacy/map-viewer-assets.md` | 历史候选、许可取得条件与已选子集保留；实际唯一源仍environment-kit/README和manifest；长期规则引用asset-pipeline |
| `todo/n-district-questionnaire.md` | adapt | `docs/dev/decisions/district-baseline.md` | 长期已定布局/店铺关系/地图选择进入decisions；首轮数量明确为历史起案不覆盖当前地图；未决条目关联G1/G2 |

## 旧 61 项逐项处理

旧计划退出活动状态源，保留每项交付与验收原文供后续展开；不保留原分母，不把本次迁移转成游戏成果

| 旧项 | 处理 | 去向 | 依据 |
| --- | --- | --- | --- |
| M0-01 现状、已有约束与体验支柱 | decision | `docs/dev/direction/demo-scope.md`<br>`todo/archive/legacy/demo-worklog.md#m0-01-accepted` | 作者真实确认的基线写回方向；不建新done卡片、不重新计数 |
| M0-02 首委托与玩家因果链 | task | `todo/tasks/TASK-012-last-toy-mechanism-review.md` | 沿已选精简方向续接固定夹/供能/运输/现实确认的待作者判断，不重选首委托 |
| M0-03 可操作角色、动作词表与镜头假设 | merge | `todo/tasks/TASK-013-demo-action-scope.md` | 依据委托真实操作收口人物/室内/路线/交互范围，不将25—40分钟等建议直接批准 |
| M0-04 可玩空间、时长与内容预算 | merge | `todo/tasks/TASK-013-demo-action-scope.md` | 依据委托真实操作收口人物/室内/路线/交互范围，不将25—40分钟等建议直接批准 |
| M0-05 UI 信息架构与操作线框 | task | `todo/tasks/TASK-015-investigation-ui-flow.md` | 只验信息架构与双输入路径；M2-04实现和手感仍留G1门槛 |
| M0-06 美术、动画与声音生产方案 | task | `todo/tasks/TASK-017-production-baseline.md` | 已有静态城市基线继续复用；实际角色/动画/声音小样才能锁定生产来源与成本 |
| M0-07 技术、存档与双平台验证基线 | task | `todo/tasks/TASK-018-platform-save-baseline.md` | 明确参考设备和离线发布/存档约束，具体实现和双平台实测仍由G1/G4证据证明 |
| M0-08 M0 范围基线验收 | milestone | `todo/roadmap.md#g0` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M1-01 正式入口、输入与最小运行骨架 | merge | `todo/tasks/TASK-014-human-scale-prototype.md` | 同一真实路线验证输入→移动/碰撞→镜头假设，保留可重复比较，不等待全部M0或重造地图 |
| M1-02 人物移动与碰撞手感 | merge | `todo/tasks/TASK-014-human-scale-prototype.md` | 同一真实路线验证输入→移动/碰撞→镜头假设，保留可重复比较，不等待全部M0或重造地图 |
| M1-03 跟随镜头、室内与遮挡 | merge | `todo/tasks/TASK-014-human-scale-prototype.md` | 同一真实路线验证输入→移动/碰撞→镜头假设，保留可重复比较，不等待全部M0或重造地图 |
| M1-04 室内灰盒、入口与进出生命周期 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M1-05 交互提示、HUD、暂停与焦点 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M1-06 代理人物、基础动画与交互反馈 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M1-07 双平台双输入与恢复检查 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M1-08 M1 人尺度游玩验收 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M2-01 任务、认知与世界状态契约 | merge | `todo/tasks/TASK-016-state-recovery-probe.md` | 最小真实状态路径验证设计数据与运行语义、幂等和保存，不把完整M2/M3作为笼统前置 |
| M2-02 店务与邻里日常片段 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M2-03 现实调查与线索顺序 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M2-04 对话、调查记录与物品 UI | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M2-05 时段、NPC 活动与重访 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M2-06 存档、幂等与任务恢复 | merge | `todo/tasks/TASK-016-state-recovery-probe.md` | 最小真实状态路径验证设计数据与运行语义、幂等和保存，不把完整M2/M3作为笼统前置 |
| M2-07 现实片段的表演、字幕与声音 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M2-08 M2 日常调查端到端验收 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M3-01 现实与梦境对应规则 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M3-02 基础动作、瞄准与手感 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M3-03 敌人行为、预警与对策 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M3-04 关键工具与异常处理机制 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M3-05 兄妹接应与 Agent 支援 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M3-06 入梦、退出、失败与恢复 | merge | `todo/tasks/TASK-016-state-recovery-probe.md` | 最小真实状态路径验证设计数据与运行语义、幂等和保存，不把完整M2/M3作为笼统前置 |
| M3-07 行动 UI、动画、预警与声音 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M3-08 M3 潜梦行动验收 | milestone | `todo/roadmap.md#g1` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M4-01 完整委托、场景与正式台词 | milestone | `todo/roadmap.md#g2` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M4-02 首尾完整的整合灰盒 | milestone | `todo/roadmap.md#g2` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M4-03 路线、节奏与回家体验 | milestone | `todo/roadmap.md#g2` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M4-04 角色与场景品质样板 | milestone | `todo/roadmap.md#g2` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M4-05 统一 UI／UX 设计与组件 | milestone | `todo/roadmap.md#g2` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M4-06 表演、音乐、音效与混音样板 | milestone | `todo/roadmap.md#g2` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M4-07 样板成本、预算与制作规则 | milestone | `todo/roadmap.md#g2` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M4-08 M4 样板验收与生产放行 | milestone | `todo/roadmap.md#g2` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M5-01 内容／资产清单与制作批次 | milestone | `todo/roadmap.md#g3` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M5-02 核心关卡与环境内容生产 | milestone | `todo/roadmap.md#g3` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M5-03 角色、NPC 与动画生产 | milestone | `todo/roadmap.md#g3` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M5-04 日常、教学、可选互动与后续 | milestone | `todo/roadmap.md#g3` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M5-05 完整 UI 页面与全部状态 | milestone | `todo/roadmap.md#g3` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M5-06 正式声音、特效、字幕与演出 | milestone | `todo/roadmap.md#g3` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M5-07 Alpha 集成与内容完整性检查 | milestone | `todo/roadmap.md#g3` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M5-08 M5 Alpha 验收与内容冻结 | milestone | `todo/roadmap.md#g3` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M6-01 试玩问题、路线与测试安排 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M6-02 陌生玩家理解与关卡修订 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M6-03 动作、难度、节奏与混音修订 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M6-04 UI／无障碍与设备切换 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M6-05 真实性能、加载与双平台兼容 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M6-06 回归、边界与存档完整性 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M6-07 M6 Beta 验收与候选放行 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M7-01 分发、资源定位与用户数据目录 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M7-02 首次启动、产品外壳与存档兼容 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M7-03 许可、署名、说明与展示资料 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M7-04 固定 Release Candidate 与校验包 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M7-05 干净环境与发布包实机验收 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |
| M7-06 M7 发布批准与 Demo 交付 | milestone | `todo/roadmap.md#g4` | 尚未到可给出真实预算与当前动作的阶段；保留成果/验收门槛和重大风险，达到前置结果后再展开卡片，不维持旧固定分母 |

## 近期任务建议

TASK-006—TASK-011 专用于本轮 R0—R5；以下只续接已有真实待办，交付与实际状态由 R4 任务卡最终确定

| ID | 范围 | 当前建议状态 | 真实依赖结果 |
| --- | --- | --- | --- |
| TASK-012 | 《最后的玩具》操作因果与恢复假设走查 | review | 已有规格与作者选择，可立即继续走查 |
| TASK-013 | Demo 最小动作、主控与可达内容范围 | backlog | TASK-012 |
| TASK-014 | 正式入口的人尺度移动与镜头实验 | backlog | TASK-013 |
| TASK-015 | 调查事实、推测与返回路径线框 | backlog | TASK-012 |
| TASK-016 | 提前找到玩具的事实、认知与退出恢复探针 | backlog | TASK-012 |
| TASK-017 | 代表角色与动作声音的生产成本小样 | backlog | TASK-013 |
| TASK-018 | 平台、用户数据与保存验证边界 | backlog | TASK-013 |

## 必须同步的消费者

- tools/validate_docs.py及其tests：目录/身份字段/模板/历史引用
- tools/validate_narrative.py及tests：dev/design/quests扫描和模板位置
- tools/validate_story_design.py及tests：四catalog路径、source_file根、subject_id
- tools/wiki-data.mjs及wiki.test.mjs：受众同源规则、JSON/CSV/public/project-assets
- docs/.vitepress/config.mjs/sidebar.mjs/components：双构建目录/导航/搜索/显式投影
- AGENTS/README/game README/tools README/package.json/.github工作流
- 本地nside/guide/work-loop/review/create-case/create-content/review-narrative Skills及manifest hashes
- source-assets各README及外部索引：旧文档相对链接
- 旧roadmap.py和tests退出活动命令，新tasks.mjs和tests接管状态
- todo历史文本和hash证据：引用修复与原始校验口径分开

## 不能机械移动的内容

- 操作页没有实际可用按键，仅进入开发动作规格；当前可玩版本指南不能将设计文案改写成已实现
- QST-002 的当前验收、轮次和下一步转任务卡，规则及恢复矩阵保留；故事事实由百科维护
- 玩法制作按操作、关卡、战斗、日常、状态、敌人与委托约束分拆，不能将整个大页换一个目录
- 美术页的视觉方向、具体样板要求与资产格式分别归位，不以概述替换原详细规范
- `characters.source_file` 相对仓库根，`quests.source_file` 相对 docs；移动后消费者必须保持各自真实语义
- 玩家地图不仅修改下载接口，还需检查组件静态import、客户端bundle及开发服务器；完整district源不能因去掉侧栏而隐身
- 历史哈希说明当时输入，不因迁移重新计算冒称旧运行基于新文件；旧引用通过新旧映射追溯

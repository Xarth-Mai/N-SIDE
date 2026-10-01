# Blender r2 运行接入独立审查

复读 Root 当前 `scene.rs`、`collision.rs`、`app.rs`、`player.rs`、`places/observation.rs`、`map_viewer.rs` 与 `appearance.json` 差异，完成正确性和 ponytail-review 两个独立检查面；未运行 Cargo、Blender 或 GPU，未修改运行代码

审查最终差异 SHA256 `92f7e911952c1864ff317d840bcb56e63e55695cb0059fa08279fc6b681a71c8`；`scene.rs` SHA256 `86257f3dac82a09159eb217dde9db418ac5b6b038d06959230bed29a265fd5c8`，`collision.rs` SHA256 `7c3dfcf71905234c1ff7eb3f0e22656ccb9a2dca5a88f4a445955c6c2046a008`

## 初始发现与解决

- P2：森林保持测试仅按 `MapSource` 查找旧实例；V-A08 屋檐和立面共享建筑来源后，第二个模型误匹配第一个，完整 lib 日志实际复现失败；Root 改为按来源及 model 配对，保留原 transform 一致性断言，来源继续表达建筑归属，没有扩展成新的 ID 框架
- P2：泛型住宅玻璃的覆盖期望和窗饰数量仍含已由 Blender 接管的 V-A08，完整 lib 日志分别复现失败；Root 保留其余九栋检查，由已有 GLB 包络／路线测试承接 V-A08；随后删除 `residential_sample` 中运行已不可达的 V-A08 项，移除相应补偿过滤，窗饰数精确改为九栋
- 文档：旧 V15／V-A08 文字及一条测试注释仍把新模型描述为纯视觉；已同步本轮三资产 README、manifest 和制作证据为实际 GLB 三角碰撞，原 V-A08 独立屋檐仍仅可视，没有改变冻结几何、脚本或模型源 hash

初始 `lib-tests.log` 是 137 passed／3 failed／1 ignored，三项失败均有明确对应修复；Root 的 `scene-tests-final.log` 为 29 passed／0 failed。其后白名单简化只删除已被外层接管分支排除的死项，最终 build 由 Root 执行；本审查不将初次完整 lib 记录改写成全通过

## 正确性核对

`CollisionWorld::from_scene` 只导入 `v_a08_facade`、`v15_mirror_hall_facade`、`street_bench`，三者共 27,328 个静态三角；没有顺带给树、草或独立屋檐添加碰撞。模型实际位置按 `prop.transform × Scale(spec.scale) × node.transform` 计算，与 `scene::spawn` 给模型根设置的父变换等价，GLB 已是 Y-up，不再次翻轴；长椅来源位于既有 `/surfaces/` 结构范围，建筑模型的 `/collision/` 后缀进入显式建筑来源范围

加载诊断保留 source、model、文件及 Scene 编号；单根节点、无子节点／skin／animation、Triangle primitives 和无 morph 的契约直接拒绝不支持的输入，位置与索引读取限内嵌 BIN，随后复用 `from_parts` 的有限坐标、索引边界、u32 容量与零面积三角检查。没有建立通用 glTF 物理导入框架，实际碰撞分件的来源带 model 和 primitive 编号，来源区间保持 TriMesh 输入三角顺序

V15 只替换南／西三层窗及29m／33m层带，并继续移除原公共入口雨棚；北／东窗、原门框与中梃、主体和屋顶保持。`RF` 已使原泛型 coping 不生成，因此没有虚报其移除数；真实生成器比较记录 V15 相对完整泛型方案移除1,236三角。V-A08 四面泛型接管与资产合同一致，独立屋檐继续存在

新长椅实际世界包络为 `[311.04,37,-237.335197]` 至 `[312.96,37.866914,-236.664803]`，位于镜厅原屋顶范围内；新碰撞窄测读取转换后的真实两足顶点，查询原屋面支承来源与37m高度，另以胶囊验证座前阻挡及建筑覆层比旧壳更早命中，不只复述放置数组。完整 lib 日志中该碰撞测试已通过

`app.rs` 实际加载与现有玩家／观察测试改用 `from_scene`，避免游戏与回归继续使用少了新资产的旧碰撞。Viewer 仅增加固定机位和地图锚点推导，既有 camera_views 有限变换及单位 scale 测试可覆盖其结构，不需要另写逐个坐标复述测试

## 文档及资产保持检查

三模型正式 GLB 与登记 hash 一致，V15／V-A08 主文件及 V15 构建／检查脚本保持冻结 hash，本次说明修订未写入任何资产脚本；长椅源 GLB 与运行 GLB 相同。仅更新 V15 文档自身在 `frozen-inputs.json` 中的记录，未修改模型数据

`code-review-docs-check.log` 为全仓文档与 Skills 检查 PASS，局部 `git diff --check` PASS；`code-review-export-check.log` 核对新增长椅放置说明未改变既有导出产物

Root 的 Wiki 构建先遇到地形测试计时超限，已安排 CPU 空闲时复验原命令；该尚待复验门禁独立于本次代码正确性结论，未要求放宽断言或超时。实机街景、贴墙路径、长椅足底观感和作者美术验收由 Root 的 GPU 证据承担

## Ponytail

实际删除了失效白名单项和补偿过滤；保留窄测、严格资产错误与来源诊断，没有新增 schema、依赖、配置层或单实现通用接口。复读最终差异后没有剩余有效简化或具体运行 bug

Lean already. Ship.

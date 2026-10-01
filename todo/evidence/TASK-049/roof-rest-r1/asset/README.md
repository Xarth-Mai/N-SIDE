# RF 条栅遮阴架源制作

Root 批准在镜厅公共屋顶休息岛内制作两后柱悬挑遮阴，继续复用 `[312,237,37]` 的已有公共长椅；本轮新增独立[源资产包](../../../../../source-assets/environment-kit/street-furniture/roof-shade/README.md)，源制作后获授权在 AST-003 登记、复制运行导出并添加 `appearance.models.roof_shade`。没有修改 R5 冻结的 V-15、既有bench、scene或地图；场景与地图由主线程分工接入

## 已制作内容

真实 Blender 5.2.2 LTS 源工程有28个可编辑mesh：2柱、2脚板、2后撑、1后横梁、3悬臂、10木条、8底部固定件。前三侧开敞，没有增加整片封闭屋面、室内、咖啡经营或植物状态；使用已有CC0木色图与scale035法线，金属沿用公共灰绿色PBR参数，长椅没有复制进GLB

源主文件 SHA-256 为 `2c0cbcbe7078ff053936308231e10b3e4e5b7574594ab5b798e7ebd9c633c0ad`，源导出 GLB 为 `fc2eddc389d472ea7be3f05e7354d356d7b8cdc12418429725bbccfab740828b`；实际1,104三角、2,612顶点、2材质、2图、1,514,200bytes，低于本次1,800三角／2材质预算

## 实际命令与结果

```fish
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/environment-kit/street-furniture/roof-shade/build.py -- --rebuild
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python todo/evidence/TASK-049/roof-rest-r1/asset/inspect_source.py
python3 -B source-assets/environment-kit/street-furniture/roof-shade/check.py
```

三个命令均实际 exit0，前两项在宿主单进程2线程执行，已与角色 r13 制作错开并释放窗口。[build.log](build.log)记录首次制作；[source-check.log](source-check.log)与[source-check.json](source-check.json)记录保存后重开、packed图片／颜色空间／Strength1及复导出完整字节相同；复验没有保存或改写源主文件。[geometry-check.json](geometry-check.json)为系统Python读取实际GLB的检查结果

导出器启动时报告可选MeshOptimizer共享库不可用；本包采用未压缩GLB，没有请求Draco/Meshopt，没有glTF扩展，实际读回三角与图片全部通过。保留原日志，不把缺少可选压缩库写成已启用功能

## 几何与边界

地图锚点为 `[312,237,37]`，总包络 x309.98..314.02、north235.90..238.46、h37..39.52。柱后侧布置，前方悬臂底高39.345m；短撑最低约38.896344m，位于座椅后侧。部件连接和独立实际几何复核见 [geometry-review.md](geometry-review.md)

实际遮阴三角分别与已装bench完整包络比较，全部分离；前方及左右1.85m高接近体内无遮阴三角。整个冠部有意位于座位上方，不能把整件AABB重叠误作穿插。六条既有RF相关路段按实际中心线、宽度和组件投影计算，最近路带边界距离约11.997m；结果只说明本独立组件的位置，没有代测人物控制器或全屋顶体验

QST-025 明确保留未种格、照料分配与取水交班。本轮地图新增实体草地预留区 `cinema-roof-planting-reserve`，检查直接读取其 x329..336、north228..236、elevation37 的源区划；实际遮阴包络距离其边界14.98m。本模型不创建植物或改故事状态，实体预留区不表示任务新增数字规则

新增区划检查首次因旧 surface 没有 `id` 字段而报 `KeyError: 'id'`，已改为通过可选 `id` 查找新增对象；复验 exit0，没有修改地图或放宽间隔条件

## 登记与运行导出

`bun tools/export-environment.ts` 与 `--check` 实际 exit0，结果均为38个运行文件、96,984,686bytes，见 [export.log](export.log)和[export-check.log](export-check.log)。AST-003 只增加一个原创混合许可条目，木材沿用既有CC0输入，新增第三方来源为0；运行副本完整字节等于源GLB。[registration-check.json](registration-check.json)核对旧37个文件哈希、旧登记字段和旧appearance语义均保持，appearance保留手工格式，仅新增5行模型绑定

## 验收与交接

技术源检查与运行复制／绑定 PASS，整体状态仍为 `needs_revision`。源制作阶段未另做CPU渲染；随后已完成真实场景整合与四组GPU复验，420帧和22项原生／22项包装断言通过，实际画面自查见[总记录](../review.md)。真实胶囊与模型脚点已检查，真人通行与作者外观验收尚未发生

源制作阶段没有产生临时媒体；随后运行的临时画面由Root在实际查看后统一清理，结论与hash归总记录。正式源文件、GLB、脚本、许可引用、日志和JSON保留，当前屋顶模型不计作作者已验收的美术效果

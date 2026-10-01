# 公共室外空调源包

沿用 AST-003，[Exterior Aircon Unit](https://polyhaven.com/a/exterior_aircon_unit) 由 Monsta3D 制作，Poly Haven 于 2023-08-16 发布；2026-10-01 取得 clean 单机所需原件，费用 0，依据[官方许可](https://polyhaven.com/license)使用 CC0-1.0，完整条款复用 [CC0-1.0.txt](../licenses/CC0-1.0.txt)

本目录已登记 AST-003 并导出完整装配，资产状态为 exported；`scene.rs`／`appearance.json` 已绑定首处实例，`CollisionWorld::from_scene` 已接纳实际 GLB 三角，R3 局部 GPU 短片和收水口支承窄测通过；作者验收仍待完成。唯一环境资产清单仍为 [AST-003](../asset-manifest.json)；[provenance.json](provenance.json)保存本件的取得凭据、原始文件 hash、已选候选版本及完整装配证据入口，正式登记时引用这些凭据，不另建总台账

## 原件与可编辑候选

- `original/` 原样保存官方 glTF、共用 bin 和 clean 单机的 7 张 1K JPEG；原 glTF 还声明 rusted 变体及其未下载贴图，不能把这个目录称为完整上游双机离线包
- `aircon-info.json` 与 `aircon-files.json` 是当时官方 API 原始响应，记录作者、版本标记与文件 MD5；`provenance.json` 只列实际保存的 9 个原件，合计 4,489,351 bytes
- `aircon-candidate.blend` 是选出的可编辑主文件，所用图片已 packed；`aircon-candidate.glb` 为该主文件的静态导出，保留作无安装件的来源对照；运行交付使用下方完整装配
- 仅采用 `exterior_aircon_unit` 节点，保持原米制几何和 UV，平移底部枢轴并烘焙变换。上游格栅材质把无 alpha 的 opacity JPEG 当颜色使用，候选接回官方 `02_diff` 并用 `01_opacity` 生成 RGBA / MASK；原件保持不变
- `prepare.py` 复用此前候选制作方法，路径已改成本目录、仅处理 clean 单机。它从原件重建主文件，会覆盖后续手工修改，当前只做语法检查，尚未在迁入后重新运行 Blender

## 原单机契约

| 项目 | 已查值 |
| --- | --- |
| 几何 | 9,493 三角、6,872 顶点行、2 材质，未缩放 |
| 宽 × 深 × 高 | `0.799971 × 0.374159 × 0.927897 m` |
| GLB 包围盒 | `[-0.399986,0,-0.187080]` 至 `[0.399986,0.927897,0.187080]` |
| 坐标与枢轴 | `Scene0` / 单根 / identity，+Y 向上、+Z 为正面；最低点是下垂软管末端，不能当作机壳底 |
| 图像 | 6 张内嵌 1K 图；颜色 sRGB，法线和 ARM 线性，格栅 RGBA / MASK cutoff 0.5 |
| ARM | 保留原接法，没有 `occlusionTexture`，不宣称红通道 AO 已启用 |
| 文件 | GLB 4,983,300 bytes；SHA-256 `fae1a9f45ccebea4eef64f675f67e0913473614f5a805abea5d1a09e9537ebb5` |

## 墙挂装配与首处位置

首处代码实例位于真实 `V-W10` 音乐街普通商住楼的西墙，保留刚完成的 V-A08 三台原创空调。该建筑西墙 X=568、南北 78–100、首层 12.666667–16.266667m，已有局部砖饰面；无安装件单机的初步枢轴位置为地图 `[567.79,95.6,15.17]`、绕世界 Y 转 `-90°`，风扇朝西，完整模型外包络约 X=567.603–567.977、南北95.2–96.0、高15.17–16.097m

上述是单机枢轴坐标，完整装配改用地面锚点 `[567.79,95.6,12.666667]`，单位缩放和 `-90°` 朝向保持；此地面锚点已在 `props()` 以 `aircon_wall` 和来源 `buildings[V-W10]/aircon-wall` 绑定：两侧首层窗中心约93.4/97.8，空调置于窗间；首层公共门在89附近，既有西侧步行路径中心在556，安装不应占通行净空。R3 实机第29／49帧可见背架、回墙管、排水与地面收水口；该画面自查不替代接点微小间隙、各方向遮挡及人物通行检查

原件近图与实际顶点核对表明，侧阀口和控制线已经汇入背面的包覆管，装配只延伸真实包覆管末端入墙，不在侧面重复接一套管。以下均为原单机 GLB 局部坐标，安装时统一加高 `2.503333m`

| 接点 | 实测与新构件 |
| --- | --- |
| 原软管末圈 | 中心约 `[0.05780,0.00110,-0.01680]`，接直径24mm套口，转入直径20mm贴墙 PVC 管 |
| 原包覆管口 | 圈包络 X `[-0.012439,0.050649]`、Y `[0.414325,0.477412]`、Z `[-0.187080,-0.170698]`；中心 `[0.019105,0.445869]` 接回墙短套管及分块收口板 |
| 原背架 | 两背板约 X `-0.26577/0.25120`、Z `-0.176136`，新增薄垫块延至砖面 `Z=-0.198`；机壳与支架保持原形 |
| 竖管与收水口 | 竖管中心 Z `-0.164`，三处管卡接墙；底部接入有实体底、侧壁、留管孔盖格的 `0.20×0.195×0.18m` 收水口，底面为地面枢轴 |

`wall-installation.blend` 是完整安装的可编辑主文件，原 CC0 单机为独立对象，新增组件分别可编辑；`mount.py` 从原单机主文件重建或从现有安装主文件导出，不修改官方原件和既有候选。原模型9,493三角与 UV 保留，六张已检查的内嵌图逐字节保留，Blender重新导出使单机壳体法线方向最大变化约0.039°；原创安装件624三角，总计10,117三角、5材质、6张内嵌1K图

源 GLB 为 `wall-installation.glb`，沿既有导出器生成 `game/assets/environment/street-furniture/aircon-wall.glb`，完整包围盒为 `[-0.399986,0,-0.235]` 至 `[0.399986,3.431230,0.187080]`，GLB 为5,027,140 bytes。短套管末端进入既有墙体约37mm，属于埋入墙面的连接；原单机朝向和尺度不变。整个成品的文件许可字段为项目 MPL-2.0，清单单独保留原单机/纹理 CC0-1.0 与新增几何的分工，原件许可不变

地面收水口不是高挂装饰，当前世界包络约 X `567.798–567.993`、南北 `95.4422–95.6422`、高 `12.666667–12.846667`。装配实际几何已列入既有导入碰撞白名单；最终 CPU 窄测从实际导入三角取收水口底部顶点，确认其与运行地面支承相差小于0.03m。该检查没有覆盖人物撞击收水口或沿墙行走，低位阻挡与人物通行仍需相应路径验证。没有排水模拟、维修交互或室内系统，当前完成的是可见安装关系

## 检查与恢复

仓库根目录执行，fish 可直接使用；检查入口只读，Blender 仅在协调后的单实例 CPU 生产窗口执行

```fish
python3 -B source-assets/environment-kit/aircon/check.py
python3 -B source-assets/environment-kit/aircon/mount-check.py
# 默认从当前安装主文件导出，只有重建初始安装才附加 -- --rebuild
env ALSOFT_DRIVERS=null timeout --signal=TERM --kill-after=5s 180s blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/environment-kit/aircon/mount.py
# 从当前安装主文件做中性背景 CPU 检视
env ALSOFT_DRIVERS=null timeout --signal=TERM --kill-after=5s 180s blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/environment-kit/aircon/mount.py -- --preview
bun tools/export-environment.ts
bun tools/export-environment.ts --check
```

重建后先检查真实产物并更新既有导出版本 hash，再登记 AST-003 和使用环境导出器，不直接修改运行 GLB。`check.py` 核对官方 API MD5/尺寸、原件 SHA、候选 hash、有限属性、单位法线、有效索引、退化三角、尺度与实际绑定的格栅 alpha，报告输出到 stdout，不重写源文件

原候选已在 Blender 5.2.2 LTS / Cycles CPU 查看正背面，细格栅、风扇、壳缝和背架可辨；原后台进程曾在音频退出阶段挂起，由宿主确认后终止，不能记为正常 exit 0。本轮安装使用安全命令、CPU 2线程完成原件3张接点近图、源装配导出和3张安装图，三个进程均正常 exit 0 并回收；R3 已补局部 Bevy 画面观察，作者尚未验收，格栅远近闪烁、帧成本及人物路径仍待对应验证

R3 的 `aircon-wall` 实机短片采集通过，主任务实际查看第29／49帧，背架、管路与收水口可见；范围与边界见[视觉记录](../../../todo/evidence/TASK-049/blender-integration-r3/visual-review.md)及[集成记录](../../../todo/evidence/TASK-049/blender-integration-r3/review.md)，支承检查见[最终碰撞窄测](../../../todo/evidence/TASK-049/blender-integration-r3/scene-models-final.log)

本轮源装配、接点近看、导出与复验记录已按原字节迁入 [public-aircon-r1](../../../todo/evidence/TASK-049/public-aircon-r1/review.md)，逐文件迁移及空占位记录见 [migration.json](../../../todo/evidence/TASK-049/public-aircon-r1/migration.json)；历史日志保留当时的 `output/assets/public-aircon-mount-r1` 路径，脚本未来仍可在该临时目录生成预览，不将路径迁移误记为重新建模

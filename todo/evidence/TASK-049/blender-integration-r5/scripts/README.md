# R5 后勤前场实拍计划

本页保存运行前计划与最终复现入口，执行结果见 [R5汇总](../review.md)。复用现有 Viewer、真实场景和 [capture入口](../../../../../tools/capture.py)，不修改场景位置、人物出生点或受验收结果

| 脚本 | 机位与输入 | 检查画面 |
| --- | --- | --- |
| [inspect-cinema-service-court.json](inspect-cinema-service-court.json)，120帧/4s | `[309,279,31] → [321,258,26.7]`，高侧向自由诊断机位；30–33帧 A，60–63帧 D，90帧 Escape | 29/49/79/119及横移连续帧：后场边界、原服务路、中央卸货净空、北墙设备和路灯落位 |
| [eye-cinema-service-aircon.json](eye-cinema-service-aircon.json)，180帧/6s | `[323,254,26.725] → [324,250.203,26.55]`，30–33帧 A、60–63帧 D；85–104帧鼠标下看，125–144帧复位，160帧 Escape | 29/49/79上部装配与墙窗关系，104/124及连续103–105看回墙管、管端和收水口，149/179恢复上下文 |
| [镜厅材质脚本](../../blender-integration-r2/captures/cinema-facade-retry/script.json)，150帧/5s | `inspect-cinema-facade`，30–33帧 W、60–63帧 A、90–93帧 D，120帧 Escape | 原有接近与横移路径中的灰泥、混凝土、木饰面和保留的海报 |
| [va08-normal.json](va08-normal.json)，120帧/4s | `eye-va08-east`，30–33帧 A、60–63帧 D，90帧 Escape | 住宅立面新法线的横移稳定性；使用本轮真实输入，未复用旧60帧固定截图作为移动证明 |
| [V-55材质脚本](../../../TASK-045/env-reflection-r1/v55-west.json)，120帧/4s | `eye-v55-west`，30–33帧 A、60–63帧 D，90帧 Escape | 包装修补店灰泥／混凝土与窗框层次；只复用脚本，不代表保留反射候选B |
| [角色动作](../../../../../game/capture/walk-character.json)，540帧/18s | 正式 `n-side --character-preview CHR-001`，真实 Jump、Walk、Run及暂停恢复 | r12袖筒动作回归；脚本含3次R恢复输入，不是零重置连续路线 |

坐标采用地图 `[东西,南北,高程]`。依据[实际GLB包络](../../cinema-service-court-r1/props-report.json)，空调装配高25–28.431m，收水口在25m；近景相机高26.725m，与墙约3.8m，初始向下约2.55°，装配上下边缘接近55°竖直视场的边界。只做横移难以充分观察地面接合，故复用 [R3 空调下看](../../blender-integration-r3/scripts/aircon-wall.json)范式，原地向下0.4rad后复位，不向闭门或墙面前冲

鼠标序列为20帧×20dot×当前 `FreeCamera.sensitivity 0.18`／180=0.4rad，依据当前安装的 Bevy 0.19.1 `free_camera.rs`，不是正式角色鼠标系数。各横移仍核对真实1–3m位移；俯仰要求0.38–0.42rad、位置稳定，最终停止检查朝向、位置、控制器关闭和实体数量。资源就绪、有限 Transform 与全帧落盘沿用原生检查

这两段只能证明场景渲染与 Viewer 相机输入；原服务道路、GLB支承和闭门接近由主任务 CPU 窄测判断。中央净空是设计使用范围，本片不会临时生成推车、车辆或人物表演

## 复现与记录

仓库根目录执行，fish可直接运行；主任务先完成本批构建和场景接入，输出目录必须不存在

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-049/blender-integration-r5/scripts/inspect-cinema-service-court.json --output output/blender-integration-r5/inspect-cinema-service-court
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-049/blender-integration-r5/scripts/eye-cinema-service-aircon.json --output output/blender-integration-r5/eye-cinema-service-aircon
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-049/blender-integration-r2/captures/cinema-facade-retry/script.json --output output/blender-integration-r5/cinema-normal
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-049/blender-integration-r5/scripts/va08-normal.json --output output/blender-integration-r5/va08-normal
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-045/env-reflection-r1/v55-west.json --output output/blender-integration-r5/v55-normal
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-001 --no-video --script game/capture/walk-character.json --output output/blender-integration-r5/yao-motion-retry
```

记录实际版本、资产hash、原 run／script／state／runtime.log，机器检查与实际看图分开；视频为明确的 `NOT RUN: --no-video`，需看关键帧及下看／横移连续帧。运行完成后归档非媒体记录，大于500,000字节的原文件沿R4使用gzip mtime=0并保存原始及存储hash；主任务完成观察后统一清理本轮截图

准备阶段 Bun 对照当时的 Rust 字段，检查前2份脚本、10段有序输入、8条断言的字段及帧范围、2个已登记机位、5个文件引用和0.4rad输入计算，结果 `PASS`；该历史静态结果不代替后续新增脚本的实际运行。`--aa` 仅供district Viewer使用，角色首次误传导致启动前失败，原失败记录保留在 [R5归档](../captures.json)，上述retry命令已修正

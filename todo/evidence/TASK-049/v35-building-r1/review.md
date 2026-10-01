# V-35 北立面候选检查

状态：源制作、数值检查与图片 self-audit 完成，root 看过三图并同意进入 R4 引擎整合；作者美术验收与真实引擎路线仍待完成

## 检查

| 实际操作 | 结果与边界 |
| --- | --- |
| `blender --background --threads 2 --python-exit-code 1 --python source-assets/buildings/V-35/build.py -- --rebuild --render` | 三张 960 × 540、16 samples、CPU 双线程预览完成，源模型与 GLB 已写入；沙箱退出阶段卡在音频清理，未将该进程的退出记为 PASS |
| `python source-assets/buildings/V-35/check.py` | PASS，3,612 三角面、7,176 顶点、七材质、四张原字节 PBR 图；有限属性、单位法线、绕序、米制 UV、源尺寸、招牌留空及 41 个入口样本通过 |
| `blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/buildings/V-35/build.py`，宿主执行 | PASS，实际退出码 0；导出编辑主文件，不重建源模型 |
| `env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/buildings/V-35/check.py -- --source`，宿主执行 | PASS，实际退出码 0；21 个可编辑网格、六台设备、八组窗、四张 packed 纹理，未混入预览地面、白盒和招牌 |
| Bevy 真实运行、人物尺度、实际碰撞、连续视频 | NOT RUN，交 R4 整合轮 |

首次生产在保存前发现循环变量覆盖建筑元数据，已修复。首次看图发现临时招牌 UV 左右翻转，修正预览平面的 UV 后重出三图；运行时原招牌与候选几何都未因此改动

本环境 sandbox 中 Blender 完成工作后会在 `pa_write() ... Operation not permitted` 后停在 `futex_wait`，`-noaudio` 单独未解除该退出问题。确认本轮全部预览已落盘、检查已输出且进程归属后，以 TERM 清理本轮五个退出等待进程，宿主重跑导出／检查取得退出 0；使用范围只包含本轮源目录与证据。README 的日常入口附 `ALSOFT_DRIVERS=null`、`-noaudio` 与 `--python-exit-code 1`，遇到环境退出异常仍需核对实际进程，不以日志完成替代退出检查

## 实际看图

- `north-street`：BYTE BEAT 原招牌正向可读，粉紫与黄绿点出经营类型，中央门与来路没有设备或候场栏杆遮挡；首层高店面和安静上层窗带可区分
- `northwest`：雨棚、厚窗框、窗台、浅展柜与机台前后关系成立；未制作的另外三面仅以临时白盒辅助观察，候选仍只覆盖完整北面
- `equipment`：六台设备各有外壳、退后屏面、按键台、检修／扬声孔大形，细节分组可读，没有现成游戏画面或新增品牌内容
- root 同样实际查看三图，认为经营辨识足够进入整合；主体仍规整，人物尺寸关联和场景光照需实机复查，不据源图宣称达到最终品质

三张 PNG 已在 root 看图后清理，尺寸、SHA-256、字节数与看图范围留在 `review.json`。保留 editable master、candidate、制作与检查脚本、manifest、日志和数值报告

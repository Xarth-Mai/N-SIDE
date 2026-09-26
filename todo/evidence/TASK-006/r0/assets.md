# F03 颜色预算修复

输入：`a0deb64e1447d84ab7161e494ee1748de279090c` 中的 `asset_report.py`；随附独立报告为故障线索，本记录来自当前项目 Python/Pillow 实际运行

根因：Pillow `getcolors(N + 1)` 能返回精确的 `N + 1` 个颜色，原检查只拒绝字符串形式的超限哨兵值。修复同时拒绝精确整数大于预算的情形；保留 N + 1 的计数能力

文件顶部显著保留 Apache 作者、许可证位置、上游版本及本次修改说明，来源清单按补丁管理，不再宣称该脚本逐字节原样

```fish
python3 -B -m unittest discover -s tools/tests -p test_asset_preview.py -v
```

PASS：4 项端到端测试，颜色边界子场景 N=3、颜色数 2/3/4/5 分别返回 0/0/1/1；原图字节保持不变。尺寸、alpha、全透明/不透明 cutout、覆盖保护与 contact sheet 同时通过

这证明工具约束，不代表游戏内视觉验收。首次使用 dotted module 命令因本仓库 tools 不是 Python package 失败，改用现有 unittest discover 入口后完成上述验证

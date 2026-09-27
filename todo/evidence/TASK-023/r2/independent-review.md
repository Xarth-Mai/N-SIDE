# 地形细化最终增量独立检查

2026-09-27；范围为 `tools/terrain-shape.ts`、其测试、最高点被铺装覆盖的几何回归及水平人眼镜头回归。未修改实现，未重复图形捕获或扩大游戏验收

检查时地图 SHA256：`bd04144820e6adefd4f4d9d87c2874090a0b1f92c0e699bc41787cc4ba82e9d2`

| 实际执行 | 结果 |
| --- | --- |
| `bun test tools/tests/terrain-shape.test.ts` | PASS，3 项通过，退出 0 |
| 下方只读探针内的 `bun tools/terrain-shape.ts --check` | PASS，748 个样点，退出 0；源文件字节与 `mtimeMs` 前后相等 |
| 隔离副本坏 JSON、展开格式、未重新烘焙的坐标修改 | 三次均正确退出 1；文件字节与 `mtimeMs` 均未改变 |
| 隔离副本正常写入模式连续两次执行 | 两次退出 0，字节稳定，第二次没有写文件 |
| `git diff --check` | PASS，退出 0 |

只读探针通过 `Bun.spawnSync` 执行真实 CLI；前后时间只比较相等，未打印具体时间值

```ts
// 使用 bun -e 执行
import {readFileSync,statSync} from "node:fs"
import {createHash} from "node:crypto"
const p="source-assets/district-map/district.json"
const before=readFileSync(p),mtime=statSync(p).mtimeMs
const r=Bun.spawnSync(["bun","tools/terrain-shape.ts","--check"])
if(r.exitCode)throw new Error(r.stderr.toString())
if(!before.equals(readFileSync(p))||mtime!==statSync(p).mtimeMs)throw new Error("--check wrote source")
console.log(r.stdout.toString().trim())
console.log("PASS readonly SHA256 "+createHash("sha256").update(before).digest("hex"))
```

隔离探针使用 `bun -` 执行，复制 `terrain-shape.ts`、`district-types.ts`、`district-geometry.ts` 及源 JSON 到 `/tmp/nside-terrain-review-gk2fV1`，保持工具实际相对路径。实际子命令为 `bun /tmp/nside-terrain-review-gk2fV1/tools/terrain-shape.ts --check`，正常重复生成时省略 `--check`

三种失败输入分别为 `{`、`JSON.stringify(source,null,2)`、将原文本首个 `[500,` 改为 `[501,` 后不重新烘焙。每次断言退出码非零，并比较输入字节与 `statSync(path).mtimeMs` 均未变化；正常重复生成比较两次退出码、输出字节与第二次执行前后的 `mtimeMs`。隔离探针整体退出 0

测试保留 authored 前缀及全部非 terrain 数据，覆盖连续道路中点和平台内部保护。几何回归检查实际峰顶铺装的渲染高度，镜头回归检查 1.7m 人眼高度、水平朝向及视锥；本轮未重新执行 Cargo 测试或把静态审查记为视觉验收

正确性审查未发现剩余阻断；复杂度审查：Lean already. Ship.

## 山脚平台边界修复后的增量复核

本节为边界修复执行者的只读增量复核；上文独立检查对应旧地图 `bd04144820e6adefd4f4d9d87c2874090a0b1f92c0e699bc41787cc4ba82e9d2`，不能直接作为此次边界修复的验收结果

当前地图 SHA256：`784ef145fb231712faa433d3ea6e9c35e73ddd3a0a03c839218d6d278ba93ba0`

旧画面中山脚休息台 `fw-e-existing-20` 左角出现三角护坡：屏幕位置对应地面约 `[346.54,609.62,104.64]`，平台高程为90m。粗网格虽保护区域内部，边角仍被邻近较高样点插值抬起；本次在生成后缀补充地面平台与建筑的精确边界控制，保留39个手工控制及全部非地形数据，采样总数由748变为848

| 实际执行及检查 | 结果 |
| --- | --- |
| `bun test tools/tests/terrain-shape.test.ts tools/tests/district-map.test.ts` | PASS，35项通过，退出0 |
| 原始控制与最高点 | PASS，手工前缀和非地形数据不变；新增边界点优先避让既有坐标，生成后缀均低于450m |
| 平台真实插值回归 | PASS，`buildGround` 检查四条边各自的起点、1/4点、中点、3/4点，共16点均为90m；断言容差 `1e-6` 仅容纳浮点误差 |
| 重复生成及只读 CLI | PASS，`bakeTerrain(bakeTerrain(data))` 无漂移；上文相同只读探针运行 `--check` 返回848样点，当前源字节及 `mtimeMs` 前后不变 |
| `git diff --check` | PASS，退出0 |

边界补点直接复用已有区域、控制点与生成后缀，未增加依赖、运行时状态或第二份地图源；本轮未重拍图形，修复后的视觉结果由新的真实捕获单独确认

复杂度复核：Lean already. Ship.

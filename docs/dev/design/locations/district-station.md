---
document_id: DOC-DISTRICT-STATION
status: draft
depends_on: ["DOC-DISTRICT-PLAN", "DOC-DISTRICT-ARCHITECTURE", "DOC-DISTRICT-PLACES"]
---

<script setup lang="ts">
import DistrictArchitecture from '../../../.vitepress/components/DistrictArchitecture.vue'
import data from '../../../../source-assets/district-map/district.json'
import { architectureStats } from '../../../../tools/district-architecture.ts'
import { routeProfile } from '../../../../tools/district-plan.ts'
const segment=data.architectures.find(a=>a.id==='P3-A2')
const stats=architectureStats(data,segment)
const supply=data.logistics.find(l=>l.id==='interest-equipment')
const distance=leg=>routeProfile(data.nodes,leg.nodes).at(-1).distance.toFixed(1)
const grade=Math.max(...routeProfile(data.nodes,supply.legs[0].nodes).slice(1).map(p=>Math.abs(p.grade))).toFixed(2)
</script>

# 站前与兴趣街建筑

站前以通勤商业、办公和住宅组成较完整的城市界面；兴趣街把游玩、观看、制作与设备服务嵌入日常街坊。两段之间沿站厅北侧与站西步行街联系，店面、公共经过与住户门厅共同构成来路

与[小店—采购街—坡地住宅](district-architecture.md)共用设计米、相对高程、楼层与统计口径。图纸从同一份地图 JSON 渲染，[总图](district-plan.md)同步显示本段建筑与道路，[场所页](district-places.md)维护活动与运营

## 街面、体量与楼层

本段覆盖 B01、B02 的 {{ segment.buildings.length }} 个单元，包含 14 个既有体量与 10 栋新增普通建筑。已设计占地 {{ stats.reduce((n,t)=>n+t.footprint,0).toLocaleString() }} m²、初算楼面 {{ stats.reduce((n,t)=>n+t.floorArea,0).toLocaleString() }} m²；规划容量与实际建筑分别见总图，普通楼层使用不增加特色目的地数量

| 街段 | 建筑安排 | 公共与私人关系 |
| --- | --- | --- |
| 站前内街 | 南排四栋通勤店屋，北排三栋六层混合楼，内街净宽按 6 m 起案 | 商业朝内街，办公与住宅另设门厅；配送沿南侧与侧向服务口进入 |
| 站西 | 食堂办公楼、普通办公楼与六层公寓形成连续东向界面 | 公众绕楼北侧抵达兴趣街；食堂、办公与住宅各有门口 |
| 兴趣南街 | 音游、模型、桌游、维修、独立游戏与旧物交换形成多面街段 | 北侧会合、观看与入店，南侧装卸和受控后场 |
| 兴趣北街 | 新增办公、工作室与普通住宅，两侧随道路错台 | +12.76 m、+14.11 m 入口分别接同高门前；住户院落沿外部公共道路到达 |

站前楼高以四至六层组织，游戏楼保留较高首层，制作楼以三层展开，站厅维持较低体量。现代感通过成组窗面、清楚的入口、连续遮蔽与设备服务位置表达

## 建筑图纸

<DistrictArchitecture section="P3-A2" />

## 站前混合楼

V-M01 的 32×22 m 轮廓承担商业、办公与住宅。首层 +8 m，办公层 +12.5、+16.3 m，住宅层 +20.1、+23.3、+26.5 m，屋面 +29.7 m。首层、办公标准层与住宅标准层分别提供平面，楼层标高和总高同步进入立面及 E-E 剖面

南门进入普通商业，东侧服务门进入收货与店务。办公门厅在东侧北段，连接办公楼梯与客梯；住户由北侧门厅进入独立楼梯、电梯与信箱区。办公层保留住户交通井，住宅层由共用走廊进入各户，第二楼梯通过受控前室联系

内街在两排建筑之间通过，短时取餐与观看橱窗靠店面布置。北侧住宅门前和东侧办公门前分别有停步位置，导向只在对应门厅显示楼层与开放状态。南侧饭馆、咖啡、寄存和书店的楼上用途与街面入口同时记录

各栋楼层用途、房间边界与核心交通为建筑方案控制。墙厚、结构网格、疏散与设备井在专项阶段核对，具体套型和住宅使用效率随全区建筑展开校准

## 游戏、制作与设备服务

### 音游与朋友会合

V-35 首层 +12 m、二层 +18 m、屋面 +23 m，首层按 6 m 层高为重型设备、可开启检修面和设备管线留空间。北侧入口先到缓冲与候场，两处公共楼梯和东侧电梯联系上层轻型试玩和创作者展示

首层南侧是整机验收与维修带，中部搬运通道接两侧设备区。机器沿后门进入，公众从北侧会合庭进入；营业时设备通道受店员管理。上层活动以轻型设备和交流为主，重型机器保留在首层

设备实际型号、动作包络、声学隔振、承重与维护尺寸作为 P4 输入。当前图纸先控制首层高度、通道和前后门关系

### 模型与维修

V-36 首层由材料验收、常规拼装、模型销售与橱窗组成，公共楼梯和电梯靠前侧，服务楼梯连接背侧材料区。二层提供预约制作、展示交流和后场走廊；有通风需求的工艺室预留独立房间，开放条件随材料与通风方案确定

V-38 顾客在东侧交接和短测，设备进入受控检修区，再分到待修、已修和零件储藏。后勤楼梯接材料与维修教学准备，公共楼梯和电梯接上层小组制作；个人设备的授权与保管由店员管理

模型、维修和音游共同提供可观察的工作过程。作品上窗、交接设备、早间检修与夜间候场分别发生在实际门口或房间，公共横街保持通过

## 配送、高差与会合

城市接入经过站西转角，沿约 69.5 m 的坡段从 +9 m 上升到兴趣街 +12 m，车辆路线最大分段坡度约 {{ grade }}%。装卸位从中央公共街旁侧分出，后场推车沿 +12 m 向左右各店分送

<table>
<thead><tr><th>方式</th><th>路径</th><th>距离</th></tr></thead>
<tbody><tr v-for="leg in supply.legs" :key="leg.mode"><td>{{ leg.mode }}</td><td>{{ leg.mode==='推车'?'旁侧装卸位 → 同高后场 → 音游南侧整机门':'城市西口 → 站西道路 → 兴趣街南口 → 旁侧装卸位' }}</td><td>{{ distance(leg) }} m</td></tr></tbody>
</table>

车辆只在装卸位完成交接，重型设备以推车进入店铺；实际车辆转弯、临时占用和搬运工具在交通专项核对。北侧 +12.76 m 与 +14.11 m 建筑沿公共街坡面错台，入口前的平台各自水平

站西座位、音游门前庭与既有兴趣街会合庭承担不同停留：通勤等候、朋友观看与游玩前后会合。座椅、状态柱、取件柜和维护接口在图中有明确位置，停用提示与人工服务保留在对应入口

## 类型依据

[R12 混合用途建筑](world-research.md#r12-站前混合楼与公共穿行)提供商业、办公、住宅入口与公共穿行的平面及剖面依据；[R13 FRAME](world-research.md#r13-模型兴趣店的展示、活动与会合)提供展示、活动与会合的实际组织；[R14 制作空间](world-research.md#r14-制作工作区与工具、作品储存)提供电子、开放制作、工具管理与独立工艺房的关系

楼层高度、面宽、进深、设备搬运路线与住宅布局均按 Null Site条件设计。图面验证建筑、入口、房间开口与道路范围；声学、结构、通风及车行包络由专项核对

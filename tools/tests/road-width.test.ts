import { test } from 'node:test'
import assert from 'node:assert/strict'
import { roadWidthConflicts, roadSurfaceConflicts } from '../check-road-width.ts'
import { pointInside, onBoundary, polygonInside, polygonArea, roadAllowed, roadOffsets, intersectionArea } from '../district-geometry.ts'
import type { District } from '../district-types.ts'
import source from '../../source-assets/district-map/district.json' with { type: 'json' }

test('shared endpoints do not hide width conflicts and truly level junctions pass',()=>{
  const data:Pick<District,'nodes'|'roads'|'routes'>={
    nodes:{center:[0,0,0],up:[10,0,10],side:[10,2,0]},
    roads:[{nodes:['center','up'],kind:'steps',width:4},{nodes:['center','side'],kind:'lane',width:4}],
    routes:[{id:'walk',name:'test walk',nodes:['center','up'],user:'public'}],
  }
  const bad=roadWidthConflicts(data,'walk')
  assert.equal(bad.length,1)
  assert.deepEqual(bad[0].sharedNodes,['center'])
  assert.ok(bad[0].maxDelta>5,'same graph node does not make overlapping slopes coplanar')
  data.nodes.up[2]=0
  assert.deepEqual(roadWidthConflicts(data,'walk'),[])
  assert.throws(()=>roadWidthConflicts(data,'typo'),/Unknown route/)
})

test('a selected route is checked against other roads without absorbing unrelated conflicts',()=>{
  const data:Pick<District,'nodes'|'roads'|'routes'>={
    nodes:{a:[0,0,0],b:[10,0,0],c:[100,0,0],d:[110,0,10],e:[110,2,0]},
    roads:[{nodes:['a','b'],kind:'lane',width:4},{nodes:['c','d'],kind:'steps',width:4},{nodes:['c','e'],kind:'lane',width:4}],
    routes:[{id:'walk',name:'test walk',nodes:['a','b'],user:'public'}],
  }
  assert.deepEqual(roadWidthConflicts(data,'walk'),[])
  assert.equal(roadWidthConflicts(data,'all').length,1)
  data.nodes.c=[5,-5,1];data.nodes.d=[5,5,1]
  assert.ok(roadWidthConflicts(data,'walk').some(c=>c.a.nodes.includes('a')&&c.b.nodes.includes('c')))
})

test('upper-street and foothill junctions join at road width with usable stair treads',()=>{
  const data:District=source
  const junctions=new Set(['old_home','home_north','west_upper','upper_homes','foothill_east','north'])
  const conflicts=roadWidthConflicts(data,'all').filter(c=>c.sharedNodes.some(id=>junctions.has(id))||[...c.a.nodes,...c.b.nodes].some(id=>id.startsWith('junction_')))
  assert.deepEqual(conflicts,[],'the repaired approach must clear every other road, including its shared node')
  for(const road of data.roads.filter(r=>r.kind==='steps'&&r.nodes.some(id=>id.startsWith('stair_old_home_level_west_upper_')||id.startsWith('stair_home_north_level_upper_homes_')||id.startsWith('stair_foothill_east_fw_e_forest_turn_')))) {
    const [a,b]=road.nodes.map(id=>data.nodes[id]),risers=Math.ceil(Math.abs(b[2]-a[2])/.17)
    assert.ok(risers<=48&&Math.hypot(b[0]-a[0],b[1]-a[1])/risers>=.28,`${road.nodes.join(' -> ')}: clearing the junction must retain a usable flight`)
  }
  assert.deepEqual(roadWidthConflicts(data,'hill-short'),[],'the previously checked short ascent remains clear')
})

test('campus stairs and market entrances meet the full road width at their own levels',()=>{
  const junctions=new Set(['market_entry','market_turn','school_steps_east_base','school_steps_east_mid','school_steps_east_rest','fw_e_school_edge_top'])
  const conflicts=roadWidthConflicts(source,'all').filter(c=>c.sharedNodes.some(id=>junctions.has(id))||[...c.a.nodes,...c.b.nodes].some(id=>/^junction_(school|market)_/.test(id)))
  assert.deepEqual(conflicts,[],'approach landings must clear both neighboring roads and the next stair flight')
  assert.ok(!source.roads.some(r=>r.nodes.some((id,i)=>id===r.nodes[i+2])),'an exact out-and-back road segment creates duplicate ribbons and an undefined turn')
})

test('music street junctions clear their neighbors and equipment reaches the level unloading route',()=>{
  const data:District=source,junctions=new Set(['music_cross','music_west_walk','fw_w_music_delivery','fw_w_music_mid'])
  const conflicts=roadWidthConflicts(data,'all').filter(c=>c.sharedNodes.some(id=>junctions.has(id))||[...c.a.nodes,...c.b.nodes].some(id=>id.startsWith('junction_music_')))
  assert.deepEqual(conflicts,[],'the music approach must clear the avenue and public lane across their full widths')
  const supply=data.logistics.find(l=>l.id==='music-equipment')!
  for(const leg of supply.legs)for(let i=1;i<leg.nodes.length;i++)assert.ok(data.roads.some(r=>roadAllowed(r,'service')&&!['steps','trail','deck','lift'].includes(r.kind)&&r.nodes.some((id,j)=>j>0&&((id===leg.nodes[i]&&r.nodes[j-1]===leg.nodes[i-1])||(id===leg.nodes[i-1]&&r.nodes[j-1]===leg.nodes[i])))),`${leg.mode}: broken equipment approach ${leg.nodes[i-1]} -> ${leg.nodes[i]}`)
  const [vehicle,trolley]=supply.legs,bay=data.surfaces.find(s=>s.id===supply.unloading)!
  assert.equal(vehicle.nodes.at(-1),trolley.nodes[0],'vehicle and trolley must meet at the same unloading node')
  assert.ok(pointInside(data.nodes[trolley.nodes[0]],bay.polygon),'equipment transfer must occur inside the unloading bay')
  assert.ok(trolley.nodes.every(id=>data.nodes[id][2]===bay.elevation),'equipment trolley route must stay at the unloading level')
})

test('upper homes stairs and greenhouse approaches retain level junctions and usable flights',()=>{
  const data:District=source,junctions=new Set(['homes_high_m','homes_high_junction','fw_f_junction22','fw_f_junction86','fw_f_junction87'])
  const conflicts=roadWidthConflicts(data,'all').filter(c=>c.sharedNodes.some(id=>junctions.has(id))||[...c.a.nodes,...c.b.nodes].some(id=>id.startsWith('junction_homes_')))
  assert.deepEqual(conflicts,[],'upper-street stairs must clear both streets, neighboring flights and doorway landings')
  for(const road of data.roads.filter(r=>r.nodes.some(id=>id.startsWith('junction_homes_')))) {
    const points=road.nodes.map(id=>data.nodes[id])
    if(road.kind==='landing')assert.ok(points.every(p=>Math.abs(p[2]-points[0][2])<.001),`${road.nodes.join(' -> ')}: junction landing must stay level`)
    if(road.kind==='steps')for(let i=1;i<points.length;i++) {
      const a=points[i-1],b=points[i],risers=Math.ceil(Math.abs(b[2]-a[2])/.17)
      assert.ok(risers<=48&&Math.hypot(b[0]-a[0],b[1]-a[1])/risers>=.28,`${road.nodes.join(' -> ')}: retain a usable flight after clearing the road width`)
    }
  }
})

test('station and music approaches plus the long ascent join at width without sharpening slopes or steps',()=>{
  const data:District=source,junctions=new Set(['square','city_w','fw_w_music_gate','fw_w_music_north','hill_east_approach','hill_east_curve_06','hill_east_curve_08'])
  const conflicts=roadWidthConflicts(data,'all').filter(c=>c.sharedNodes.some(id=>junctions.has(id))||[...c.a.nodes,...c.b.nodes].some(id=>/^junction_(square|city_w|music_gate|music_north|trail)_/.test(id)))
  assert.deepEqual(conflicts,[],'both sides of each new level approach must clear every neighboring road')
  const stairEnds=new Set(['junction_trail_approach_in','junction_trail_curve06_in','junction_trail_curve06_out','hill_east_arc_08_10','hill_east_arc_09_01'])
  for(const r of data.roads)for(let i=1;i<r.nodes.length;i++) {
    const ids=r.nodes.slice(i-1,i+1),[a,b]=ids.map(id=>data.nodes[id]),run=Math.hypot(b[0]-a[0],b[1]-a[1]),rise=Math.abs(b[2]-a[2])
    if(ids.some(id=>/^junction_(square|city_w|music_gate|music_north)_/.test(id)))assert.ok(rise/run<=.100001,`${ids.join(' -> ')}: level junction must not create a steeper lowland approach`)
    if(r.kind==='steps'&&ids.some(id=>stairEnds.has(id))) {
      const risers=Math.ceil(rise/.17)
      assert.ok(risers>0&&risers<=48&&run/risers>=.28,`${ids.join(' -> ')}: the shortened stair flight needs usable treads`)
    }
  }
  const shrine=data.buildings.find(b=>b.id==='V-22')!
  for(const road of data.roads.filter(r=>r.nodes.includes('shrine'))) {
    const points=road.nodes.map(id=>data.nodes[id]),offsets=roadOffsets(points,road.width)
    for(let i=1;i<points.length;i++) {
      const [a,b]=points.slice(i-1,i+1),[u,v]=offsets.slice(i-1,i+1)
      const ribbon=[[a[0]-u[0],a[1]-u[1]],[b[0]-v[0],b[1]-v[1]],[b[0]+v[0],b[1]+v[1]],[a[0]+u[0],a[1]+u[1]]]
      assert.ok(intersectionArea(ribbon,shrine.polygon)<1e-6,`${road.nodes.slice(i-1,i+1).join(' -> ')}: an exterior trail must not pass underneath the shrine floor`)
    }
  }
  assert.deepEqual(roadWidthConflicts(data,'hill-short'),[],'the short summit alternative remains clear')
})

test('foothill housing and west plateau approaches clear streets, stairs and resident side paths',()=>{
  const junctions=new Set(['foothill_house_gate','platform_west','res_n','fw_f_plateau_w2','fw_f_plateau_w3','fw_f_plateau_w4','fw_f_junction83','fw_f_plateau_w5','fw_f_plateau_w_rear0','fw_f_plateau_w_rear1'])
  const relevant=(id:string)=>junctions.has(id)||/^junction_(plateau_|foothill_house_)/.test(id)
  const conflicts=roadWidthConflicts(source,'all').filter(c=>[...c.a.nodes,...c.b.nodes].some(relevant))
  assert.deepEqual(conflicts,[],'level approaches must clear neighboring roads and resident entrances across the full width')
  assert.deepEqual(roadWidthConflicts(source,'hill'),[],'the complete long summit route must clear neighboring ribbons too')
  assert.deepEqual(roadWidthConflicts(source,'hill-short'),[],'the shorter summit route remains clear')
})

function checkExteriorApproaches(data:District, roads:District['roads']) {
  for(const road of roads) {
    const p=road.nodes.map(id=>data.nodes[id]),o=roadOffsets(p,road.width)
    for(let i=1;i<p.length;i++) {
      const [a,b]=[p[i-1],p[i]],[u,v]=[o[i-1],o[i]],run=Math.hypot(b[0]-a[0],b[1]-a[1]),rise=Math.abs(b[2]-a[2])
      if(road.kind==='steps') {
        const risers=Math.ceil(rise/.17)
        assert.ok(risers>0&&risers<=48&&run/risers>=.28,`${road.nodes[i-1]} -> ${road.nodes[i]}: keep a usable stair flight`)
      } else assert.ok(rise/run<=.100001,`${road.nodes[i-1]} -> ${road.nodes[i]}: exterior approach is too steep`)
      const ribbon=[[a[0]-u[0],a[1]-u[1]],[b[0]-v[0],b[1]-v[1]],[b[0]+v[0],b[1]+v[1]],[a[0]+u[0],a[1]+u[1]]]
      for(const building of data.buildings)assert.ok(intersectionArea(ribbon,building.polygon)<1e-6,`${road.nodes[i-1]} -> ${road.nodes[i]}: exterior path crosses ${building.id}`)
    }
  }
}

for(const [name,junctions,prefix] of [
  ['east terraces',['fw_f_wood_homes1','fw_f_wood_homes2','fw_f_plateau_e1','fw_f_plateau_e2','fw_f_plateau_e4','fw_f_plateau_e_mid0','platform_east','fw_f_neighbors3'],'junction_east_'],
  ['west old housing',['fw_f_old_low1','interest_n','west_link','west_north','fw_f_old_court0'],'junction_old_'],
] as const) test(`${name} clear neighboring road ribbons and retain usable exterior approaches`,()=>{
  const relevant=(id:string)=>(junctions as readonly string[]).includes(id)||id.startsWith(prefix)
  assert.deepEqual(roadWidthConflicts(source,'all').filter(c=>[...c.a.nodes,...c.b.nodes].some(relevant)),[],'level exits must clear neighboring ribbons')
  checkExteriorApproaches(source,source.roads.filter(r=>r.nodes.some(id=>id.startsWith(prefix))))
})

test('food street bypasses shop footprints and waterfront stairs descend beside the waiting court',()=>{
  const data:District=source
  const junctions=new Set(['food_ramp_lower','community','cross_w','river_w','dock'])
  const relevant=(id:string)=>junctions.has(id)||/^junction_(food_lower|community_|river_|dock_)/.test(id)
  assert.deepEqual(roadWidthConflicts(data,'all').filter(c=>[...c.a.nodes,...c.b.nodes].some(relevant)),[],'food and waterfront approaches must clear neighboring ribbons')
  checkExteriorApproaches(data,data.roads.filter(r=>r.nodes.some(id=>/^junction_(food_lower|community_|river_|dock_)/.test(id)||id==='fw_f_food_north3')))
  const court=data.surfaces.find(s=>s.id==='ferry_waiting_surface')!
  for(const r of data.roads.filter(r=>r.kind==='steps'&&r.nodes.some(id=>id.startsWith('junction_dock_')))) {
    const p=r.nodes.map(id=>data.nodes[id]),o=roadOffsets(p,r.width)
    for(let i=1;i<p.length;i++) {
      const [a,b]=[p[i-1],p[i]],[u,v]=[o[i-1],o[i]],ribbon=[[a[0]-u[0],a[1]-u[1]],[b[0]-v[0],b[1]-v[1]],[b[0]+v[0],b[1]+v[1]],[a[0]+u[0],a[1]+u[1]]]
      assert.ok(intersectionArea(ribbon,court.polygon)<1e-6,'dock descent must not cut through the upper waiting court')
    }
  }
})

test('all ground-road junctions clear neighboring ribbons while retaining usable exterior approaches',()=>{
  const data:District=source
  assert.deepEqual(roadWidthConflicts(data,'all'),[],'all source ground-road widths must meet at compatible elevations')
  checkExteriorApproaches(data,data.roads.filter(r=>r.nodes.some(id=>/^junction_(e9_|shop_)/.test(id))))
})

test('platform checks keep concave cutouts and actual stair treads while excluding explicitly elevated surfaces',()=>{
  const data:Pick<District,'nodes'|'roads'|'routes'|'surfaces'>={
    nodes:{a:[2,2,0],b:[8,2,0]},roads:[{nodes:['a','b'],kind:'lane',width:1}],routes:[],
    surfaces:[{id:'court',kind:'plaza',elevation:2,polygon:[[0,-4],[10,-4],[10,4],[6,4],[6,-2],[4,-2],[4,4],[0,4]]}],
  }
  const hit=roadSurfaceConflicts(data)
  assert.equal(hit.length,1)
  assert.ok(Math.abs(hit[0].area-4)<1e-6,'the concave cutout is not a filled convex envelope')
  assert.equal(hit[0].maxDelta,1.975)
  data.surfaces[0].elevation=0
  assert.deepEqual(roadSurfaceConflicts(data),[],'same-level circulation may overlap a court')
  data.nodes.a[2]=.34;data.nodes.b[2]=.34
  assert.ok(Math.abs(roadSurfaceConflicts(data)[0].maxDelta-.365)<1e-6,'the rendered road top includes its 2.5 cm surface offset')
  data.surfaces[0].elevation=2;data.surfaces[0].elevated=true
  assert.deepEqual(roadSurfaceConflicts(data),[],'elevated structures require a separate clearance check')
  data.surfaces[0].elevated=false;data.surfaces[0].polygon=[[4,-1],[6,-1],[6,1],[4,1]]
  data.nodes={a:[0,0,0],b:[10,0,4]};data.roads[0].kind='steps'
  assert.ok(roadSurfaceConflicts(data)[0].maxDelta>.4,'horizontal stair patches extend beyond the continuous-slope interpolation')
  data.surfaces[0].elevation=NaN
  assert.throws(()=>roadSurfaceConflicts(data),/invalid elevation/)
})

test('ground platforms meet adjacent road ribbons and stair treads at their own level',()=>{
  assert.deepEqual(roadSurfaceConflicts(source),[],'platform edges must clear slopes and stair flights before their level arrival')
})

test('cinema forecourt joins existing level entrances and keeps green islands inside its parcel',()=>{
  const data:District=source
  const court=data.surfaces.find(s=>s.id==='cinema-arrival-court')!
  const parcel=data.parcels.find(p=>p.id==='P09-A')!
  const gardens=['cinema-west-garden-south','cinema-west-garden-north'].map(id=>data.surfaces.find(s=>s.id===id)!)
  assert.ok(court&&gardens.every(Boolean),'the paved arrival and both planting beds must exist in the shared map')
  assert.equal(court.kind,'court');assert.equal(court.access,'public')
  assert.ok(!court.elevated&&!court.building,'the forecourt is ground-level public space')
  assert.ok(polygonArea(court.polygon)>350&&polygonArea(court.polygon)<450&&polygonInside(court.polygon,parcel.polygon),'retain a bounded usable arrival within P09-A')
  for(const id of ['fw_w_cinema_front','cinema_entry','fw_w_cinema_lift_front','cinema_lift_low']) {
    assert.equal(data.nodes[id][2],court.elevation,`${id}: the court must meet the actual entrance level`)
    assert.ok(pointInside(data.nodes[id],court.polygon),`${id}: preserve the public arrival through the court`)
  }
  for(const garden of gardens) {
    assert.equal(garden.kind,'park');assert.equal(garden.elevation,court.elevation)
    assert.ok(polygonInside(garden.polygon,parcel.polygon),'planting stays inside the existing cinema parcel')
    assert.ok(intersectionArea(garden.polygon,court.polygon)<1e-6,'planting and paving must meet without overlapping in either Viewer or Wiki')
    assert.ok(garden.polygon.filter(p=>onBoundary(p,court.polygon)).length>=2,'the planting edge must join the paved forecourt')
    assert.ok(polygonArea(garden.polygon)>8,'retain real soil area rather than decorative points')
    assert.ok(!pointInside([295.8,233.5],garden.polygon),'leave the seating gap unplanted')
  }
  assert.ok(intersectionArea(gardens[0].polygon,gardens[1].polygon)<1e-6,'planting beds remain separate')
  for(const building of data.buildings)assert.ok(intersectionArea(court.polygon,building.polygon)<1e-6,`${building.id}: do not enlarge public paving into a building`)
})

test('cinema forecourt excludes actual sloped road widths rather than relying on the coarse step threshold',()=>{
  const data:District=source
  const court=data.surfaces.find(s=>s.id==='cinema-arrival-court')!
  const gardens=['cinema-west-garden-south','cinema-west-garden-north'].map(id=>data.surfaces.find(s=>s.id===id)!)
  const check=(polygon:number[][])=>{
    for(const road of data.roads.filter(r=>!r.building&&!['interior','lift','bridge','deck'].includes(r.kind))) {
      const points=road.nodes.map(id=>data.nodes[id]),offsets=roadOffsets(points,road.width)
      for(let i=1;i<points.length;i++) {
        const [a,b]=[points[i-1],points[i]],[u,v]=[offsets[i-1],offsets[i]]
        const ribbon=[[a[0]-u[0],a[1]-u[1]],[b[0]-v[0],b[1]-v[1]],[b[0]+v[0],b[1]+v[1]],[a[0]+u[0],a[1]+u[1]]]
        if(intersectionArea(ribbon,polygon)>1e-6)assert.ok(Math.abs(a[2]-court.elevation)<1e-6&&Math.abs(b[2]-court.elevation)<1e-6,`${road.nodes.slice(i-1,i+1).join(' -> ')}: the court must not cap a sloping approach`)
        for(const garden of gardens)assert.ok(intersectionArea(ribbon,garden.polygon)<1e-6,`${garden.id}: planting must leave the full existing road width clear`)
      }
    }
  }
  check(court.polygon)
  assert.throws(()=>check([[295,213.5],[342.5,213.5],[342.5,220],[300,220],[300,242],[294.5,242],[294.5,220]]),/must not cap a sloping approach/,'the rejected rectangle produces a real step despite passing the general .35m screening threshold')
})

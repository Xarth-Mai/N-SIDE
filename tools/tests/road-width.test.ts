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

test('cinema service court supports the original loading route without changing its access or filling the doorway',()=>{
  const data:District=source
  const court=data.surfaces.find(s=>s.id==='cinema-service-court')!
  const parcel=data.parcels.find(p=>p.id==='P09-A')!
  const service=data.roads.find(r=>r.nodes.includes('cinema_loading')&&r.nodes.includes('cinema_service_entry'))!
  assert.ok(court&&service,'retain the authored court and its existing service approach')
  assert.equal(court.kind,'service');assert.equal(court.access,'service')
  assert.ok(!court.elevated&&!court.building,'the equipment court remains a ground surface')
  assert.equal(polygonArea(court.polygon),198,'keep the approved 12 x 16.5 m service footprint')
  assert.ok(polygonInside(court.polygon,parcel.polygon),'service paving stays in the existing cinema parcel')
  assert.equal(service.kind,'service');assert.equal(service.access,'service');assert.equal(service.width,3)
  for(const id of ['cinema_loading','cinema_service_entry']) {
    assert.equal(data.nodes[id][2],court.elevation,`${id}: court must meet the existing working level`)
    assert.ok(pointInside(data.nodes[id],court.polygon),`${id}: court must join the original route and door`)
  }
  for(const reserved of [
    [[318.2,250],[321.8,250],[321.8,266.5],[318.2,266.5]],
    [[317,258],[325,258],[325,266.5],[317,266.5]],
  ])assert.ok(polygonInside(reserved,court.polygon),'retain the 3.6 m clear approach and 8 x 8.5 m unloading area')
  const check=(polygon:number[][],elevation:number)=>{
    for(const building of data.buildings)assert.ok(intersectionArea(polygon,building.polygon)<1e-6,`${building.id}: service paving must stay outside the building`)
    for(const other of data.surfaces.filter(s=>s.id!==court.id&&!s.elevated&&!s.building))assert.ok(intersectionArea(polygon,other.polygon)<1e-6,`${other.id}: service paving must not overlap other ground surfaces`)
    let crossings=0
    for(const road of data.roads.filter(r=>!r.building&&!['interior','lift','bridge','deck'].includes(r.kind))) {
      const points=road.nodes.map(id=>data.nodes[id]),offsets=roadOffsets(points,road.width)
      for(let i=1;i<points.length;i++) {
        const [a,b]=[points[i-1],points[i]],[u,v]=[offsets[i-1],offsets[i]]
        const ribbon=[[a[0]-u[0],a[1]-u[1]],[b[0]-v[0],b[1]-v[1]],[b[0]+v[0],b[1]+v[1]],[a[0]+u[0],a[1]+u[1]]]
        if(intersectionArea(ribbon,polygon)<1e-6)continue
        crossings++
        assert.equal(road.access,'service','the service court must not absorb a public road')
        assert.ok(Math.abs(a[2]-elevation)<1e-6&&Math.abs(b[2]-elevation)<1e-6,`${road.nodes.slice(i-1,i+1).join(' -> ')}: service court must meet the full road ribbon at its actual level`)
      }
    }
    assert.ok(crossings>0,'the court must join an actual service road ribbon')
  }
  check(court.polygon,court.elevation)
  assert.throws(()=>check(court.polygon,court.elevation-.1),/must meet the full road ribbon/,'reject a 10 cm road lip below the coarse .35 m screening threshold')
  assert.throws(()=>check([[314,249],[326,249],[326,266.5],[314,266.5]],court.elevation),/must stay outside the building/,'reject paving extended through the existing north wall')
})

test('cinema roof deck retains four metres of supported width from its lift to the upper-street connection',()=>{
  const data:District=source
  const roof=data.surfaces.find(s=>s.id==='cinema_roof_surface')!
  const platform=data.surfaces.find(s=>s.id==='cinema_upper_platform')!
  const parcel=data.parcels.find(p=>p.id==='P09-A')!
  const building=data.buildings.find(b=>b.id==='V-15')!
  const deck=data.roads.find(r=>r.nodes[0]==='cinema_lift_high'&&r.nodes.at(-1)==='cinema_roof')!
  const formerOutline=[[340,240],[344,240],[355,257],[355,345],[345,345],[345,262],[340,250]]
  const addition=[[340,220],[342.15,220],[342.15,240],[340,240]]
  assert.equal(deck.width,4);assert.equal(deck.kind,'deck');assert.equal(deck.building,'V-15')
  assert.ok(roadAllowed(deck,'public'),'public roof access remains outside the ticketed zone')
  assert.equal(roof.elevation,37);assert.equal(platform.elevation,roof.elevation)
  assert.ok(platform.elevated&&!platform.building,'exterior platform stays distinct from the original roof')
  assert.ok(polygonInside(addition,parcel.polygon),'the new outer half and 0.15m structural edge stay in P09-A')
  assert.ok(Math.abs(polygonArea(platform.polygon)-polygonArea(formerOutline)-43)<1e-6,'extend only the approved 43 square metres')
  assert.ok(Math.abs(intersectionArea(platform.polygon,formerOutline)-polygonArea(formerOutline))<1e-6,'retain the entire original northern platform')
  assert.ok(Math.abs(intersectionArea(platform.polygon,addition)-polygonArea(addition))<1e-6,'the floor must actually include the proposed strip')
  assert.ok(intersectionArea(addition,building.polygon)<1e-6,'the added exterior slab does not replace the roof')
  assert.deepEqual(platform.bearingEdges,[{edge:0,building:'V-15'},{edge:1,building:'V-15'},{edge:8,building:'V-15'}],'the new outer edge and southern corner bear on the existing building without adding ground columns')
  const bearing=platform.bearingEdges!.find(e=>e.edge===8&&e.building==='V-15')!
  const bearingEnds=[platform.polygon[bearing.edge],platform.polygon[(bearing.edge+1)%platform.polygon.length]]
  assert.deepEqual(bearingEnds,[[340,250],[340,220]],'the shifted bearing index must still identify the full east wall contact')
  assert.ok(bearingEnds.every(p=>onBoundary(p,building.polygon)))
  const points=deck.nodes.map(id=>data.nodes[id]),offsets=roadOffsets(points,deck.width)
  const check=(outline:number[][])=>{
    let samples=0
    for(let i=1;i<points.length;i++)for(let step=0;step<=100;step++)for(let side=0;side<=16;side++) {
      const t=step/100,k=side/8-1,a=points[i-1],b=points[i],u=offsets[i-1],v=offsets[i]
      const p=[a[0]+(b[0]-a[0])*t+(u[0]+(v[0]-u[0])*t)*k,a[1]+(b[1]-a[1])*t+(u[1]+(v[1]-u[1])*t)*k]
      assert.ok(Math.abs(a[2]-roof.elevation)<1e-6&&Math.abs(b[2]-roof.elevation)<1e-6)
      assert.ok(pointInside(p,roof.polygon)||pointInside(p,outline),`unsupported roof deck width at ${p}: a centreline on the roof edge is insufficient`)
      samples++
    }
    assert.equal(samples,1717,'check the entire 25m by 4m deck at 0.25m spacing')
  }
  check(platform.polygon)
  assert.throws(()=>check(formerOutline),/unsupported roof deck width/,'the former outer-half gap must fail despite the centreline remaining on the roof boundary')
})

test('cinema roof paving joins the public seat and preserves the unfilled planting area',()=>{
  const data:District=source,roof=data.surfaces.find(s=>s.id==='cinema_roof_surface')!
  const ids=['cinema-roof-east-walk','cinema-roof-garden-entry','cinema-roof-service-walk','cinema-roof-seat-walk','cinema-roof-rest-court']
  const paving=ids.map(id=>data.surfaces.find(s=>s.id===id)!)
  const reserve=data.surfaces.find(s=>s.id==='cinema-roof-planting-reserve')!
  assert.equal(reserve.kind,'park');assert.equal(reserve.building,'V-15');assert.equal(reserve.elevation,37)
  assert.ok(data.surfaces.indexOf(reserve)<data.surfaces.indexOf(roof)&&polygonInside(reserve.polygon,roof.polygon))
  assert.ok(polygonArea(reserve.polygon)>=56,'retain the agreed unfilled planting area')
  const check=(surfaces:District['surfaces'])=>{
    for(const p of paving) {
      assert.ok(p&&surfaces.indexOf(p)<surfaces.indexOf(roof),'roof paving must precede grass at the same level')
      assert.equal(p.building,'V-15');assert.equal(p.elevation,37);assert.ok(p.elevated)
      assert.ok(polygonInside(p.polygon,roof.polygon),'building-bound paving stays on the supported roof')
      assert.ok(intersectionArea(p.polygon,reserve.polygon)<1e-6,'preserve the unfilled planting area as grass')
    }
  }
  check(data.surfaces)
  const rest=paving[4],branch=paving[3]
  assert.deepEqual(rest.polygon,[[309,235],[315,235],[315,240],[309,240]])
  assert.ok(pointInside([312,237],rest.polygon),'retain the existing bench at the original 37m floor')
  assert.ok(intersectionArea(branch.polygon,rest.polygon)>1&&intersectionArea(branch.polygon,paving[1].polygon)>.1,'the short walk must join both the existing garden entrance and rest court')
  const routes=data.roads.filter(r=>r.building==='V-15'&&['deck','bridge'].includes(r.kind)&&r.nodes.every(id=>data.nodes[id][2]===37)).map(r=>({points:r.nodes.map(id=>data.nodes[id]),width:r.width}))
  routes.push({points:[data.nodes.cinema_garden_entry,[312,237,37]],width:2})
  for(const {points,width} of routes) {
    const offsets=roadOffsets(points,width)
    for(let i=1;i<points.length;i++)for(let step=0;step<=100;step++)for(let side=0;side<=16;side++) {
      const t=step/100,k=side/8-1,a=points[i-1],b=points[i],u=offsets[i-1],v=offsets[i]
      const p=[a[0]+(b[0]-a[0])*t+(u[0]+(v[0]-u[0])*t)*k,a[1]+(b[1]-a[1])*t+(u[1]+(v[1]-u[1])*t)*k]
      if(pointInside(p,roof.polygon))assert.ok(paving.some(s=>pointInside(p,s.polygon)),`unpaved roof route width at ${p}`)
    }
  }
  assert.throws(()=>check([roof,...data.surfaces.filter(s=>s!==roof)]),/must precede grass/,'same-height paving after the park would be fully masked at runtime')
})

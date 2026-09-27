import { test } from 'node:test'
import assert from 'node:assert/strict'
import { intersectionArea, pointInside, polygonArea, polygonInside, roadAllowed, roadOffsets } from '../district-geometry.ts'
import { roadWidthConflicts } from '../check-road-width.ts'
import type { District } from '../district-types.ts'
import source from '../../source-assets/district-map/district.json' with { type: 'json' }

test('station meeting court retains usable space with a level public arrival clear of sloping roads',()=>{
  const data:District=source,place=data.places.find(p=>p.id==='02')!,court=data.surfaces.find(s=>s.place===place.id)!,parcel=data.parcels.find(p=>p.id===place.parcel)!
  assert.ok(polygonArea(court.polygon)>=150&&polygonInside(court.polygon,parcel.polygon),'repair must retain a usable meeting court inside its existing parcel')
  for(const road of data.roads.filter(r=>!r.building&&!['interior','lift','bridge','deck'].includes(r.kind))) {
    const points=road.nodes.map(id=>data.nodes[id]),offsets=roadOffsets(points,road.width)
    for(let i=1;i<points.length;i++) {
      const [a,b]=points.slice(i-1,i+1),[u,v]=offsets.slice(i-1,i+1)
      const ribbon=[[a[0]-u[0],a[1]-u[1]],[b[0]-v[0],b[1]-v[1]],[b[0]+v[0],b[1]+v[1]],[a[0]+u[0],a[1]+u[1]]]
      if(intersectionArea(ribbon,court.polygon)>1e-6)assert.ok(a[2]===court.elevation&&b[2]===court.elevation,`${road.nodes.slice(i-1,i+1).join(' -> ')}: a sloping road must not run through the horizontal meeting court`)
    }
  }
  assert.ok(data.buildings.every(b=>intersectionArea(b.polygon,court.polygon)<1e-6),'meeting space must clear existing buildings')
  const arrival=place.arrivals?.public
  assert.ok(arrival,'meeting court needs an explicit public arrival')
  assert.equal(arrival.nodes[0],place.access)
  for(let i=1;i<arrival.nodes.length;i++)assert.ok(data.roads.some(r=>roadAllowed(r)&&r.nodes.some((id,j)=>j>0&&((id===arrival.nodes[i]&&r.nodes[j-1]===arrival.nodes[i-1])||(id===arrival.nodes[i-1]&&r.nodes[j-1]===arrival.nodes[i])))),`${place.id}: broken public arrival`)
  const end=data.nodes[arrival.nodes.at(-1)!]
  assert.ok(pointInside(end,court.polygon)&&pointInside(place.position,court.polygon),'arrival and place marker must lie in the actual court')
  assert.equal(end[2],court.elevation)
  assert.equal(place.position[2],court.elevation)
  for(const id of ['station','station_entry','station_service_entry','station_front_start','station_north_start','v_m01_north','fw_f_station_north0'])assert.equal(data.nodes[id][2],8,`${id}: repairing the court must retain station and commercial doorway levels`)
  const courtNode=arrival.nodes.at(-1)!
  assert.deepEqual(roadWidthConflicts(data,'all').filter(c=>[...c.a.nodes,...c.b.nodes].includes(courtNode)),[],'new court approach must clear neighboring roads across its full width')
})

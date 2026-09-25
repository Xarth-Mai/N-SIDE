import { test } from 'node:test'
import assert from 'node:assert/strict'
import data from '../../source-assets/district-map/district.json' with { type: 'json' }
import { terrainHeight, heightColor } from '../district-map.mjs'

test('map places and roads have valid shared references', () => {
  assert.deepEqual(data.places.map(p=>p.id).sort(), Array.from({length:29},(_,i)=>String(i+1).padStart(2,'0')))
  for(const point of Object.values(data.nodes)) assert.ok(point.length===3 && point.every(Number.isFinite))
  for(const road of data.roads) assert.ok(road.nodes.length>=2 && road.nodes.every(id=>data.nodes[id]))
  for(const place of data.places){
    assert.ok(data.groups[place.group])
    assert.ok(place.position.length===3 && place.position.every(Number.isFinite))
  }
})

test('terrain colors follow interpolated ground elevation', () => {
  const points=[[0,0,0],[10,0,80]]
  assert.equal(terrainHeight(points,0,0),0)
  assert.equal(terrainHeight(points,5,0),40)
  assert.equal(terrainHeight(points,10,0),80)
  assert.equal(heightColor(0),'rgb(223,213,175)')
  assert.equal(heightColor(80),'rgb(103,139,112)')
  assert.equal(heightColor(-10),heightColor(0))
  assert.equal(heightColor(100),heightColor(80))
})

test('simplified map retains the shopping short loop and public hillside connection', () => {
  const linked=(a,b)=>data.roads.some(r=>r.nodes.some((n,i)=>i>0 && ((n===a && r.nodes[i-1]===b)||(n===b && r.nodes[i-1]===a))))
  for(const nodes of [
    ['home','court','market_turn','alley_corner','market_back','community','home'],
    ['home','steps_mid','upper_homes','north','forest','summit'],
  ]) for(let i=1;i<nodes.length;i++)assert.ok(linked(nodes[i-1],nodes[i]))
})

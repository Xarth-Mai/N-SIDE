export const svgPath = points => points.map((p, i) => `${i ? 'L' : 'M'}${p[0]},${-p[1]}`).join(' ')

// Road elevations guide the schematic terrain; bridge decks are not ground samples
export function terrainHeight(points, x, y) {
  const nearest = points.map(p => ({ height:p[2], distance:(p[0]-x)**2+(p[1]-y)**2 })).sort((a,b)=>a.distance-b.distance).slice(0,4)
  if (nearest[0].distance === 0) return nearest[0].height
  const weights = nearest.map(p=>1/p.distance**2)
  return nearest.reduce((sum,p,i)=>sum+p.height*weights[i],0)/weights.reduce((a,b)=>a+b,0)
}

export function heightColor(height) {
  const t = Math.max(0, Math.min(1, height / 80))
  return `rgb(${[223,213,175].map((value,i)=>Math.round(value+([103,139,112][i]-value)*t)).join(',')})`
}

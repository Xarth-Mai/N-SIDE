// Authored district data and the public drawing contract
export type District = {
  version: number
  groups: Record<string, string>
  terrain: {
    water: number[][]
    samples: number[][]
  }
  nodes: Record<string, number[]>
  roads: {
    nodes: string[]
    kind: string
    width: number
    access?: string
    surface?: string
    building?: string
    users?: string[]
  }[]
  buildings: {
    id: string
    bank: string
    parcel?: string
    polygon: number[][]
    elevation: number
    height: number
    kind: string
    place?: string
    design?: {
      type: string
      status: string
      floors: {
        name: string
        z: number
        use: string
        rooms?: {
          name: string
          kind: string
          polygon: number[][]
          label?: number[]
        }[]
        openings?: {
          line: number[][]
          kind: string
        }[]
      }[]
      entries: {
        node: string
        role: string
        level: string
      }[]
      front?: number[][]
      canopy?: number
      lightwell?: number[][]
    }
  }[]
  places: {
    id: string
    name: string
    block: string
    space?: string
    use: string
    entry: string
    users?: string
    time?: string
    activity?: string
    operations?: string
    detail?: boolean
    brief?: {
      space: string
      flow: string
      service: string
      public: string
      reference: string[]
    }
    featured: boolean
    group: string
    parcel?: string
    position: number[]
    access: string
    building?: string
    arrivals?: {
      public: {
        label: string
        nodes: string[]
        level: string
        user?: string
      }
      service?: {
        user?: string
        label: string
        nodes: string[]
        level: string
      }
    }
    role?: string
    page?: string
    parent?: string
  }[]
  trees: number[][]
  surfaces: {
    kind: string
    place?: string
    elevation: number
    polygon: number[][]
    id?: string
    name?: string
    boundary?: number[][]
    access?: string
    use?: string
    elevated?: boolean
    building?: string
    users?: string[]
    baseElevation?: number
  }[]
  blocks: {
    id: string
    name: string
    polygon: number[][]
    label: number[]
    role: string
    targetBuildings: number
    targetPlaces: number
    urbanPolygon?: number[][]
  }[]
  units: {
    horizontal: string
    vertical: string
    basis: string
  }
  parcels: {
    id: string
    block: string
    polygon: number[][]
    elevation: number
    use: string
    capacity: number
    access: string
    housing?: {
      coverage: number
      floors: number
      share: number
    }
  }[]
  sections: {
    id: string
    name: string
    nodes: string[]
    labels: Record<string, string | undefined>
    note: string
  }[]
  routes: {
    id: string
    name: string
    nodes: string[]
    user: string
    places?: string[]
  }[]
  connections: {
    id: string
    name: string
    from: string
    to: number[]
    use: string
  }[]
  elevatedNodes: string[]
  housingAssumptions: {
    netRatio: number
    unitArea: number
    occupancy: number[]
    household: number[]
  }
  logistics: {
    id: string
    place?: string
    label?: string
    legs: {
      mode: string
      nodes: string[]
    }[]
    unloading: string
    note?: string
    name?: string
  }[]
  architectures: {
    id: string
    name: string
    boundary: number[][]
    buildings: string[]
    types: {
      id: string
      name: string
      reference: string[]
      sample: string
      notes: string
    }[]
    baseline: {
      commit: string
      buildings: {
        id: string
        polygon: number[][]
        elevation: number
        height: number
      }[]
    }
    street: {
      nodes: string[]
      facades: {
        name: string
        line: number[][]
        buildings: string[]
        upper?: boolean
      }[]
    }
    sections: {
      id: string
      name: string
      line: number[][]
      ground: number[][]
    }[]
    scenes: {
      id: string
      theme: string
      label: string
      position: number[]
      who: string
      action: string
      change: string
      passage: string
      towards?: number[]
      targetBuilding?: string
    }[]
    fixtures: {
      name: string
      kind: string
      polygon: number[][]
      elevation: number
    }[]
  }[]
}

export type Building = District['buildings'][number]
export type Road = District['roads'][number]
export type Surface = District['surfaces'][number]
export type Architecture = District['architectures'][number]
export type Point = number[]
export type Shape = { d: string; fill: string; stroke?: string; width?: number; opacity?: number; fillRule?: "evenodd" }
export type SceneObject = { key: number; depth: number; kind: string; shapes: Shape[]; place?: string }
export type Scene = { terrain: (Shape & { height: number })[]; surfaces: SceneObject[]; water: Shape; objects: SceneObject[] }
export type PlayerMap = { groups: Record<string, string>; places: Pick<District["places"][number], "id" | "name" | "group" | "position" | "use" | "entry" | "page">[]; scene: Scene }

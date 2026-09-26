type Transform = { source_height: number; height_m: number; materials?: Record<string, Record<string, unknown>> }
type Image = { mip_levels: number; dimensions: number[] }
type Gltf = {
  scene?: number; scenes: {nodes: number[]}[]
  nodes: {children?: number[]; matrix?: number[]; rotation?: number[]; scale?: number[]; translation?: number[]; mesh: number}[]
  meshes: {primitives: {attributes: {POSITION: number}}[]}[]
  accessors: {min: number[]; max: number[]}[]
  materials: {name: string; pbrMetallicRoughness: Record<string, unknown>}[]
}
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = fileURLToPath(new URL('../', import.meta.url))
const source = resolve(root, 'source-assets/environment-kit')
const destination = resolve(root, 'game/assets/environment')
const manifest: { files: {source: string; output: string; sha256: string; transform?: Transform; image?: Image}[] } = JSON.parse(await readFile(resolve(source, 'asset-manifest.json'), 'utf8'))
const check = process.argv.includes('--check')
assert(process.argv.slice(2).every(arg => arg === '--check'), 'Usage: bun tools/export-environment.ts [--check]')
const sha256 = (data: Uint8Array) => createHash('sha256').update(data).digest('hex')

function exportGlb(input: Buffer, settings: Transform) {
  assert.equal(input.toString('ascii', 0, 4), 'glTF')
  assert.equal(input.readUInt32LE(4), 2)
  assert.equal(input.readUInt32LE(8), input.length)
  assert.equal(input.readUInt32LE(16), 0x4e4f534a)
  const jsonLength = input.readUInt32LE(12)
  const gltf: Gltf = JSON.parse(input.toString('utf8', 20, 20 + jsonLength))
  const scene = gltf.scenes[gltf.scene ?? 0]
  assert.equal(scene.nodes.length, 1, 'Selected models must have one scene root')
  const node = gltf.nodes[scene.nodes[0]]
  assert.equal(node.children, undefined, 'Selected root must directly contain its mesh')
  assert.equal(node.matrix, undefined)
  assert.deepEqual(node.rotation ?? [0, 0, 0, 1], [0, 0, 0, 1])
  assert.deepEqual(node.scale ?? [1, 1, 1], [1, 1, 1])
  const positions = gltf.meshes[node.mesh].primitives.map(p => gltf.accessors[p.attributes.POSITION])
  const bottom = Math.min(...positions.map(a => a.min[1]))
  const height = Math.max(...positions.map(a => a.max[1])) - bottom
  assert(Math.abs(height - settings.source_height) < 1e-6, 'Source height changed')
  const scale = settings.height_m / height
  node.translation = [0, -bottom * scale, 0]
  node.scale = [scale, scale, scale]
  for (const material of gltf.materials) {
    const override = settings.materials?.[material.name]
    if (override) Object.assign(material.pbrMetallicRoughness, override)
  }
  const json = Buffer.from(JSON.stringify(gltf))
  const padded = Buffer.alloc(Math.ceil(json.length / 4) * 4, 0x20)
  json.copy(padded)
  const remaining = input.subarray(20 + jsonLength)
  const header = Buffer.alloc(20)
  header.write('glTF')
  header.writeUInt32LE(2, 4)
  header.writeUInt32LE(20 + padded.length + remaining.length, 8)
  header.writeUInt32LE(padded.length, 12)
  header.writeUInt32LE(0x4e4f534a, 16)
  return Buffer.concat([header, padded, remaining])
}

function exportDds(path: string, image: Image) {
  // Explicit opaque alpha selects BGRA8; Bevy can load it without RGB24 transcoding
  assert.equal(image.mip_levels, 1 + Math.floor(Math.log2(Math.max(...image.dimensions))))
  const result = Bun.spawnSync(['magick', path, '-alpha', 'on', '-define', 'dds:compression=none', 'dds:-'])
  assert.equal(result.exitCode, 0, `DDS export failed: ${Buffer.from(result.stderr).toString('utf8')}`)
  const output = Buffer.from(result.stdout)
  assert.equal(output.toString('ascii', 0, 4), 'DDS ')
  assert.equal(output.readUInt32LE(12), image.dimensions[1])
  assert.equal(output.readUInt32LE(16), image.dimensions[0])
  assert.equal(output.readUInt32LE(28), image.mip_levels, 'DDS must contain the complete mip chain')
  assert.equal(output.readUInt32LE(88), 32, 'DDS must contain uncompressed BGRA8')
  assert.equal(output.readUInt32LE(92), 0x00ff0000)
  assert.equal(output.readUInt32LE(96), 0x0000ff00)
  assert.equal(output.readUInt32LE(100), 0x000000ff)
  assert.equal(output.readUInt32LE(104), 0xff000000)
  let bytes = 128
  for (let level = 0; level < image.mip_levels; level++) {
    bytes += Math.max(1, image.dimensions[0] >> level) * Math.max(1, image.dimensions[1] >> level) * 4
  }
  assert.equal(output.length, bytes, 'DDS mip data must be complete')
  return output
}

const outputs = []
for (const file of manifest.files) {
  const sourcePath = resolve(source, file.source)
  const input = await readFile(sourcePath)
  assert.equal(sha256(input), file.sha256, `Source checksum differs: ${file.source}`)
  const output = file.transform ? exportGlb(input, file.transform) : file.image ? exportDds(sourcePath, file.image) : input
  outputs.push({ file: file.output, bytes: output.length, sha256: sha256(output) })
  const path = resolve(destination, file.output)
  if (check) {
    assert(output.equals(await readFile(path)), `Export differs: ${file.output}`)
  } else {
    await mkdir(dirname(path), { recursive: true })
    await writeFile(path, output)
  }
}
const generated = Buffer.from(`${JSON.stringify({ source: 'source-assets/environment-kit/asset-manifest.json', license: 'CC0-1.0', files: outputs }, null, 2)}\n`)
const generatedPath = resolve(destination, 'manifest.json')
if (check) assert(generated.equals(await readFile(generatedPath)), 'Runtime manifest differs')
else await writeFile(generatedPath, generated)
console.log(`PASS: ${check ? 'verified' : 'exported'} ${outputs.length} environment files (${outputs.reduce((sum, file) => sum + file.bytes, 0)} bytes)`)

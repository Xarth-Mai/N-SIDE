import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = fileURLToPath(new URL('../', import.meta.url))
const source = resolve(root, 'source-assets/ui-kit')
const destination = resolve(root, 'game/assets/ui')
const manifest: { files: { source: string; output: string; sha256: string; bytes: number }[] } = JSON.parse(await readFile(resolve(source, 'asset-manifest.json'), 'utf8'))
assert(process.argv.slice(2).every(arg => arg === '--check'), 'Usage: bun tools/export-ui.ts [--check]')
const check = process.argv.includes('--check')
assert(manifest.files.length > 0, 'UI asset manifest must contain files')
for (const file of manifest.files) {
  const input = await readFile(resolve(source, file.source))
  assert.equal(createHash('sha256').update(input).digest('hex'), file.sha256, `UI source checksum differs: ${file.source}`)
  assert.equal(input.length, file.bytes, `UI source size differs: ${file.source}`)
  const path = resolve(destination, file.output)
  if (check) assert(input.equals(await readFile(path)), `UI export differs: ${file.output}`)
  else {
    await mkdir(dirname(path), { recursive: true })
    await writeFile(path, input)
  }
}
console.log(`PASS: ${check ? 'verified' : 'exported'} ${manifest.files.length} UI files`)

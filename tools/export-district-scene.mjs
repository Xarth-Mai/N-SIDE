import { mkdtemp, readFile, rm, mkdir } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'

const check = process.argv.includes('--check')
const root = resolve(import.meta.dir, '..')
const temporary = await mkdtemp(join(tmpdir(), 'nside-signs-'))
try {
  const output = join(root, 'game/assets/environment/signs')
  if (!check) await mkdir(output, { recursive: true })
  const names = ['shop', 'grocer', 'cafe', 'bakery', 'drygoods'].flatMap(name => [name, `${name}-display`])
  for (const name of names) {
    const generated = join(temporary, `${name}.png`)
    const process = Bun.spawn(['magick', '-background', 'none', join(root, `source-assets/district-scene/${name}.svg`), '-define', 'png:exclude-chunks=date,time', generated], { stdout: 'inherit', stderr: 'inherit' })
    if (await process.exited !== 0) throw new Error(`Sign export failed: ${name}`)
    const bytes = await readFile(generated)
    const destination = join(output, `${name}.png`)
    if (check) {
      if (!bytes.equals(await readFile(destination))) throw new Error(`Stale sign: ${destination}`)
    } else await Bun.write(destination, bytes)
  }
  console.log(`${check ? 'Verified' : 'Exported'} ${names.length} original storefront graphics`)
} finally { await rm(temporary, { recursive: true, force: true }) }

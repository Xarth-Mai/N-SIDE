import { mkdtemp, readFile, rm, mkdir } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'

const check = process.argv.includes('--check')
const root = resolve(import.meta.dir, '..')
const temporary = await mkdtemp(join(tmpdir(), 'nside-signs-'))
try {
  // Use the project font for editable Chinese SVG text without installing it system-wide
  if (!await Bun.file(join(root, 'source-assets/ui-kit/fonts/NotoSansSC-VF.ttf')).exists()) throw new Error('Missing project font: source-assets/ui-kit/fonts/NotoSansSC-VF.ttf')
  const fontconfig = join(temporary, 'fonts.conf')
  const fontDirectory = join(root, 'source-assets/ui-kit/fonts').replaceAll('&', '&amp;').replaceAll('<', '&lt;')
  await Bun.write(fontconfig, `<fontconfig><include ignore_missing="yes">/etc/fonts/fonts.conf</include><dir>${fontDirectory}</dir><cachedir>${temporary}/font-cache</cachedir></fontconfig>`)
  const output = join(root, 'game/assets/environment/signs')
  if (!check) await mkdir(output, { recursive: true })
  const citySigns = ['station', 'byte-beat', 'frame', 'playroom', 'after9']
  const names = [...['shop', 'grocer', 'cafe', 'bakery', 'drygoods'].flatMap(name => [name, `${name}-display`]), ...citySigns]
  for (const name of names) {
    const generated = join(temporary, `${name}.png`)
    const process = Bun.spawn(['magick', '-background', 'none', join(root, `source-assets/district-scene/${name}.svg`), '-define', 'png:exclude-chunks=date,time', generated], { stdout: 'inherit', stderr: 'inherit', env: name === 'shop' || citySigns.includes(name) ? { ...Bun.env, FONTCONFIG_FILE: fontconfig, XDG_CACHE_HOME: temporary } : Bun.env })
    if (await process.exited !== 0) throw new Error(`Sign export failed: ${name}`)
    const bytes = await readFile(generated)
    const destination = join(output, `${name}.png`)
    if (check) {
      if (!bytes.equals(await readFile(destination))) throw new Error(`Stale sign: ${destination}`)
    } else await Bun.write(destination, bytes)
  }
  console.log(`${check ? 'Verified' : 'Exported'} ${names.length} original storefront graphics`)
} finally { await rm(temporary, { recursive: true, force: true }) }

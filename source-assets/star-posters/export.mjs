import { mkdtemp, readFile, mkdir, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const source = dirname(fileURLToPath(import.meta.url))
const root = resolve(source, '../..')
const output = join(root, 'game/assets/environment/posters/anke-sunny.png')
const temporary = await mkdtemp(join(tmpdir(), 'nside-star-poster-'))
try {
  const font = join(root, 'source-assets/ui-kit/fonts/NotoSansSC-VF.ttf')
  if (!await Bun.file(font).exists()) throw new Error(`Missing project font: ${font}`)
  const fontconfig = join(temporary, 'fonts.conf')
  const fontDirectory = dirname(font).replaceAll('&', '&amp;').replaceAll('<', '&lt;')
  await Bun.write(fontconfig, `<fontconfig><include ignore_missing="yes">/etc/fonts/fonts.conf</include><dir>${fontDirectory}</dir><cachedir>${temporary}/font-cache</cachedir></fontconfig>`)
  const generated = join(temporary, 'anke-sunny.png')
  // librsvg blocks external image references; embed the retained source at export time
  const svg = (await Bun.file(join(source, 'anke-sunny.svg')).text()).replace('anke-portrait.png', `data:image/png;base64,${(await readFile(join(source, 'anke-portrait.png'))).toString('base64')}`)
  const embedded = join(temporary, 'anke-sunny.svg')
  await Bun.write(embedded, svg)
  const child = Bun.spawn(['magick', '-background', 'none', embedded, '-define', 'png:exclude-chunks=date,time', generated], {
    cwd: source, stdout: 'inherit', stderr: 'inherit', env: { ...Bun.env, FONTCONFIG_FILE: fontconfig, XDG_CACHE_HOME: temporary },
  })
  if (await child.exited !== 0) throw new Error('Anke poster export failed')
  const bytes = await readFile(generated)
  if (process.argv.includes('--check')) {
    if (!bytes.equals(await readFile(output))) throw new Error(`Stale poster: ${output}`)
  } else {
    await mkdir(dirname(output), { recursive: true })
    await Bun.write(output, bytes)
  }
  console.log(`${process.argv.includes('--check') ? 'Verified' : 'Exported'} Anke poster: 1600 × 900`)
} finally {
  await rm(temporary, { recursive: true, force: true })
}

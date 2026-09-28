import { createHash } from 'node:crypto'
import { createReadStream, constants, chmodSync, copyFileSync, existsSync, lstatSync, mkdirSync, openSync, closeSync, readSync, readFileSync, readdirSync, realpathSync, rmSync, statSync, writeFileSync } from 'node:fs'
import { dirname, isAbsolute, join, relative, resolve } from 'node:path'

const MANIFEST = 'package-manifest.json'
const COPIES = [
  ['game/assets', 'game/assets'],
  ['source-assets/district-map/district.json', 'source-assets/district-map/district.json'],
  ['source-assets/district-scene/appearance.json', 'source-assets/district-scene/appearance.json'],
  ['source-assets/district-scene/daylight.json', 'source-assets/district-scene/daylight.json'],
  ['source-assets/environment-kit/licenses', 'licenses/environment'],
  ['source-assets/environment-kit/asset-manifest.json', 'licenses/environment-assets.json'],
  ['source-assets/ui-kit/licenses', 'licenses/ui'],
  ['source-assets/ui-kit/asset-manifest.json', 'licenses/ui-assets.json'],
  ['third_party/parry', 'licenses/parry'],
  ['LICENSE', 'LICENSE'],
  ['game/Cargo.toml', 'game/Cargo.toml'],
  ['game/Cargo.lock', 'game/Cargo.lock'],
  ['game/src', 'game/src'],
  ['game/capture', 'game/capture'],
  ['source-assets/ui-kit/tokens.json', 'source-assets/ui-kit/tokens.json'],
] as const
const REQUIRED = [
  'game/n-side', 'game/map_viewer', 'run-game.sh', 'run-walk-preview.sh', 'run-viewer.sh',
  'README.md', 'LICENSE', 'game/Cargo.toml', 'game/Cargo.lock', 'game/src/main.rs',
  'source-assets/district-map/district.json', 'source-assets/district-scene/appearance.json',
  'source-assets/district-scene/daylight.json', 'source-assets/ui-kit/tokens.json',
  'licenses/environment/CC0-1.0.txt', 'licenses/environment/kenney-nature.txt',
  'licenses/environment/kenney-roads.txt', 'licenses/ui/OFL.txt', 'licenses/ui/COPYRIGHT.txt',
  'licenses/parry/LICENSE', 'licenses/parry/UPSTREAM.md', 'licenses/environment-assets.json',
  'licenses/ui-assets.json', 'game/assets/ui/fonts/NotoSansSC-VF.ttf',
]
type FileEntry = { path: string; bytes: number; sha256: string; mode: number }
type Manifest = {
  version: 1; kind: 'n-side-linux-local-preview'; stripped: boolean
  source: { commit: string; worktree_changes: string[] }
  inputs: { game: { bytes: number; sha256: string }; viewer: { bytes: number; sha256: string } }
  files: FileEntry[]
}

function files(directory: string): string[] {
  if (!lstatSync(directory).isDirectory()) throw new Error(`not a directory: ${directory}`)
  return readdirSync(directory).sort().flatMap(name => {
    const path = join(directory, name), info = lstatSync(path)
    if (info.isDirectory()) return files(path)
    if (!info.isFile()) throw new Error(`symlinks and special files are not supported: ${path}`)
    return [path]
  })
}

async function digest(path: string) {
  const hash = createHash('sha256')
  for await (const chunk of createReadStream(path)) hash.update(chunk)
  return { bytes: statSync(path).size, sha256: hash.digest('hex') }
}

function copy(source: string, target: string, mode = 0o644) {
  if (!lstatSync(source).isFile()) throw new Error(`expected a regular source file: ${source}`)
  mkdirSync(dirname(target), { recursive: true })
  // Copy-on-write when available; never hard-link a binary which --strip will modify
  copyFileSync(source, target, constants.COPYFILE_FICLONE)
  chmodSync(target, mode)
}

function linuxBinary(path: string) {
  const info = lstatSync(path)
  if (!info.isFile() || !(info.mode & 0o111)) throw new Error(`not an executable regular file: ${path}`)
  const descriptor = openSync(path, 'r'), magic = Buffer.alloc(4)
  try { readSync(descriptor, magic, 0, magic.length, 0) } finally { closeSync(descriptor) }
  if (!magic.equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46]))) throw new Error(`not a Linux ELF binary: ${path}`)
}

function launcher(binary: string, walk = false) {
  return `#!/bin/sh
set -eu
package_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
exec "$package_root/game/${binary}" --project-root "$package_root"${walk ? ' --walk-preview' : ''} "$@"
`
}

const README = `# N:SIDE Linux 室外预览

完整解压并保留目录结构，在图形桌面中运行 ./run-walk-preview.sh，再选择「进入街区」
当前为白天室外街区与中性人物预览，尚未开放室内、委托、潜梦和战斗，不是完整 Demo
不需要安装 Rust 或 Bun，系统仍需提供兼容的 Linux 动态库与 Vulkan 显卡驱动

- ./run-walk-preview.sh：室外步行
- ./run-game.sh：标题与固定镜头街景
- ./run-viewer.sh：自由相机街区 Viewer
- 启动脚本可以从其他目录调用，也可透传 --help 等参数

步行：WASD／左摇杆移动，右键拖动／Q E／右摇杆观察，F／南键查看近处地点，Esc／东键返回，Tab／Start暂停，R／Select回到起点
地点说明支持点击返回；失焦或手柄断开会暂停，放开旧输入后明确继续
标题或暂停中的设置提供100%／125%文字、100%／65%镜头；显示已保存后下次启动继续使用，行走位置不存档
从月台杂货沿上坡街道、上街住宅、山脚休息台与三处回望台到摘星台，可沿原路返回

## 来源与许可

这是本地预览运行目录，不代表公开发布批准
项目许可证全文在 LICENSE；项目源代码与构建声明在 game/src、game/Cargo.toml、game/Cargo.lock，编译所需配置与录制样例保留原路径
源码按项目 MPL-2.0 提供；第三方素材声明在 licenses/ 与 game/assets/ 中，保留各自来源与条款
package-manifest.json 记录来源提交、相关工作区改动、输入二进制和包内文件的 SHA-256；输入二进制由打包者提供，清单不独立证明其编译来源
公开分发前还需完成所有链接依赖的许可与运行兼容性审计；当前保留的素材及 Parry 声明不等于全部 crate 许可清单
清单不包含自身哈希，也不是数字签名；若需要复验，使用项目中的 bun tools/package-preview.ts --check 解压目录
`

export async function packagePreview(options: { root: string; binary: string; viewer: string; output: string; strip?: boolean }) {
  const root = realpathSync(options.root), output = resolve(options.output)
  if (existsSync(output)) throw new Error(`output must be a new directory: ${output}`)
  const binaries = { game: realpathSync(options.binary), viewer: realpathSync(options.viewer) }
  for (const path of Object.values(binaries)) linuxBinary(path)
  const strip = options.strip ? Bun.which('strip') : undefined
  if (options.strip && !strip) throw new Error('--strip requested but strip is unavailable')
  const git = (args: string[]) => {
    const result = Bun.spawnSync(['git', '-C', root, ...args], { stdout: 'pipe', stderr: 'pipe' })
    if (result.exitCode !== 0) throw new Error(result.stderr.toString().trim())
    return result.stdout.toString().trimEnd()
  }
  const commit = git(['rev-parse', 'HEAD'])
  if (!/^[a-f0-9]{40}$/.test(commit)) throw new Error('source requires a Git commit')
  const worktree = git(['status', '--porcelain', '--untracked-files=all', '--', ...COPIES.map(([source]) => source)])
  const inputs = { game: await digest(binaries.game), viewer: await digest(binaries.viewer) }
  // Inspect every selected source before creating output, including all license files
  const copies = COPIES.flatMap(([source, target]) => {
    const from = join(root, source)
    if (output === from || output.startsWith(`${from}/`)) throw new Error(`output must not be inside a source: ${source}`)
    if (lstatSync(from).isDirectory()) return files(from).map(path => [path, join(output, target, relative(from, path))])
    if (!lstatSync(from).isFile()) throw new Error(`unsupported source: ${from}`)
    return [[from, join(output, target)]]
  })
  mkdirSync(dirname(output), { recursive: true })
  mkdirSync(output)
  try {
    for (const [source, target] of copies) copy(source, target)
    copy(binaries.game, join(output, 'game/n-side'), 0o755)
    copy(binaries.viewer, join(output, 'game/map_viewer'), 0o755)
    for (const [name, input] of [['n-side', inputs.game], ['map_viewer', inputs.viewer]] as const) {
      const copied = await digest(join(output, 'game', name))
      if (copied.sha256 !== input.sha256 || copied.bytes !== input.bytes) throw new Error(`input binary changed during packaging: ${name}`)
    }
    for (const [name, body] of [
      ['run-game.sh', launcher('n-side')],
      ['run-walk-preview.sh', launcher('n-side', true)],
      ['run-viewer.sh', launcher('map_viewer')],
      ['README.md', README],
    ]) {
      writeFileSync(join(output, name), body)
      chmodSync(join(output, name), name.endsWith('.sh') ? 0o755 : 0o644)
    }
    if (strip) {
      const command = Bun.spawn([strip, '--strip-debug', join(output, 'game/n-side'), join(output, 'game/map_viewer')], { stdout: 'pipe', stderr: 'pipe' })
      const error = await new Response(command.stderr).text()
      if (await command.exited !== 0) throw new Error(`strip failed: ${error.trim()}`)
    }
    const manifest: Manifest = { version: 1, kind: 'n-side-linux-local-preview', stripped: !!strip,
      source: { commit, worktree_changes: worktree ? worktree.split('\n') : [] }, inputs, files: [] }
    for (const path of files(output)) manifest.files.push({ path: relative(output, path), ...await digest(path), mode: statSync(path).mode & 0o777 })
    writeFileSync(join(output, MANIFEST), JSON.stringify(manifest, null, 2) + '\n')
    chmodSync(join(output, MANIFEST), 0o644)
    await checkPreview(output)
    return manifest
  } catch (error) {
    // Only the new directory owned by this invocation is removed on failure
    rmSync(output, { recursive: true, force: true })
    throw error
  }
}

export async function checkPreview(directory: string) {
  const root = resolve(directory), manifestPath = join(root, MANIFEST)
  const actual = files(root).map(path => relative(root, path)).filter(path => path !== MANIFEST)
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8')) as Manifest
  if (manifest?.version !== 1 || manifest.kind !== 'n-side-linux-local-preview' || typeof manifest.stripped !== 'boolean'
    || !/^[a-f0-9]{40}$/.test(manifest.source?.commit ?? '') || !Array.isArray(manifest.source?.worktree_changes)
    || !manifest.source.worktree_changes.every(value => typeof value === 'string')
    || !Array.isArray(manifest.files) || !manifest.files.length) throw new Error('invalid package manifest schema')
  for (const input of [manifest.inputs?.game, manifest.inputs?.viewer]) {
    if (!input || !Number.isSafeInteger(input.bytes) || input.bytes <= 0 || !/^[a-f0-9]{64}$/.test(input.sha256)) throw new Error('invalid input binary provenance')
  }
  const expected = new Set<string>()
  for (const entry of manifest.files) {
    if (!entry || typeof entry.path !== 'string' || !entry.path || isAbsolute(entry.path) || entry.path.includes('\\')
      || entry.path.split('/').some(part => !part || part === '.' || part === '..') || entry.path === MANIFEST
      || expected.has(entry.path) || !Number.isSafeInteger(entry.bytes) || entry.bytes < 0
      || !/^[a-f0-9]{64}$/.test(entry.sha256) || ![0o644, 0o755].includes(entry.mode)) throw new Error('invalid package file entry')
    expected.add(entry.path)
  }
  for (const path of REQUIRED) if (!expected.has(path)) throw new Error(`manifest omits required file: ${path}`)
  for (const path of expected) if (!actual.includes(path)) throw new Error(`missing package file: ${path}`)
  for (const path of actual) if (!expected.has(path)) throw new Error(`unexpected package file: ${path}`)
  for (const entry of manifest.files) {
    const path = join(root, entry.path), value = await digest(path)
    if (value.sha256 !== entry.sha256 || value.bytes !== entry.bytes) throw new Error(`changed package file: ${entry.path}`)
    if ((statSync(path).mode & 0o777) !== entry.mode) throw new Error(`changed package permissions: ${entry.path}`)
  }
  return manifest
}

if (import.meta.main) {
  try {
    const args = process.argv.slice(2)
    if (args.length === 2 && args[0] === '--check') {
      const manifest = await checkPreview(args[1])
      console.log(`PASS: ${manifest.files.length} package files`)
    } else {
      const options: Record<string, string> = {}
      let strip = false
      for (let index = 0; index < args.length; index++) {
        const arg = args[index]
        if (arg === '--strip' && !strip) { strip = true; continue }
        if (!['--binary', '--viewer', '--output'].includes(arg) || options[arg] || !args[index + 1] || args[index + 1].startsWith('--')) throw new Error('usage: --binary PATH --viewer PATH --output NEW_DIRECTORY [--strip] | --check DIRECTORY')
        options[arg] = args[++index]
      }
      if (!options['--binary'] || !options['--viewer'] || !options['--output']) throw new Error('--binary, --viewer and --output are required')
      const manifest = await packagePreview({ root: resolve(import.meta.dir, '..'), binary: options['--binary'], viewer: options['--viewer'], output: options['--output'], strip })
      console.log(`PASS: created ${resolve(options['--output'])}, ${manifest.files.length} package files`)
    }
  } catch (error) { console.error(`FAIL: ${error instanceof Error ? error.message : String(error)}`); process.exitCode = 1 }
}

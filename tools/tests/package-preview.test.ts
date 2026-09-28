import { expect, test } from 'bun:test'
import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, symlinkSync, unlinkSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { checkPreview, packagePreview } from '../package-preview.ts'

function fixture() {
  const base = mkdtempSync(join(tmpdir(), 'n-side-package-')), root = join(base, 'source'), output = join(base, 'preview with spaces')
  mkdirSync(root)
  const put = (path: string, contents = path) => {
    const full = join(root, path)
    mkdirSync(dirname(full), { recursive: true }); writeFileSync(full, contents)
    return full
  }
  for (const path of [
    'LICENSE', 'game/Cargo.toml', 'game/Cargo.lock', 'game/src/main.rs', 'game/capture/game-entry.json',
    'game/assets/ui/fonts/NotoSansSC-VF.ttf', 'game/assets/ui/licenses/OFL.txt',
    'source-assets/district-map/district.json', 'source-assets/district-scene/appearance.json',
    'source-assets/district-scene/daylight.json', 'source-assets/ui-kit/tokens.json',
    'source-assets/environment-kit/licenses/CC0-1.0.txt', 'source-assets/environment-kit/licenses/kenney-nature.txt',
    'source-assets/environment-kit/licenses/kenney-roads.txt', 'source-assets/environment-kit/asset-manifest.json',
    'source-assets/ui-kit/licenses/OFL.txt', 'source-assets/ui-kit/licenses/COPYRIGHT.txt', 'source-assets/ui-kit/asset-manifest.json',
    'third_party/parry/LICENSE', 'third_party/parry/UPSTREAM.md',
  ]) put(path)
  for (const args of [
    ['init', '--quiet'],
    ['-c', 'user.name=Package Test', '-c', 'user.email=package-test@example.invalid', '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', 'commit', '--allow-empty', '--quiet', '-m', 'fixture'],
  ]) {
    const result = Bun.spawnSync(['git', '-C', root, ...args], { stdout: 'pipe', stderr: 'pipe' })
    if (result.exitCode) throw new Error(result.stderr.toString())
  }
  const binary = join(base, 'source game'), viewer = join(base, 'source viewer')
  copyFileSync('/bin/echo', binary); chmodSync(binary, 0o755)
  copyFileSync('/bin/false', viewer); chmodSync(viewer, 0o755)
  return { base, root, output, binary, viewer, put, cleanup: () => rmSync(base, { recursive: true, force: true }) }
}

test('packages explicit ELF binaries, sources and licenses; launchers handle cwd, spaces, argv and exit status', async () => {
  const f = fixture()
  try {
    const manifest = await packagePreview(f)
    expect(manifest.source.commit).toMatch(/^[a-f0-9]{40}$/)
    expect(manifest.source.worktree_changes.length).toBeGreaterThan(0)
    expect(readFileSync(join(f.output, 'game/src/main.rs'), 'utf8')).toBe('game/src/main.rs')
    expect(readFileSync(join(f.output, 'licenses/ui/OFL.txt'), 'utf8')).toBe('source-assets/ui-kit/licenses/OFL.txt')
    expect(readFileSync(join(f.output, 'LICENSE'), 'utf8')).toBe('LICENSE')
    expect(readFileSync(join(f.output, 'README.md'), 'utf8')).toContain('不等于全部 crate 许可清单')
    const argument = 'two lines\nwith spaces; $(not-a-command)'
    for (const script of ['run-game.sh', 'run-walk-preview.sh']) {
      const result = Bun.spawnSync([join(f.output, script), argument], { cwd: tmpdir(), stdout: 'pipe', stderr: 'pipe' })
      expect(result.exitCode).toBe(0)
      expect(result.stdout.toString()).toBe(`--project-root ${f.output}${script === 'run-walk-preview.sh' ? ' --walk-preview' : ''} ${argument}\n`)
    }
    expect(Bun.spawnSync([join(f.output, 'run-viewer.sh')], { cwd: tmpdir(), stdout: 'pipe', stderr: 'pipe' }).exitCode).toBe(1)
    const path = join(f.output, 'package-manifest.json'), before = readFileSync(path), mtime = statSync(path).mtimeMs
    await checkPreview(f.output)
    const cli = Bun.spawnSync(['bun', resolve('tools/package-preview.ts'), '--check', f.output], { cwd: tmpdir(), stdout: 'pipe', stderr: 'pipe' })
    expect(cli.exitCode).toBe(0)
    expect(cli.stdout.toString()).toContain('PASS:')
    expect(readFileSync(path)).toEqual(before)
    expect(statSync(path).mtimeMs).toBe(mtime)
  } finally { f.cleanup() }
})

test('read-only check rejects missing, modified, additional and linked files plus broken manifests', async () => {
  const f = fixture()
  try {
    await packagePreview(f)
    const file = join(f.output, 'source-assets/district-map/district.json'), original = readFileSync(file)
    unlinkSync(file)
    await expect(checkPreview(f.output)).rejects.toThrow('missing package file')
    writeFileSync(file, 'changed')
    await expect(checkPreview(f.output)).rejects.toThrow('changed package file')
    writeFileSync(file, original)
    chmodSync(file, 0o644)
    writeFileSync(join(f.output, 'extra.txt'), 'extra')
    await expect(checkPreview(f.output)).rejects.toThrow('unexpected package file')
    unlinkSync(join(f.output, 'extra.txt'))
    symlinkSync(f.binary, join(f.output, 'link'))
    await expect(checkPreview(f.output)).rejects.toThrow('symlinks')
    unlinkSync(join(f.output, 'link'))
    chmodSync(join(f.output, 'run-game.sh'), 0o644)
    await expect(checkPreview(f.output)).rejects.toThrow('changed package permissions')
    chmodSync(join(f.output, 'run-game.sh'), 0o755)
    const path = join(f.output, 'package-manifest.json'), text = readFileSync(path, 'utf8'), manifest = JSON.parse(text)
    for (const value of [{}, { ...manifest, version: 2 }, { ...manifest, files: [] }, { ...manifest, files: [{ ...manifest.files[0], path: '../outside' }] }]) {
      writeFileSync(path, JSON.stringify(value))
      await expect(checkPreview(f.output)).rejects.toThrow()
    }
    writeFileSync(path, JSON.stringify({ ...manifest, files: manifest.files.filter((entry: { path: string }) => entry.path !== 'LICENSE') }))
    await expect(checkPreview(f.output)).rejects.toThrow('omits required file: LICENSE')
    writeFileSync(path, text)
    await checkPreview(f.output)
  } finally { f.cleanup() }
})

test('creation refuses existing output, non-ELF and linked assets without changing inputs', async () => {
  const f = fixture()
  try {
    mkdirSync(f.output)
    writeFileSync(join(f.output, 'keep'), 'existing data')
    await expect(packagePreview(f)).rejects.toThrow('new directory')
    expect(readFileSync(join(f.output, 'keep'), 'utf8')).toBe('existing data')
    rmSync(f.output, { recursive: true })
    await expect(packagePreview({ ...f, output: join(f.root, 'game/assets/preview') })).rejects.toThrow('inside a source')
    expect(existsSync(join(f.root, 'game/assets/preview'))).toBe(false)
    const invalid = f.put('invalid', '#!/bin/sh\nexit 0\n'); chmodSync(invalid, 0o755)
    await expect(packagePreview({ ...f, binary: invalid })).rejects.toThrow('ELF')
    expect(existsSync(f.output)).toBe(false)
    symlinkSync(f.binary, join(f.root, 'game/assets/external'))
    await expect(packagePreview(f)).rejects.toThrow('symlinks')
    expect(existsSync(f.output)).toBe(false)
  } finally { f.cleanup() }
})

test.skipIf(!Bun.which('strip'))('explicit strip modifies only package copies and records resulting hashes', async () => {
  const f = fixture()
  try {
    const before = readFileSync(f.binary), viewerBefore = readFileSync(f.viewer)
    const manifest = await packagePreview({ ...f, strip: true })
    expect(manifest.stripped).toBe(true)
    expect(readFileSync(f.binary)).toEqual(before)
    expect(readFileSync(f.viewer)).toEqual(viewerBefore)
    expect(statSync(join(f.output, 'game/n-side')).ino).not.toBe(statSync(f.binary).ino)
    await checkPreview(f.output)
  } finally { f.cleanup() }
})

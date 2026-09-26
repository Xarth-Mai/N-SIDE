export {}
// vue-tsc patches the JS compiler through Node's require hook; Bun skips that hook
// TypeScript 7 checks tools; Vue uses the latest official TypeScript 6 compatibility API
const result = Bun.spawn(['node', '-e', `
  process.argv.splice(1, 0, 'vue-tsc')
  require('vue-tsc').run(require.resolve('@typescript/typescript6/lib/tsc'))
`, '--', ...process.argv.slice(2)], { stdout: 'inherit', stderr: 'inherit' })
process.exitCode = await result.exited

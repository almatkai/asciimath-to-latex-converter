import { defineConfig } from 'vitest/config'
import path from 'node:path'
const dependencies=['react','react-dom','katex','dompurify','marked','marked-highlight','highlight.js']
export default defineConfig({
  esbuild:{jsx:'automatic'},
  resolve:{alias:[{find:'~beta',replacement:path.resolve('../../harness-work/renderer/src/renderer')},...dependencies.map(name=>({find:name,replacement:path.resolve('node_modules',name)}))],dedupe:['react','react-dom']},
  test:{environment:'jsdom',include:['*.test.ts','*.test.tsx'],pool:'forks',maxWorkers:1,minWorkers:1},
})

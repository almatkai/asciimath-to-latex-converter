// @vitest-environment node
import { build } from 'esbuild'
import { it, expect } from 'vitest'
import path from 'node:path'
import fs from 'node:fs'
import { createHash } from 'node:crypto'
it('bundles actual KaTeX CSS and fonts locally with the production renderer',async()=>{
 const result=await build({entryPoints:['../host-files/src/renderer/ui/NativeAsciiMathPreview.tsx'],bundle:true,write:false,outdir:'../../harness-work/math-bundle',platform:'browser',jsx:'automatic',nodePaths:[path.resolve('node_modules')],loader:{'.woff':'file','.woff2':'file','.ttf':'file'}})
 const css=result.outputFiles.find(f=>f.path.endsWith('.css'))!.text
 expect(result.outputFiles.filter(f=>/\.(woff2?|ttf)$/.test(f.path)).length).toBeGreaterThan(50)
 expect(css).toContain('KaTeX_Main');expect(css).not.toMatch(/url\(["']?https?:/)
 const hash=(b:Buffer)=>createHash('sha256').update(b).digest('hex')
 expect(hash(fs.readFileSync('node_modules/katex/dist/katex.min.css'))).toBe(hash(fs.readFileSync('../../fixtures/katex/package/dist/katex.min.css')))
 for(const name of fs.readdirSync('../../fixtures/katex/package/dist/fonts'))expect(hash(fs.readFileSync('node_modules/katex/dist/fonts/'+name))).toBe(hash(fs.readFileSync('../../fixtures/katex/package/dist/fonts/'+name)))
})

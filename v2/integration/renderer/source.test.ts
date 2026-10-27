// @vitest-environment node
import React from 'react'
import TestRenderer, { act } from 'react-test-renderer'
import { describe, expect, it } from 'vitest'
import { build } from 'esbuild'
import fs from 'node:fs'
import path from 'node:path'
import { createRequire } from 'node:module'
import vm from 'node:vm'
const root=path.resolve('../..')
const stub=`import React from 'react';export const getPreferenceValues=()=>globalThis.__source.pref;export const useNavigation=()=>({push:v=>{globalThis.__source.preview=v},pop:()=>{globalThis.__source.preview=null}});const c=name=>props=>React.createElement(name,props,props.children,props.actions);export const Form=Object.assign(c('Form'),{TextField:c('TextField'),Dropdown:Object.assign(c('Dropdown'),{Item:c('Item')}),Description:c('Description')});export const Detail=c('Detail');export const ActionPanel=c('ActionPanel');export const Action=Object.assign(c('Action'),{CopyToClipboard:c('Copy'),Paste:c('Paste')});`
async function ownCommand(fail=false){
 const result=await build({entryPoints:[root+'/fixtures/own/src/convert-asciimath-to-latex.tsx'],bundle:true,write:false,format:'cjs',platform:'node',jsx:'automatic',external:['react','react/jsx-runtime'],plugins:[{name:'actual-locked-source',setup(b){b.onResolve({filter:/^@raycast\/api$/},()=>({path:'api',namespace:'stub'}));b.onLoad({filter:/.*/,namespace:'stub'},()=>({contents:stub,loader:'js'}));b.onResolve({filter:/^asciimath-to-latex$/},()=>({path:root+'/fixtures/library/package/build/index.js'}));if(fail)b.onLoad({filter:/library\/package\/build\/index\.js$/},()=>({contents:'exports.default=()=>{throw new Error("private")}',loader:'js'}))}}]})
 const exports:any={};const source:any={pref:{defaultWrapLatex:'single'},preview:null};const module={exports};const sandbox:any={exports,module,require:createRequire(import.meta.url),__source:source};sandbox.globalThis=sandbox;vm.runInNewContext(result.outputFiles[0]!.text,sandbox)
 return {Command:module.exports.default,source}
}
describe('executable own V1 TSX/components/helper and actual locked library',()=>{
 it('runs preference variants, real callbacks, wrappers, navigation and source actions',async()=>{
  for(const wrap of ['none','single','double']){
   const {Command,source}=await ownCommand();source.pref.defaultWrapLatex=wrap;let tree:any
   act(()=>{tree=TestRenderer.create(React.createElement(Command))})
   const copy=()=>tree.root.findByType('Copy').props.content;const paste=()=>tree.root.findByType('Paste').props.content
   expect(copy()).toBe(wrap==='none'?'':wrap==='single'?'$$':'$$$$');expect(paste()).toBe(copy())
   expect(tree.root.findAllByType('Action')).toHaveLength(0)
   act(()=>tree.root.findByType('TextField').props.onChange('(x+1)/sqrt(y)'))
   expect(tree.root.findAllByType('Action')[0].props.title).toBe('Show Rendered Preview')
   act(()=>tree.root.findByType('Dropdown').props.onChange('double'))
   const copied=copy();expect(copied.startsWith('$$')).toBe(true)
   act(()=>tree.root.findAllByType('Action')[0].props.onAction())
   let preview:any;act(()=>{preview=TestRenderer.create(source.preview)})
   expect(preview.root.findByType('Detail').props.markdown).toBe(`\n$$\\huge ${copied.slice(2,-2)}$$\n`)
   expect(preview.root.findByType('Copy').props.content).toBe(copied);expect(preview.root.findByType('Paste').props.content).toBe(copied)
   expect(preview.root.findByType('Action').props.title).toBe('Edit Asciimath Expression')
   act(()=>preview.root.findByType('Action').props.onAction());expect(source.preview).toBeNull()
   expect(tree.root.findByType('TextField').props.value).toBe('(x+1)/sqrt(y)');expect(tree.root.findByType('Dropdown').props.value).toBe('double')
   act(()=>tree.root.findByType('TextField').props.onChange('(x_1^)'))
   expect(copy()).toBe('$$Error parsing expression$$')
   act(()=>{tree.unmount();preview.unmount()})
  }
 })
 it('executes the actual helper catch-all using an injected throwing library boundary',async()=>{
  const {Command}=await ownCommand(true);let tree:any;act(()=>{tree=TestRenderer.create(React.createElement(Command))})
  act(()=>tree.root.findByType('TextField').props.onChange('x'))
  expect(tree.root.findByType('Copy').props.content).toBe('$Error parsing expression$')
  expect(tree.root.findAllByType('Description').some((n:any)=>n.props.title==='Conversion Error')).toBe(false)
  act(()=>tree.unmount())
 })
})
it('executes all frozen library-oracle inputs through the actual own helper and live callback',async()=>{
 const {Command}=await ownCommand();let tree:any;act(()=>{tree=TestRenderer.create(React.createElement(Command))})
 const rows=JSON.parse(fs.readFileSync(root+'/fixtures/oracle.json','utf8'))
 for(const row of rows){
  act(()=>tree.root.findByType('TextField').props.onChange(row.input))
  const output=tree.root.findByType('Copy').props.content.slice(1,-1)
  const units=Array.from({length:output.length},(_,i)=>output.charCodeAt(i))
  expect(units,row.input).toEqual(row.outputUnits)
 }
 act(()=>tree.unmount())
})

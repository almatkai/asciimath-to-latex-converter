import React, { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import fs from 'node:fs'
import { ExtensionRuntimeSurface } from '~beta/src/raycast-api/index'
import { FormRuntime, reconcileFormValues } from '~beta/src/raycast-api/form-runtime'
import { DetailRuntime } from '~beta/src/raycast-api/detail-runtime'
import { NativeAsciiMathPreview } from '~beta/ui/NativeAsciiMathPreview'
import { NativeAsciiMathLane, olderAsciiMathView } from '~beta/src/raycast-api/native-asciimath-lane'
import ExtensionRuntimeView from '~beta/ExtensionRuntimeView'
const views=JSON.parse(fs.readFileSync('../../harness-work/views.json','utf8'))
let element:HTMLDivElement,root:Root
beforeEach(()=>{(globalThis as any).IS_REACT_ACT_ENVIRONMENT=true;element=document.createElement('div');document.body.append(element);root=createRoot(element)})
afterEach(async()=>{await act(async()=>root.unmount());element.remove()})
const noop=()=>{}
const click=async(e:Element|null)=>{expect(e).not.toBeNull();await act(async()=>{e!.dispatchEvent(new MouseEvent('click',{bubbles:true}))})}
async function edit(e:HTMLInputElement|HTMLSelectElement,value:string) {
  await act(async()=>{
    const proto=e instanceof HTMLInputElement?HTMLInputElement.prototype:HTMLSelectElement.prototype
    Object.getOwnPropertyDescriptor(proto,'value')!.set!.call(e,value)
    e.dispatchEvent(new Event(e instanceof HTMLSelectElement?'change':'input',{bubbles:true}))
  })
}
function surface(view:any,invoke:any) {return <ExtensionRuntimeSurface {...view} sessionId="native-extension:test" extensionId="com.tezbar.native.asciimath-to-latex-converter" commandName="convert-asciimath-to-latex" onBack={noop} onSearchTextChanged={noop} onLoadMore={noop} onInvokeAction={invoke}/>}
describe('actual patched beta Form, Dropdown and Surface',()=>{
  it('initializes defaults, uses value callbacks and preserves newer queued drafts through stale refreshes',async()=>{
    const calls:any[]=[]
    await act(async()=>root.render(surface(views[0],(...args:any[])=>{calls.push(args)})))
    const input=element.querySelector('input')!,select=element.querySelector('select')!
    expect(select.value).toBe('single')
    await edit(input,'newer draft');await edit(select,'double')
    expect(calls).toEqual([[views[0].root.children[0].props.actionId,{value:'newer draft'}],[views[0].root.children[1].props.actionId,{value:'double'}]])
    for(const view of [views[1],views[0],views[2]])await act(async()=>root.render(surface(view,(...args:any[])=>calls.push(args))))
    expect(input.value).toBe('newer draft');expect(select.value).toBe('double')
    expect(element.textContent).toContain(views[2].root.children[2].props.text)
    // Form parser/reconciliation keeps defaultValue initialization distinct from replacement.
    const fields=[{key:'input:Form.TextField',id:'input',title:'input',type:'Form.TextField',defaultValue:'server',actionId:'a'}]
    expect(reconcileFormValues(fields)).toEqual({input:'server'})
    expect(reconcileFormValues(fields,{input:'typed'},fields)).toEqual({input:'typed'})
  })
  it('submits complete maps and supplies maps to every overlay action in source order',async()=>{
    const calls:any[]=[];const v=views[2]
    await act(async()=>root.render(surface(v,(...args:any[])=>calls.push(args))))
    const form=element.querySelector('form')!
    await act(async()=>{form.dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}))})
    expect(calls.pop()).toEqual([v.actions[0].id,{input:'(x+1)/sqrt(y)',wrapStyle:'double'}])
    await click(element.querySelector('[aria-label="Open actions"]'))
    const actions=Array.from(element.querySelectorAll('button')).filter(b=>v.actions.some((a:any)=>b.textContent?.includes(a.title)))
    expect(actions.map(b=>v.actions.find((a:any)=>b.textContent?.includes(a.title)).title)).toEqual(v.actions.map((a:any)=>a.title))
    await click(actions[2]!)
    expect(calls.pop()).toEqual([v.actions[2].id,{input:'(x+1)/sqrt(y)',wrapStyle:'double'}])
  })
  it('back remount restores exact canonical input and wrap; delayed operations cannot replace drafts',async()=>{
    const calls:any[]=[]
    await act(async()=>root.render(surface(views[3],(...args:any[])=>calls.push(args))))
    await click(Array.from(element.querySelectorAll('button')).find(b=>b.textContent==='Back')!)
    expect(calls[0]).toEqual([views[3].root.props.backActionId,undefined])
    await act(async()=>root.render(surface(views[4],(...args:any[])=>calls.push(args))))
    expect(element.querySelector('input')!.value).toBe('(x+1)/sqrt(y)')
    expect(element.querySelector('select')!.value).toBe('double')
    await edit(element.querySelector('input')!,'typed again')
    await act(async()=>root.render(surface(views[4],noop)))
    expect(element.querySelector('input')!.value).toBe('typed again')
  })
})
describe('offline actual math rendering',()=>{
  it('renders actual Detail math DOM including fraction, root, MathML and huge sizing',async()=>{
    await act(async()=>root.render(<DetailRuntime root={views[3].root} title="LaTeX Preview" onBack={noop} onRunPrimaryAction={noop} onOpenActions={noop}/>))
    expect(element.querySelector('.katex-display .katex .katex-html .mfrac')).not.toBeNull()
    expect(element.querySelector('.katex-html .sqrt')).not.toBeNull()
    // KaTeX's radical is an SVG path; a surviving wrapper alone is not a preview.
    expect(element.querySelector('.katex-html .sqrt svg path')).not.toBeNull()
    expect(element.querySelector('math mfrac')).not.toBeNull()
    expect(element.querySelector('.katex-html .sizing.size10')).not.toBeNull()
    expect(element.querySelector('math msqrt')).not.toBeNull()
    expect(element.querySelector('pre, .tezbar-md-code-copy, img, iframe, script')).toBeNull()
  })
  it('handles unsupported/malformed/oversized math safely and never trusts links or HTML',async()=>{
    for(const latex of ['\\unknownCommand{a}','\\frac{','x'.repeat(16385),'\\href{javascript:alert(1)}{x}','<img src=x onerror=alert(1)>']){
      await act(async()=>root.render(<NativeAsciiMathPreview value={{latex,displayMode:true,size:'huge'}}/>))
      expect(element.querySelector('a,script,img,iframe,[onerror]')).toBeNull()
      expect(element.querySelector('[role="status"],.katex')).not.toBeNull()
    }
  })
})
describe('reconciliation ordering and deliberate effects',()=>{
  it('serializes callbacks, skips queued effects when drafts change and never retries rejected effects',async()=>{
    const lane=new NativeAsciiMathLane();const calls:string[]=[];let release!:()=>void
    const first=lane.send(async()=>{calls.push('old');await new Promise<void>(r=>release=r)},true,noop)
    await Promise.resolve()
    const stale=lane.send(()=>{calls.push('paste')},false,()=>calls.push('skipped'))
    const latest=lane.send(()=>{calls.push('latest')},true,noop)
    release();await Promise.all([first,stale,latest]);expect(calls).toEqual(['old','skipped','latest'])
    await expect(lane.send(()=>{calls.push('copy');throw new Error('private')},false,noop)).rejects.toThrow()
    await lane.send(()=>{calls.push('explicit retry')},false,noop)
    expect(calls.slice(-2)).toEqual(['copy','explicit retry'])
  })
  it('ignores older refreshes in actual RuntimeView and preserves the latest math/output/actions',async()=>{
    let resolveRefresh!:(v:any)=>void
    const initial={...views[0],sessionId:'native-extension:test',extensionId:'com.tezbar.native.asciimath-to-latex-converter',commandName:'convert-asciimath-to-latex'}
    ;(window as any).tezbar={extensionRefreshSession:()=>new Promise(r=>resolveRefresh=r),extensionDisposeSession:async()=>true,extensionInvokeAction:async()=>({...views[2],...Object.fromEntries(['sessionId','extensionId','commandName'].map(k=>[k,initial[k]]))})}
    await act(async()=>root.render(<ExtensionRuntimeView initial={initial} onBack={noop}/>))
    await act(async()=>{await new Promise(r=>setTimeout(r,140))})
    await edit(element.querySelector('input')!,'(x+1)/sqrt(y)')
    expect(element.textContent).toContain(views[2].root.children[2].props.text)
    await act(async()=>resolveRefresh(initial))
    expect(element.textContent).toContain(views[2].root.children[2].props.text)
    expect(element.querySelector('input')!.value).toBe('(x+1)/sqrt(y)')
    expect(olderAsciiMathView(views[2].root,views[0].root)).toBe(true)
  })
})
it('drops old-session action replies after RuntimeView switches sessions',async()=>{
 let release!:(v:any)=>void
 const info={sessionId:'native-extension:old',extensionId:'com.tezbar.native.asciimath-to-latex-converter',commandName:'convert-asciimath-to-latex'}
 const initial={...views[0],...info}
 ;(window as any).tezbar={extensionRefreshSession:async()=>({ok:true,mode:'unchanged'}),extensionDisposeSession:async()=>true,extensionInvokeAction:()=>new Promise(r=>release=r)}
 await act(async()=>root.render(<ExtensionRuntimeView initial={initial} onBack={noop}/>))
 await act(async()=>{element.querySelector('form')!.dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}))})
 await act(async()=>root.render(<ExtensionRuntimeView initial={{...initial,sessionId:'native-extension:new'}} onBack={noop}/>))
 await act(async()=>release({...views[3],...info}))
 expect(element.querySelector('form')).not.toBeNull()
 expect(element.querySelector('.katex')).toBeNull()
})
it('does not send disposed-lifetime callbacks after StrictMode reactivation',async()=>{
 const lane=new NativeAsciiMathLane();const calls:string[]=[]
 const old=lane.send(()=>calls.push('old'),true,noop)
 lane.dispose();lane.activate()
 const next=lane.send(()=>calls.push('new'),true,noop)
 await Promise.all([old,next]);expect(calls).toEqual(['new'])
})

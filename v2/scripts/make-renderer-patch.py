from pathlib import Path
import difflib,json
root=Path(__file__).resolve().parents[1]; fixtures=root/'fixtures/beta'; out=root/'integration/host-files'
files=['src/renderer/src/raycast-api/form-runtime.tsx','src/renderer/src/raycast-api/index.tsx','src/renderer/src/raycast-api/detail-runtime.tsx','src/renderer/ExtensionRuntimeView.tsx','package.json']
for p in files:
 s=(fixtures/p).read_text()
 if p.endswith('form-runtime.tsx'):
  s=s.replace('  value?: unknown','  value?: unknown\n  defaultValue?: unknown')
  s=s.replace('          value: node.props?.value,','          value: node.props?.value,\n          defaultValue: node.props?.defaultValue,')
  s=s.replace("    } else if (field.type === 'Form.Checkbox')", "    } else if (field.defaultValue !== undefined) {\n      next[field.id] = field.defaultValue\n    } else if (field.type === 'Form.Checkbox')")
 elif p.endswith('detail-runtime.tsx'):
  s="import { NativeAsciiMathPreview } from '../../ui/NativeAsciiMathPreview'\n"+s
  s=s.replace("{imageOnly && imageOnly.alt", "{root.props?.nativeAsciiMath ? (\n            <NativeAsciiMathPreview value={root.props.nativeAsciiMath} />\n          ) : imageOnly && imageOnly.alt")
 elif p.endswith('index.tsx'):
  s="import { NativeAsciiMathLane } from './native-asciimath-lane'\n"+s
  at="  const formRuntimeRef = useRef<FormRuntimeHandle>(null)"
  s=s.replace(at,at+'''
  const asciiMath = !!root.props?.nativeAsciiMath
  const [localNotice, setLocalNotice] = useState<string | null>(null)
  const lane = useMemo(() => new NativeAsciiMathLane(), [props.sessionId])
  useEffect(() => { lane.activate(); return () => lane.dispose() }, [lane])
  const invoke = useCallback((id: string, values?: Record<string, unknown>, change = false): Promise<void> => {
    const send = () => onInvokeAction(id, values)
    if (!asciiMath) return Promise.resolve(send())
    setLocalNotice(null)
    return lane.send(send, change, () => setLocalNotice('Expression changed; choose the action again.'))
      .catch(() => { setLocalNotice('Action failed or acknowledgement is uncertain. Retry deliberately.') })
  }, [asciiMath, lane, onInvokeAction])
''')
  # Route existing callbacks through the lane, leaving the send closure above alone.
  a=s.index('  const primaryAction =')
  s=s[:a]+s[a:].replace('void onInvokeAction(', 'void invoke(').replace('[onBack, onInvokeAction, root]', '[onBack, invoke, root]').replace('[onInvokeAction]', '[invoke]').replace('[kind, onInvokeAction, primaryAction]', '[kind, invoke, primaryAction]').replace('[onInvokeAction, submitAction]', '[invoke, submitAction]')
  s=s.replace('onChangeField={(actionId, value) => onInvokeAction(actionId, { value })}', 'onChangeField={(actionId, value) => invoke(actionId, { value }, true)}')
  s=s.replace('onInvokeAction={onInvokeAction}', 'onInvokeAction={invoke}')
  s=s.replace('void invoke(action.id)','void invoke(action.id, asciiMath && kind === \'form\' ? formRuntimeRef.current?.getValues() : undefined)')
  # Notices are plain text, never source private errors.
  s=s.replace('          {showActions && actions.length > 0 ? (', '''          {localNotice ? <p role="status">{localNotice}</p> : null}
          {showActions && actions.length > 0 ? (''')
 elif p.endswith('ExtensionRuntimeView.tsx'):
  s="import { olderAsciiMathView } from './src/raycast-api/native-asciimath-lane'\n"+s
  marker='  const [state, setState] = useState<RuntimeViewState>(() => fromRunResult(initial))'
  s=s.replace(marker,marker+'\n  const currentSessionRef = useRef(state.sessionId)\n  currentSessionRef.current = state.sessionId')
  marker='              const result = await window.tezbar.extensionInvokeAction({\n                sessionId: state.sessionId,\n                actionId,\n                formValues,\n              })'
  s=s.replace(marker,marker+'\n              if (currentSessionRef.current !== state.sessionId) return')

  # Functional updates reject older same-process revisions, including delayed refreshes.
  import re
  pattern=r'setState\(\{(\s*sessionId: result\.sessionId,.*?\n\s*)\}\)'
  s,n=re.subn(pattern,r'setState((previous) => olderAsciiMathView(previous.root, result.root) ? previous : ({\1}))',s,flags=re.S)
  assert n==4,n
 elif p=='package.json':
  pkg=json.loads(s);pkg['devDependencies']['katex']='0.16.25';s=json.dumps(pkg,indent=2)+'\n'
 dest=out/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(s)
# Lock is intentionally produced with pnpm during integration; exact library/SRI fixture provided.
paths=files+['src/renderer/ui/NativeAsciiMathPreview.tsx','src/renderer/src/raycast-api/native-asciimath-lane.ts']
patch=''
for p in paths:
 base=(fixtures/p).read_text() if (fixtures/p).exists() else ''
 patch+=''.join(difflib.unified_diff(base.splitlines(True),(out/p).read_text().splitlines(True),fromfile='a/'+p if base else '/dev/null',tofile='b/'+p))
(root/'integration/required-renderer.patch').write_text(patch)

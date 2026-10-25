import { useMemo } from 'react'
import katex from 'katex'
import DOMPurify from 'dompurify'
import 'katex/dist/katex.min.css'

// This typed Detail surface is deliberately separate from Markdown. Formula text
// never becomes markdown, HTML, a URL, a code block or a clipboard button.
export function NativeAsciiMathPreview({ value }: { value: unknown }): JSX.Element {
  const rendered = useMemo(() => {
    if (!value || typeof value !== 'object') return { error: 'Invalid formula preview.' }
    const data = value as Record<string, unknown>
    if (typeof data.latex !== 'string' || data.latex.length > 16_384 ||
        data.displayMode !== true || data.size !== 'huge') {
      return { error: 'Formula preview exceeds the supported format or size.' }
    }
    try {
      const html = katex.renderToString(`\\huge ${data.latex}`, {
        displayMode: true, output: 'htmlAndMathml', trust: false,
        throwOnError: true, strict: 'ignore', maxExpand: 1000, maxSize: 20,
      })
      return { html: DOMPurify.sanitize(html, { USE_PROFILES: { html: true, mathMl: true, svg: true } }) }
    } catch {
      return { error: 'This LaTeX cannot be rendered. Edit the expression or copy its code.' }
    }
  }, [value])
  return rendered.error
    ? <p role="status">{rendered.error}</p>
    : <div className="overflow-auto py-4" aria-label="Rendered LaTeX" dangerouslySetInnerHTML={{ __html: rendered.html ?? '' }} />
}

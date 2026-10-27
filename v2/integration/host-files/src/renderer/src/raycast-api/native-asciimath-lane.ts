// Session-only invocation lane for the compiled AsciiMath surface. No storage.
export class NativeAsciiMathLane {
  private tail: Promise<void> = Promise.resolve()
  private version = 0
  private disposed = false
  private epoch = 0
  activate(): void { this.disposed = false }
  dispose(): void { this.disposed = true; this.version += 1; this.epoch += 1 }
  send(callback: () => Promise<void> | void, change: boolean, skipped: () => void): Promise<void> {
    if (change) this.version += 1
    const version = this.version
    const epoch = this.epoch
    const task = this.tail.then(async () => {
      if (this.disposed || epoch !== this.epoch) return
      if (!change && version !== this.version) { skipped(); return }
      await callback()
    })
    // A failed/cancelled effect never retries automatically; the lane stays usable.
    this.tail = task.catch(() => undefined)
    return task
  }
}
export function olderAsciiMathView(previous: { props?: Record<string, unknown> }, next: { props?: Record<string, unknown> }): boolean {
  const parse = (value: unknown): RegExpExecArray | null => typeof value === 'string'
    ? /^am:([a-f0-9-]+):([0-9]+)$/.exec(value) : null
  const a = parse(previous.props?.nativeAsciiMathRevision)
  const b = parse(next.props?.nativeAsciiMathRevision)
  return !!(a && b && a[1] === b[1] && BigInt(b[2]!) < BigInt(a[2]!))
}

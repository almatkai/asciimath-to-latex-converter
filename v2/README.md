# AsciiMath to LaTeX Converter — Native V2

Reviewable implementation only. Nothing is installed in Tezbar. Native V2 is compiled into the host; the manifest cannot load Rust code. Both macOS and Windows verification entries remain `not-run`. Host registration/rebuild/install and actual desktop UI, dropdown, formula fonts, clipboard, paste, focus and keyboard QA have not run.

Source base: own V1 `9b40a9cff89c7cfc50360d282ee3a80a43d7ef35`, fetched from fresh `origin/main`; branch `feat/native-v2-rust`. Locked converter is **asciimath-to-latex 0.5.1**, not merely the declared `^0.5.1`. Original source/artwork pin: `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Beta architecture pin: `fd300e276356587726566026f072563eb9c34cb1`. See [PROVENANCE.json](PROVENANCE.json), [NOTICE](NOTICE) and [INDEX.json](INDEX.json) for sources, exact bytes, hashes, SRI and license evidence.

## Behavior and bounded adaptations

The Rust core translates the locked library's native grammar, 342 initialized symbol entries, definition expansion, functions, brackets, text, accents, fractions, scripts and matrix-output quirks. It does not execute JavaScript. UTF-16 intermediate strings reproduce JavaScript indexing. Own TSX/components/helper and actual locked library execute in the oracle/source harness.

The frozen corpus has 3,951 expressions: 3,672 valid-Unicode results match exactly, including the literal helper result `Error parsing expression`; 279 malformed expressions involving astral characters produce unpaired UTF-16 surrogates in the library. Rust/serde JSON cannot represent those as Unicode strings, so those cases return an explicit conversion error instead of replacing or truncating their bytes. This is a supported boundary, not universal parity. All initialized tokens, five token contexts, deterministic mixed malformed expressions, nested fractions/scripts/matrices, empty/whitespace/entities, Unicode and unmatched input are exercised.

Input is limited to 8,192 UTF-8 bytes; parser call depth 96; work budget 250,000; output 131,072 UTF-16 units. Exceeding a resource budget rejects the whole conversion. Live resource/Unicode errors appear as Conversion Error; a library parsing exception still becomes the source's literal output and remains previewable/copyable. Oversized callbacks fail visibly and retain the previous committed conversion. There is no silent truncation.

Command preference `defaultWrapLatex` remains a dropdown, default `single`, with `none`, `single`, `double`. Initial input/output are empty. A wrap change reconverts current input. Copy/paste remain available before there is output: none copies empty, single `$$`, double `$$$$`. Preview precedes Copy/Paste when output is truthy. Preview action order is Copy Latex Code, Edit Asciimath Expression, Paste Latex Code. Edit/Back restores exact input/wrap/output. No new persistence, clipboard initialization, history or V1 store migration exists; the host supplies `payload.preferences`, and undeclared `payload.argumentValues` do not initialize input. Invalid typed preferences are rejected.

The beta Markdown component has no math support. **The separate required renderer patch must be integrated**: typed Detail data is rendered offline by KaTeX 0.16.25, always display math with `\huge`, independently of the chosen copy wrapper. The source-equivalent markdown is retained, but the actual preview uses a dedicated sanitized math component. Unsupported LaTeX and preview inputs over 16,384 UTF-16 units get a safe visible error. KaTeX runs with trust disabled, bounded expansion/size, and local CSS/fonts. No remote renderer or subprocess formula service is used. Markdown code-block copy buttons are bypassed; their unrelated acknowledgement issue is not changed.

Auto-focus and source action-specific shortcuts are not advertised because beta does not implement them. The supported beta Actions menu, form submission, Detail primary action and Back navigation are used. Feedback is ordinary Form/status/message UI, not a claimed HUD/toast/popToRoot. Copy does not claim dismissal or focus changes. Paste uses the existing platform route whose implementation owns copy/hide/focus/wait/paste.

## Session and effect safety

The concrete adapter is [adapter/v2_asciimath.rs](adapter/v2_asciimath.rs). It handles run-command, invoke-action, search-text-changed, refresh-session, load-more and dispose-session through the real beta SDK/common/NativeSessions contracts. Search/load-more have no form mutation; refresh renders committed capabilities without generating state. Session-only callback IDs remain valid for queued field edits; hidden callbacks, old effect/navigation actions, cross-session and forged IDs are rejected. A process UUID plus monotonic counter allocates capabilities outside JSON clones, so discarded clones cannot reuse them.

Fields keep stable IDs and omit controlled `value` in every response. A focused `defaultValue` renderer addition initializes mounts/back without replacing existing drafts. No controlled replacement operation is used, hence no replacement replay/consumption protocol is necessary. Callbacks use `formValues.value`; submits/actions use complete maps. Effects validate that the complete map matches committed formula/wrapper. A per-session renderer lane orders changes/actions and skips an unsent action if a newer draft arrived. Revision guards reject delayed old refresh/action views. Effects already submitted across the OS boundary can be uncertain if cancelled; they are never retried automatically.

Copy invokes genuine SDK `clipboard:write` with `[text]`. Paste invokes `core:platform:paste-text` with `[text]` exactly once. No second clipboard call, window hide, focus, selection, reveal or paste API is invented. Effect intent, canonical text and attempt ID are checkpointed before await. Only an object with `ok:true`, absent/null `error`, absent/true `success` acknowledges success; malformed/primitive/null/transport/contradictory replies fail. Private router errors are sanitized. Acknowledgement is checkpointed before rendering, and later failure cannot erase it. Refresh after cancellation only reports uncertainty. Retry is a new deliberate action and new capability/attempt, using the current committed formula/wrapper.

## Host integration artifacts (unapplied)

1. Vendor this package into `src-tauri/native-extensions/asciimath-to-latex-converter`. `scripts/package-check.py` verifies the actual Cargo archive and extracted offline checks; `scripts/prepare-integration.py` creates a review-only vendor layout. No external Rust plugin loader is involved.
2. Review/apply [integration/registration.patch](integration/registration.patch) against the pinned beta. It adds the Cargo path dependency, sibling `super::v2_sdk` adapter module via `#[path="../../../native-extensions/asciimath-to-latex-converter/adapter/v2_asciimath.rs"]`, runtime slug/commands/handle, appends the manifest and changes the catalog-count assertion 11→12. Existing entries are preserved. `scripts/registration.py --source <exported host tree> --output <patch>` regenerates against an exported target and its actual count without editing that source. Preserve all preceding ports/dependencies/catalog entries; never replace the target catalog wholesale.
3. Review/apply [integration/required-renderer.patch](integration/required-renderer.patch): dedicated math renderer, exact KaTeX dependency, field defaults, complete form maps, invocation lane and stale-view protection. Update the target pnpm lock using pinned `katex@0.16.25` (exact tar/SRI retained); no host lock was edited here. The fixture renderer npm lock is a reproducible test harness, not a replacement host lock. The host frontend build must bundle KaTeX CSS and all fonts locally.
4. Copy exact [assets/icon.png](assets/icon.png) to `src/renderer/public/extensions/icons/asciimath-to-latex-converter.png`; its URL is `/extensions/icons/asciimath-to-latex-converter.png`. It is 16,920 bytes, SHA256 `85d2d2a64b001b214cd2b88d01260f81b9c393863013abf3f7422d08612a1b88`.
5. The committed host observed here is `fe71ca87f21ac4ff31505010c228f8abda0f2a89`; its complete relevant diff from beta is [integration/host-committed-delta.patch](integration/host-committed-delta.patch). Its registry stopped accepting bundled `/extensions/...` icon URLs; [integration/current-host-icon-prerequisite.patch](integration/current-host-icon-prerequisite.patch) restores support for this exact icon only and is for that committed host, not needed on the pinned beta. Other catalog/icon/cache changes are preserved. No preceding vendored ports existed in this observed committed snapshot; the parent's later target may differ.
6. Rebuild the host binary and renderer, then install the canonical `com.tezbar.native.asciimath-to-latex-converter` through Native V2 install. Confirm compiled coverage for `convert-asciimath-to-latex`. Canonical launches require an installed compiled command. The actual beta registry first resolves a Raycast alias to installed canonical state, retaining the legacy package. Before canonical installation, installed Raycast aliases can still enter compiled dispatch according to beta's policy; unrecognized own V1 IDs stay in compatibility execution. Existing preference fallback follows beta registry paths. This port adds no migration or deletion policy.

Registration, canonical install, aliases and preferences are tested **in temporary headless profiles**, using actual beta registry/catalog and patched runtime dispatch. Unrelated compiled adapters and Tauri are explicit stubs. This is not a real host install or desktop QA.

## Reproduction

From this repository, with Rust stable and 1.85.0, both Windows GNU std targets, cached crates, Node/npm and the renderer lock's npm packages available:

```sh
node v2/scripts/oracle.cjs
npm ci --prefix v2/integration/renderer --ignore-scripts --no-audit --no-fund
node v2/scripts/schema.cjs
python3 v2/scripts/validate-rust.py
python3 v2/scripts/validate-renderer.py
python3 v2/scripts/validate-v1.py
python3 v2/scripts/package-check.py
python3 v2/scripts/check-index.py
```

Use `npm ci --offline` once the lock's packages are cached; test execution itself requires no network. `validate-rust.py` runs locked offline core, direct production-adapter and real NativeSessions/registry harness tests; fmt and strict all-target Clippy on stable **and** 1.85.0; Windows GNU check/Clippy only (no Windows execution). Nested harness manifests use `Cargo.toml.template`; fixture host manifests use `Cargo.toml.fixture`, with their original bytes and source paths recorded. The scripts instantiate harness manifests only in disposable copies, ensuring Cargo does not omit nested packages from the archive. Pinned common/SDK fixture tests are included in the direct harness; their temporary storage tests do not mean this adapter persists anything.

`validate-renderer.py` executes own TSX/helper against the full frozen corpus and real library, actual patched beta Form/Dropdown/Detail/Surface/RuntimeView in JSDOM, KaTeX math DOM and font bundling, and a strict TypeScript check with explicit test OS bridge typing. Presentation/OS stubs are stated in the harness. Rendered HTML/MathML DOM and actual CSS/font compilation are evidence; desktop pixels, real font loading and OS interaction remain not-run.

`validate-v1.py` copies immutable own fixtures to a disposable directory under `v2/harness-work`, uses `npm ci --legacy-peer-deps --ignore-scripts`, then original test/typecheck and unchanged-lock verification. It never repairs V1. `package-check.py` extracts an actual `.crate` to a fresh directory, checks every indexed file is present (including nested package/build/dist/font fixtures), then runs cached offline Rust tests without any host or absolute source dependency. Normalized Cargo metadata is expected; immutable original manifest bytes are available as `Cargo.toml.orig` in the archive.

Raw validation logs stay ignored under `v2/test-results`; a compact reviewable evidence report is generated separately. The root README's discovery link is the only permitted V1 tracked-byte change. Everything else is new under `v2/`; all work remains unstaged/uncommitted and the clone remains intact.

XERJ skill was read first. Canonical clone prefix `ax-ebd3516cc229` indexing failed with HTTP429/RSS watermark; only its owned indexing process was stopped. Coverage is incomplete; full current files, `git show` and `rg` supplied exhaustive checks. No global search restart, schema or prefix workaround was used.

Parent review reproduced a missing radical: the HTML/MathML-only sanitizer stripped KaTeX SVG paths while leaving the root wrappers. The dedicated preview now also enables DOMPurify’s SVG profile with KaTeX trust disabled. Its renderer regression requires the actual radical SVG path, while the existing unsafe-link/HTML checks still pass. Host files remain unapplied; desktop QA remains not-run.

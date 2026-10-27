// Test-only OS bridge typing. Real desktop integration and OS behavior remain not-run.
interface Window { tezbar: {
 getAssetIconDataUrl(kind: string, path: string): Promise<string | null>;
 getAppIconDataUrl(path: string): Promise<string | null>;
 clipboardWriteText(text: string): Promise<{ok?:boolean} | null>;
 [key: string]: any;
} }

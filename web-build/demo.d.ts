/* tslint:disable */
/* eslint-disable */

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: (a: number, b: number) => number;
  readonly wasm_bindgen__convert__closures_____invoke__h1969764f01d3ddd9: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__hf0a97ad3a7d40afb: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h270e774e62b586ab: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h1ba2ea4e4548722a: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h08b90047d022d8b7: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h08999c1a5d646b8c: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hbe12ccfcb4ba9183: (a: number, b: number, c: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h90b60eec2016b26b: (a: number, b: number, c: any, d: any) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hf563398f37e960ef: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

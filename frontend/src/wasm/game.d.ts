/* tslint:disable */
/* eslint-disable */

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly main: (a: number, b: number) => number;
    readonly wasm_bindgen__closure__destroy__h03a1e8c6e2342f32: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h9149ef0305527ab1: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h3cd972c9dc7cd6ff: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h6a9eb582f12f865d: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h4a2ad4347a313b6d: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h3bb56ec522261029: (a: number, b: number, c: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h7710776ac70fe669: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hbac8b47fa261c71e: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h8ef78c0ba6911c23: (a: number, b: number) => void;
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

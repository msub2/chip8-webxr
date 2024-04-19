/* tslint:disable */
/* eslint-disable */
/**
* Which particular CHIP-8 interpreter to emulate
*/
export enum Variant {
  CHIP8 = 0,
  SCHIP_LEGACY = 1,
  SCHIP_MODERN = 2,
  XOCHIP = 3,
}
/**
*/
export class Chip8 {
  free(): void;
/**
* Create a new Chip8 instance
* @param {Variant} variant
*/
  constructor(variant: Variant);
/**
* Load the default font into memory at 0x0050
*/
  load_font(): void;
/**
* Load a ROM into memory at 0x0200 from a file
* @param {string} rom
*/
  load_rom_from_file(rom: string): void;
/**
* Load a ROM into memory at 0x0200 from a sequence of Uint8s
* @param {Uint8Array} bytes
*/
  load_rom_from_bytes(bytes: Uint8Array): void;
/**
* Get screen pixel data as a sequence of Uint8s
* @returns {Uint8Array}
*/
  get_display(): Uint8Array;
/**
* Execute the next instruction at the program counter
*/
  run(): void;
/**
* @param {number} key_index
* @param {boolean} value
*/
  set_keypad_state(key_index: number, value: boolean): void;
/**
*/
  decrement_timers(): void;
/**
* @returns {number}
*/
  get_sound_timer(): number;
/**
* @returns {boolean}
*/
  displayed_this_frame(): boolean;
/**
* @returns {boolean}
*/
  hires_mode(): boolean;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_chip8_free: (a: number) => void;
  readonly chip8_new: (a: number) => number;
  readonly chip8_load_font: (a: number) => void;
  readonly chip8_load_rom_from_file: (a: number, b: number, c: number) => void;
  readonly chip8_load_rom_from_bytes: (a: number, b: number, c: number) => void;
  readonly chip8_get_display: (a: number, b: number) => void;
  readonly chip8_run: (a: number) => void;
  readonly chip8_set_keypad_state: (a: number, b: number, c: number) => void;
  readonly chip8_decrement_timers: (a: number) => void;
  readonly chip8_get_sound_timer: (a: number) => number;
  readonly chip8_displayed_this_frame: (a: number) => number;
  readonly chip8_hires_mode: (a: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;

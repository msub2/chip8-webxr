# SILK-8

SILK-8 is a CHIP-8 interpreter written in Rust that also can run in the browser via WebAssembly.
This repo includes a three.js visualization that is usable on desktop and in WebXR.

## Support

Currently this interpreter can emulate programs for CHIP-8 and SCHIP (legacy and modern), with support for XOCHIP planned.

## Caveats

- Does not support half-pixel scrolling in legacy superchip.
- The web export has some divergent behavior that I haven't yet been able to trace the source of. This results in some ROMs behaving incorrectly.

## Development

For Rust, simply build and run with cargo. For the web export, ensure you have followed the setup instructions for [wasm-pack](https://rustwasm.github.io/docs/wasm-pack/introduction.html), then build with `wasm-pack build --target web`. This will place the WASM files in the `pkg` folder. From there, either copy the new files over to the demo folder or adjust the initialization in the demo page to point to the `pkg` folder instead.

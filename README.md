# `dash-minifb`
Bindings to [minifb](https://github.com/emoon/rust_minifb), a cross-platform window and framebuffer Rust crate for [dash](https://github.com/y21/dash).

### Example usage
```js
// Load dylib
import dll from '@std/dlloader';
const { Window } = dll.load('../target/release/libdash_minifb.so');

// Create window
const window = new Window();
window.limitUpdateRateMs(30);

// Define some constants
const WIDTH = 50;
const HEIGHT = 50;
const SIZE = WIDTH * HEIGHT;

// Create a buffer for the framebuffer
const buf = new ArrayBuffer(4 * SIZE);
const view = new Uint32Array(buf);

while (window.isOpen()) {
    for (let i = 0; i < SIZE; i++) {
        // Draw the pixels
        view[i] = 0xFF0000;
    }
    window.updateWithBuffer(buf, WIDTH, HEIGHT);
}
```

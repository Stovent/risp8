# risp8

Experimental Chip8 interpreter, cached interpreter and JIT compiler written in Rust.

The JIT is only available on x86_64.

## Controls

The Chip8 controls are mapped on the numpad.

Numpad layout:

|   | / | * | - |
|---|---|---|---|
| 7 | 8 | 9 | + |
| 4 | 5 | 6 | + |
| 1 | 2 | 3 | enter |
|   | 0 | . | enter |

Chip8 layout:

|   | A | B | C |
|---|---|---|---|
| 1 | 2 | 3 | D |
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
|   | 0 | F | E |

## Keyboard shortcuts

| key | action |
| --- | --- |
|  B  | Cached interpreter 3 |
|  C  | Cached interpreter |
|  I  | Interpreter |
|  J  | JIT |
|  P  | Play/Pause toggle |
|  S  | Single Step |
|  V  | Cached interpreter 2 |

## License

risp8 is distributed under the terms of the MIT license. Refer to the LICENSE file for more information.

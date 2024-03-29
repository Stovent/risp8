# risp8

Experimental Chip8 interpreter, cached interpreter and JIT compiler written in Rust.
There is a GUI frontend and a TUI frontend.

The JIT is only available on x86_64.

## Controls

The Chip8 controls are mapped on the keyboard and on the numpad.
TUI only has the keyboard mapping.

For each table below, the key in each layout square is the corresponding Chip8 key.

### Keyboard mapping

Keyboard key:

| | | | |
|---|---|---|---|
| 3 | 4 | 5 | 6 |
| E | R | T | Y |
| D | F | G | H |
| C | V | B | N |

Chip8 key:

| | | | |
|---|---|---|---|
| 1 | 2 | 3 | C |
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

### Numpad mapping

Numpad key:

| | | | |
|---|---|---|---|
|   | / | * | - |
| 7 | 8 | 9 | + |
| 4 | 5 | 6 | + |
| 1 | 2 | 3 | enter |
| 0 | 0 | . | enter |

Chip8 key:

| | | | |
|---|---|---|---|
|   | A | B | C |
| 1 | 2 | 3 | D |
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| 0 | 0 | F | E |

## GUI Control hotkeys

| key | action |
|:---:| --- |
|  P  | Play/Pause toggle |
|  S  | Single Step |
|  I  | Interpreter |
|  K  | Cached interpreter |
|  L  | Cached interpreter 2 |
|  M  | Cached interpreter 3 |
|  J  | JIT |

## TUI Control hotkeys

| key | action |
|:---:| --- |
|  Q  | Close the emulator |
|  P  | Play/Pause toggle |
|  S  | Single Step |
|  I  | Interpreter |
|  K  | Cached interpreter |
|  L  | Cached interpreter 2 |
|  M  | Cached interpreter 3 |
|  J  | JIT |
|  A  | Change the display charactere |

## License

risp8 is distributed under the terms of the MIT license. Refer to the LICENSE file for more information.

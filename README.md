# Chip'd
A Chip-8 emulator written in Rust using SDL2. 
<img width="515" height="290" alt="image" src="https://github.com/user-attachments/assets/08bf3ce9-11f4-473d-a1ca-93872a906969" />

---

## Usage
`chipd-rust [rom location]`

### Controls:

Ctrl + R: Reset

Ctrl + P: Pause

Keyboard mapping:
```

┌───┬───┬───┬───┐  ┌───┬───┬───┬───┐
│ 1 │ 2 │ 3 │ C │  │ 1 │ 2 │ 3 │ 4 │
├───┼───┼───┼───┤  ├───┼───┼───┼───┤
│ 4 │ 5 │ 6 │ D │  │ Q │ W │ E │ R │
├───┼───┼───┼───┤  ├───┼───┼───┼───┤
│ 7 │ 8 │ 9 │ E │  │ A │ S │ D │ F │
├───┼───┼───┼───┤  ├───┼───┼───┼───┤
│ A │ 0 │ B │ F │  │ Z │ X │ C │ V │
└───┴───┴───┴───┘  └───┴───┴───┴───┘

```

---

## Build steps on MacOS
1. Install SDL with `brew install sdl`
2. Add sdl2 to the PATH (temporary):
   ```bash
   export LIBRARY_PATH="/opt/homebrew/opt/sdl2-compat/lib:$LIBRARY_PATH"
   export CPATH="/opt/homebrew/opt/sdl2-compat/include:$CPATH"
   export PKG_CONFIG_PATH="/opt/homebrew/opt/sdl2-compat/lib/pkgconfig"
   ```
3. Run `cargo run`. You will get the usage printout
4. Run `cd target/debug`
5. Run `chipd-rust [rom location]` to load a rom and start playing

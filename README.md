
# 💡 shdoku

A basic tui sudoku game for your shell.

---

### Controls

The current control scheme adheres to vim-like keybindings:

- `h, j, k, l` to move `left, down, up, right`
- `H, J, K, L` to move 3 spaces at once
- `x` to delete a number
- `1-9` to place a number
- `q` to quit

This shall be reworked. 😼


### Preview

This is what the sudoku will be displayed like.

```
┌────────┬────────┬────────┐
│ 5 3    │   7    │        │
│ 6      │ 1 9 5  │        │
│   9 8  │        │   6    │
├────────┼────────┼────────┤
│ 8      │   6    │     3  │
│ 4      │ 8   3  │     1  │
│ 7      │   2    │     6  │
├────────┼────────┼────────┤
│   6    │        │ 2 8    │
│        │ 4 1 9  │     5  │
│        │   8    │   7 9  │
└────────┴────────┴────────┘
```


### Todo

  - [ ] Game logic
    - [x] Validate Sudokus
    - [x] Generate Sudokus
      - [x] Difficulties to choose
    - [ ] Timer
      - [ ] Scoreboard per difficulty
    - [ ] Undo functionality
  
  
  - [x] Basic UI
    - [x] Basic controls
    - [x] Basic rendering
    - [x] Centered UI
    - [x] Reset terminal state
  
  
  - [ ] Final UI
      - [ ] Final controls
        - [ ] Preselect numbers
        - [ ] Cell markups (perhaps with unicode block thingies?)
      - [ ] Colored UI
        - [ ] Hightlight selected numbers
        - [ ] Hightlight selected markups
        - [ ] Color chooser
      - [ ] Live timer
      - [ ] Scoreboard access
      - [ ] Difficulty selection


# 💡 shdoku

A basic tui sudoku game for your shell.

---

### Controls

The current control scheme adheres to vim-like keybindings and is modal.

- `h, j, k, l` to move `left, down, up, right`
- `H, J, K, L` to move 3 spaces at once

- `1-9` to preselect a number

- Modes:
  - `a` to enter Markup mode
  - `i` to enter Edit mode
  - `g` to enter Go mode
    - `1-9` to move to block
    - you then return to the previous mode
  - `A` and `I` to enter Edit/Markup mode "once"
    - do a single edit/mark
    - you then return to the previous mode
  - `<esc>` to return to Edit mode

- `<space>` to place/unplace preselected number/mark
- `x` to delete a number/mark
- `q` to quit


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
      - [x] Final controls
        - [x] Preselect numbers
        - [x] Edit Mode to (re)place numbers
        - [x] Markup Mode to mark where numbers could go
        - [x] Go Mode to move to blocks 1-9
        - [x] Toggle Number/Mark with Space
      - [ ] Colored UI
        - [ ] Hightlight selected numbers
        - [ ] Hightlight selected markups
        - [ ] Color chooser
      - [ ] Live timer
      - [ ] Scoreboard access
      - [ ] Difficulty selection


The Final UI design should include a sidebar,
that will look something like the following:


```
┌────────┬────────┬────────┐    ┌───────┐
│        │        │ 8 9    │    │ Hard  │ <- Difficulty
│ 4 9 7  │        │     6  │    │ 36/81 │ <- Completion in number of cells
│     2  │ 3   1  │   7    │    ├───────┤
├────────┼────────┼────────┤    │ 01:22 │ <- Elapsed Time
│     6  │ 9 7    │     3  │    ├───────┤
│   3    │ 5   2  │        │    │> Edit │ <- Active Mode
│ 7 2    │ 1   3  │ 5   4  │    │  Mark │
├────────┼────────┼────────┤    │  Go   │
│ 2   1  │   3 7  │ 9 5    │    ├───────┤
│ 5      │     9  │ 3 4    │    │  [9]  │ <- Preselected Number
│        │ 4      │ 6   1  │    │ 4 / 9 │ <- Completion of preselected Number
└────────┴────────┴────────┘    └───────┘
```

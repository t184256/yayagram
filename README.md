# yayagram - Play nonograms/picross in your terminal

**yayagram** is a puzzle game in which you fill out a grid with cells based on logic and number clues.

The game goes by many names: nonogram, picross, paint by numbers, griddlers, pic-a-pix, hanjie and others.

![Showcase](showcase.png)

*A randomized grid where white signifies a filled cell and red signifies a crossed out cell.*

See [this example](#Example) to find out how to play.

Playing it is as easy as:

```console
cargo install yayagram
yayagram
```

Binaries are also provided in the [Releases](https://github.com/r00ster91/yayagram/releases).

# Features

* Runs in the terminal!
* Cross-platform: runs on Linux, Windows and macOS.
* Helpful features and tools like undo, redo and clear.
* [Create your own grids](#Editor) for others to play.
* A new kind of cell: [maybed](#Maybed).
* Very well suited for [big grids](#Big-grids).
* Intuitive to play.

## Controls

The game is primarily played with both mouse and keyboard but can also be played exclusively with the keyboard.

- Mouse movement or arrow keys: select a cell.
- Left-click or `Q`: place a cell.
- Middle-click or `W`: [maybe a cell](#Maybed).
- Right-click or `E`: cross out a cell.
- `C`: clear the grid.
- `A`: undo cell placements or a grid clear.
- `D`: redo cell placements or a grid clear.
- `F`: flood-fill multiple cells.
- `X`: set [measurement point](#Measurement-tool).
- `Tab`: toggle the [editor](#Editor).
- `S`: save the [edited](#Editor) grid as a file locally.
- `Esc`: exit.

## Editor

Press `Tab` to toggle the editor and start placing the cells for your grid.
You can make use of all cell kinds.
To export your grid, press `Enter` to save while in editor mode. This will create a new `.yaya` grid file.
Note that in the same session it will always write the grid to the same file again unless renamed.

On Windows you can drag `.yaya` grid files onto the `.exe` to play the grid.
On Linux and macOS the `.yaya` file is passed via the command line.

## img2yaya

As an alternative to the editor you can generate `.yaya` grids using [@AaronErhardt](https://github.com/AaronErhardt)'s amazing [**img2yaya**](https://github.com/AaronErhardt/img2yaya) to convert images to playable `.yaya` files!

## Command line arguments

The program takes a single number for a squared grid size, two numbers for a width and height or the filename of a `.yaya` grid file.

```shell
yayagram # a random 5x5 grid
yayagram 10 # a random 10x10 grid
yayagram 5 15 # a random 5x15 grid
yayagram example.yaya # a custom grid
```

`--help`, `-h` and `--version`, `-V` are also supported.

## Measurement tool

Particularly on bigger grids it can sometimes become hard to count all the cells.
For this you can use the measurement tool. Simply press `X` to set your first point and then `X` again to set your second point.
You will then be able to see the distance between those two points with the measured cells that appear.
Measured cells never overwrite cell kinds other than empty cells and its own.

If you save a grid that contains measured cells, their distance indices won't be saved
and the measured cells will only appear as green when that grid is loaded.

## Maybed

The blue "maybed" cell kind can be placed on the grid with middle-click and is supposed to make "what if?" reasoning and trying out things easier,
as an alternative to using crossed out or filled cells which might be confusing.
It can help you imagine theoretical situations better.

## Big grids

yayagram is very well suited for big grids, up to size 99x99. Here are the reasons:

* The [measurement tool](#Measurement-tool) makes counting many cells far less error-prone and a lot easier.
* There is a fill tool that easily lets you flood-fill multiple cells at once.
* Cells surrounding the pointer are highlighted so that you don't lose track of the cell row you are focusing on.
* The grid is shown in smaller form on the top left, making it easier to see the whole picture.

## Other Tips

- Try to avoid guesssing and play it safe! Guessing can come back later to bite you. Guessing is `unsafe`.
- Don't forget to cross out cells that you are sure won't be filled.
  This helps immensely at ruling out possibilities.
- Be careful about accidentally pasting in your clipboard data. Some terminals paste with the press of a mouse button.
  If the data contains a `'c'` for instance, the grid will be cleared. This clear can be undone using the `A` key of course, but it may be confusing.

## Example

Here's a simple example to help you grasp the game.

|       | 2 | 3 | 2 |
|-------|---|---|---|
| **2** |   |   |   |
| **3** |   |   |   |
| **2** |   |   |   |

This is our grid where we need to fill out these 9 empty fields with the help of the 6 clue numbers on the top and on the left.
First, let's take a look at the 3 clues on the very top: the first one is **2**.
It tells us that exactly 2 cells in a row of the 3 cells below it are filled.
Through logic we can determine that this means that at least the middle cell is definitely filled:

|       | 2 | 3 | 2 |
|-------|---|---|---|
| **2** |   |   |   |
| **3** | ◯ |   |   |
| **2** |   |   |   |

Let's look at what the clue next to it to the right tells us (**3**).
This clue means that there is exactly 3 cells in a row filled below.
Because we only have 3 cells here (because it's a 3x3 grid), we can simply fill out all of those 3 cells safely:

|       | 2 | 3 | 2 |
|-------|---|---|---|
| **2** |   | ◯ |   |
| **3** | ◯ | ◯ |   |
| **2** |   | ◯ |   |

Next clue on the top: **2**. Again, through logic we can determine that at least the cell in the middle is definitely filled:

|       | 2 | 3 | 2 |
|-------|---|---|---|
| **2** |   | ◯ |   |
| **3** | ◯ | ◯ | ◯ |
| **2** |   | ◯ |   |

Now let's continue with the clues on the left, starting with the top one: **2**.
Here it's the same, there is definitely at least the middle cell filled,
which is already the case so we can simply move on with the clue below it: **3**.
The row is already filled out so let's move on to the next one: **2**.
Again, nothing to do.
It turns out that we have a pretty rare case here where **two solutions are correct**:

|       | 2 | 3 | 2 |
|-------|---|---|---|
| **2** | ◯ | ◯ |   |
| **3** | ◯ | ◯ | ◯ |
| **2** |   | ◯ | ◯ |

or

|       | 2 | 3 | 2 |
|-------|---|---|---|
| **2** |   | ◯ | ◯ |
| **3** | ◯ | ◯ | ◯ |
| **2** | ◯ | ◯ |   |

Both solutions fulfill the requirements.

I hope this helped you grasp the game a little bit.
Now you can try to apply some of what you learned on a small grid like 3x3 or 5x5 by simply passing the grid size as a command line argument to the program:
`yayagram 3` or `yayagram 5` respectively.

You can also load the above example grid into the game by passing this file as a command line argument to the binary: [example.yaya](example.yaya).

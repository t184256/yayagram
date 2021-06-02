use crate::undo_redo_buffer::UndoRedoBuffer;
use itertools::Itertools;
use std::{borrow::Cow, cell};
use terminal::{
    util::{Color, Point, Size},
    Terminal,
};
pub mod builder;
mod colors;
#[cfg(debug_assertions)]
pub mod debug;
mod random;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    /// An umarked cell.
    Empty,
    /// Used to mark filled cells.
    Filled,
    /// Used to mark cells that may be filled. Useful for doing "what if" reasoning.
    ///
    /// NOTE: in VS Code's terminal there's some weird bug that can only be reproduced sometimes
    ///       where you set this cell with the middle mouse wheel and it somehow clears the whole grid.
    ///       I believe the terminal takes a middle click as both a middle click and an R key press, causing the grid to be reset.
    ///       Maybe report this?
    Maybed,
    /// Used to mark cells that are certainly empty.
    Crossed,
    /// Used for indicating cells that were measured using the measurement tool.
    ///
    /// When this cell is saved, the index is not preserved.
    Measured(Option<usize>),
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

impl From<bool> for Cell {
    fn from(filled: bool) -> Self {
        filled.then(|| Cell::Filled).unwrap_or_default()
    }
}

impl Cell {
    pub fn draw(&self, terminal: &mut Terminal, point: Point, dark: bool) {
        const SEPARATING_POINT: u16 = 5;

        let (cell_color, content): (u8, Cow<'static, str>) = match self {
            Cell::Empty => {
                let x_reached_point = point.x / SEPARATING_POINT % 2 == 0;
                let y_reached_point = point.y / SEPARATING_POINT % 2 == 0;
                let cell_color = if x_reached_point ^ y_reached_point {
                    237
                } else {
                    239
                };

                (cell_color, "  ".into())
            }
            Cell::Filled => (255, "  ".into()),
            Cell::Crossed => (124, "  ".into()),
            Cell::Maybed => (39, "  ".into()),
            Cell::Measured(index) => {
                let cell_color = 46;

                let content = if let Some(index) = index {
                    terminal.set_foreground_color(Color::Black);
                    format!("{:>2}", index).into()
                } else {
                    "  ".into()
                };

                (cell_color, content)
            }
        };

        let cell_color = if dark {
            Color::Byte(cell_color - 2)
        } else {
            Color::Byte(cell_color)
        };

        terminal.set_background_color(cell_color);
        terminal.write(&content);
        terminal.reset_colors();
    }

    fn get_color(&self) -> Color {
        match self {
            Cell::Empty => unreachable!(), // TODO
            Cell::Filled => Color::Byte(255),
            Cell::Maybed => Color::Byte(39),
            Cell::Crossed => Color::Byte(124),
            Cell::Measured(_) => Color::Byte(46),
        }
    }

    pub fn get_dark_color(&self) -> Color {
        match self {
            Cell::Empty => Color::Byte(236),
            Cell::Filled => Color::Byte(253),
            Cell::Maybed => Color::Byte(38),
            Cell::Crossed => Color::Byte(88),
            Cell::Measured(_) => Color::Byte(40),
        }
    }

    pub fn get_darkest_color(&self) -> Color {
        match self {
            Cell::Empty => Color::Byte(235),
            Cell::Filled => Color::Byte(251),
            Cell::Maybed => Color::Byte(37),
            Cell::Crossed => Color::Byte(52),
            Cell::Measured(_) => Color::Byte(34),
        }
    }
}

/// A single clue specifying how many cells there are in a row at some point.
type Clue = u16;
/// A complete set of clues.
type Clues = Vec<Clue>;

#[derive(Debug)]
pub struct SolvedClues {
    pub horizontal_clues: Vec<Cell>,
    pub vertical_clues: Vec<Cell>,
}

pub struct Grid {
    pub size: Size,
    /// This is where the player's input is stored. It is initially empty.
    pub cells: Vec<Cell>,
    /// The horizontal clue solutions generated out of the initial input.
    pub horizontal_clues_solutions: Vec<Clues>,
    /// The vertical clue solutions generated out of the initial input.
    pub vertical_clues_solutions: Vec<Clues>,
    pub max_clues_size: Size,
    pub undo_redo_buffer: UndoRedoBuffer,
}

fn get_index(width: u16, point: Point) -> usize {
    point.y as usize * width as usize + point.x as usize
}

fn get_horizontal_clues(cells: &[Cell], width: u16, y: u16) -> impl Iterator<Item = Clue> + '_ {
    (0..width)
        .map(move |x| cells[get_index(width, Point { x, y })] == Cell::Filled)
        .dedup_with_count()
        .filter(|(_, filled)| *filled)
        .map(|(count, _)| count as Clue)
}

fn get_vertical_clues(
    cells: &[Cell],
    width: u16,
    height: u16,
    x: u16,
) -> impl Iterator<Item = Clue> + '_ {
    (0..height)
        .map(move |y| cells[get_index(width, Point { x, y })] == Cell::Filled)
        .dedup_with_count()
        .filter(|(_, filled)| *filled)
        .map(|(count, _)| count as Clue)
}

impl Grid {
    /// Creates a new grid. `cells` must have a length of `size.width * size.height`.
    pub fn new(size: Size, mut cells: Vec<Cell>) -> Self {
        assert_eq!(cells.len(), (size.width as usize * size.height as usize));

        let mut horizontal_clues_solutions = Vec::<Clues>::new();
        for y in 0..size.height {
            let horizontal_clues_solution: Clues =
                get_horizontal_clues(&cells, size.width, y).collect();
            horizontal_clues_solutions.push(horizontal_clues_solution);
        }
        let max_clues_width = horizontal_clues_solutions
            .iter()
            .map(|horizontal_clues_solution| horizontal_clues_solution.len() * 2)
            .max()
            .unwrap() as u16;

        let mut vertical_clues_solutions = Vec::<Clues>::new();
        for x in 0..size.width {
            let vertical_clues_solution: Clues =
                get_vertical_clues(&cells, size.width, size.height, x).collect();
            vertical_clues_solutions.push(vertical_clues_solution);
        }
        let max_clues_height = vertical_clues_solutions
            .iter()
            .map(|vertical_clues_solution| vertical_clues_solution.len())
            .max()
            .unwrap() as u16;

        for cell in &mut cells {
            if *cell == Cell::Filled {
                *cell = Cell::Empty;
            }
        }

        let max_clues_size = Size::new(max_clues_width, max_clues_height);

        let undo_redo_buffer = UndoRedoBuffer::default();

        Self {
            size,
            cells,
            horizontal_clues_solutions,
            vertical_clues_solutions,
            max_clues_size,
            undo_redo_buffer,
        }
    }

    fn cell_panic(point: Point, index: usize) -> ! {
        panic!(
            "cell access at {} with index {} is out of bounds",
            point, index
        );
    }

    pub fn get_cell(&self, point: Point) -> Cell {
        let index = get_index(self.size.width, point);
        *self
            .cells
            .get(index)
            .unwrap_or_else(|| Self::cell_panic(point, index))
    }

    pub fn get_mut_cell(&mut self, point: Point) -> &mut Cell {
        let index = get_index(self.size.width, point);
        self.cells
            .get_mut(index)
            .unwrap_or_else(|| Self::cell_panic(point, index))
    }

    pub fn get_horizontal_clues(&self, y: u16) -> impl Iterator<Item = Clue> + '_ {
        get_horizontal_clues(&self.cells, self.size.width, y)
    }

    pub fn get_vertical_clues(&self, x: u16) -> impl Iterator<Item = Clue> + '_ {
        get_vertical_clues(&self.cells, self.size.width, self.size.height, x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Grid {
        /// Creates a new grid from the given line of strings.
        /// A `1` represents a filled cell, a ` ` represents an empty cell.
        ///
        /// # Examples
        ///
        /// ```
        /// let lines = [
        ///    "111 1",
        ///    "11111",
        ///    "1 111"
        /// ]
        /// let grid = Grid::from_lines(lines);
        /// ```
        fn from_lines(lines: &[&str]) -> Grid {
            let width = lines.iter().map(|line| line.len()).max().unwrap();
            let height = lines.len();
            let size = Size::new(width as u16, height as u16);
            let mut cells = Vec::<Cell>::with_capacity(size.product() as usize);
            for line in lines {
                for char in line.chars() {
                    cells.push(match char {
                        '1' => Cell::Filled,
                        ' ' => Cell::Empty,
                        _ => panic!("the strings must only contain '1' or ' '"),
                    });
                }
            }
            Grid::new(size, cells)
        }
    }

    #[test]
    fn test_squared_grid() {
        let grid = Grid::from_lines(&[
            "1 1 111 1 ",
            " 1 11 111 ",
            "1111 11  1",
            "1 11 1  11",
            "1  111  11",
        ]);

        assert_eq!(
            grid.horizontal_clues_solutions,
            [
                vec![1, 1, 3, 1],
                vec![1, 2, 3],
                vec![4, 2, 1],
                vec![1, 2, 1, 2],
                vec![1, 3, 2],
            ]
        );

        assert_eq!(
            grid.vertical_clues_solutions,
            [
                vec![1, 3],
                vec![2],
                vec![1, 2],
                vec![4],
                vec![2, 1],
                vec![1, 3],
                vec![3],
                vec![1],
                vec![2, 2],
                vec![3]
            ]
        );
    }

    #[test]
    fn test_non_squared_grid() {
        #[rustfmt::skip]
            let grid = Grid::from_lines(&[
                " 111",
                " 1 1",
                "11 1",
                "1 1 ",
                "1  1",
                "  1 ",
            ]);

        assert_eq!(
            grid.horizontal_clues_solutions,
            [
                vec![3],
                vec![1, 1],
                vec![2, 1],
                vec![1, 1],
                vec![1, 1],
                vec![1],
            ]
        );

        assert_eq!(
            grid.vertical_clues_solutions,
            [vec![3], vec![3], vec![1, 1, 1], vec![3, 1]]
        );
    }
}

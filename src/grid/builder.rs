use super::{Cell, Grid};
use std::borrow::Cow;
use terminal::{
    util::{Color, Point},
    Terminal,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Cursor {
    pub point: Point,
}

impl Cursor {
    pub fn centered(terminal: &Terminal, grid: &Grid) -> Self {
        let grid_width = grid.size.width; // No division because blocks are 2 characters
        let grid_height = grid.size.height / 2;

        let max_clues_width = grid.max_clues_size.width / 2;
        let max_clues_height = grid.max_clues_size.height / 2;

        Self {
            point: Point {
                x: terminal.size.width / 2 - grid_width + max_clues_width,
                y: terminal.size.height / 2 - grid_height + max_clues_height,
            },
        }
    }

    pub fn update(&mut self, terminal: &mut Terminal) {
        terminal.set_cursor(self.point);
    }
}

/// Builds and draws the grid to the screen.
pub struct Builder {
    pub grid: Grid,
    pub cursor: Cursor,
}

impl Builder {
    pub fn new(terminal: &Terminal, grid: Grid) -> Self {
        let cursor = Cursor::centered(terminal, &grid);

        Self { grid, cursor }
    }

    /// Checks whether the point is within the grid.
    pub fn contains(&self, point: Point) -> bool {
        (self.cursor.point.y..self.cursor.point.y + self.grid.size.height).contains(&point.y)
            && (self.cursor.point.x..self.cursor.point.x + self.grid.size.width * 2)
                .contains(&point.x)
    }

    /// Draws the top clues while also returning whether all of them were solved ones.
    fn draw_top_clues(&mut self, terminal: &mut Terminal) -> usize {
        let mut highlighted = true;
        let mut all_solved = true;
        let mut solved_cell_count = 0;

        for (x, vertical_clues_solution) in self.grid.vertical_clues_solutions.iter().enumerate() {
            let vertical_clues = self.grid.get_vertical_clues(x as u16);
            let solved = vertical_clues
                .clone()
                .eq(vertical_clues_solution.iter().copied());

            if highlighted {
                terminal.set_background_color(Color::Byte(238));
            }
            if solved {
                for y in 0..self.grid.size.height {
                    // NOTE: the problem is that it also counts cells that already have been counted
                    if let Cell::Filled = self.grid.get_cell(Point { x: x as u16, y }) {
                        solved_cell_count += 1;
                    }
                }
                terminal.set_foreground_color(Color::DarkGray);
            } else if !vertical_clues_solution.is_empty() {
                all_solved = false;
            }

            let previous_cursor_y = self.cursor.point.y;
            for clue in vertical_clues_solution.iter().rev() {
                self.cursor.point.y -= 1;
                self.cursor.update(terminal);
                terminal.write(&format!("{:<2}", clue));
            }
            terminal.reset_colors();
            highlighted = !highlighted;
            self.cursor.point.y = previous_cursor_y;
            self.cursor.point.x += 2;
        }

        solved_cell_count
    }
    /// Clears the top clues, only graphically.
    fn clear_top_clues(&mut self, terminal: &mut Terminal) {
        let mut highlighted = true;
        for vertical_clues_solution in self.grid.vertical_clues_solutions.iter() {
            // NOTE (1): perhaps this temporary storing and loading (or "pushing and popping") for state preservation could be improved into a nicer API
            //           to prevent errors like accidentally forgetting to pop/put `previous_something` back into the variable
            let previous_cursor_y = self.cursor.point.y;

            for _ in vertical_clues_solution.iter().rev() {
                self.cursor.point.y -= 1;
                self.cursor.update(terminal);
                terminal.write("  ");
            }
            highlighted = !highlighted;

            // NOTE (2): this is the loading/"popping" part
            self.cursor.point.y = previous_cursor_y;

            self.cursor.point.x += 2;
        }
    }

    /// Draws the left clues while also returning whether all of them were solved ones.
    fn draw_left_clues(&mut self, terminal: &mut Terminal) -> usize {
        terminal.move_cursor_left(2);
        self.cursor.point.x -= 2;
        let mut highlighted = true;
        let mut all_solved = true;
        let mut solved_cell_count = 0;
        for (y, horizontal_clues_solution) in
            self.grid.horizontal_clues_solutions.iter().enumerate()
        {
            let horizontal_clues = self.grid.get_horizontal_clues(y as u16);
            let solved = horizontal_clues
                .clone()
                .eq(horizontal_clues_solution.iter().copied());

            if highlighted {
                terminal.set_background_color(Color::Byte(238));
            }
            if solved {
                for x in 0..self.grid.size.width {
                    // NOTE: the problem is that it also counts cells that already have been counted
                    if let Cell::Filled = self.grid.get_cell(Point { y: y as u16, x }) {
                        solved_cell_count += 1;
                    }
                }
                terminal.set_foreground_color(Color::DarkGray);
            } else if !horizontal_clues_solution.is_empty() {
                all_solved = false;
            }

            let previous_cursor_x = self.cursor.point.x;
            for clue in horizontal_clues_solution.iter().rev() {
                terminal.write(&format!("{:>2}", clue));
                terminal.move_cursor_left(4);
                self.cursor.point.x -= 4;
            }
            terminal.reset_colors();
            highlighted = !highlighted;
            self.cursor.point.x = previous_cursor_x;
            self.cursor.point.y += 1;
            self.cursor.update(terminal);
        }

        solved_cell_count
    }
    /// Clears the left clues, only graphically.
    fn clear_left_clues(&mut self, terminal: &mut Terminal) {
        terminal.move_cursor_left(2);
        self.cursor.point.x -= 2;
        let mut highlighted = true;
        for horizontal_clues_solution in self.grid.horizontal_clues_solutions.iter() {
            let previous_cursor_x = self.cursor.point.x;
            for _ in horizontal_clues_solution.iter().rev() {
                terminal.write("  ");
                terminal.move_cursor_left(4);
                self.cursor.point.x -= 4;
            }
            terminal.reset_colors();
            highlighted = !highlighted;
            self.cursor.point.x = previous_cursor_x;
            self.cursor.point.y += 1;
            self.cursor.update(terminal);
        }
    }

    /// Draws all clues, the top clues and the left clues while also returning whether all the drawn clues were solved ones.
    fn draw_clues(&mut self, terminal: &mut Terminal) -> usize {
        self.cursor.update(terminal);

        let mut solved_cell_count = 0;

        let previous_cursor_point = self.cursor.point;
        solved_cell_count += self.draw_top_clues(terminal);
        self.cursor.point = previous_cursor_point;

        self.cursor.update(terminal);

        let previous_cursor_point = self.cursor.point;
        solved_cell_count += self.draw_left_clues(terminal);
        self.cursor.point = previous_cursor_point;

        self.cursor.update(terminal);

        solved_cell_count
    }
    /// Clears all clues, only graphically.
    pub fn clear_clues(&mut self, terminal: &mut Terminal) {
        self.cursor.update(terminal);

        let previous_cursor_point = self.cursor.point;
        self.clear_top_clues(terminal);
        self.cursor.point = previous_cursor_point;

        self.cursor.update(terminal);

        let previous_cursor_point = self.cursor.point;
        self.clear_left_clues(terminal);
        self.cursor.point = previous_cursor_point;

        self.cursor.update(terminal);
    }

    fn draw_cells(&mut self, terminal: &mut Terminal) {
        /// Every 5 cells, the color changes to make the grid and its cells easier to look at and distinguish.
        const SEPARATING_POINT: u16 = 5;

        let previous_cursor_y = self.cursor.point.y;
        for (y, cells) in self
            .grid
            .cells
            .chunks(self.grid.size.width as usize)
            .enumerate()
        {
            let previous_cursor_x = self.cursor.point.x;
            for (x, cell) in cells.iter().enumerate() {
                let (cell_color, content): (Color, Cow<'static, str>) = match cell {
                    Cell::Empty => {
                        let x_reached_point = x / SEPARATING_POINT as usize % 2 == 0;
                        let y_reached_point = y / SEPARATING_POINT as usize % 2 == 0;
                        let cell_color = if x_reached_point ^ y_reached_point {
                            Color::Byte(236)
                        } else {
                            Color::Byte(238)
                        };

                        (cell_color, "  ".into())
                    }
                    Cell::Measured(index) => {
                        let cell_color = cell.get_color();

                        let content = if let Some(index) = index {
                            terminal.set_foreground_color(Color::Black);
                            format!("{:>2}", index).into()
                        } else {
                            "  ".into()
                        };

                        (cell_color, content)
                    }
                    _ => (cell.get_color(), "  ".into()),
                };

                terminal.set_background_color(cell_color);
                terminal.write(&content);
                terminal.reset_colors();

                self.cursor.point.x += 2;
            }
            self.cursor.point.x = previous_cursor_x;
            self.cursor.point.y += 1;
            self.cursor.update(terminal);
        }
        self.cursor.point.y = previous_cursor_y;
    }

    fn draw_percentage(&mut self, terminal: &mut Terminal, solved_cell_count: usize) {
        let previous_cursor_point = self.cursor.point;

        let point = Point {
            x: self.cursor.point.x + self.grid.size.width * 2 + 1,
            y: self.cursor.point.y + self.grid.size.height / 2,
        };

        self.cursor.point = point;
        self.cursor.update(terminal);

        terminal.write(&format!(
            "{}%  {} / {}",
            ((solved_cell_count as f64 / self.grid.cells_to_be_filled as f64) * 100.) as usize,
            solved_cell_count as f64,
            self.grid.cells_to_be_filled as f64,
        ));

        self.cursor.point = previous_cursor_point;
    }

    /// Draws the clues and the cells while also returning whether all the drawn clues were solved ones.
    #[must_use]
    pub fn draw(&mut self, terminal: &mut Terminal) -> bool {
        let solved_cell_count = self.draw_clues(terminal);

        self.draw_cells(terminal);

        self.draw_percentage(terminal, solved_cell_count);

        solved_cell_count == self.grid.cells_to_be_filled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terminal::util::Size;

    #[test]
    fn test_draw() {
        let mut terminal = Terminal::new().unwrap();
        let size = Size::new(5, 5);
        let grid = Grid::new(size.clone(), vec![Cell::Empty; size.product() as usize]);
        let mut builder = Builder::new(&terminal, grid);

        let previous_cursor = builder.cursor.clone();
        let all_clues_solved = builder.draw(&mut terminal);
        assert!(all_clues_solved);
        assert_eq!(builder.cursor, previous_cursor);
    }
}

use std::cmp;
use std::fmt;
use std::fs::File;
use std::io::Read;

use world::World;

/// Loads a world from the given filename. Results either in the
/// loaded world or a human readable error description.
pub fn load_from_file(filename: &str) -> Result<World, String> {
    macro_rules! err {
        ($expr:expr) => {
            match $expr {
                Err(e) => return Err(format!("{}: {}", filename, e)),
                Ok(v) => v,
            }
        }
    }
    let mut f = err!(File::open(filename));
    let mut s = String::with_capacity(err!(f.metadata()).len() as usize);
    err!(f.read_to_string(&mut s));
    let w = err!(cells_parse(&s));
    Ok(w)
}

// --------------------------------------------------------------------

#[derive(Debug)]
struct Error {
    row: usize,
    col: usize,
    reason: String,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}:{}: {}", self.row, self.col, self.reason)
    }
}

// --------------------------------------------------------------------

/// Scans and determines the dimension of the given world as a
/// `(width, height)` tuple.
fn cells_dimension(world: &str) -> (usize, usize) {
    world.lines()
         .skip_while(|line| line.chars().next() == Some('!'))
         .enumerate()
         .map(|(line_i, line)| {
             (// width; 1 + `index of the last 'O' char in this line`
              line.chars()
                  .enumerate()
                  .filter_map(|(i, c)| {
                      if c == 'O' {
                          Some(i + 1)
                      } else {
                          None
                      }
                  })
                  .max()
                  .unwrap_or(0),
              // heigth; 1 + `index of this line`
              line_i + 1)
         })
         .fold((0, 0),
               |(acc_w, acc_h), (w, h)| (cmp::max(acc_w, w), cmp::max(acc_h, h)))
}

#[test]
fn test_cells_dimension() {
    assert_eq!((0, 0), cells_dimension(""));
    assert_eq!((0, 0), cells_dimension("!only a comment"));
    assert_eq!((7, 5),
               cells_dimension(r#"!7x5 world
....O
..O
...O..O
.O
O"#));
}

fn cells_parse(world: &str) -> Result<World, Error> {
    let mut w = {
        let dim = cells_dimension(world);
        World::empty(dim.0 + 2, dim.1 + 2)
    };
    let mut lines = world.lines()
                         .skip_while(|line| line.chars().next() == Some('!'))
                         .enumerate();
    while let Some((line_i, line)) = lines.next() {
        for (col_i, c) in line.chars().enumerate() {
            match c {
                'O' => w.set_alive(col_i + 1, line_i + 1, true),
                '.' => {}
                c => return Err(Error {
                    row: line_i + 1, // XXX this is not fully correct due to the skipped header
                    col: col_i + 1,
                    reason: format!("Invalid character: {}", c),
                }),
            }
        }
    }
    Ok(w)
}

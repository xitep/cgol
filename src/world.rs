use std::fmt::{self, Write};

use bit_vec::BitVec;

pub struct World {
    width:      usize,
    height:     usize,

    generation: usize,  // current generation of cells
    cells:      BitVec, // cells addressable by: `x + y*width`
}

impl fmt::Debug for World {
    fn fmt(&self, fmt: &mut fmt::Formatter)
           -> Result<(), fmt::Error>
    {
        write!(fmt, "{}x{} (alive: {}, capacity: {})",
               self.width, self.height,
               self.cells.iter().filter(|&x| x).count(),
               self.cells.capacity())
    }
}

impl World {
    pub fn new(width: usize, height: usize, cells: BitVec)
           -> Result<World, &'static str>
    {
        if width*height != cells.len() {
            Err("width*height != cells")
        } else {
            Ok(World {
                width: width,
                height: height,
                generation: 0,
                cells: cells,
            })
        }
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn generation(&self) -> usize { self.generation }

    pub fn render_line<F: fmt::Write>(&self, line: usize, fmt: &mut F) {
        assert!(line < self.height);
        for i in 0..self.width {
            let _ = fmt.write_char(if self.is_alive(i, line) { '#' } else { '.' });
        }
    }

    /*
     Rules (https://en.wikipedia.org/wiki/Conway's_Game_of_Life):

     1. Any live cell with fewer than two live neighbours dies, as if
        caused by under-population.
     2. Any live cell with two or three live neighbours lives on to the
        next generation.
     3. Any live cell with more than three live neighbours dies, as if
        by over-population.
     4. Any dead cell with exactly three live neighbours becomes a live
        cell, as if by reproduction.
     */
    pub fn advance_generation(&mut self) {
        let mut changes = Vec::new();
        for h in 0..self.height {
            for w in 0..self.width {
                match self.count_living_neighbours(w, h) {
                    (true, n) if n < 2 || n > 3 => changes.push((w, h, false)),
                    (false, 3) => changes.push((w, h, true)),
                    _ => {}
                }
            }
        }
        for (w, h, change) in changes {
            trace!("setting {}x{} = {}", w, h, change);
            self.set_alive(w, h, change);
        }
        self.generation += 1;
    }

    fn set_alive(&mut self, w: usize, h: usize, alive: bool) {
        debug_assert!(w < self.width);
        debug_assert!(h < self.height);

        self.cells.set(h*self.width + w, alive);
    }

    fn is_alive(&self, w: usize, h: usize) -> bool {
        debug_assert!(w < self.width);
        debug_assert!(h < self.height);

        self.cells[h*self.width + w]
    }

    fn count_living_neighbours(&mut self, w: usize, h: usize) -> (bool, usize) {
        debug_assert!(w < self.width);
        debug_assert!(h < self.height);

        let mut living_neighbours = 0usize;
        for &(x, y) in &[(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
            let (w1, h1) = (wrapped(w, x, self.width), wrapped(h, y, self.height));
            let alive = self.is_alive(w1, h1);
            trace!("{}x{}: {}x{} => {}", w, h, w1, h1, alive);
            if alive {
                living_neighbours += 1;
            }
        }
        let x = (self.is_alive(w, h), living_neighbours);
        trace!("count_living_neighbours: {}x{} => {:?}", w, h, x);
        x
    }
}

fn wrapped(w: usize, offs: isize, wrap: usize) -> usize {
    if offs >= 0 {
        (w + offs as usize) % wrap
    } else {
        let offs = offs.abs() as usize;
        if w >= offs {
            w - offs
        } else {
            let mut u = (offs - w) % wrap;
            if u > 0 {
                u = wrap - u;
            }
            u
        }
    }
}

#[test]
fn test_wrapped() {
    assert_eq!(wrapped(1, 1, 5), 2);
    assert_eq!(wrapped(1, -1, 5), 0);
    assert_eq!(wrapped(0, -1, 5), 4);
    assert_eq!(wrapped(0, -2, 5), 3);
    assert_eq!(wrapped(0, -11, 5), 4);
    assert_eq!(wrapped(0, -10, 5), 0);
    assert_eq!(wrapped(1, -11, 5), 0);
    assert_eq!(wrapped(2, -11, 5), 1);
}

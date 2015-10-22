use std::fmt::{self, Write};

use rand::Rng;

pub struct World {
    width: usize,
    height: usize,

    generation: usize, // current generation of cells
    alive: usize, // current number of live cells
    cells: Vec<bool>,   // cells addressable by: `x + y*width`
}

impl fmt::Debug for World {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt,
               "{}x{} (alive: {})",
               self.width,
               self.height,
               self.cells.iter().filter(|&x| *x).count())
    }
}

impl World {
    pub fn empty(width: usize, height: usize) -> World {
        World::from_cells(width, height, vec![false; width * height])
    }

    pub fn random<R: Rng>(r: &mut R, width: usize, height: usize) -> World {
        let mut v = Vec::with_capacity(width * height);
        for _ in 0..(width * height) {
            v.push(r.gen::<f64>() < 0.3);
        }
        World::from_cells(width, height, v)
    }

    fn from_cells(width: usize, height: usize, cells: Vec<bool>) -> World {
        assert_eq!(width * height, cells.len());
        World {
            width: width,
            height: height,
            generation: 0,
            alive: cells.iter().filter(|&x| *x).count(),
            cells: cells,
        }
    }

    pub fn alive(&self) -> usize {
        self.alive
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn expand_to(&mut self, new_width: usize, new_height: usize) {
        if new_width == self.width && new_height == self.height {
            return;
        }
        let mut ncells = vec![false; new_width * new_height];
        for h in 0..self.height {
            for w in 0..self.width {
                if self.is_alive(w, h) {
                    let (nw, nh) = (w % new_width, h % new_height);
                    *ncells.get_mut(nh * new_width + nw).unwrap() = true;
                }
            }
        }
        self.width = new_width;
        self.height = new_height;
        self.cells = ncells;
        self.alive = self.cells.iter().filter(|&x| *x).count();
    }

    //
    // Rules (https://en.wikipedia.org/wiki/Conway's_Game_of_Life):
    //
    // 1. Any live cell with fewer than two live neighbours dies, as if
    // caused by under-population.
    // 2. Any live cell with two or three live neighbours lives on to the
    // next generation.
    // 3. Any live cell with more than three live neighbours dies, as if
    // by over-population.
    // 4. Any dead cell with exactly three live neighbours becomes a live
    // cell, as if by reproduction.
    //
    pub fn advance_generation<F: FnMut(usize, usize, bool)>(&mut self, mut cb: F) {
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
        for &(w, h, change) in changes.iter() {
            self.set_alive(w, h, change);
        }
        self.generation += 1;
        for &(w, h, change) in changes.iter() {
            cb(w, h, change);
        }
    }

    pub fn set_alive(&mut self, w: usize, h: usize, alive: bool) {
        debug_assert!(w < self.width);
        debug_assert!(h < self.height);

        *self.cells.get_mut(h * self.width + w).unwrap() = alive;
        if alive {
            self.alive += 1;
        } else {
            self.alive -= 1;
        }
    }

    pub fn is_alive(&self, w: usize, h: usize) -> bool {
        debug_assert!(w < self.width);
        debug_assert!(h < self.height);

        self.cells[h * self.width + w]
    }

    fn count_living_neighbours(&mut self, w: usize, h: usize) -> (bool, usize) {
        debug_assert!(w < self.width);
        debug_assert!(h < self.height);

        let mut living_neighbours = 0usize;
        for &(x, y) in &[(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
            let (w1, h1) = (wrapped(w, x, self.width), wrapped(h, y, self.height));
            if self.is_alive(w1, h1) {
                living_neighbours += 1;
            }
        }
        let x = (self.is_alive(w, h), living_neighbours);
        x
    }
}

fn wrapped(w: usize, offs: isize, wrap: usize) -> usize {
    let (w, wrap) = (w as isize, wrap as isize);
    let n = (w + offs) % wrap;
    if n < 0 {
        (wrap + n) as usize
    } else {
        n as usize
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

#[cfg(test)]
mod benches {
    use super::World;

    use rand::XorShiftRng;
    use test::{black_box, Bencher};

    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;

    #[bench]
    fn advance_generation_random_world(b: &mut Bencher) {
        let mut w = World::random(&mut XorShiftRng::new_unseeded(), WIDTH, HEIGHT);
        b.iter(|| w.advance_generation(|_, _, state| {
            black_box(state);
        }));
    }
}

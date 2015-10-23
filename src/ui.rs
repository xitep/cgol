use std::fmt::{self, Write};

use rustbox::{self, RustBox, InitOptions, Event, Color};
use rustbox::keyboard::Key;
use time::Duration;
use rand::thread_rng;
use world::World;

enum Error {
    RustboxInit(rustbox::InitError),
    RustboxEvent(rustbox::EventError),
}

impl From<rustbox::InitError> for Error {
    fn from(e: rustbox::InitError) -> Self {
        Error::RustboxInit(e)
    }
}
impl From<rustbox::EventError> for Error {
    fn from(e: rustbox::EventError) -> Self {
        Error::RustboxEvent(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::RustboxEvent(ref e) => write!(fmt, "rustbox::event: {}", e),
            Error::RustboxInit(ref e) => write!(fmt, "rustbox::init: {}", e),
        }
    }
}

struct UI {
    terminal: RustBox,
    width: usize,
    height: usize,

    line_buf: String,

    alive_char: char,
    dead_char: char,
}

impl UI {
    fn init(alive: char, dead: char) -> Result<UI, Error> {
        let t = try!(RustBox::init(InitOptions { buffer_stderr: true, ..Default::default() }));
        let (width, height) = (t.width(), t.height());
        Ok(UI {
            terminal: t,
            width: width,
            height: height,
            line_buf: String::with_capacity(width),
            alive_char: alive,
            dead_char: dead,
        })
    }

    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }

    fn expand_to_screen(&mut self, world: &mut World) {
        let (w, h) = (self.terminal.width(), self.terminal.height());
        if w == self.width && h == self.height {
            return;
        }
        world.expand_to(w, h);
        self.width = w;
        self.height = h;
    }

    fn get_drawing_char(&self, alive: bool) -> char {
        if alive {
            self.alive_char
        } else {
            self.dead_char
        }
    }

    fn render_line(&mut self, world: &World, h: usize) {
        self.line_buf.clear();
        for w in 0..world.width() {
            let c = self.get_drawing_char(world.is_alive(w, h));
            self.line_buf.push(c);
        }
    }

    fn print_world(&mut self, world: &World) {
        for h in 0..(world.height() - 1) {
            self.render_line(world, h);
            self.print_line(0, h, &self.line_buf);
        }
        self.update_status(world);
    }

    fn update_status(&mut self, world: &World) {
        let line_is_clean = if world.height() >= self.height() {
            let h = self.height() - 1;
            self.render_line(world, h);
            self.print_line(0, h, &self.line_buf);
            true
        } else {
            false
        };
        self.print_status(line_is_clean,
                          format_args!("Gen: {} / Alive: {}", world.generation(), world.alive()));
    }

    fn print_status(&mut self, clear: bool, args: fmt::Arguments) {
        self.line_buf.clear();
        let _ = self.line_buf.write_fmt(args);

        if !clear {
            for _ in 0..(self.line_buf.len() - self.width()) {
                self.line_buf.push(' ');
            }
        }
        self.print_line(0, self.height - 1, &self.line_buf);
    }

    fn print_line(&self, x: usize, y: usize, line: &str) {
        self.terminal.print(x,
                            y,
                            rustbox::RB_NORMAL,
                            Color::Default,
                            Color::Default,
                            line);
    }

    fn print_char(&self, x: usize, y: usize, c: char) {
        self.terminal.print_char(x, y, rustbox::RB_NORMAL, Color::Default, Color::Default, c);
    }

    fn clear(&self) {
        self.terminal.clear();
    }

    fn flush(&self) {
        self.terminal.present();
    }

    fn set_cursor(&self, w: usize, h: usize) {
        self.terminal.set_cursor(w as isize, h as isize);
    }

    fn redraw_scene(&mut self, world: &World, clear: bool) {
        if clear {
            self.clear();
        }
        self.print_world(world);
        self.set_cursor(self.width - 1, self.height - 1);
        self.flush();
    }
}

pub fn run(world: Option<World>, alive: char, dead: char) -> Result<(), String> {
    run_(world, alive, dead).map_err(|e| format!("error: {}", e))
}

fn run_(world: Option<World>, alive: char, dead: char) -> Result<(), Error> {
    let mut ui = try!(UI::init(alive, dead));
    // ~ if no world was explicitely specified, generated one
    let mut world = match world {
        Some(w) => w,
        None => World::random(&mut thread_rng(), ui.width(), ui.height()),
    };
    // ~ expand the give world to the size of the ui and draw the world
    {
        world.expand_to(ui.width(), ui.height());
        ui.redraw_scene(&world, false);
    }

    let mut maxdelay = Duration::milliseconds(100);
    let mut nextdelay = maxdelay;
    let mut animate = false;

    // ~ start the event loop
    loop {
        let e = try!(if animate {
            ui.terminal.peek_event(nextdelay, false)
        } else {
            ui.terminal.poll_event(false)
        });
        match e {
            Event::NoEvent => {
                // ~ advance generation
                advance_one_step(&mut ui, &mut world);
                nextdelay = maxdelay;
            }
            Event::KeyEvent(Some(key)) => {
                match key {
                    Key::Char('q') => {
                        // ~ quit
                        break;
                    }
                    Key::Char('r') => {
                        // ~ regenerate (random) world
                        animate = false;
                        nextdelay = Duration::nanoseconds(0);
                        world = World::random(&mut thread_rng(), ui.width(), ui.height());
                        ui.redraw_scene(&world, true);
                    }
                    Key::Char('s') => {
                        // ~ advance generation
                        advance_one_step(&mut ui, &mut world);
                    }
                    Key::Char('-') => {
                        maxdelay = maxdelay * 2;
                        nextdelay = maxdelay;
                    }
                    Key::Char('+') => {
                        if maxdelay.num_milliseconds() > 0 {
                            maxdelay = maxdelay / 2;
                            nextdelay = maxdelay;
                        }
                    }
                    Key::Ctrl('l') => {
                        // ~ redraw screen
                        ui.expand_to_screen(&mut world);
                        ui.redraw_scene(&world, true);
                    }
                    Key::Char(' ') => {
                        animate ^= true;
                        nextdelay = Duration::nanoseconds(0);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn advance_one_step(ui: &mut UI, world: &mut World) {
    world.advance_generation(|w, h, alive| {
        ui.print_char(w, h, ui.get_drawing_char(alive));
    });
    ui.update_status(&world);
    ui.flush();
}

use std::fmt::{self, Write};

use rustbox::{self, RustBox, InitOptions, Event, Color};
use rustbox::keyboard::{Key};
use time::{Duration};
use world::World;

pub fn run(world: World) -> Result<(), String> {
    run_(world).map_err(|e| format!("error: {}", e))
}

enum Error {
    RustboxInit(rustbox::InitError),
    RustboxEvent(rustbox::EventError),
}

impl From<rustbox::InitError> for Error {
    fn from(e: rustbox::InitError) -> Self { Error::RustboxInit(e) }
}
impl From<rustbox::EventError> for Error {
    fn from(e: rustbox::EventError) -> Self { Error::RustboxEvent(e) }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::RustboxEvent(ref e) =>
                write!(fmt, "rustbox::event: {}", e),
            Error::RustboxInit(ref e) =>
                write!(fmt, "rustbox::init: {}", e),
        }
    }
}

struct UI {
    terminal: RustBox,
    width:    usize,
    height:   usize,

    line_buf: String,
}

impl UI {
    fn init() -> Result<UI, Error> {
        let t = try!(RustBox::init(InitOptions {
            buffer_stderr: true,
            .. Default::default()
        }));
        let (width, height) = (t.width(), t.height());
        Ok(UI {
            terminal: t,
            width: width,
            height: height,
            line_buf: String::with_capacity(width),
        })
    }

    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    fn print_world(&mut self, world: &World) {
        for h in 0..world.height() {
            self.line_buf.clear();
            world.render_line(h, &mut self.line_buf);
            self.print_line(0, h, &self.line_buf);
        }
        self.print_status(
            format_args!("Gen: {} / Alive: {}",
                         world.generation(), world.alive()));
    }

    fn print_status(&mut self, args: fmt::Arguments) {
        self.line_buf.clear();
        let _ = self.line_buf.write_fmt(args);
        self.print_line(0, self.height - 1, &self.line_buf);
    }

    fn print_line(&self, x: usize, y: usize, line: &str) {
        self.terminal.print(
            x, y, rustbox::RB_NORMAL,
            Color::Default, Color::Default, line);
    }

    fn clear(&self) {
        self.terminal.clear();
    }

    fn flush(&self) {
        self.terminal.present();
    }

    fn redraw_scene(&mut self, world: &World, clear: bool) {
        if clear {
            self.clear();
        }
        self.print_world(world);
        self.flush();
    }
}

fn run_(mut world: World) -> Result<(), Error> {
    let mut ui = try!(UI::init());
    // ~ expand the give world to the size of the ui and draw the world
    {
        let (ew, eh) = (ui.width(), ui.height());
        debug!("resizing to {}x{}", ew, eh);
        world.expand_to(ew, eh);
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
                world.advance_generation();
                ui.redraw_scene(&world, false);
                nextdelay = maxdelay;
            }
            Event::KeyEvent(Some(key)) => {
                match key {
                    Key::Char('q') => {
                        // ~ quit
                        break;
                    }
                    Key::Char('+') => {
                        maxdelay = maxdelay * 2;
                        nextdelay = maxdelay;
                    }
                    Key::Char('-') => {
                        if maxdelay.num_milliseconds() > 0 {
                            maxdelay = maxdelay / 2;
                            nextdelay = maxdelay;
                        }
                    }
                    Key::Ctrl('l') => {
                        // ~ redraw screen
                        ui.redraw_scene(&world, true);
                    }
                    Key::Char(' ') => {
                        // ~ advance generation
                        animate ^= true;
                        nextdelay = Duration::nanoseconds(0);
                    }
                    _ => {},
                }
            }
            Event::ResizeEvent(w, h) => {
                let (w, h) = (w.abs() as usize, h.abs() as usize);
                world.expand_to(ui.width(), ui.height());
                ui.set_size(w, h);
                ui.redraw_scene(&world, true);
            }
            _ => {},
        }
    }
    Ok(())
}

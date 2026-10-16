#[derive(Debug, PartialEq)]
pub enum Request {
    Fullscreen(bool),
    Size(u32, u32),
}

#[derive(Default)]
pub struct WindowMode {
    pub fullscreen: bool,
    shortcut_held: bool,
}

impl WindowMode {
    pub fn shortcut(&mut self, alt: bool, enter: bool) -> bool {
        let held = alt && enter;
        let triggered = held && !self.shortcut_held;
        self.shortcut_held = held;
        triggered
    }

    pub fn change(&mut self, fullscreen: bool, size: Option<(u32, u32)>) -> Vec<Request> {
        // A resolution choice describes a movable window, never a shrunken borderless desktop.
        let fullscreen = fullscreen && size.is_none();
        let mut requests = vec![];
        if self.fullscreen != fullscreen {
            self.fullscreen = fullscreen;
            requests.push(Request::Fullscreen(fullscreen));
        }
        if let Some((width, height)) = size {
            requests.push(Request::Size(width, height));
        }
        requests
    }
}

pub fn apply(requests: Vec<Request>) {
    for request in requests {
        match request {
            Request::Fullscreen(enabled) => macroquad::window::set_fullscreen(enabled),
            Request::Size(width, height) => {
                macroquad::window::request_new_screen_size(width as f32, height as f32)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/window_mode.rs"]
mod tests;

/// Windows reports both focus loss and minimization through the activity events.
pub struct Activity {
    subscriber: usize,
    active: bool,
}

impl Activity {
    pub fn new() -> Self {
        Self {
            subscriber: macroquad::input::utils::register_input_subscriber(),
            active: true,
        }
    }

    pub fn poll(&mut self) -> bool {
        macroquad::input::utils::repeat_all_miniquad_input(self, self.subscriber);
        self.active
    }
}

impl macroquad::miniquad::EventHandler for Activity {
    fn update(&mut self) {}
    fn draw(&mut self) {}
    fn window_minimized_event(&mut self) {
        self.active = false;
    }
    fn window_restored_event(&mut self) {
        self.active = true;
    }
}

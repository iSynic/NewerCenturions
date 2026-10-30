/// A change to make to the window: fullscreen on or off, or a new size.
#[derive(Debug, PartialEq)]
pub enum Request {
    Fullscreen(bool),
    Size(u32, u32),
}

/// Whether the window is fullscreen, and whether Alt+Return is still held.
#[derive(Default)]
pub struct WindowMode {
    pub fullscreen: bool,
    shortcut_held: bool,
}

impl WindowMode {
    /// Whether Alt+Return was just pressed.
    pub fn shortcut(&mut self, alt: bool, enter: bool) -> bool {
        let held = alt && enter;
        let triggered = held && !self.shortcut_held;
        self.shortcut_held = held;
        triggered
    }

    /// The requests that bring the window to `fullscreen` and `size`; a size
    /// choice always leaves fullscreen.
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

/// Carries out the window requests.
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
    /// Starts listening to the window's events, active.
    pub fn new() -> Self {
        Self {
            subscriber: macroquad::input::utils::register_input_subscriber(),
            active: true,
        }
    }

    /// Whether the window is active, after taking in its latest events.
    pub fn poll(&mut self) -> bool {
        macroquad::input::utils::repeat_all_miniquad_input(self, self.subscriber);
        self.active
    }
}

impl macroquad::miniquad::EventHandler for Activity {
    /// Nothing to update each frame.
    fn update(&mut self) {}
    /// Nothing to draw; the game draws the screen.
    fn draw(&mut self) {}
    /// The window stops being active when it is minimized.
    fn window_minimized_event(&mut self) {
        self.active = false;
    }
    /// The window is active again when it is restored.
    fn window_restored_event(&mut self) {
        self.active = true;
    }
}

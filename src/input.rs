use glfw::Glfw;

use crate::{debugger::Debugger, renderer::Renderer};

pub struct GBInputState {
    pub up: bool,
    pub right: bool,
    pub left: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}

impl Default for GBInputState {
    fn default() -> Self {
        Self {
            up: Default::default(),
            right: Default::default(),
            left: Default::default(),
            down: Default::default(),
            a: Default::default(),
            b: Default::default(),
            start: Default::default(),
            select: Default::default(),
        }
    }
}

pub fn handle_input(
    glfw: &mut Glfw,
    renderer: &mut Renderer,
    events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    debugger: &mut Debugger,
    input_state: &mut GBInputState,
) {
    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&events) {
        match event {
            glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                renderer.window.set_should_close(true)
            }
            glfw::WindowEvent::Key(glfw::Key::P, _, glfw::Action::Press, _) => {
                debugger.pause();
            }
            glfw::WindowEvent::Key(glfw::Key::Up, _, glfw::Action::Press, _) => {
                input_state.up = true;
            }
            glfw::WindowEvent::Key(glfw::Key::Up, _, glfw::Action::Release, _) => {
                input_state.up = false;
            }
            glfw::WindowEvent::Key(glfw::Key::Right, _, glfw::Action::Press, _) => {
                input_state.right = true;
            }
            glfw::WindowEvent::Key(glfw::Key::Right, _, glfw::Action::Release, _) => {
                input_state.right = false;
            }
            glfw::WindowEvent::Key(glfw::Key::Left, _, glfw::Action::Press, _) => {
                input_state.left = true;
            }
            glfw::WindowEvent::Key(glfw::Key::Left, _, glfw::Action::Release, _) => {
                input_state.left = false;
            }
            glfw::WindowEvent::Key(glfw::Key::Down, _, glfw::Action::Press, _) => {
                input_state.down = true;
            }
            glfw::WindowEvent::Key(glfw::Key::Down, _, glfw::Action::Release, _) => {
                input_state.down = false;
            }
            glfw::WindowEvent::Key(glfw::Key::Z, _, glfw::Action::Press, _) => {
                input_state.b = true;
            }
            glfw::WindowEvent::Key(glfw::Key::Z, _, glfw::Action::Release, _) => {
                input_state.b = false;
            }
            glfw::WindowEvent::Key(glfw::Key::X, _, glfw::Action::Press, _) => {
                input_state.a = true;
            }
            glfw::WindowEvent::Key(glfw::Key::X, _, glfw::Action::Release, _) => {
                input_state.a = false;
            }
            _ => {}
        }
    }
}

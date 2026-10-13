//! Player.ifMesbox presentation shared by protected content notices.
use crate::startup::Names;
use gielinor_world::ui::Component;
use std::io;
#[derive(Clone, Copy)]
pub struct Presentation {
    pub parent: Component,
    pub interface: u16,
    pub text: Component,
    pub button: Component,
}
impl Presentation {
    pub fn resolve(names: &Names) -> io::Result<Self> {
        Ok(Self {
            parent: Component(names.id("component", "chatbox:chatmodal")?),
            interface: names.short("interface", "messagebox")?,
            text: Component(names.id("component", "messagebox:text")?),
            button: Component(names.id("component", "messagebox:continue")?),
        })
    }
}
pub(crate) struct Active {
    pub token: u64,
    pub remaining: std::collections::VecDeque<(String, usize)>,
}
pub fn line_height(lines: usize) -> i32 {
    match lines {
        2 => 31,
        3 => 24,
        4 => 17,
        _ => 0,
    }
}

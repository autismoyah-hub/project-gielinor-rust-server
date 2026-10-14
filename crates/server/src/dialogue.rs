use crate::startup::Names;
use gielinor_world::{
    dialogue::{Expression, Page},
    dialogue_text::Font,
    simulation::NpcId,
    ui::Component,
};
use std::{collections::VecDeque, io};
#[derive(Clone)]
pub struct Presentation {
    pub parent: Component,
    pub unclamp: u32,
    pub player: u16,
    pub npc: u16,
    pub choice: u16,
    pub animations: [[u16; 4]; 5],
    pub font: Font,
}
impl Presentation {
    pub fn resolve(names: &Names, font: Font) -> io::Result<Self> {
        let mut animations = [[0; 4]; 5];
        for (row, prefix) in ["chathap", "chatneu", "chatang", "chatcon", "chatquiz"]
            .into_iter()
            .enumerate()
        {
            for line in 0..4 {
                animations[row][line] = names.short("seq", &format!("{prefix}{}", line + 1))?;
            }
        }
        Ok(Self {
            parent: Component(names.id("component", "chatbox:chatmodal")?),
            unclamp: names.id("varbit", "chatmodal_unclamp")?,
            player: names.short("interface", "chat_right")?,
            npc: names.short("interface", "chat_left")?,
            choice: names.short("interface", "chatmenu")?,
            animations,
            font,
        })
    }
    pub fn animation(&self, expression: Expression, lines: usize) -> u16 {
        let row = match expression {
            Expression::Happy => 0,
            Expression::Neutral => 1,
            Expression::Angry => 2,
            Expression::Confused => 3,
            Expression::Quiz => 4,
        };
        self.animations[row][lines.saturating_sub(1).min(3)]
    }
}
#[derive(Clone)]
pub(super) struct Active {
    pub token: u64,
    pub npc: NpcId,
    pub page: Page,
    pub remaining: VecDeque<(String, usize)>,
    pub button: Component,
}

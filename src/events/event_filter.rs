use super::{Audience, Event, Level};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventFilter {
    audiences: Vec<Audience>,
    levels: Vec<Level>,
    include_markers: bool,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            audiences: vec![Audience::User],
            levels: Level::all().to_vec(),
            include_markers: false,
        }
    }
}

impl EventFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_audiences(mut self, audiences: Vec<Audience>) -> Self {
        self.audiences = audiences;
        self
    }

    pub fn with_levels(mut self, levels: Vec<Level>) -> Self {
        self.levels = levels;
        self
    }

    pub fn include_markers(mut self, include_markers: bool) -> Self {
        self.include_markers = include_markers;
        self
    }

    pub(crate) fn matches(&self, event: &Event) -> bool {
        match event {
            Event::Message(message) => {
                self.audiences.contains(&message.audience) && self.levels.contains(&message.level)
            }
            Event::Marker(_) => self.include_markers,
        }
    }
}

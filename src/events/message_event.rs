use std::path::PathBuf;

use super::{Audience, EventLocation, Level};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageEvent {
    pub message: String,
    pub location: Option<EventLocation>,
    pub level: Level,
    pub audience: Audience,
    pub origin: Option<String>,
}

impl MessageEvent {
    pub fn new(
        message: impl Into<String>,
        level: Level,
        audience: Audience,
        location: Option<EventLocation>,
        origin: Option<String>,
    ) -> Self {
        Self {
            message: message.into(),
            location,
            level,
            audience,
            origin,
        }
    }

    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    pub fn with_origin_option(mut self, origin: Option<&str>) -> Self {
        self.origin = origin.map(str::to_owned);
        self
    }

    pub fn with_file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.location = Some(match self.location.take() {
            Some(location) => location.with_file_path(path),
            None => EventLocation::file_path(path),
        });
        self
    }
}

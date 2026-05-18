use super::check_in;
use crate::events::{Audience, Event, EventLog, Level};
use crate::mlg::collection::{find_collection_root, resolve_source_files};
use crate::mlg::config::default_config_contents;
use std::fs;
use std::path::{Path, PathBuf};

mod collection;
mod events;
mod paths;
mod semantic;
mod support;

use support::*;

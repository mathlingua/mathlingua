use crate::events::{EventLog, EventLogListener};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

const ORIGIN: &str = "mlg_whte_rbt_obj";
const LINE_DELAY: Duration = Duration::from_millis(700);
const LONG_DELAY: Duration = Duration::from_millis(1000);
const TYPE_DELAY: Duration = Duration::from_millis(100);
const MAGIC_WORD_DELAY: Duration = Duration::from_millis(50);

pub struct WhteRbtObjResult {
    pub event_log: EventLog,
    pub successful: bool,
}

pub fn whte_rbt_obj(listener: Option<Box<dyn EventLogListener>>) -> WhteRbtObjResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let successful = match run_whte_rbt_obj() {
        Ok(()) => true,
        Err(error) => {
            event_log.user_error(Some(ORIGIN), format!("whte_rbt.obj failed: {error}"));
            false
        }
    };

    WhteRbtObjResult {
        event_log,
        successful,
    }
}

fn run_whte_rbt_obj() -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    delayed_line(
        &mut stdout,
        "Jurassic Park, System Security Interface",
        LINE_DELAY,
    )?;
    delayed_line(&mut stdout, "Version 4.0.5, Alpha E", LINE_DELAY)?;
    delayed_line(&mut stdout, "Ready...", LINE_DELAY)?;

    typed_prompt(&mut stdout, "access security")?;
    delayed_line(&mut stdout, "access: PERMISSION DENIED.", LINE_DELAY)?;

    typed_prompt(&mut stdout, "access security grid")?;
    delayed_line(&mut stdout, "access: PERMISSION DENIED.", LINE_DELAY)?;

    typed_prompt(&mut stdout, "access main security grid")?;
    thread::sleep(LONG_DELAY);
    write!(stdout, "access: PERMISSION DENIED.")?;
    stdout.flush()?;
    thread::sleep(LINE_DELAY);
    writeln!(stdout, "...and....")?;
    stdout.flush()?;
    thread::sleep(LONG_DELAY);

    loop {
        writeln!(stdout, "YOU DIDN'T SAY THE MAGIC WORD!")?;
        stdout.flush()?;
        thread::sleep(MAGIC_WORD_DELAY);
    }
}

fn delayed_line(stdout: &mut impl Write, message: &str, delay_after: Duration) -> io::Result<()> {
    writeln!(stdout, "{message}")?;
    stdout.flush()?;
    thread::sleep(delay_after);
    Ok(())
}

fn typed_prompt(stdout: &mut impl Write, text: &str) -> io::Result<()> {
    write!(stdout, "> ")?;
    stdout.flush()?;

    for ch in text.chars() {
        thread::sleep(TYPE_DELAY);
        write!(stdout, "{ch}")?;
        stdout.flush()?;
    }

    writeln!(stdout)?;
    stdout.flush()
}

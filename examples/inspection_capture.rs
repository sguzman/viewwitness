use std::{env, io};

use viewwitness::{InspectionObserver, to_agent_text, to_yaml};

fn main() -> io::Result<()> {
    let options = Options::from_args()?;
    let mut observer = InspectionObserver::connect(&options.addr)?;

    let witness = if let Some(max_steps) = options.settle_steps {
        let (settle, witness) = observer.settle_and_capture(max_steps)?;
        eprintln!("settle settled={} steps={}", settle.settled, settle.steps);
        witness
    } else {
        observer.capture()?
    };

    let Some(witness) = witness else {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "inspection peer has not produced an AccessKit tree yet",
        ));
    };

    if options.agent_text {
        print!("{}", to_agent_text(&witness));
    } else {
        let yaml = to_yaml(&witness)
            .map_err(|error| io::Error::other(format!("failed to serialize witness: {error}")))?;
        print!("{yaml}");
    }

    Ok(())
}

struct Options {
    addr: String,
    agent_text: bool,
    settle_steps: Option<u64>,
}

impl Options {
    fn from_args() -> io::Result<Self> {
        let mut addr = None;
        let mut agent_text = false;
        let mut settle_steps = None;

        for arg in env::args().skip(1) {
            if arg == "--agent" {
                agent_text = true;
            } else if let Some(value) = arg.strip_prefix("--settle=") {
                settle_steps = Some(value.parse::<u64>().map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("invalid --settle step count {value:?}: {error}"),
                    )
                })?);
            } else if arg.starts_with('-') {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown option {arg:?}"),
                ));
            } else if addr.replace(arg).is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "only one inspection address may be supplied",
                ));
            }
        }

        Ok(Self {
            addr: addr.unwrap_or_else(|| "127.0.0.1:5719".to_owned()),
            agent_text,
            settle_steps,
        })
    }
}

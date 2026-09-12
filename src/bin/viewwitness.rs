use std::{env, fs, io, process};

use viewwitness::{
    Witness, diff_to_agent_text, diff_to_yaml, diff_witnesses, from_yaml, to_agent_text, to_yaml,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("viewwitness: {error}");
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage_error();
    };

    match command.as_str() {
        "validate" => validate_command(args.collect()),
        "inspect" => inspect_command(args.collect()),
        "inspect-exact" => inspect_exact_command(args.collect()),
        "derive" => derive_command(args.collect()),
        "diff" => diff_command(args.collect()),
        "diff-exact" => diff_exact_command(args.collect()),
        "capture" => capture_command(args.collect()),
        "capture-exact" => capture_exact_command(args.collect()),
        "screenshot" => screenshot_command(args.collect()),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        _ => Err(invalid(format!("unknown command {command:?}"))),
    }
}

fn validate_command(args: Vec<String>) -> io::Result<()> {
    let [path] = args.as_slice() else {
        return Err(invalid("usage: viewwitness validate <file>"));
    };
    let witness = load_witness(path)?;
    validate_witness(&witness)?;
    println!("ok {path}");
    Ok(())
}

fn inspect_command(args: Vec<String>) -> io::Result<()> {
    let (positionals, mode) = output_args(args)?;
    let [path] = positionals.as_slice() else {
        return Err(invalid(
            "usage: viewwitness inspect <file> [--agent|--yaml]",
        ));
    };
    let witness = load_witness(path)?;
    validate_witness(&witness)?;
    print_witness(&witness, mode)
}

#[cfg(feature = "egui")]
fn inspect_exact_command(args: Vec<String>) -> io::Result<()> {
    let mut path = None;
    let mut mode = OutputMode::Agent;
    let mut focus = ExactFocus::default();

    for arg in args {
        match arg.as_str() {
            "--agent" => mode = OutputMode::Agent,
            "--yaml" => mode = OutputMode::Yaml,
            _ if parse_exact_focus_option(&arg, &mut focus)? => {}
            _ if arg.starts_with('-') => {
                return Err(invalid(format!("unknown inspect-exact option {arg:?}")));
            }
            _ if path.replace(arg).is_some() => {
                return Err(invalid("only one correlated capture path may be supplied"));
            }
            _ => {}
        }
    }

    let Some(path) = path else {
        return Err(invalid(
            "usage: viewwitness inspect-exact <file> [--agent|--yaml] [--object=ID [--binding=ID]]",
        ));
    };
    validate_exact_focus(&focus, mode, false)?;

    let capture = load_correlated_capture(&path)?;
    validate_witness(&capture.witness)?;
    print_correlated_capture(&capture, mode, &focus)
}

#[cfg(not(feature = "egui"))]
fn inspect_exact_command(_args: Vec<String>) -> io::Result<()> {
    Err(invalid(
        "exact correlated inspection requires the `egui` feature; rebuild with `--features egui`",
    ))
}

fn derive_command(args: Vec<String>) -> io::Result<()> {
    let (positionals, mode) = output_args(args)?;
    let [path] = positionals.as_slice() else {
        return Err(invalid("usage: viewwitness derive <file> [--agent|--yaml]"));
    };
    let mut witness = load_witness(path)?;
    validate_witness(&witness)?;
    enrich_geometry(&mut witness);
    print_witness(&witness, mode)
}

fn diff_command(args: Vec<String>) -> io::Result<()> {
    let (positionals, mode) = output_args(args)?;
    let [before_path, after_path] = positionals.as_slice() else {
        return Err(invalid(
            "usage: viewwitness diff <before> <after> [--agent|--yaml]",
        ));
    };

    let before = load_witness(before_path)?;
    let after = load_witness(after_path)?;
    validate_witness(&before)?;
    validate_witness(&after)?;
    let diff = diff_witnesses(&before, &after);

    match mode {
        OutputMode::Agent => print!("{}", diff_to_agent_text(&diff)),
        OutputMode::Yaml => print!("{}", map_yaml(diff_to_yaml(&diff))?),
    }
    Ok(())
}

#[cfg(feature = "egui")]
fn diff_exact_command(args: Vec<String>) -> io::Result<()> {
    use viewwitness::{correlated_diff_to_agent_text, diff_correlated_captures};

    let (positionals, mode) = output_args(args)?;
    let [before_path, after_path] = positionals.as_slice() else {
        return Err(invalid(
            "usage: viewwitness diff-exact <before> <after> [--agent|--yaml]",
        ));
    };

    let before = load_correlated_capture(before_path)?;
    let after = load_correlated_capture(after_path)?;
    validate_witness(&before.witness)?;
    validate_witness(&after.witness)?;
    let diff = diff_correlated_captures(&before, &after);

    match mode {
        OutputMode::Agent => print!("{}", correlated_diff_to_agent_text(&diff)),
        OutputMode::Yaml => {
            let yaml = serde_yaml_ng::to_string(&diff).map_err(|error| {
                io::Error::other(format!("failed to serialize correlated diff YAML: {error}"))
            })?;
            print!("{yaml}");
        }
    }
    Ok(())
}

#[cfg(not(feature = "egui"))]
fn diff_exact_command(_args: Vec<String>) -> io::Result<()> {
    Err(invalid(
        "exact correlated diff requires the `egui` feature; rebuild with `--features egui`",
    ))
}

#[cfg(feature = "observer")]
fn capture_command(args: Vec<String>) -> io::Result<()> {
    use viewwitness::InspectionObserver;

    let mut addr = None;
    let mut mode = OutputMode::Agent;
    let mut settle_steps = None;
    let mut derive = false;

    for arg in args {
        match arg.as_str() {
            "--agent" => mode = OutputMode::Agent,
            "--yaml" => mode = OutputMode::Yaml,
            "--derive" => derive = true,
            _ if arg.starts_with("--settle=") => {
                let value = arg.trim_start_matches("--settle=");
                settle_steps = Some(value.parse::<u64>().map_err(|error| {
                    invalid(format!("invalid --settle step count {value:?}: {error}"))
                })?);
            }
            _ if arg.starts_with('-') => {
                return Err(invalid(format!("unknown capture option {arg:?}")));
            }
            _ if addr.is_none() => addr = Some(arg),
            _ => return Err(invalid("only one inspection address may be supplied")),
        }
    }

    let addr = addr.unwrap_or_else(|| "127.0.0.1:5719".to_owned());
    let mut observer = InspectionObserver::connect(&addr)?;
    let witness = if let Some(max_steps) = settle_steps {
        let (settle, witness) = observer.settle_and_capture(max_steps)?;
        eprintln!("settle settled={} steps={}", settle.settled, settle.steps);
        witness
    } else {
        observer.capture()?
    };
    let Some(mut witness) = witness else {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "inspection peer has not produced an AccessKit tree yet",
        ));
    };

    validate_witness(&witness)?;
    if derive {
        enrich_geometry(&mut witness);
    }
    print_witness(&witness, mode)
}

#[cfg(not(feature = "observer"))]
fn capture_command(_args: Vec<String>) -> io::Result<()> {
    Err(invalid(
        "live capture requires the `observer` feature; rebuild with `--features observer`",
    ))
}

#[cfg(feature = "egui")]
fn capture_exact_command(args: Vec<String>) -> io::Result<()> {
    use viewwitness::{DEFAULT_EGUI_CAPTURE_ADDR, EguiCaptureObserver};

    let mut addr = None;
    let mut mode = OutputMode::Agent;
    let mut derive = false;
    let mut focus = ExactFocus::default();

    for arg in args {
        match arg.as_str() {
            "--agent" => mode = OutputMode::Agent,
            "--yaml" => mode = OutputMode::Yaml,
            "--derive" => derive = true,
            _ if parse_exact_focus_option(&arg, &mut focus)? => {}
            _ if arg.starts_with('-') => {
                return Err(invalid(format!("unknown capture-exact option {arg:?}")));
            }
            _ if addr.is_none() => addr = Some(arg),
            _ => return Err(invalid("only one exact-capture address may be supplied")),
        }
    }

    validate_exact_focus(&focus, mode, derive)?;

    let addr = addr.unwrap_or_else(|| DEFAULT_EGUI_CAPTURE_ADDR.to_owned());
    let mut observer = EguiCaptureObserver::connect(&addr)?;
    let mut capture = observer.capture()?;
    validate_witness(&capture.witness)?;
    if derive {
        enrich_geometry(&mut capture.witness);
    }
    print_correlated_capture(&capture, mode, &focus)
}

#[cfg(not(feature = "egui"))]
fn capture_exact_command(_args: Vec<String>) -> io::Result<()> {
    Err(invalid(
        "exact correlated capture requires the `egui` feature; rebuild with `--features egui`",
    ))
}

#[cfg(feature = "observer")]
fn screenshot_command(args: Vec<String>) -> io::Result<()> {
    use viewwitness::InspectionObserver;

    let mut output_path = None;
    let mut addr = "127.0.0.1:5719".to_owned();
    let mut scale = None;

    for arg in args {
        if let Some(value) = arg.strip_prefix("--address=") {
            if value.is_empty() {
                return Err(invalid("--address must not be empty"));
            }
            addr = value.to_owned();
        } else if let Some(value) = arg.strip_prefix("--scale=") {
            scale =
                Some(value.parse::<f32>().map_err(|error| {
                    invalid(format!("invalid --scale value {value:?}: {error}"))
                })?);
        } else if arg.starts_with('-') {
            return Err(invalid(format!("unknown screenshot option {arg:?}")));
        } else if output_path.replace(arg).is_some() {
            return Err(invalid("only one screenshot output path may be supplied"));
        }
    }

    let Some(output_path) = output_path else {
        return Err(invalid(
            "usage: viewwitness screenshot <output.png> [--address=HOST:PORT] [--scale=N]",
        ));
    };

    let mut observer = InspectionObserver::connect(&addr)?;
    let screenshot = observer.screenshot(scale)?;
    fs::write(&output_path, screenshot.png_bytes).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to write screenshot {output_path}: {error}"),
        )
    })?;
    println!(
        "saved {output_path} {}x{}",
        screenshot.size[0], screenshot.size[1]
    );
    Ok(())
}

#[cfg(not(feature = "observer"))]
fn screenshot_command(_args: Vec<String>) -> io::Result<()> {
    Err(invalid(
        "screenshot capture requires the `observer` feature; rebuild with `--features observer`",
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    Agent,
    Yaml,
}

#[cfg(feature = "egui")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ExactFocus {
    object_id: Option<String>,
    binding_id: Option<String>,
}

#[cfg(feature = "egui")]
impl ExactFocus {
    fn is_active(&self) -> bool {
        self.object_id.is_some()
    }
}

#[cfg(feature = "egui")]
fn parse_exact_focus_option(arg: &str, focus: &mut ExactFocus) -> io::Result<bool> {
    if let Some(value) = arg.strip_prefix("--object=") {
        if value.is_empty() {
            return Err(invalid("--object must not be empty"));
        }
        if focus.object_id.replace(value.to_owned()).is_some() {
            return Err(invalid("--object may only be supplied once"));
        }
        return Ok(true);
    }
    if let Some(value) = arg.strip_prefix("--binding=") {
        if value.is_empty() {
            return Err(invalid("--binding must not be empty"));
        }
        if focus.binding_id.replace(value.to_owned()).is_some() {
            return Err(invalid("--binding may only be supplied once"));
        }
        return Ok(true);
    }
    Ok(false)
}

#[cfg(feature = "egui")]
fn validate_exact_focus(focus: &ExactFocus, mode: OutputMode, derive: bool) -> io::Result<()> {
    if focus.binding_id.is_some() && focus.object_id.is_none() {
        return Err(invalid("--binding requires --object"));
    }
    if focus.is_active() && mode == OutputMode::Yaml {
        return Err(invalid(
            "authored focus is an agent-text projection; --object/--binding cannot be combined with --yaml",
        ));
    }
    if focus.is_active() && derive {
        return Err(invalid(
            "authored focus omits canonical semantics; --derive cannot be combined with --object/--binding",
        ));
    }
    Ok(())
}

#[cfg(feature = "egui")]
fn print_correlated_capture(
    capture: &viewwitness::EguiCorrelatedCapture,
    mode: OutputMode,
    focus: &ExactFocus,
) -> io::Result<()> {
    use viewwitness::{
        correlated_capture_authored_focus_to_agent_text, correlated_capture_to_agent_text,
    };

    match (mode, focus.object_id.as_deref()) {
        (OutputMode::Agent, Some(object_id)) => print!(
            "{}",
            correlated_capture_authored_focus_to_agent_text(
                capture,
                object_id,
                focus.binding_id.as_deref(),
            )
        ),
        (OutputMode::Agent, None) => print!("{}", correlated_capture_to_agent_text(capture)),
        (OutputMode::Yaml, None) => {
            let yaml = serde_yaml_ng::to_string(capture).map_err(|error| {
                io::Error::other(format!(
                    "failed to serialize correlated capture YAML: {error}"
                ))
            })?;
            print!("{yaml}");
        }
        (OutputMode::Yaml, Some(_)) => {
            unreachable!("focused YAML is rejected by validate_exact_focus")
        }
    }
    Ok(())
}

fn output_args(args: Vec<String>) -> io::Result<(Vec<String>, OutputMode)> {
    let mut positionals = Vec::new();
    let mut mode = OutputMode::Agent;

    for arg in args {
        match arg.as_str() {
            "--agent" => mode = OutputMode::Agent,
            "--yaml" => mode = OutputMode::Yaml,
            _ if arg.starts_with('-') => {
                return Err(invalid(format!("unknown output option {arg:?}")));
            }
            _ => positionals.push(arg),
        }
    }

    Ok((positionals, mode))
}

fn load_witness(path: &str) -> io::Result<Witness> {
    let source = fs::read_to_string(path)
        .map_err(|error| io::Error::new(error.kind(), format!("failed to read {path}: {error}")))?;
    from_yaml(&source).map_err(|error| invalid(format!("failed to parse {path}: {error}")))
}

#[cfg(feature = "egui")]
fn load_correlated_capture(path: &str) -> io::Result<viewwitness::EguiCorrelatedCapture> {
    let source = fs::read_to_string(path)
        .map_err(|error| io::Error::new(error.kind(), format!("failed to read {path}: {error}")))?;
    serde_yaml_ng::from_str(&source).map_err(|error| {
        invalid(format!(
            "failed to parse correlated capture {path}: {error}"
        ))
    })
}

fn validate_witness(witness: &Witness) -> io::Result<()> {
    let issues = witness.validation_issues();
    if issues.is_empty() {
        return Ok(());
    }

    Err(invalid(format!(
        "witness validation failed:\n- {}",
        issues.join("\n- ")
    )))
}

fn enrich_geometry(witness: &mut Witness) {
    for relation in viewwitness::derive_geometry_relations(witness) {
        if !witness.relations.contains(&relation) {
            witness.relations.push(relation);
        }
    }
    witness.relations.sort_by_cached_key(|relation| {
        serde_json::to_string(relation).expect("ViewWitness relations must serialize")
    });
}

fn print_witness(witness: &Witness, mode: OutputMode) -> io::Result<()> {
    match mode {
        OutputMode::Agent => print!("{}", to_agent_text(witness)),
        OutputMode::Yaml => print!("{}", map_yaml(to_yaml(witness))?),
    }
    Ok(())
}

fn map_yaml(result: Result<String, serde_yaml_ng::Error>) -> io::Result<String> {
    result.map_err(|error| io::Error::other(format!("failed to serialize YAML: {error}")))
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn usage_error<T>() -> io::Result<T> {
    print_usage();
    Err(invalid("a command is required"))
}

fn print_usage() {
    eprintln!(
        "ViewWitness\n\
         usage:\n\
           viewwitness validate <file>\n\
           viewwitness inspect <file> [--agent|--yaml]\n\
           viewwitness inspect-exact <file> [--agent|--yaml] [--object=ID [--binding=ID]]\n\
           viewwitness derive <file> [--agent|--yaml]\n\
           viewwitness diff <before> <after> [--agent|--yaml]\n\
           viewwitness diff-exact <before> <after> [--agent|--yaml]\n\
           viewwitness capture [address] [--agent|--yaml] [--derive] [--settle=N]\n\
           viewwitness capture-exact [address] [--agent|--yaml] [--derive] [--object=ID [--binding=ID]]\n\
           viewwitness screenshot <output.png> [--address=HOST:PORT] [--scale=N]\n\
         \n\
         Agent text is the default output for inspect, inspect-exact, derive, diff, diff-exact, capture, and capture-exact.\n\
         `capture` reads semantic state through egui_inspection; `capture-exact` reads ViewWitness's\n\
         same-pass semantic + viewport + paint evidence. `inspect-exact` reads a saved full correlated\n\
         envelope. `--object`/`--binding` produce an authored-focus agent projection and therefore cannot\n\
         be combined with `--yaml`; `--binding` requires `--object`. `diff-exact` compares two saved full\n\
         correlated capture envelopes. Screenshots remain separate raster evidence."
    );
}

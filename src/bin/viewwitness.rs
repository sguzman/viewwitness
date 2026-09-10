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
        "derive" => derive_command(args.collect()),
        "diff" => diff_command(args.collect()),
        "capture" => capture_command(args.collect()),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    Agent,
    Yaml,
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
           viewwitness derive <file> [--agent|--yaml]\n\
           viewwitness diff <before> <after> [--agent|--yaml]\n\
           viewwitness capture [address] [--agent|--yaml] [--derive] [--settle=N]\n\
         \n\
         Agent text is the default output for inspect, derive, diff, and capture."
    );
}

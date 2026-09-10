use std::{env, io};

use viewwitness::{InspectionObserver, to_yaml};

fn main() -> io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5719".to_owned());

    let mut observer = InspectionObserver::connect(&addr)?;
    let Some(witness) = observer.capture()? else {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "inspection peer has not produced an AccessKit tree yet",
        ));
    };

    let yaml = to_yaml(&witness)
        .map_err(|error| io::Error::other(format!("failed to serialize witness: {error}")))?;
    print!("{yaml}");
    Ok(())
}

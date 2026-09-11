use std::{
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::{EguiCorrelatedCapture, EguiFrameProbe};

/// Default loopback endpoint reserved by ViewWitness's exact egui capture API.
///
/// `5719` remains egui inspection and `5720` remains the continuous paint
/// stream. Exact correlated capture is intentionally a distinct request/response
/// instrument with a different cost and cadence.
pub const DEFAULT_EGUI_CAPTURE_ADDR: &str = "127.0.0.1:5721";

/// Human-readable protocol identifier sent before capture requests.
pub const EGUI_CAPTURE_PROTOCOL_MAGIC: &str = "VIEWWITNESS-EGUI-CAPTURE";

/// Version of the ViewWitness-owned exact capture protocol.
pub const EGUI_CAPTURE_PROTOCOL_VERSION: u32 = 1;

/// Defensive upper bound for one serialized correlated capture response.
pub const MAX_EGUI_CAPTURE_MESSAGE_BYTES: usize = 64 * 1024 * 1024;

const MAX_REQUEST_BYTES: usize = 1024;
const CLIENT_WRITE_TIMEOUT: Duration = Duration::from_millis(250);
const CAPTURE_COMMAND: &[u8] = b"CAPTURE";

/// Run the read-only exact egui capture server on the current thread.
///
/// This function is deliberately blocking and belongs on a worker thread. The
/// owned [`EguiFrameProbe`] communicates with egui through an atomic request,
/// repaint notification, and bounded response queue. Waiting, canonical model
/// conversion, JSON serialization, and socket I/O therefore remain outside the
/// GUI/render thread.
///
/// Each connected observer may request captures sequentially. A capture error
/// such as timeout, queue saturation, or invalid observed evidence is returned
/// to that observer as structured protocol evidence and does not crash the
/// server. A disconnected/malformed client is disposable.
pub fn run_egui_capture_server(
    listener: TcpListener,
    mut probe: EguiFrameProbe,
    capture_timeout: Duration,
) -> io::Result<()> {
    loop {
        let (stream, _) = listener.accept()?;
        match serve_client(stream, &mut probe, capture_timeout) {
            Ok(()) => {}
            Err(error) if disposable_client_error(&error) => {}
            Err(error) => return Err(error),
        }
    }
}

/// Read-only client for ViewWitness's exact correlated egui capture endpoint.
pub struct EguiCaptureObserver {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl EguiCaptureObserver {
    /// Connect and validate the exact-capture protocol handshake.
    pub fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        let writer_stream = stream.try_clone()?;
        let mut observer = Self {
            reader: BufReader::new(stream),
            writer: BufWriter::new(writer_stream),
        };
        observer.read_handshake()?;
        Ok(observer)
    }

    /// Request one exact same-pass semantic + viewport + paint capture.
    ///
    /// The server-side timeout is configured by the application hosting the
    /// probe. Server capture failures are mapped back into ordinary I/O error
    /// kinds so callers can distinguish timeout, bounded-queue loss, invalid
    /// evidence, and broken probe state.
    pub fn capture(&mut self) -> io::Result<EguiCorrelatedCapture> {
        self.writer.write_all(CAPTURE_COMMAND)?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()?;

        let line = read_bounded_line(
            &mut self.reader,
            MAX_EGUI_CAPTURE_MESSAGE_BYTES,
            "egui capture response",
        )?;
        let response: CaptureResponse = serde_json::from_slice(&line)
            .map_err(|error| invalid_data(format!("invalid egui capture response: {error}")))?;

        match response {
            CaptureResponse::Ok { capture } => Ok(capture),
            CaptureResponse::Error { kind, message } => Err(io::Error::new(kind.into(), message)),
        }
    }

    fn read_handshake(&mut self) -> io::Result<()> {
        let line = read_bounded_line(&mut self.reader, MAX_REQUEST_BYTES, "egui capture handshake")?;
        let expected = format!("{EGUI_CAPTURE_PROTOCOL_MAGIC} {EGUI_CAPTURE_PROTOCOL_VERSION}");
        if line == expected.as_bytes() {
            Ok(())
        } else {
            Err(invalid_data(format!(
                "unexpected egui capture handshake {:?}; expected {expected:?}",
                String::from_utf8_lossy(&line)
            )))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum CaptureResponse {
    Ok { capture: EguiCorrelatedCapture },
    Error { kind: WireErrorKind, message: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WireErrorKind {
    TimedOut,
    WouldBlock,
    InvalidData,
    BrokenPipe,
    Other,
}

impl From<io::ErrorKind> for WireErrorKind {
    fn from(kind: io::ErrorKind) -> Self {
        match kind {
            io::ErrorKind::TimedOut => Self::TimedOut,
            io::ErrorKind::WouldBlock => Self::WouldBlock,
            io::ErrorKind::InvalidData => Self::InvalidData,
            io::ErrorKind::BrokenPipe => Self::BrokenPipe,
            _ => Self::Other,
        }
    }
}

impl From<WireErrorKind> for io::ErrorKind {
    fn from(kind: WireErrorKind) -> Self {
        match kind {
            WireErrorKind::TimedOut => Self::TimedOut,
            WireErrorKind::WouldBlock => Self::WouldBlock,
            WireErrorKind::InvalidData => Self::InvalidData,
            WireErrorKind::BrokenPipe => Self::BrokenPipe,
            WireErrorKind::Other => Self::Other,
        }
    }
}

fn serve_client(
    stream: TcpStream,
    probe: &mut EguiFrameProbe,
    capture_timeout: Duration,
) -> io::Result<()> {
    stream.set_nodelay(true)?;
    stream.set_write_timeout(Some(CLIENT_WRITE_TIMEOUT))?;
    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);
    let mut writer = BufWriter::new(stream);

    write_handshake(&mut writer)?;

    loop {
        let request = match read_bounded_line(&mut reader, MAX_REQUEST_BYTES, "egui capture request") {
            Ok(request) => request,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(error),
        };

        if request != CAPTURE_COMMAND {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "unexpected egui capture request {:?}; expected CAPTURE",
                    String::from_utf8_lossy(&request)
                ),
            ));
        }

        let response = match probe
            .capture_timeout(capture_timeout)
            .and_then(|evidence| evidence.into_correlated_capture())
        {
            Ok(capture) => CaptureResponse::Ok { capture },
            Err(error) => CaptureResponse::Error {
                kind: error.kind().into(),
                message: error.to_string(),
            },
        };
        write_response(&mut writer, &response)?;
    }
}

fn write_handshake(writer: &mut BufWriter<TcpStream>) -> io::Result<()> {
    writeln!(
        writer,
        "{EGUI_CAPTURE_PROTOCOL_MAGIC} {EGUI_CAPTURE_PROTOCOL_VERSION}"
    )?;
    writer.flush()
}

fn write_response(writer: &mut BufWriter<TcpStream>, response: &CaptureResponse) -> io::Result<()> {
    let encoded = serde_json::to_vec(response).map_err(|error| {
        invalid_data(format!(
            "failed to serialize exact egui capture response: {error}"
        ))
    })?;
    if encoded.len() > MAX_EGUI_CAPTURE_MESSAGE_BYTES {
        return Err(invalid_data(format!(
            "egui capture response is {} bytes; maximum is {MAX_EGUI_CAPTURE_MESSAGE_BYTES}",
            encoded.len()
        )));
    }

    writer.write_all(&encoded)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

fn read_bounded_line(
    reader: &mut impl BufRead,
    max_bytes: usize,
    description: &str,
) -> io::Result<Vec<u8>> {
    let mut line = Vec::new();
    let mut limited = reader.take((max_bytes + 1) as u64);
    let read = limited.read_until(b'\n', &mut line)?;

    if read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("{description} stream closed"),
        ));
    }
    if line.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{description} exceeds {max_bytes} bytes"),
        ));
    }
    if line.last() != Some(&b'\n') {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("{description} ended before newline terminator"),
        ));
    }

    line.pop();
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    Ok(line)
}

fn disposable_client_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::UnexpectedEof
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::TimedOut
            | io::ErrorKind::InvalidInput
    )
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

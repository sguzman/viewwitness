use std::{
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use crate::EguiPaintFrame;

/// Default loopback endpoint reserved by ViewWitness's egui paint side channel.
///
/// This is intentionally distinct from egui inspection's default `5719` port.
pub const DEFAULT_EGUI_PAINT_ADDR: &str = "127.0.0.1:5720";

/// Human-readable protocol identifier sent before any paint frames.
pub const EGUI_PAINT_PROTOCOL_MAGIC: &str = "VIEWWITNESS-EGUI-PAINT";

/// Version of the small ViewWitness-owned paint stream protocol.
pub const EGUI_PAINT_PROTOCOL_VERSION: u32 = 1;

/// Defensive upper bound for one compact JSON paint frame received from a peer.
pub const MAX_EGUI_PAINT_MESSAGE_BYTES: usize = 64 * 1024 * 1024;

const SERVER_POLL_INTERVAL: Duration = Duration::from_millis(20);
const CLIENT_WRITE_TIMEOUT: Duration = Duration::from_millis(250);

/// Run the read-only egui paint side channel on the current thread.
///
/// This function is deliberately blocking and should be run on a worker thread.
/// The GUI-facing [`crate::EguiPaintReporter`] communicates with it only through
/// a bounded nonblocking channel, so listener activity, JSON serialization, and
/// socket I/O never execute in egui's render/output hook.
///
/// The server keeps only the latest delivered paint frame while no observer is
/// connected. A newly connected observer receives that frame immediately, then
/// subsequent frames as newline-delimited compact JSON. At most one observer is
/// active; a successfully initialized newer connection replaces the older one.
///
/// The server exits successfully when all reporter senders have disconnected.
pub fn run_egui_paint_server(
    listener: TcpListener,
    receiver: Receiver<EguiPaintFrame>,
) -> io::Result<()> {
    listener.set_nonblocking(true)?;

    let mut client: Option<BufWriter<TcpStream>> = None;
    let mut latest: Option<EguiPaintFrame> = None;

    loop {
        accept_pending_clients(&listener, &mut client, latest.as_ref())?;

        match receiver.recv_timeout(SERVER_POLL_INTERVAL) {
            Ok(frame) => {
                if let Some(writer) = client.as_mut() {
                    match write_frame(writer, &frame) {
                        Ok(()) => {}
                        Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                            return Err(error);
                        }
                        Err(_) => {
                            // A slow, closed, or otherwise unhealthy observer is
                            // disposable. The reporter remains isolated behind its
                            // bounded queue and the worker keeps the latest frame.
                            client = None;
                        }
                    }
                }
                latest = Some(frame);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}

/// Read-only consumer for ViewWitness's egui paint side channel.
pub struct EguiPaintObserver {
    reader: BufReader<TcpStream>,
}

impl EguiPaintObserver {
    /// Connect and validate the ViewWitness paint-stream handshake.
    pub fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        let mut observer = Self {
            reader: BufReader::new(stream),
        };
        observer.read_handshake()?;
        Ok(observer)
    }

    /// Block until the next paint frame is available on the side channel.
    pub fn next_frame(&mut self) -> io::Result<EguiPaintFrame> {
        let line = read_bounded_line(&mut self.reader)?;
        serde_json::from_slice(&line)
            .map_err(|error| invalid_data(format!("invalid egui paint frame: {error}")))
    }

    fn read_handshake(&mut self) -> io::Result<()> {
        let line = read_bounded_line(&mut self.reader)?;
        let expected = format!("{EGUI_PAINT_PROTOCOL_MAGIC} {EGUI_PAINT_PROTOCOL_VERSION}");
        if line == expected.as_bytes() {
            Ok(())
        } else {
            Err(invalid_data(format!(
                "unexpected egui paint handshake {:?}; expected {expected:?}",
                String::from_utf8_lossy(&line)
            )))
        }
    }
}

fn accept_pending_clients(
    listener: &TcpListener,
    client: &mut Option<BufWriter<TcpStream>>,
    latest: Option<&EguiPaintFrame>,
) -> io::Result<()> {
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                if let Some(writer) = prepare_client(stream, latest)? {
                    *client = Some(writer);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(error) => return Err(error),
        }
    }
}

fn prepare_client(
    stream: TcpStream,
    latest: Option<&EguiPaintFrame>,
) -> io::Result<Option<BufWriter<TcpStream>>> {
    stream.set_nodelay(true)?;
    stream.set_write_timeout(Some(CLIENT_WRITE_TIMEOUT))?;
    let mut writer = BufWriter::new(stream);

    if let Err(error) = write_handshake(&mut writer) {
        if error.kind() == io::ErrorKind::InvalidData {
            return Err(error);
        }
        return Ok(None);
    }

    if let Some(frame) = latest {
        if let Err(error) = write_frame(&mut writer, frame) {
            if error.kind() == io::ErrorKind::InvalidData {
                return Err(error);
            }
            return Ok(None);
        }
    }

    Ok(Some(writer))
}

fn write_handshake(writer: &mut BufWriter<TcpStream>) -> io::Result<()> {
    writeln!(
        writer,
        "{EGUI_PAINT_PROTOCOL_MAGIC} {EGUI_PAINT_PROTOCOL_VERSION}"
    )?;
    writer.flush()
}

fn write_frame(writer: &mut BufWriter<TcpStream>, frame: &EguiPaintFrame) -> io::Result<()> {
    let encoded = serde_json::to_vec(frame)
        .map_err(|error| invalid_data(format!("failed to serialize egui paint frame: {error}")))?;
    if encoded.len() > MAX_EGUI_PAINT_MESSAGE_BYTES {
        return Err(invalid_data(format!(
            "egui paint frame is {} bytes; maximum is {MAX_EGUI_PAINT_MESSAGE_BYTES}",
            encoded.len()
        )));
    }

    writer.write_all(&encoded)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

fn read_bounded_line(reader: &mut impl BufRead) -> io::Result<Vec<u8>> {
    let mut line = Vec::new();
    let mut limited = reader.take((MAX_EGUI_PAINT_MESSAGE_BYTES + 1) as u64);
    let read = limited.read_until(b'\n', &mut line)?;

    if read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "egui paint stream closed",
        ));
    }
    if line.len() > MAX_EGUI_PAINT_MESSAGE_BYTES {
        return Err(invalid_data(format!(
            "egui paint message exceeds {MAX_EGUI_PAINT_MESSAGE_BYTES} bytes"
        )));
    }
    if line.last() != Some(&b'\n') {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "egui paint message ended before newline terminator",
        ));
    }

    line.pop();
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    Ok(line)
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

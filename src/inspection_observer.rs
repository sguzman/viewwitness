use std::{io, net::TcpStream};

use egui::accesskit::TreeUpdate;
use egui_inspection::{PROTOCOL_VERSION, Request, Response, read_message, write_message};
use serde_json::json;

use crate::{EguiCaptureContext, Viewport, Witness, witness_from_egui_tree_update};

/// Result of asking an inspected egui application to run until it considers
/// itself idle, bounded by a caller-supplied maximum number of steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettleResult {
    pub settled: bool,
    pub steps: u64,
}

/// Read-only client for the egui inspection protocol.
///
/// The observer lives outside the GUI process. It requests fresh semantic state
/// and converts that observed frame into the canonical ViewWitness model;
/// serialization, diffing, and further analysis can therefore happen entirely
/// off the GUI thread.
pub struct InspectionObserver {
    stream: TcpStream,
}

impl InspectionObserver {
    /// Connect to a running egui inspection endpoint and verify its wire
    /// protocol version before exchanging MessagePack requests.
    ///
    /// # Errors
    /// Returns an I/O error when the endpoint is unavailable, the handshake is
    /// malformed, or the peer speaks a different protocol version.
    pub fn connect(addr: &str) -> io::Result<Self> {
        let mut stream = TcpStream::connect(addr)?;
        let peer_version = egui_inspection::protocol::read_handshake(&mut stream)?;
        if peer_version != PROTOCOL_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "egui inspection protocol mismatch: peer={peer_version}, supported={PROTOCOL_VERSION}"
                ),
            ));
        }
        stream.set_nodelay(true)?;
        Ok(Self { stream })
    }

    /// Ask the peer to run until idle, up to `max_steps` frames.
    ///
    /// This is synchronization for observation, not an input action: ViewWitness
    /// does not synthesize user events or mutate widgets through this method.
    /// A `settled: false` result is preserved as evidence rather than converted
    /// into an error, because callers may still want to capture a busy UI.
    ///
    /// # Errors
    /// Returns an error for transport/protocol failures, peer-side errors, or
    /// an unexpected response variant.
    pub fn settle(&mut self, max_steps: u64) -> io::Result<SettleResult> {
        write_message(&mut self.stream, &Request::Settle { max_steps })?;
        let response: Response = read_message(&mut self.stream)?;

        match response {
            Response::Settled { settled, steps } => Ok(SettleResult { settled, steps }),
            Response::Error { message } => Err(peer_error(message)),
            other => Err(unexpected_response("Settle", other)),
        }
    }

    /// Request the current semantic tree and translate it into a witness.
    ///
    /// `Ok(None)` means the peer replied successfully but has not produced an
    /// AccessKit tree yet.
    ///
    /// # Errors
    /// Returns an error for transport/protocol failures, unexpected responses,
    /// peer-side errors, or a tree whose root lacks bounds from which the
    /// viewport can be established without guessing.
    pub fn capture(&mut self) -> io::Result<Option<Witness>> {
        write_message(&mut self.stream, &Request::GetTree)?;
        let response: Response = read_message(&mut self.stream)?;

        match response {
            Response::Tree {
                step,
                pixels_per_point,
                accesskit,
            } => accesskit
                .map(|update| witness_from_tree(update, step, pixels_per_point))
                .transpose(),
            Response::Error { message } => Err(peer_error(message)),
            other => Err(unexpected_response("GetTree", other)),
        }
    }

    /// Settle the inspected application and then capture the resulting semantic
    /// frame using the same connection.
    ///
    /// The capture is still attempted when the peer reports `settled: false`;
    /// the caller receives both facts and can decide whether a non-idle witness
    /// is acceptable for its verification task.
    ///
    /// # Errors
    /// Propagates errors from [`Self::settle`] or [`Self::capture`].
    pub fn settle_and_capture(
        &mut self,
        max_steps: u64,
    ) -> io::Result<(SettleResult, Option<Witness>)> {
        let settle = self.settle(max_steps)?;
        let witness = self.capture()?;
        Ok((settle, witness))
    }
}

fn witness_from_tree(update: TreeUpdate, step: u64, pixels_per_point: f32) -> io::Result<Witness> {
    let viewport = viewport_from_root(&update, pixels_per_point).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "egui inspection tree root has no usable bounds; refusing to guess viewport geometry",
        )
    })?;

    let mut witness =
        witness_from_egui_tree_update(&update, EguiCaptureContext::new(viewport).with_frame(step));
    witness
        .capture
        .metadata
        .insert("transport".into(), json!("egui_inspection"));
    witness.capture.metadata.insert(
        "inspection_protocol_version".into(),
        json!(PROTOCOL_VERSION),
    );
    witness
        .capture
        .metadata
        .insert("inspection_step".into(), json!(step));
    Ok(witness)
}

fn viewport_from_root(update: &TreeUpdate, pixels_per_point: f32) -> Option<Viewport> {
    if !pixels_per_point.is_finite() || pixels_per_point <= 0.0 {
        return None;
    }

    let root_id = update.tree.as_ref()?.root;
    let root = update
        .nodes
        .iter()
        .find_map(|(id, node)| (*id == root_id).then_some(node))?;
    let bounds = root.bounds()?;
    let width = (bounds.x1 - bounds.x0) as f32;
    let height = (bounds.y1 - bounds.y0) as f32;

    if !width.is_finite() || !height.is_finite() || width < 0.0 || height < 0.0 {
        return None;
    }

    Some(Viewport {
        width,
        height,
        scale_factor: pixels_per_point,
    })
}

fn peer_error(message: String) -> io::Error {
    io::Error::other(format!("egui inspection peer error: {message}"))
}

fn unexpected_response(request: &str, response: Response) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("unexpected egui inspection response to {request}: {response:?}"),
    )
}

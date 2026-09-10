use std::{io, net::TcpStream};

use egui::accesskit::TreeUpdate;
use egui_inspection::{Request, Response, PROTOCOL_VERSION, read_message, write_message};
use serde_json::json;

use crate::{EguiCaptureContext, Viewport, Witness, witness_from_egui_tree_update};

/// Read-only client for the egui inspection protocol.
///
/// The observer lives outside the GUI process. It requests a fresh AccessKit
/// tree and converts that observed frame into the canonical ViewWitness model;
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
            Response::Error { message } => Err(io::Error::other(format!(
                "egui inspection peer error: {message}"
            ))),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected egui inspection response to GetTree: {other:?}"),
            )),
        }
    }
}

fn witness_from_tree(
    update: TreeUpdate,
    step: u64,
    pixels_per_point: f32,
) -> io::Result<Witness> {
    let viewport = viewport_from_root(&update, pixels_per_point).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "egui inspection tree root has no usable bounds; refusing to guess viewport geometry",
        )
    })?;

    let mut witness = witness_from_egui_tree_update(
        &update,
        EguiCaptureContext::new(viewport).with_frame(step),
    );
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

use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};

use serde::{Deserialize, Serialize};

use crate::{EguiPaintObservation, paint_observations_from_egui_output};

/// One renderer-evidence frame copied from egui's public output boundary.
///
/// This remains egui-specific integration data rather than canonical
/// `Witness` state. `pass_nr` is egui's cumulative pass number for the active
/// viewport; `viewport_id` is the raw egui viewport identity and disambiguates
/// independent viewport pass streams without carrying framework wrapper state.
///
/// Serialization exists for worker-side transport only. `output_hook` itself
/// never serializes a frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiPaintFrame {
    pub viewport_id: u64,
    pub pass_nr: u64,
    pub pixels_per_point: f32,
    /// Number of reporter frames dropped since the previous successfully
    /// delivered frame because the bounded consumer queue was full.
    pub dropped_before: u64,
    pub observations: Vec<EguiPaintObservation>,
}

/// Lightweight egui plugin that copies renderer-facing paint metadata into a
/// bounded nonblocking queue.
///
/// The reporter deliberately performs no serialization, persistence, network
/// I/O, diffing, searching, or model work inside [`egui::Plugin::output_hook`].
/// When the consumer is behind, `try_send` drops the current paint frame rather
/// than blocking the GUI thread.
pub struct EguiPaintReporter {
    sender: SyncSender<EguiPaintFrame>,
    dropped_since_delivery: u64,
    connected: bool,
}

impl EguiPaintReporter {
    /// Create a reporter and its off-thread consumer endpoint.
    ///
    /// A requested capacity of zero is promoted to one. ViewWitness does not
    /// use a rendezvous channel here because that would make successful
    /// delivery depend on a consumer waiting at exactly the right instant.
    #[must_use]
    pub fn channel(capacity: usize) -> (Self, Receiver<EguiPaintFrame>) {
        let (sender, receiver) = sync_channel(capacity.max(1));
        (
            Self {
                sender,
                dropped_since_delivery: 0,
                connected: true,
            },
            receiver,
        )
    }
}

impl egui::Plugin for EguiPaintReporter {
    fn debug_name(&self) -> &'static str {
        "ViewWitness paint reporter"
    }

    fn output_hook(&mut self, ctx: &egui::Context, output: &mut egui::FullOutput) {
        if !self.connected {
            return;
        }

        let frame = EguiPaintFrame {
            viewport_id: ctx.viewport_id().0.value(),
            pass_nr: ctx.cumulative_pass_nr(),
            pixels_per_point: output.pixels_per_point,
            dropped_before: self.dropped_since_delivery,
            observations: paint_observations_from_egui_output(output),
        };

        match self.sender.try_send(frame) {
            Ok(()) => self.dropped_since_delivery = 0,
            Err(TrySendError::Full(_)) => {
                self.dropped_since_delivery = self.dropped_since_delivery.saturating_add(1);
            }
            Err(TrySendError::Disconnected(_)) => {
                // Stop even collecting metadata once there is nobody to receive it.
                self.connected = false;
            }
        }
    }
}

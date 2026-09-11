use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError, sync_channel},
    },
    time::{Duration, Instant},
};

use egui::accesskit::TreeUpdate;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::egui_paint_annotation::{
    PendingPaintObjects, clear_pending_paint_objects, new_pending_paint_objects,
    resolve_pending_paint_objects,
};
use crate::{
    EguiAuthoredPaintObject, EguiCaptureContext, EguiPaintAnnotator, EguiPaintObservation, Rect,
    Viewport, Witness, paint_observations_from_egui_output, witness_from_egui_tree_update,
};

const DROP_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Raw egui evidence copied from one exact `FullOutput` in response to an
/// explicit ViewWitness capture request.
///
/// Unlike the independent external inspection and continuous paint channels,
/// `accesskit`, `viewport_rect`, `paint`, and `authored_objects` in this value
/// are known to originate from the same requested egui pass. The type remains
/// egui-specific integration evidence rather than canonical `Witness` state.
#[derive(Debug, Clone)]
pub struct EguiFrameEvidence {
    pub request_id: u64,
    pub viewport_id: u64,
    pub pass_nr: u64,
    pub pixels_per_point: f32,
    pub viewport_rect: Rect,
    pub accesskit: Option<TreeUpdate>,
    pub paint: Vec<EguiPaintObservation>,
    pub authored_objects: Vec<EguiAuthoredPaintObject>,
}

/// Off-thread product of one exact same-`FullOutput` egui capture.
///
/// The semantic half is converted into the canonical `Witness`; the paint and
/// authored-object halves deliberately remain provisional egui evidence.
/// `witness.capture.frame` uses `pass_nr`, and metadata names that clock
/// explicitly so consumers do not confuse it with `egui_inspection`'s
/// unrelated step counter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EguiCorrelatedCapture {
    pub request_id: u64,
    pub viewport_id: u64,
    pub pass_nr: u64,
    pub viewport_rect: Rect,
    pub witness: Witness,
    pub paint: Vec<EguiPaintObservation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authored_objects: Vec<EguiAuthoredPaintObject>,
}

impl EguiFrameEvidence {
    /// Convert raw same-pass evidence into a canonical semantic witness plus
    /// its correlated provisional egui evidence.
    ///
    /// This is worker-side work. Viewport size comes from egui's observed
    /// viewport rectangle copied from the same output-hook invocation; AccessKit
    /// root geometry is not treated as a viewport surrogate. Invalid observed
    /// geometry is an error rather than permission to invent dimensions.
    pub fn into_correlated_capture(self) -> io::Result<EguiCorrelatedCapture> {
        let update = self.accesskit.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "same-pass egui capture produced no AccessKit tree",
            )
        })?;
        let viewport = viewport_from_observed_rect(self.viewport_rect, self.pixels_per_point)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "same-pass egui viewport evidence is invalid; refusing to guess viewport geometry",
                )
            })?;

        let mut witness = witness_from_egui_tree_update(
            &update,
            EguiCaptureContext::new(viewport).with_frame(self.pass_nr),
        );
        witness
            .capture
            .metadata
            .insert("transport".into(), json!("viewwitness_egui_frame_probe"));
        witness
            .capture
            .metadata
            .insert("frame_clock".into(), json!("egui_cumulative_pass_nr"));
        witness.capture.metadata.insert(
            "semantic_paint_correlation".into(),
            json!("same_full_output"),
        );
        witness.capture.metadata.insert(
            "viewport_evidence_source".into(),
            json!("egui_input_state_viewport_rect"),
        );
        witness
            .capture
            .metadata
            .insert("frame_probe_request_id".into(), json!(self.request_id));
        witness
            .capture
            .metadata
            .insert("egui_viewport_id".into(), json!(self.viewport_id));
        witness
            .capture
            .metadata
            .insert("egui_pass_nr".into(), json!(self.pass_nr));
        witness.capture.metadata.insert(
            "egui_viewport_rect".into(),
            json!({
                "x": self.viewport_rect.x,
                "y": self.viewport_rect.y,
                "width": self.viewport_rect.width,
                "height": self.viewport_rect.height,
            }),
        );
        if !self.authored_objects.is_empty() {
            witness.capture.metadata.insert(
                "authored_paint_evidence".into(),
                json!("application_semantics_plus_verified_egui_paint_handles"),
            );
        }

        Ok(EguiCorrelatedCapture {
            request_id: self.request_id,
            viewport_id: self.viewport_id,
            pass_nr: self.pass_nr,
            viewport_rect: self.viewport_rect,
            witness,
            paint: self.paint,
            authored_objects: self.authored_objects,
        })
    }
}

#[derive(Default)]
struct ProbeState {
    requested: AtomicU64,
    dropped: AtomicU64,
}

struct EguiFrameProbePlugin {
    state: Arc<ProbeState>,
    sender: SyncSender<EguiFrameEvidence>,
    handled_request_id: u64,
    connected: bool,
    active_capture_request: Option<u64>,
    active_annotation_request: Arc<AtomicU64>,
    pending_annotations: PendingPaintObjects,
    resolved_annotations: Vec<EguiAuthoredPaintObject>,
}

impl egui::Plugin for EguiFrameProbePlugin {
    fn debug_name(&self) -> &'static str {
        "ViewWitness frame probe"
    }

    fn setup(&mut self, ctx: &egui::Context) {
        // A requested correlated capture needs egui to emit semantic evidence.
        ctx.enable_accesskit();
    }

    fn on_begin_pass(&mut self, _ui: &mut egui::Ui) {
        self.active_capture_request = None;
        self.active_annotation_request.store(0, Ordering::Release);
        self.resolved_annotations.clear();

        if !self.connected {
            return;
        }

        let request_id = self.state.requested.load(Ordering::Acquire);
        if request_id == 0 || request_id <= self.handled_request_id {
            return;
        }

        // Capture eligibility is fixed at pass start. A request arriving halfway
        // through a pass waits for the next repaint so explicit authored paint
        // annotations cannot describe only a suffix of the captured frame.
        clear_pending_paint_objects(&self.pending_annotations);
        self.active_capture_request = Some(request_id);
        self.active_annotation_request
            .store(request_id, Ordering::Release);
    }

    fn on_end_pass(&mut self, ui: &mut egui::Ui) {
        if self.active_capture_request.is_some() {
            self.resolved_annotations =
                resolve_pending_paint_objects(ui.ctx(), &self.pending_annotations);
        }
        self.active_annotation_request.store(0, Ordering::Release);
    }

    fn output_hook(&mut self, ctx: &egui::Context, output: &mut egui::FullOutput) {
        if !self.connected {
            return;
        }

        let Some(request_id) = self.active_capture_request.take() else {
            return;
        };

        // Mark this request handled before copying. A full response queue must
        // never cause an implicit retry/repaint loop on later GUI passes.
        self.handled_request_id = request_id;

        let viewport_rect = ctx.input(|input| input.viewport_rect());
        let evidence = EguiFrameEvidence {
            request_id,
            viewport_id: ctx.viewport_id().0.value(),
            pass_nr: ctx.cumulative_pass_nr(),
            pixels_per_point: output.pixels_per_point,
            viewport_rect: Rect {
                x: viewport_rect.min.x,
                y: viewport_rect.min.y,
                width: viewport_rect.width(),
                height: viewport_rect.height(),
            },
            accesskit: output.platform_output.accesskit_update.clone(),
            paint: paint_observations_from_egui_output(output),
            authored_objects: std::mem::take(&mut self.resolved_annotations),
        };

        match self.sender.try_send(evidence) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                // Surface loss to the waiting worker without blocking or
                // retrying on the GUI thread.
                self.state.dropped.store(request_id, Ordering::Release);
            }
            Err(TrySendError::Disconnected(_)) => {
                self.connected = false;
            }
        }
    }
}

/// Handle for requesting exact same-pass semantic + paint evidence from egui.
///
/// Install this once for an `egui::Context`, then use the handle from worker
/// code. A request only wakes egui; all waiting/conversion/serialization remains
/// outside the GUI hook. Only one request may be pending through this handle at
/// a time.
pub struct EguiFrameProbe {
    ctx: egui::Context,
    state: Arc<ProbeState>,
    receiver: Receiver<EguiFrameEvidence>,
    next_request_id: u64,
    pending_request_id: Option<u64>,
    active_annotation_request: Arc<AtomicU64>,
    pending_annotations: PendingPaintObjects,
}

impl EguiFrameProbe {
    /// Install the on-demand probe plugin and return its worker-side handle.
    ///
    /// A requested response capacity of zero is promoted to one so delivery
    /// never depends on a receiver rendezvous at the exact output-hook instant.
    #[must_use]
    pub fn install(ctx: &egui::Context, response_capacity: usize) -> Self {
        let state = Arc::new(ProbeState::default());
        let (sender, receiver) = sync_channel(response_capacity.max(1));
        let active_annotation_request = Arc::new(AtomicU64::new(0));
        let pending_annotations = new_pending_paint_objects();

        ctx.add_plugin(EguiFrameProbePlugin {
            state: Arc::clone(&state),
            sender,
            handled_request_id: 0,
            connected: true,
            active_capture_request: None,
            active_annotation_request: Arc::clone(&active_annotation_request),
            pending_annotations: Arc::clone(&pending_annotations),
            resolved_annotations: Vec::new(),
        });

        Self {
            ctx: ctx.clone(),
            state,
            receiver,
            next_request_id: 0,
            pending_request_id: None,
            active_annotation_request,
            pending_annotations,
        }
    }

    /// Return a lightweight application-side annotator tied to this exact probe.
    ///
    /// The annotator may be cloned and kept by UI code after the worker-side
    /// probe itself is moved into a capture server thread.
    #[must_use]
    pub fn annotator(&self) -> EguiPaintAnnotator {
        EguiPaintAnnotator::new(
            Arc::clone(&self.active_annotation_request),
            Arc::clone(&self.pending_annotations),
        )
    }

    /// Request one correlated capture and wake an idle egui integration.
    ///
    /// This method does not wait for or process GUI output. Call
    /// [`Self::recv_timeout`] later, normally from worker code after egui has
    /// produced another pass.
    pub fn request_capture(&mut self) -> io::Result<u64> {
        if self.pending_request_id.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "an egui frame capture request is already pending",
            ));
        }

        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or_else(|| io::Error::other("egui frame probe request id exhausted"))?;
        let request_id = self.next_request_id;
        self.pending_request_id = Some(request_id);
        self.state.requested.store(request_id, Ordering::Release);
        self.ctx.request_repaint();
        Ok(request_id)
    }

    /// Wait for the currently pending capture request.
    ///
    /// Stale responses from requests that previously timed out are discarded.
    /// If the GUI hook had to drop the current response because the bounded
    /// response queue was full, this returns `WouldBlock` rather than hiding
    /// the loss or retrying work on the render thread.
    pub fn recv_timeout(&mut self, timeout: Duration) -> io::Result<EguiFrameEvidence> {
        let request_id = self.pending_request_id.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "no egui frame capture request is pending",
            )
        })?;
        let deadline = Instant::now() + timeout;

        loop {
            if self.state.dropped.load(Ordering::Acquire) == request_id {
                self.pending_request_id = None;
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    format!(
                        "egui frame capture request {request_id} was dropped because the bounded response queue was full"
                    ),
                ));
            }

            let now = Instant::now();
            if now >= deadline {
                self.pending_request_id = None;
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("egui frame capture request {request_id} timed out"),
                ));
            }
            let remaining = deadline.saturating_duration_since(now);
            let wait = remaining.min(DROP_POLL_INTERVAL);

            match self.receiver.recv_timeout(wait) {
                Ok(evidence) if evidence.request_id == request_id => {
                    self.pending_request_id = None;
                    return Ok(evidence);
                }
                Ok(evidence) if evidence.request_id < request_id => {
                    // A response can arrive after its caller timed out. It is
                    // stale evidence for this request and must not be relabeled.
                }
                Ok(evidence) => {
                    self.pending_request_id = None;
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "egui frame probe received future request {} while waiting for {request_id}",
                            evidence.request_id
                        ),
                    ));
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    self.pending_request_id = None;
                    return Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "egui frame probe response channel disconnected",
                    ));
                }
            }
        }
    }

    /// Request and wait for one exact same-pass semantic + paint capture.
    ///
    /// The application must continue servicing egui while this blocks. In a
    /// native application this convenience method therefore belongs on a worker
    /// thread, never the GUI/render thread.
    pub fn capture_timeout(&mut self, timeout: Duration) -> io::Result<EguiFrameEvidence> {
        self.request_capture()?;
        self.recv_timeout(timeout)
    }
}

fn viewport_from_observed_rect(rect: Rect, pixels_per_point: f32) -> Option<Viewport> {
    if !pixels_per_point.is_finite()
        || pixels_per_point <= 0.0
        || !rect.x.is_finite()
        || !rect.y.is_finite()
        || !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.width < 0.0
        || rect.height < 0.0
    {
        return None;
    }

    Some(Viewport {
        width: rect.width,
        height: rect.height,
        scale_factor: pixels_per_point,
    })
}

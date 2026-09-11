#![cfg(feature = "egui")]

use std::{io::Write, net::TcpListener, thread};

use viewwitness::{
    EguiPaintKind, EguiPaintObserver, EguiPaintReporter, Rect, run_egui_paint_server,
};

#[test]
fn paint_side_channel_replays_latest_then_streams_new_frames() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind paint side channel");
    let addr = listener.local_addr().expect("paint side-channel address");
    let (reporter, receiver) = EguiPaintReporter::channel(4);
    let server = thread::spawn(move || run_egui_paint_server(listener, receiver));

    let ctx = egui::Context::default();
    ctx.add_plugin(reporter);

    run_painted_pass(&ctx, 10.0);

    let mut observer =
        EguiPaintObserver::connect(&addr.to_string()).expect("connect paint observer");
    let first = observer
        .next_frame()
        .expect("latest frame is replayed after connect");
    assert!(has_rect(&first, 10.0));
    assert_eq!(first.dropped_before, 0);

    run_painted_pass(&ctx, 30.0);
    let second = observer.next_frame().expect("new live frame arrives");
    assert!(has_rect(&second, 30.0));
    assert!(second.pass_nr > first.pass_nr);
    assert_eq!(second.viewport_id, first.viewport_id);

    drop(observer);
    drop(ctx);
    server
        .join()
        .expect("paint server worker does not panic")
        .expect("paint server exits cleanly when reporter disconnects");
}

#[test]
fn paint_observer_rejects_an_unrecognized_protocol() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock paint peer");
    let addr = listener.local_addr().expect("mock peer address");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept observer");
        stream
            .write_all(b"SOMETHING-ELSE 1\n")
            .expect("write wrong handshake");
    });

    let error = EguiPaintObserver::connect(&addr.to_string())
        .err()
        .expect("wrong protocol must be rejected");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

    server.join().expect("mock peer exits cleanly");
}

fn has_rect(frame: &viewwitness::EguiPaintFrame, x: f32) -> bool {
    frame.observations.iter().any(|observation| {
        observation.kind == EguiPaintKind::Rect
            && observation.bounds
                == Rect {
                    x,
                    y: 20.0,
                    width: 40.0,
                    height: 30.0,
                }
    })
}

fn run_painted_pass(ctx: &egui::Context, x: f32) {
    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 120.0),
            )),
            ..Default::default()
        },
        |ui| {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(egui::pos2(x, 20.0), egui::vec2(40.0, 30.0)),
                0.0,
                egui::Color32::WHITE,
            );
        },
    );
    output.drop_without_applying_deltas();
}

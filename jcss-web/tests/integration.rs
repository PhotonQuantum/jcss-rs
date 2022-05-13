use std::io::Write;
use std::net::{SocketAddr, UdpSocket};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use assert_cmd::cargo::CommandCargoExt;
use attohttpc::{MultipartBuilder, MultipartFile, StatusCode};
use serde_json::{json, Value};
use tempfile::NamedTempFile;

const MODEL: &[u8] = include_bytes!("../../model.onnx");

fn get_bind_addr() -> SocketAddr {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("to bind");
    socket.local_addr().expect("to acquire addr")
}

#[test]
fn tests() {
    let (stop_tx, stop_rx) = channel();
    let (ready_tx, ready_rx) = channel();
    let bind_addr = get_bind_addr();

    let handle = thread::spawn(move || {
        let mut model = NamedTempFile::new().expect("to be created");
        model.write_all(MODEL).expect("to write");

        let mut cmd = Command::cargo_bin("jcss-web").expect("to build");
        let mut child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .env("APP_BIND", bind_addr.to_string())
            .env("APP_MODEL", model.path())
            .spawn()
            .expect("to spawn");

        while attohttpc::get(format!("http://{}", bind_addr))
            .send()
            .is_err()
        {
            thread::sleep(Duration::from_millis(100));
        }
        ready_tx.send(()).expect("to send");
        stop_rx.recv().expect("to receive");
        child.kill().expect("to finish");
    });

    ready_rx.recv().expect("server to be ready");

    must_recognize(bind_addr, include_bytes!("../../captcha.jpg"), "gbmke");
    must_recognize(bind_addr, include_bytes!("../../captcha.jpeg"), "tbrxm");
    must_reject_missing_image(bind_addr);
    must_reject_invalid_image(bind_addr);

    stop_tx.send(()).expect("to stop server");
    handle.join().expect("to join");
}

fn must_recognize(bind_addr: SocketAddr, image: &[u8], expected: &str) {
    let payload = MultipartBuilder::new()
        .with_file(MultipartFile::new("image", image))
        .build()
        .expect("to construct");

    let resp = attohttpc::post(format!("http://{}", bind_addr))
        .body(payload)
        .send()
        .expect("to receive response");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().expect("to deserialize");
    assert_eq!(
        body.pointer("/status").expect("to_present"),
        &json!("success")
    );
    assert_eq!(
        body.pointer("/data/prediction").expect("to_present"),
        &json!(expected)
    );
    body.pointer("/data/elapsed_time").expect("to_present");
}

fn must_reject_missing_image(bind_addr: SocketAddr) {
    let payload = MultipartBuilder::new()
        .with_file(MultipartFile::new(
            "boom",
            include_bytes!("../../captcha.jpg"),
        ))
        .build()
        .expect("to construct");

    let resp = attohttpc::post(format!("http://{}", bind_addr))
        .body(payload)
        .send()
        .expect("to receive response");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: Value = resp.json().expect("to deserialize");
    assert_eq!(
        body,
        json!({"status": "fail", "data": {"image": "A captcha image is required."}})
    );
}

fn must_reject_invalid_image(bind_addr: SocketAddr) {
    let payload = MultipartBuilder::new()
        .with_file(MultipartFile::new("image", MODEL))
        .build()
        .expect("to construct");

    let resp = attohttpc::post(format!("http://{}", bind_addr))
        .body(payload)
        .send()
        .expect("to receive response");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: Value = resp.json().expect("to deserialize");
    assert_eq!(
        body,
        json!({"status": "fail", "data": {"image": "Invalid image format."}})
    );
}

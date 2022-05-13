use std::fs;

use assert_cmd::Command;
use rstest::rstest;

#[rstest]
#[case(include_bytes!("../../captcha.jpg"), "gbmke")]
#[case(include_bytes!("../../captcha.jpeg"), "tbrxm")]
fn must_recognize(#[case] image: &[u8], #[case] expected: &str) {
    let test_dir = tempfile::tempdir().expect("to be created");
    let model_path = test_dir.path().join("model.onnx");
    let captcha_path = test_dir.path().join("captcha.jpg");
    fs::write(&model_path, include_bytes!("../../model.onnx")).expect("to write");
    fs::write(&captcha_path, image).expect("to write");

    let mut cmd = Command::cargo_bin("jcss-cli").expect("to build");
    let assert = cmd.arg(captcha_path).arg("-m").arg(model_path).assert();
    assert.success().stdout(format!("{}\n", expected));
}

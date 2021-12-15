use std::fs;

use assert_cmd::Command;

#[test]
fn must_recognize() {
    let test_dir = tempfile::tempdir().expect("to be created");
    let model_path = test_dir.path().join("model.onnx");
    let captcha_path = test_dir.path().join("captcha.jpg");
    fs::write(&model_path, include_bytes!("../../model.onnx")).expect("to write");
    fs::write(&captcha_path, include_bytes!("../../captcha.jpg")).expect("to write");

    let mut cmd = Command::cargo_bin("jcss-cli").expect("to build");
    let assert = cmd.arg(captcha_path).arg("-m").arg(model_path).assert();
    assert.success().stdout("gbmke\n");
}

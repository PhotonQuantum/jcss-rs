use std::io::Cursor;

use jcss::Predictor;

#[test]
fn must_recognize() {
    let predictor =
        Predictor::new(Cursor::new(include_bytes!("../../model.onnx"))).expect("predictor");
    let result = predictor
        .predict(
            image::io::Reader::new(Cursor::new(include_bytes!("../../captcha.jpg")))
                .with_guessed_format()
                .expect("can seek")
                .decode()
                .expect("decode"),
        )
        .expect("predict");
    assert_eq!(result, "gbmke");
}

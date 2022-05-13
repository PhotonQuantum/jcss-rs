use std::io::Cursor;

use rstest::rstest;

use jcss::Predictor;

#[rstest]
#[case(include_bytes!("../../captcha.jpg"), "gbmke")]
#[case(include_bytes!("../../captcha.jpeg"), "tbrxm")]
fn must_recognize(#[case] image: &[u8], #[case] expected: &str) {
    let predictor =
        Predictor::new(Cursor::new(include_bytes!("../../model.onnx"))).expect("predictor");
    let result = predictor
        .predict(
            image::io::Reader::new(Cursor::new(image))
                .with_guessed_format()
                .expect("can seek")
                .decode()
                .expect("decode"),
        )
        .expect("predict");
    assert_eq!(result, expected);
}

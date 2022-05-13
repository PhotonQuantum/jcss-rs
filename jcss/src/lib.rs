use std::fmt::{Display, Formatter};
use std::io::Read;

use image::DynamicImage;
use image::imageops::FilterType;
use ndarray::{Array4, Axis};
use nshare::ToNdarray2;
use tracing::debug;
use tract_onnx::prelude::tract_itertools::Itertools;
use tract_onnx::prelude::{
    tvec, Datum, Framework, InferenceFact, InferenceModelExt, RunnableModel, Tensor, TractResult,
    TypedFact, TypedModel, TypedOp,
};

pub struct Predictor {
    model: RunnableModel<TypedFact, Box<dyn TypedOp>, TypedModel>,
}

enum Item {
    Char(char),
    Nothing,
}

impl From<usize> for Item {
    fn from(u: usize) -> Self {
        if u < 26 {
            Self::Char(
                char::try_from(u32::from('a') + u32::try_from(u).expect("no truncate value"))
                    .expect("no truncate value"),
            )
        } else {
            Self::Nothing
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Char(c) => c.fmt(f),
            Item::Nothing => Ok(()),
        }
    }
}

impl Predictor {
    pub fn new(mut model: impl Read) -> TractResult<Self> {
        debug!("loading model");
        let model = tract_onnx::onnx()
            .model_for_read(&mut model)?
            .with_input_fact(
                0,
                // TODO can this be u8?
                InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 1, 40, 100)),
            )?
            .into_optimized()?
            .into_runnable()?;

        debug!("model loaded");
        Ok(Self { model })
    }
    pub fn predict(&self, input: DynamicImage) -> TractResult<String> {
        debug!("preprocessing image");
        let resized = input.resize_exact(100, 40, FilterType::Lanczos3);
        let grayscale = resized.into_luma8();
        let binarized = grayscale.into_ndarray2();
        let normalized: Array4<f32> = binarized
            .map(|pixel| if *pixel <= 156 { 0. } else { 1. })
            .insert_axis(Axis(0))
            .insert_axis(Axis(0));
        let tensor = Tensor::from(normalized);

        debug!("running inference");
        let model_output = self.model.run(tvec!(tensor))?;

        debug!("transforming output");
        Ok(model_output
            .into_iter()
            .map(|tensor| {
                tensor
                    .to_array_view::<f32>()
                    .expect("access tensor as view")
                    .into_iter()
                    .position_max_by(|x, y| x.partial_cmp(y).expect("normal floats"))
                    .expect("tensor is not empty")
            })
            .map(Item::from)
            .join(""))
    }
}

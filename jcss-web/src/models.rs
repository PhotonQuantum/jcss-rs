use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Fail,
    Error,
}

#[derive(Serialize)]
pub struct Response<T> {
    status: Status,
    #[serde(flatten)]
    data: T,
}

impl<T> Response<T> {
    pub const fn new(status: Status, data: T) -> Self {
        Self { status, data }
    }
}

#[derive(Serialize)]
pub struct Data<T> {
    data: T,
}

impl<T> Data<T> {
    pub const fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Serialize)]
pub struct Message<T> {
    message: T,
}

impl<T> Message<T> {
    pub const fn new(message: T) -> Self {
        Self { message }
    }
}

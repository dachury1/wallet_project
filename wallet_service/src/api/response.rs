use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: &'static str,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            status: "success",
            data,
        }
    }
}

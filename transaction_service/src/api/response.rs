use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub status: &'static str,
    #[schema(value_type = Object)] // Mappeamos T genérico a un Object genérico por defecto
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

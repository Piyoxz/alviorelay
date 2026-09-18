use crate::handler::{
    whip_delete_handler, whip_options_handler, whip_patch_handler, whip_publish_handler, WhipState,
};
use axum::{
    routing::{options, post},
    Router,
};

/// Creates the Axum Router mounting all WHIP Ingress endpoints.
pub fn create_whip_router(state: WhipState) -> Router {
    Router::new()
        .route("/{room_id}", post(whip_publish_handler).options(whip_options_handler))
        .route(
            "/resource/{resource_id}",
            options(whip_options_handler)
                .delete(whip_delete_handler)
                .patch(whip_patch_handler),
        )
        .with_state(state)
}

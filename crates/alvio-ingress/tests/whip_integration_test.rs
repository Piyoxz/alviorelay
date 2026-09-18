use alvio_ingress::{create_whip_router, WhipRegistry, WhipState};
use alvio_signal::RoomRegistry;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use std::net::SocketAddr;
use std::time::Instant;
use tower::ServiceExt;

fn create_valid_client_sdp_offer() -> String {
    let mut client_rtc = str0m::Rtc::builder().build(Instant::now());
    let mut change = client_rtc.sdp_api();
    let _ = change.add_channel("whip_meta".into());
    let (offer, _pending) = change.apply().expect("Failed to generate test SDP offer");
    offer.to_sdp_string()
}

#[tokio::test]
async fn test_whip_options_cors_preflight() {
    let whip_reg = WhipRegistry::new();
    let room_reg = RoomRegistry::new(10);
    let bind_addr: SocketAddr = "127.0.0.1:40000".parse().unwrap();
    let state = WhipState::new(whip_reg, room_reg, bind_addr, None);
    let router = create_whip_router(state);

    let req = Request::builder()
        .method("OPTIONS")
        .uri("/room-gaming-1")
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let headers = resp.headers();
    assert_eq!(
        headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
        "*"
    );
    assert!(headers
        .get(header::ALLOW)
        .unwrap()
        .to_str()
        .unwrap()
        .contains("POST"));
    assert_eq!(headers.get("accept-post").unwrap(), "application/sdp");
}

#[tokio::test]
async fn test_whip_unsupported_content_type() {
    let whip_reg = WhipRegistry::new();
    let room_reg = RoomRegistry::new(10);
    let bind_addr: SocketAddr = "127.0.0.1:40001".parse().unwrap();
    let state = WhipState::new(whip_reg, room_reg, bind_addr, None);
    let router = create_whip_router(state);

    let req = Request::builder()
        .method("POST")
        .uri("/room-gaming-2")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"not": "sdp"}"#))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn test_whip_empty_sdp_returns_bad_request() {
    let whip_reg = WhipRegistry::new();
    let room_reg = RoomRegistry::new(10);
    let bind_addr: SocketAddr = "127.0.0.1:40002".parse().unwrap();
    let state = WhipState::new(whip_reg, room_reg, bind_addr, None);
    let router = create_whip_router(state);

    let req = Request::builder()
        .method("POST")
        .uri("/room-gaming-3")
        .header(header::CONTENT_TYPE, "application/sdp")
        .body(Body::from("   "))
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_whip_authentication_verification() {
    let whip_reg = WhipRegistry::new();
    let room_reg = RoomRegistry::new(10);
    let bind_addr: SocketAddr = "127.0.0.1:40003".parse().unwrap();
    let state = WhipState::new(
        whip_reg,
        room_reg,
        bind_addr,
        Some("secret_stream_key_999".into()),
    );
    let router = create_whip_router(state.clone());

    let sdp_offer = create_valid_client_sdp_offer();

    // 1. Missing Authorization header -> 401
    let req_unauth = Request::builder()
        .method("POST")
        .uri("/live-stream-auth")
        .header(header::CONTENT_TYPE, "application/sdp")
        .body(Body::from(sdp_offer.clone()))
        .unwrap();

    let resp_unauth = router.clone().oneshot(req_unauth).await.unwrap();
    assert_eq!(resp_unauth.status(), StatusCode::UNAUTHORIZED);

    // 2. Correct Authorization header -> 201 Created
    let req_auth = Request::builder()
        .method("POST")
        .uri("/live-stream-auth")
        .header(header::CONTENT_TYPE, "application/sdp")
        .header(header::AUTHORIZATION, "Bearer secret_stream_key_999")
        .body(Body::from(sdp_offer))
        .unwrap();

    let resp_auth = router.oneshot(req_auth).await.unwrap();
    assert_eq!(resp_auth.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_whip_full_lifecycle() {
    let whip_reg = WhipRegistry::new();
    let room_reg = RoomRegistry::new(10);
    let bind_addr: SocketAddr = "127.0.0.1:40004".parse().unwrap();
    let state = WhipState::new(whip_reg.clone(), room_reg, bind_addr, None);
    let router = create_whip_router(state);

    let client_sdp = create_valid_client_sdp_offer();

    // 1. Ingest SDP Offer via POST
    let post_req = Request::builder()
        .method("POST")
        .uri("/obs-broadcast-room")
        .header(header::CONTENT_TYPE, "application/sdp")
        .body(Body::from(client_sdp))
        .unwrap();

    let post_resp = router.clone().oneshot(post_req).await.unwrap();
    assert_eq!(post_resp.status(), StatusCode::CREATED);

    let location = post_resp
        .headers()
        .get(header::LOCATION)
        .expect("Location header must be present")
        .to_str()
        .unwrap()
        .to_string();

    assert!(location.starts_with("/whip/resource/whip_res_"));
    assert!(post_resp.headers().contains_key(header::LINK));
    assert!(post_resp.headers().contains_key(header::ETAG));

    // Read SDP Answer body
    let body_bytes = post_resp.into_body().collect().await.unwrap().to_bytes();
    let sdp_answer = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(sdp_answer.contains("v=0"), "SDP answer must contain v=0");

    assert_eq!(whip_reg.active_count(), 1);

    // Extract resource_id from Location path: /whip/resource/{resource_id}
    let resource_id = location.strip_prefix("/whip/resource/").unwrap();
    let resource_uri = format!("/resource/{}", resource_id);

    // 2. Send Trickle ICE Candidate via PATCH
    let patch_req = Request::builder()
        .method("PATCH")
        .uri(&resource_uri)
        .header(header::CONTENT_TYPE, "application/trickle-ice-sdpfrag")
        .body(Body::from(
            "candidate:1 1 UDP 2130706431 192.168.1.100 5004 typ host",
        ))
        .unwrap();

    let patch_resp = router.clone().oneshot(patch_req).await.unwrap();
    assert_eq!(patch_resp.status(), StatusCode::NO_CONTENT);

    let session = whip_reg.get(resource_id).unwrap();
    assert_eq!(session.candidate_count(), 1);

    // 3. Terminate publisher stream via DELETE
    let delete_req = Request::builder()
        .method("DELETE")
        .uri(&resource_uri)
        .body(Body::empty())
        .unwrap();

    let delete_resp = router.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);
    assert_eq!(whip_reg.active_count(), 0);

    // 4. Repeated DELETE returns 404 Not Found
    let repeat_delete_req = Request::builder()
        .method("DELETE")
        .uri(&resource_uri)
        .body(Body::empty())
        .unwrap();

    let repeat_resp = router.oneshot(repeat_delete_req).await.unwrap();
    assert_eq!(repeat_resp.status(), StatusCode::NOT_FOUND);
}

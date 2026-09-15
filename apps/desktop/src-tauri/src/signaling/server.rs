use super::{
    messages::{SignalingEnvelope, SignalingRole},
    phone_assets,
    router::SignalingHub,
};
use crate::{
    errors::{IMirrorError, IMirrorResult},
    pairing::{session::SessionManager, token::random_token},
    security::tls::LocalTlsIdentity,
};
use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct SignalingServer {
    port: u16,
    session_manager: Arc<SessionManager>,
    hub: SignalingHub,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignalQuery {
    session_id: String,
    token: String,
    role: SignalingRole,
}

#[derive(Clone)]
struct BootstrapState {
    certificate_der: Vec<u8>,
    secure_url: String,
}

impl SignalingServer {
    pub fn new(port: u16, session_manager: Arc<SessionManager>, hub: SignalingHub) -> Self {
        Self {
            port,
            session_manager,
            hub,
        }
    }

    pub async fn run_secure(self, tls: LocalTlsIdentity) -> IMirrorResult<()> {
        let state = Arc::new(self);
        let app = Router::new()
            .route("/health", get(health))
            .route("/signal", get(signal))
            .fallback(get(phone_assets::serve))
            .layer(CorsLayer::permissive())
            .with_state(state.clone());

        let addr = SocketAddr::from(([0, 0, 0, 0], state.port));
        let config = axum_server::tls_rustls::RustlsConfig::from_pem(tls.cert_pem, tls.key_pem)
            .await
            .map_err(|err| {
                IMirrorError::new(
                    "tls-config-failed",
                    "Could not configure the local HTTPS server.",
                )
                .with_detail(err.to_string())
            })?;

        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .map_err(|err| {
                IMirrorError::new(
                    "signaling-server-failed",
                    "Local HTTPS signaling server stopped unexpectedly.",
                )
                .with_detail(err.to_string())
            })
    }

    pub async fn run_local(mut self, port: u16) -> IMirrorResult<()> {
        self.port = port;
        let state = Arc::new(self);
        let app = Router::new()
            .route("/health", get(health))
            .route("/signal", get(signal))
            .layer(CorsLayer::permissive())
            .with_state(state.clone());
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr).await.map_err(|err| {
            IMirrorError::new(
                "signaling-bind-failed",
                "Could not start desktop signaling server.",
            )
            .with_detail(err.to_string())
        })?;

        axum::serve(listener, app).await.map_err(|err| {
            IMirrorError::new(
                "signaling-server-failed",
                "Desktop signaling server stopped unexpectedly.",
            )
            .with_detail(err.to_string())
        })
    }

    pub async fn run_bootstrap(
        host: String,
        bootstrap_port: u16,
        secure_port: u16,
        certificate_der: Vec<u8>,
    ) -> IMirrorResult<()> {
        let state = Arc::new(BootstrapState {
            certificate_der,
            secure_url: format!("https://{host}:{secure_port}/"),
        });
        let app = Router::new()
            .route("/", get(bootstrap))
            .route("/imirror-local-ca.cer", get(certificate))
            .with_state(state);
        let addr = SocketAddr::from(([0, 0, 0, 0], bootstrap_port));
        let listener = TcpListener::bind(addr).await.map_err(|err| {
            IMirrorError::new(
                "bootstrap-bind-failed",
                "Could not start the iPhone setup page.",
            )
            .with_detail(err.to_string())
        })?;

        axum::serve(listener, app).await.map_err(|err| {
            IMirrorError::new(
                "bootstrap-server-failed",
                "The iPhone setup page stopped unexpectedly.",
            )
            .with_detail(err.to_string())
        })
    }
}

async fn bootstrap(State(state): State<Arc<BootstrapState>>) -> Html<String> {
    const PAGE: &str = r##"<!doctype html>
<html lang="es"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">
<meta name="theme-color" content="#08152e"><title>Configurar iMirror</title>
<style>body{margin:0;background:#061020;color:#eaf8ff;font:16px system-ui,sans-serif;min-height:100vh;display:grid;place-items:center}.card{width:min(92vw,560px);padding:28px;border:1px solid #1a4568;border-radius:22px;background:#0a1930}h1{margin:0 0 8px;font-size:30px}p,li{line-height:1.55;color:#b9d2e8}.actions{display:grid;gap:12px;margin:24px 0}a{display:block;padding:15px;border-radius:12px;text-align:center;font-weight:750;text-decoration:none}.primary{background:#16d9f4;color:#04101d}.secondary{border:1px solid #32739e;color:#eaf8ff}.note{font-size:13px;color:#85a9c4}</style></head>
<body><main class="card"><h1>iMirror</h1><p>Safari necesita una conexión HTTPS confiable para usar la cámara. Esta configuración se hace una sola vez para esta red.</p>
<ol><li>Descarga el certificado local.</li><li>Abre Ajustes → General → VPN y gestión de dispositivos e instálalo.</li><li>En Ajustes → General → Información → Ajustes de confianza de certificados, activa <strong>iMirror Local Camera</strong>.</li><li>Regresa aquí y abre la cámara segura.</li></ol>
<div class="actions"><a class="secondary" href="/imirror-local-ca.cer">1. Descargar certificado</a><a class="primary" id="open" href="#">2. Abrir cámara segura</a></div>
<p class="note">El certificado y el vídeo permanecen en tu red local. iMirror no sube imágenes a Internet.</p></main>
<script>document.getElementById('open').href='__SECURE_URL__'+location.search;</script></body></html>"##;
    Html(PAGE.replace("__SECURE_URL__", &state.secure_url))
}

async fn certificate(State(state): State<Arc<BootstrapState>>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-x509-ca-cert")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=imirror-local-ca.cer",
        )
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(state.certificate_der.clone()))
        .expect("valid certificate response")
}

async fn health(State(state): State<Arc<SignalingServer>>) -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "name": "iMirror Signaling",
        "port": state.port
    }))
}

async fn signal(
    ws: WebSocketUpgrade,
    State(state): State<Arc<SignalingServer>>,
    Query(query): Query<SignalQuery>,
) -> impl IntoResponse {
    if !state
        .session_manager
        .validate(&query.session_id, &query.token)
    {
        return (
            StatusCode::UNAUTHORIZED,
            "Invalid or expired iMirror pairing session.",
        )
            .into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state, query.role, query.session_id))
        .into_response()
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<SignalingServer>,
    role: SignalingRole,
    session_id: String,
) {
    let client_id = random_token(16);
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.hub.subscribe();

    let outbound_client_id = client_id.clone();
    let outbound_session_id = session_id.clone();
    let outbound = tokio::spawn(async move {
        while let Ok(envelope) = rx.recv().await {
            if envelope.client_id.as_deref() == Some(outbound_client_id.as_str()) {
                continue;
            }
            if envelope_session_id(&envelope) != Some(outbound_session_id.as_str()) {
                continue;
            }
            if envelope.to.is_some_and(|target| target != role) {
                continue;
            }

            match serde_json::to_string(&envelope) {
                Ok(payload) => {
                    if sender.send(Message::Text(payload)).await.is_err() {
                        break;
                    }
                }
                Err(error) => {
                    eprintln!("Could not serialize signaling envelope: {error}");
                }
            }
        }
    });

    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => match serde_json::from_str::<SignalingEnvelope>(&text) {
                Ok(mut envelope) => {
                    if envelope_session_id(&envelope) != Some(session_id.as_str()) {
                        eprintln!("Dropped signaling envelope with mismatched session id.");
                        continue;
                    }
                    if let Some(ack) =
                        hello_ack_envelope(&state, &envelope, role, session_id.as_str())
                    {
                        let accepted = ack
                            .message
                            .get("accepted")
                            .and_then(|value| value.as_bool())
                            .unwrap_or(false);
                        state.hub.broadcast(ack);
                        if !accepted {
                            continue;
                        }
                    }

                    envelope.from = role;
                    envelope.client_id = Some(client_id.clone());
                    state.hub.broadcast(envelope);
                }
                Err(error) => {
                    eprintln!("Invalid signaling envelope: {error}");
                }
            },
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(error) => {
                eprintln!("Signaling socket error: {error}");
                break;
            }
        }
    }

    outbound.abort();
}

fn envelope_session_id(envelope: &SignalingEnvelope) -> Option<&str> {
    envelope
        .message
        .get("sessionId")
        .and_then(|value| value.as_str())
}

fn hello_ack_envelope(
    state: &SignalingServer,
    envelope: &SignalingEnvelope,
    role: SignalingRole,
    session_id: &str,
) -> Option<SignalingEnvelope> {
    if envelope
        .message
        .get("type")
        .and_then(|value| value.as_str())
        != Some("hello")
    {
        return None;
    }

    let token = envelope
        .message
        .get("token")
        .and_then(|value| value.as_str());
    let message_role = envelope
        .message
        .get("role")
        .and_then(|value| value.as_str());
    let accepted = token.is_some_and(|token| state.session_manager.validate(session_id, token))
        && message_role == Some(role_name(role));

    Some(SignalingEnvelope {
        from: opposite_role(role),
        to: Some(role),
        message: if accepted {
            json!({
                "type": "hello-ack",
                "sessionId": session_id,
                "accepted": true
            })
        } else {
            json!({
                "type": "hello-ack",
                "sessionId": session_id,
                "accepted": false,
                "reason": "iMirror hello did not match the authenticated pairing session."
            })
        },
        sent_at: chrono::Utc::now().to_rfc3339(),
        client_id: None,
    })
}

fn opposite_role(role: SignalingRole) -> SignalingRole {
    match role {
        SignalingRole::Desktop => SignalingRole::Phone,
        SignalingRole::Phone => SignalingRole::Desktop,
    }
}

fn role_name(role: SignalingRole) -> &'static str {
    match role {
        SignalingRole::Desktop => "desktop",
        SignalingRole::Phone => "phone",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_session_id_from_signaling_message() {
        let envelope = SignalingEnvelope {
            from: SignalingRole::Phone,
            to: Some(SignalingRole::Desktop),
            message: json!({ "sessionId": "session-123", "type": "offer" }),
            sent_at: "2026-06-13T00:00:00.000Z".to_string(),
            client_id: None,
        };

        assert_eq!(envelope_session_id(&envelope), Some("session-123"));
    }

    #[test]
    fn ignores_messages_without_session_id() {
        let envelope = SignalingEnvelope {
            from: SignalingRole::Phone,
            to: Some(SignalingRole::Desktop),
            message: json!({ "type": "candidate" }),
            sent_at: "2026-06-13T00:00:00.000Z".to_string(),
            client_id: None,
        };

        assert_eq!(envelope_session_id(&envelope), None);
    }

    #[test]
    fn accepts_valid_hello_message() {
        let session_manager = Arc::new(SessionManager::new(
            "dev".into(),
            "127.0.0.1".into(),
            48173,
            48174,
            48172,
        ));
        let payload = session_manager.current_payload();
        let server = SignalingServer::new(48173, session_manager, SignalingHub::new());
        let envelope = SignalingEnvelope {
            from: SignalingRole::Phone,
            to: Some(SignalingRole::Desktop),
            message: json!({
                "type": "hello",
                "role": "phone",
                "sessionId": payload.session_id,
                "token": payload.token
            }),
            sent_at: "2026-06-13T00:00:00.000Z".to_string(),
            client_id: None,
        };

        let ack = hello_ack_envelope(
            &server,
            &envelope,
            SignalingRole::Phone,
            &payload.session_id,
        )
        .expect("hello should produce ack");

        assert_eq!(ack.to, Some(SignalingRole::Phone));
        assert_eq!(
            ack.message
                .get("accepted")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn rejects_hello_with_wrong_token() {
        let session_manager = Arc::new(SessionManager::new(
            "dev".into(),
            "127.0.0.1".into(),
            48173,
            48174,
            48172,
        ));
        let payload = session_manager.current_payload();
        let server = SignalingServer::new(48173, session_manager, SignalingHub::new());
        let envelope = SignalingEnvelope {
            from: SignalingRole::Phone,
            to: Some(SignalingRole::Desktop),
            message: json!({
                "type": "hello",
                "role": "phone",
                "sessionId": payload.session_id,
                "token": "wrong"
            }),
            sent_at: "2026-06-13T00:00:00.000Z".to_string(),
            client_id: None,
        };

        let ack = hello_ack_envelope(
            &server,
            &envelope,
            SignalingRole::Phone,
            &payload.session_id,
        )
        .expect("hello should produce ack");

        assert_eq!(
            ack.message
                .get("accepted")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
    }
}

use std::{env, io::Cursor, str::FromStr};

use axum::{Json, Router, extract, http::StatusCode, response::IntoResponse, routing, serve};
use axum_extra::{TypedHeader, headers::ContentType};
use futures::future::try_join_all;
use serde::Deserialize;
use std::time::Instant;
use synth_server::{State, setting::Setting};
use tokio::{net::TcpListener, signal};

#[derive(Deserialize)]
struct PreprocessQuery {
    text: String,
}

async fn preprocess(
    extract::State(state): extract::State<State>,
    extract::Json(PreprocessQuery { text }): extract::Json<PreprocessQuery>,
) -> impl IntoResponse {
    let span = tracing::info_span!("preprocess_handler");
    let _entered = span.enter();

    let start = Instant::now();
    let texts = state.splice.splice(&text);
    let response =
        match try_join_all(texts.into_iter().map(|text| state.preprocess.run(text))).await {
            Ok(texts) => Json(texts).into_response(),
            Err(e) => {
                tracing::error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        };
    tracing::info!(elapsed_ms = start.elapsed().as_millis(), "handler finished");
    response
}

fn default_true() -> bool {
    true
}

fn default_one() -> f64 {
    1.0
}

static SPEC: hound::WavSpec = hound::WavSpec {
    channels: 1,
    sample_rate: 48_000,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};

#[derive(Deserialize)]
struct HTSQuery {
    text: String,
    model: String,
    #[serde(default = "default_one")]
    speed: f64,
    #[serde(default = "default_true")]
    preprocess: bool,
}

async fn hts(
    extract::State(state): extract::State<State>,
    extract::Json(HTSQuery {
        text,
        model,
        speed,
        preprocess,
    }): extract::Json<HTSQuery>,
) -> impl IntoResponse {
    let span = tracing::info_span!("hts_handler");
    let _entered = span.enter();
    let start = Instant::now();
    let mut texts = state.splice.splice(&text);
    if preprocess {
        texts = match try_join_all(texts.into_iter().map(|text| state.preprocess.run(text))).await {
            Ok(text) => text,
            Err(e) => {
                tracing::error!("{e:?}");
                tracing::info!(elapsed_ms = start.elapsed().as_millis(), "handler finished");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
    };
    let response = match try_join_all(
        texts
            .into_iter()
            .map(|text| state.cached_run(model.clone(), text, speed)),
    )
    .await
    {
        Ok(wavs) => {
            let wavs = wavs.into_iter().flatten().collect::<Vec<_>>();
            if wavs.is_empty() {
                return StatusCode::NOT_FOUND.into_response();
            }
            let mut buf = vec![];
            match (|| {
                let mut w = hound::WavWriter::new(Cursor::new(&mut buf), SPEC)?;
                for wav in wavs {
                    for b in wav {
                        w.write_sample(b)?
                    }
                }
                w.finalize()?;
                anyhow::Ok((TypedHeader(ContentType::from_str("audio/wav")?), buf))
            })() {
                Ok(r) => r.into_response(),
                Err(e) => {
                    tracing::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    };
    tracing::info!(elapsed_ms = start.elapsed().as_millis(), "handler finished");
    response
}

async fn hts_models(extract::State(state): extract::State<State>) -> impl IntoResponse {
    Json(state.models).into_response()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let setting =
        Setting::load(&env::var("SETTING_PATH").unwrap_or("./data/config.toml".to_string()))
            .await?;
    let state = State::new(
        setting
            .models
            .into_iter()
            .map(|(k, v)| (k, v.path))
            .collect(),
        setting.cache_size,
    )
    .await?;
    let router = Router::new()
        .route(
            "/v1/preprocess",
            routing::post(preprocess).with_state(state.clone()),
        )
        .route("/v1/hts", routing::post(hts).with_state(state.clone()))
        .route("/v1/hts/models", routing::get(hts_models).with_state(state));
    let tcp = TcpListener::bind(&setting.bind_addr).await?;
    tracing::info!("Listening on addr: {}", setting.bind_addr);
    serve(tcp, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
}

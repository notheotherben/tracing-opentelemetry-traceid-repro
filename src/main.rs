use std::{borrow::Cow, collections::HashMap};

use opentelemetry::{trace::TracerProvider, sdk::propagation::TraceContextPropagator, propagation::{TextMapPropagator, Extractor}};
use tracing::info_span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

macro_rules! map {
    () => {
        std::collections::HashMap::new()
    };

    ($($key:expr => $val:expr),+) => {
        {
            let mut m = std::collections::HashMap::new();

            $(
                m.insert($key.into(), $val.into());
            )+

            m
        }
    };
}

fn main() {
    let tracer_provider = opentelemetry::sdk::trace::TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporterBuilder::default().build())
        .build();

    let tracer = tracing_opentelemetry::layer()
        .with_tracer(tracer_provider.versioned_tracer("example", Some("v0.0.0"), Option::<Cow<'static, str>>::None, None));

    tracing_subscriber::registry()
        .with(tracer)
        .init();

    {
        eprintln!("\n\n----- without propagation -----\n\n");

        repro_with_context(None);
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    {
        eprintln!("\n\n----- with propagation (empty) -----\n\n");

        repro_with_context(Some(map![]));
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    {
        eprintln!("\n\n----- with propagation (non-empty) -----\n\n");

        repro_with_context(Some(map![
            "traceparent" => "00-d92b48609ff5c2a7952d45bd6c6a2802-a2b6ccc696a3a41f-01"
        ]));
    }
}

fn repro_with_context(carrier: Option<HashMap<String, String>>)
{
    let root = info_span!("Root Span", carrier = ?carrier);

    if let Some(carrier) = carrier
    {
        let propagator = TraceContextPropagator::new();
        let context = propagator.extract(&HashMapExtractor::from(&carrier));
        root.set_parent(context);
    }

    let _root = root.entered();
    let _child1 = info_span!("Child 1").entered();
}


struct HashMapExtractor<'a> {
    headers: &'a HashMap<String, String>,
}

impl<'a> From<&'a HashMap<String, String>> for HashMapExtractor<'a> {
    fn from(headers: &'a HashMap<String, String>) -> Self {
        HashMapExtractor { headers }
    }
}

impl<'a> Extractor for HashMapExtractor<'a> {
    fn get(&self, key: &str) -> Option<&'a str> {
        self.headers.get(key).map(|v| v.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|v| v.as_str()).collect()
    }
}
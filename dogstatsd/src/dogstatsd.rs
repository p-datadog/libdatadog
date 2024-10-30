// Copyright 2023-Present Datadog, Inc. https://www.datadoghq.com/
// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddr;
use std::str::Split;
use std::sync::{Arc, Mutex};

use tracing::{debug, error};

use crate::aggregator::Aggregator;
use crate::metric::{parse, Metric};

pub struct DogStatsD {
    cancel_token: tokio_util::sync::CancellationToken,
    aggregator: Arc<Mutex<Aggregator>>,
    buffer_reader: BufferReader,
}

pub struct DogStatsDConfig {
    pub host: String,
    pub port: u16,
}

enum BufferReader {
    UdpSocketReader(tokio::net::UdpSocket),
    #[allow(dead_code)]
    MirrorReader(Vec<u8>, SocketAddr),
}

impl BufferReader {
    async fn read(&self) -> std::io::Result<(Vec<u8>, SocketAddr)> {
        match self {
            BufferReader::UdpSocketReader(socket) => {
                // TODO(astuyve) this should be dynamic
                // Max buffer size is configurable in Go Agent and the default is 8KB
                // https://github.com/DataDog/datadog-agent/blob/85939a62b5580b2a15549f6936f257e61c5aa153/pkg/config/config_template.yaml#L2154-L2158
                let mut buf = [0; 8192];
                let (amt, src) = socket
                    .recv_from(&mut buf)
                    .await
                    .expect("didn't receive data");
                Ok((buf[..amt].to_owned(), src))
            }
            BufferReader::MirrorReader(data, socket) => Ok((data.clone(), *socket)),
        }
    }
}

impl DogStatsD {
    #[must_use]
    pub async fn new(
        config: &DogStatsDConfig,
        aggregator: Arc<Mutex<Aggregator>>,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> DogStatsD {
        let addr = format!("{}:{}", config.host, config.port);
        // TODO (UDS socket)
        let socket = tokio::net::UdpSocket::bind(addr)
            .await
            .expect("couldn't bind to address");
        DogStatsD {
            cancel_token,
            aggregator,
            buffer_reader: BufferReader::UdpSocketReader(socket),
        }
    }

    pub async fn spin(self) {
        let mut spin_cancelled = false;
        while !spin_cancelled {
            self.consume_statsd().await;
            spin_cancelled = self.cancel_token.is_cancelled();
        }
    }

    async fn consume_statsd(&self) {
        let (buf, src) = self
            .buffer_reader
            .read()
            .await
            .expect("didn't receive data");
        let msgs = std::str::from_utf8(&buf).expect("couldn't parse as string");
        debug!("Received message: {} from {}", msgs, src);
        let statsd_metric_strings = msgs.split('\n');
        self.insert_metrics(statsd_metric_strings);
    }

    fn insert_metrics(&self, msg: Split<char>) {
        let all_valid_metrics: Vec<Metric> = msg
            .filter(|m| !m.is_empty() && !m.starts_with("_sc|") && !m.starts_with("_e{")) // exclude empty messages, service checks, and events
            .map(|m| m.replace('\n', ""))
            .filter_map(|m| match parse(m.as_str()) {
                Ok(metric) => Some(metric),
                Err(e) => {
                    error!("Failed to parse metric {}: {}", m, e);
                    None
                }
            })
            .collect();
        if !all_valid_metrics.is_empty() {
            let mut guarded_aggregator = self.aggregator.lock().expect("lock poisoned");
            for a_valid_value in all_valid_metrics {
                let _ = guarded_aggregator.insert(a_valid_value);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::aggregator::tests::assert_sketch;
    use crate::aggregator::tests::assert_value;
    use crate::aggregator::Aggregator;
    use crate::dogstatsd::{BufferReader, DogStatsD};
    use crate::metric::EMPTY_TAGS;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::{Arc, Mutex};
    use tracing_test::traced_test;

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_dogstatsd_multi_distribution() {
        let locked_aggregator = setup_dogstatsd(
            "single_machine_performance.rouster.api.series_v2.payload_size_bytes:269942|d
single_machine_performance.rouster.metrics_min_timestamp_latency:1426.90870216|d
single_machine_performance.rouster.metrics_max_timestamp_latency:1376.90870216|d
",
        )
        .await;
        let aggregator = locked_aggregator.lock().expect("lock poisoned");

        let parsed_metrics = aggregator.distributions_to_protobuf();

        assert_eq!(parsed_metrics.sketches.len(), 3);
        assert_eq!(aggregator.to_series().len(), 0);
        drop(aggregator);

        assert_sketch(
            &locked_aggregator,
            "single_machine_performance.rouster.api.series_v2.payload_size_bytes",
            269_942_f64,
        );
        assert_sketch(
            &locked_aggregator,
            "single_machine_performance.rouster.metrics_min_timestamp_latency",
            1_426.908_702_16,
        );
        assert_sketch(
            &locked_aggregator,
            "single_machine_performance.rouster.metrics_max_timestamp_latency",
            1_376.908_702_16,
        );
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_dogstatsd_multi_metric() {
        let locked_aggregator = setup_dogstatsd(
            "metric3:3|c|#tag3:val3,tag4:val4\nmetric1:1|c\nmetric2:2|c|#tag2:val2\n",
        )
        .await;
        let aggregator = locked_aggregator.lock().expect("lock poisoned");

        let parsed_metrics = aggregator.to_series();

        assert_eq!(parsed_metrics.len(), 3);
        assert_eq!(aggregator.distributions_to_protobuf().sketches.len(), 0);
        drop(aggregator);

        assert_value(&locked_aggregator, "metric1", 1.0, "");
        assert_value(&locked_aggregator, "metric2", 2.0, "tag2:val2");
        assert_value(&locked_aggregator, "metric3", 3.0, "tag3:val3,tag4:val4");
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_dogstatsd_single_metric() {
        let locked_aggregator = setup_dogstatsd("metric123:99123|c").await;
        let aggregator = locked_aggregator.lock().expect("lock poisoned");
        let parsed_metrics = aggregator.to_series();

        assert_eq!(parsed_metrics.len(), 1);
        assert_eq!(aggregator.distributions_to_protobuf().sketches.len(), 0);
        drop(aggregator);

        assert_value(&locked_aggregator, "metric123", 99_123.0, "");
    }

    #[tokio::test]
    #[traced_test]
    #[cfg_attr(miri, ignore)]
    async fn test_dogstatsd_filter_service_check() {
        let locked_aggregator = setup_dogstatsd("_sc|servicecheck|0").await;
        let aggregator = locked_aggregator.lock().expect("lock poisoned");
        let parsed_metrics = aggregator.to_series();

        assert!(!logs_contain("Failed to parse metric"));
        assert_eq!(parsed_metrics.len(), 0);
    }

    #[tokio::test]
    #[traced_test]
    #[cfg_attr(miri, ignore)]
    async fn test_dogstatsd_filter_event() {
        let locked_aggregator = setup_dogstatsd("_e{5,10}:event|test event").await;
        let aggregator = locked_aggregator.lock().expect("lock poisoned");
        let parsed_metrics = aggregator.to_series();

        assert!(!logs_contain("Failed to parse metric"));
        assert_eq!(parsed_metrics.len(), 0);
    }

    async fn setup_dogstatsd(statsd_string: &str) -> Arc<Mutex<Aggregator>> {
        let aggregator_arc = Arc::new(Mutex::new(
            Aggregator::new(EMPTY_TAGS, 1_024).expect("aggregator creation failed"),
        ));
        let cancel_token = tokio_util::sync::CancellationToken::new();

        let dogstatsd = DogStatsD {
            cancel_token,
            aggregator: Arc::clone(&aggregator_arc),
            buffer_reader: BufferReader::MirrorReader(
                statsd_string.as_bytes().to_vec(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(111, 112, 113, 114)), 0),
            ),
        };
        dogstatsd.consume_statsd().await;

        aggregator_arc
    }
}

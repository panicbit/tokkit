use std::time::Instant;

/// Collects metrics for token introspection
pub trait MetricsCollector {
    /// An incoming request for token introspection
    fn incoming_introspection_request(&self);
    /// The complete introspection workflow was finished regardless of the
    /// result.
    fn introspection_request(&self, request_started: Instant);
    /// The complete introspection workflow was finished and successful
    fn introspection_request_success(&self, request_started: Instant);
    /// The complete introspection workflow was finished and failed
    fn introspection_request_failure(&self, request_started: Instant);

    /// The token introspections was called regardless of the result.
    fn introspection_service_call(&self, request_started: Instant);
    /// The token introspections was called and the call was a failure.
    fn introspection_service_call_failure(&self, request_started: Instant);
    /// The token introspections was called and the call was a success.
    fn introspection_service_call_success(&self, request_started: Instant);
}

#[derive(Clone)]
pub struct DevNullMetricsCollector;

impl MetricsCollector for DevNullMetricsCollector {
    fn incoming_introspection_request(&self) {}
    fn introspection_request(&self, _request_started: Instant) {}
    fn introspection_request_success(&self, _request_started: Instant) {}
    fn introspection_request_failure(&self, _request_started: Instant) {}

    fn introspection_service_call(&self, _request_started: Instant) {}
    fn introspection_service_call_failure(&self, _request_started: Instant) {}
    fn introspection_service_call_success(&self, _request_started: Instant) {}
}

#[cfg(feature = "metrix")]
pub mod metrix {
    use std::time::Instant;

    use metrix::instruments::*;
    use metrix::processor::*;
    use metrix::TelemetryTransmitter;
    use metrix::TransmitsTelemetryData;
    use metrix::*;

    #[derive(Clone, PartialEq, Eq)]
    enum MetricsIntrospectionRequest {
        IncomingIntrospectionRequest,
        IntrospectionRequest,
        IntrospectionRequestSuccess,
        IntrospectionRequestFailure,
    }

    #[derive(Clone, PartialEq, Eq)]
    enum MetricsIntrospectionService {
        IntrospectionServiceCall,
        IntrospectionServiceCallSuccess,
        IntrospectionServiceCallFailure,
    }

    /// A `MetricsCollector` that works with the [`metrix`](https://crates.io/crates/metrix)
    ///  library
    #[derive(Clone)]
    pub struct MetrixCollector {
        introspection_transmitter: TelemetryTransmitter<MetricsIntrospectionRequest>,
        service_transmitter: TelemetryTransmitter<MetricsIntrospectionService>,
    }

    impl MetrixCollector {
        /// Creates a new collector that
        /// is attached to `add_metrics_to`.
        pub fn new<T>(add_metrics_to: &mut T) -> MetrixCollector
        where
            T: AggregatesProcessors,
        {
            let (introspection_tx, introspection_rx) = create_introspection_metrics();
            let (service_tx, service_rx) = create_introspection_service_metrics();

            add_metrics_to.add_processor(introspection_rx);
            add_metrics_to.add_processor(service_rx);

            MetrixCollector {
                introspection_transmitter: introspection_tx,
                service_transmitter: service_tx,
            }
        }
    }

    impl super::MetricsCollector for MetrixCollector {
        fn incoming_introspection_request(&self) {
            self.introspection_transmitter
                .observed_one_now(MetricsIntrospectionRequest::IncomingIntrospectionRequest);
        }
        fn introspection_request(&self, request_started: Instant) {
            self.introspection_transmitter.measure_time(
                MetricsIntrospectionRequest::IntrospectionRequest,
                request_started,
            );
        }
        fn introspection_request_success(&self, request_started: Instant) {
            self.introspection_transmitter.measure_time(
                MetricsIntrospectionRequest::IntrospectionRequestSuccess,
                request_started,
            );
        }
        fn introspection_request_failure(&self, request_started: Instant) {
            self.introspection_transmitter.measure_time(
                MetricsIntrospectionRequest::IntrospectionRequestFailure,
                request_started,
            );
        }

        fn introspection_service_call(&self, request_started: Instant) {
            self.service_transmitter.measure_time(
                MetricsIntrospectionService::IntrospectionServiceCall,
                request_started,
            );
        }
        fn introspection_service_call_failure(&self, request_started: Instant) {
            self.service_transmitter.measure_time(
                MetricsIntrospectionService::IntrospectionServiceCallFailure,
                request_started,
            );
        }
        fn introspection_service_call_success(&self, request_started: Instant) {
            self.service_transmitter.measure_time(
                MetricsIntrospectionService::IntrospectionServiceCallSuccess,
                request_started,
            );
        }
    }

    fn create_introspection_metrics() -> (
        TelemetryTransmitter<MetricsIntrospectionRequest>,
        TelemetryProcessor<MetricsIntrospectionRequest>,
    ) {
        let mut cockpit: Cockpit<MetricsIntrospectionRequest> = Cockpit::without_name();

        let panel = Panel::named(
            MetricsIntrospectionRequest::IncomingIntrospectionRequest,
            "incoming",
        );
        add_counting_instruments_to_cockpit(&mut cockpit, panel);

        let panel = Panel::named(MetricsIntrospectionRequest::IntrospectionRequest, "all");
        add_counting_and_time_us_instruments_to_cockpit(&mut cockpit, panel);

        let panel = Panel::named(
            MetricsIntrospectionRequest::IntrospectionRequestSuccess,
            "successful",
        );
        add_counting_and_time_us_instruments_to_cockpit(&mut cockpit, panel);

        let panel = Panel::named(
            MetricsIntrospectionRequest::IntrospectionRequestFailure,
            "failed",
        );
        add_counting_and_time_us_instruments_to_cockpit(&mut cockpit, panel);

        let (tx, rx) = TelemetryProcessor::new_pair("introspection");

        tx.add_cockpit(cockpit);

        (tx, rx)
    }

    fn create_introspection_service_metrics() -> (
        TelemetryTransmitter<MetricsIntrospectionService>,
        TelemetryProcessor<MetricsIntrospectionService>,
    ) {
        let mut cockpit: Cockpit<MetricsIntrospectionService> = Cockpit::without_name();

        let panel = Panel::named(MetricsIntrospectionService::IntrospectionServiceCall, "all");
        add_counting_and_time_us_instruments_to_cockpit(&mut cockpit, panel);

        let panel = Panel::named(
            MetricsIntrospectionService::IntrospectionServiceCallSuccess,
            "successful",
        );
        add_counting_and_time_us_instruments_to_cockpit(&mut cockpit, panel);

        let panel = Panel::named(
            MetricsIntrospectionService::IntrospectionServiceCallFailure,
            "failed",
        );
        add_counting_and_time_us_instruments_to_cockpit(&mut cockpit, panel);

        let (tx, rx) = TelemetryProcessor::new_pair("service_calls");

        tx.add_cockpit(cockpit);

        (tx, rx)
    }

    fn add_counting_instruments_to_cockpit<L>(cockpit: &mut Cockpit<L>, mut panel: Panel<L>)
    where
        L: Clone + Eq + Send + 'static,
    {
        panel.set_counter(Counter::new_with_defaults("count"));
        let mut meter = Meter::new_with_defaults("per_second");
        meter.set_five_minute_rate_enabled(true);
        meter.set_fifteen_minute_rate_enabled(true);
        panel.set_meter(meter);
        cockpit.add_panel(panel);
    }

    fn add_counting_and_time_us_instruments_to_cockpit<L>(
        cockpit: &mut Cockpit<L>,
        mut panel: Panel<L>,
    ) where
        L: Clone + Eq + Send + 'static,
    {
        panel.set_counter(Counter::new_with_defaults("count"));
        let mut meter = Meter::new_with_defaults("per_second");
        meter.set_five_minute_rate_enabled(true);
        meter.set_fifteen_minute_rate_enabled(true);
        panel.set_meter(meter);
        panel.set_histogram(
            Histogram::new_with_defaults("time_us").display_time_unit(TimeUnit::Microseconds),
        );
        cockpit.add_panel(panel);
    }
}

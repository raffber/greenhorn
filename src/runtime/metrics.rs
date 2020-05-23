//! This modules allows collecting runtime information about a greenhorn application
//!
//! The [Metrics](struct.Metrics.html) object collects performance information during runtime.
//! The [Runtime](../struct.Runtime.html) objects maintains such a `Metrics` objects and returns
//! it after running the application to completion.
//!
//! The performance data is collected in a histogram. Refer to the [Histogram](struct.Histogram.html)
//! object for details.
//!

use crate::Id;
use hdrhistogram::Histogram as HdrHistogram;
use hdrhistogram::{CreationError, RecordError};
use instant::Instant;
use serde::{Serialize, Serializer};
use serde_json::json;
use std::cmp::max;
use std::collections::HashMap;
use std::io;
use std::result::Result as StdResult;
use std::time::Duration;

/// Histogram type to collect performance information as u64.
///
/// This is just a new-type for `HdrHistogram` to implement a custom
/// `serde::Serialize`.
/// Refer to [HdrHistogram](https://docs.rs/hdrhistogram/7.1.0/hdrhistogram/struct.Histogram.html).
struct Histogram(HdrHistogram<u64>);

impl Histogram {
    /// Refer to [HdrHistogram::new_with_bounds](https://docs.rs/hdrhistogram/7.1.0/hdrhistogram/struct.Histogram.html#method.new_with_bounds).
    pub fn new_with_bounds(low: u64, high: u64, sigfig: u8) -> Result<Self, CreationError> {
        HdrHistogram::new_with_bounds(low, high, sigfig).map(Histogram)
    }

    /// Refer to [HdrHistogram::record_n](https://docs.rs/hdrhistogram/7.1.0/hdrhistogram/struct.Histogram.html#method.record_n).
    pub fn record_n(&mut self, value: u64, count: u64) -> Result<(), RecordError> {
        self.0.record_n(value, count)
    }

    /// Refer to [HdrHistogram::record](https://docs.rs/hdrhistogram/7.1.0/hdrhistogram/struct.Histogram.html#method.record).
    pub fn record(&mut self, value: u64) -> Result<(), RecordError> {
        self.0.record(value)
    }
}

impl Serialize for Histogram {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let num = 16;
        let step = max((self.0.max() / num) + 1, 1);
        let quantiles: Vec<_> = self
            .0
            .iter_linear(step)
            .take(num as usize)
            .enumerate()
            .map(|(k, x)| {
                (
                    k * (step as usize),
                    x.quantile(),
                    x.count_since_last_iteration() as f64,
                )
            })
            .collect();

        let json = json!({
            "len": self.0.len(),
            "min": self.0.min(),
            "max": self.0.max(),
            "mean": self.0.mean(),
            "quantiles": quantiles,
        });
        json.serialize(serializer)
    }
}

/// Marks a serializeable type which supplies a [Histogram](struct.Histogram.html)
trait Metric: Serialize {
    fn histogram(&self) -> &Histogram;
}

/// Allows measuring hit count per second of a function.
///
/// Data is collected in hits per second. The measurement
/// range is from 1 to 1000 hits per second.
#[derive(Serialize)]
pub struct Throughput {
    hist: Histogram,

    #[serde(skip_serializing)]
    last_update: Option<Instant>,

    #[serde(skip_serializing)]
    last_count: u64,
}

impl Throughput {
    /// Create a new `Throughput` object.
    pub fn new() -> Self {
        Self {
            hist: Histogram::new_with_bounds(1, 1000, 3).unwrap(),
            last_update: None,
            last_count: 0,
        }
    }

    /// Record a hit
    pub fn hit(&mut self) {
        self.update();
        self.last_count += 1;
    }

    /// Update the histogram
    fn update(&mut self) {
        let now = Instant::now();
        if let Some(last_update) = self.last_update {
            let delta = now.duration_since(last_update).as_secs_f64();
            let delta_int = delta as u64;
            if delta_int >= 2 {
                self.hist.record_n(0, delta_int - 1).unwrap();
            }
            if delta_int >= 1 {
                self.hist.record(self.last_count).unwrap();
                self.last_update = Some(now);
                self.last_count = 0;
            }
        } else {
            self.last_update = Some(now);
        }
    }
}

impl Default for Throughput {
    fn default() -> Self {
        Self::new()
    }
}

impl Metric for Throughput {
    fn histogram(&self) -> &Histogram {
        &self.hist
    }
}


/// Allows measures execution time of a function.
///
/// Data is collected in us. The supported execution time range is from
/// 1us to 1s.
#[derive(Serialize)]
pub struct ResponseTime {
    hist: Histogram,
}

impl ResponseTime {
    /// Create a new `ResponseTime` object.
    pub fn new() -> Self {
        Self {
            hist: Histogram::new_with_bounds(1, 1e6 as u64, 3).unwrap(),
        }
    }

    /// Run a closure and measures it's execution time
    pub fn run<T, F: FnOnce() -> T>(&mut self, fun: F) -> T {
        let before = Instant::now();
        let ret = fun();
        let after = Instant::now();
        let delta = after.duration_since(before);
        let delta = delta.as_micros();
        self.hist.record(delta as u64).unwrap();
        ret
    }

    /// Record an execution time in the underlying histogram
    pub fn record(&mut self, delta: Duration) {
        self.hist.record(delta.as_micros() as u64).unwrap();
    }
}

impl Default for ResponseTime {
    fn default() -> Self {
        Self::new()
    }
}

/// Collects `render()` performance information about a [Component](../struct.Component.html).
///
/// Records how often the a function of a `Component` is called and records the time
/// each call requires to complete.
#[derive(Serialize, Default)]
pub struct ComponentMetric {
    time: ResponseTime,
    throughput: Throughput,
}

impl ComponentMetric {
    pub fn new() -> Self {
        Default::default()
    }

    /// Measure and execute a function and collect throughput and response time information.
    pub fn run<T, F: FnOnce() -> T>(&mut self, fun: F) -> T {
        let ret = self.time.run(fun);
        self.throughput.hit();
        ret
    }
}

/// Aggregation of metrics collected during execution of a [Runtime](struct.Runtime.html) object.
#[derive(Serialize, Default)]
pub struct Metrics {
    /// `render()` performance and hit count for each component
    pub components: HashMap<Id, ComponentMetric>,

    /// `render()` performance and hit count for the root component
    pub root: ComponentMetric,

    /// Collects the response time of creating a patch after rendering a VDom
    pub diff: ResponseTime,

    /// Collects the time required to diff the VDom which resulted in an empty patch
    ///
    /// This condition might be avoided by correctly reporting whether a component should
    /// re-render using an [Updated](struct.Updated.html) object.
    pub empty_patch: ResponseTime,
}

impl Metrics {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    /// Run a function and record its execution time in the component with the associated `id`.
    pub(crate) fn run_comp<T, F>(&mut self, id: Id, fun: F) -> T
    where
        F: FnOnce() -> T,
    {
        let metric = if let Some(metric) = self.components.get_mut(&id) {
            metric
        } else {
            self.components.insert(id, ComponentMetric::new());
            self.components.get_mut(&id).unwrap()
        };
        metric.run(fun)
    }

    /// JSON serialize this object.
    pub fn write(&self, out: impl io::Write) -> StdResult<(), String> {
        serde_json::to_writer(out, self).map_err(|x| format!("{}", x))
    }
}

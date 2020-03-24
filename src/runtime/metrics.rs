use std::collections::HashMap;
use crate::Id;
use std::io;
use serde::{Serialize, Serializer};
use std::time::Instant;
use hdrhistogram::Histogram as HdrHistogram;
use hdrhistogram::{CreationError, RecordError};
use std::result::Result as StdResult;
use serde_json::json;

// newtype for histogram to impl Serialize
struct Histogram(HdrHistogram<u64>);


impl Histogram {
    pub fn new_with_bounds(low: u64, high: u64, sigfig: u8) -> Result<Self, CreationError> {
        HdrHistogram::new_with_bounds(low, high, sigfig).map(|x| Histogram(x))
    }

    pub fn record_n(&mut self, value: u64, count: u64) -> Result<(), RecordError> {
        self.0.record_n(value, count)
    }

    pub fn record(&mut self, value: u64) -> Result<(), RecordError> {
        self.0.record(value)
    }
}

impl Serialize for Histogram {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where S: Serializer
    {
        let quantiles: Vec<_> = self.0
            .iter_quantiles(16)
            .map(|x| x.quantile())
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

trait Metric<'de> : Serialize {
    fn histogram(&self) -> &Histogram;
}

#[derive(Serialize)]
struct ResponseTime {
    hist: Histogram,
}

#[derive(Serialize)]
struct Throughput {
    hist: Histogram,

    #[serde(skip_serializing)]
    last_update: Option<Instant>,

    #[serde(skip_serializing)]
    last_count: u64,
}

impl Throughput {
    fn new() -> Self {
        Self {
            hist: Histogram::new_with_bounds(0, 10000, 3).unwrap(),
            last_update: None,
            last_count: 0,
        }
    }

    fn hit(&mut self) {
        self.update();
        self.last_count += 1;
    }

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
            }
            self.last_count = 0;
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

impl Metric<'_> for Throughput {
    fn histogram(&self) -> &Histogram {
        &self.hist
    }
}


impl ResponseTime {
    fn new() -> Self {
        Self {
            hist: Histogram::new_with_bounds(0, 1e6 as u64, 3).unwrap()
        }
    }

    fn run<T, F: FnOnce() -> T>(&mut self, fun: F) -> T {
        let before = Instant::now();
        let ret = fun();
        let after = Instant::now();
        let delta = after.duration_since(before);
        let delta = delta.as_micros();
        self.hist.record(delta as u64).unwrap();
        ret
    }
}

impl Default for ResponseTime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Default)]
pub struct ComponentMetric {
    time: ResponseTime,
    throughput: Throughput,
}

impl ComponentMetric {
    fn new() -> Self { Default::default() }
}

#[derive(Serialize, Default)]
pub struct Metrics {
    pub components: HashMap<Id, ComponentMetric>,
    pub root: ComponentMetric,
}


impl Metrics {
    fn new() -> Self {
        Default::default()
    }

    fn run_comp<T, F>(&mut self, id: Id, fun: F) -> T
        where
            F: FnOnce() -> T
    {
        let metric = if let Some(metric) = self.components.get_mut(&id) {
            metric
        } else {
            self.components.insert(id, ComponentMetric::new());
            self.components.get_mut(&id).unwrap()
        };
        let ret = metric.time.run(fun);
        metric.throughput.hit();
        ret
    }

    fn write(&self, out: impl io::Write) -> StdResult<(), String> {
        serde_json::to_writer(out, self)
            .map_err(|x| format!("{}", x))
    }
}
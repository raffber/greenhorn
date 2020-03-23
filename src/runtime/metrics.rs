use std::collections::HashMap;
use crate::Id;
use std::io;
use serde::Serialize;
use std::time::Instant;
use hdrhistogram::Histogram;
use serde_json::Value as JsonValue;

trait Metric {
    fn histogram(&self) -> &Histogram<u64>;
    fn dump(&self) -> JsonValue;
}

struct ResponseTime {
    hist: Histogram<u64>,
}

struct Throughput {
    hist: Histogram<u64>,
    last_update: Option<Instant>,
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
                self.hist.record_n(0, delta_int - 1);
            }
            if delta_int >= 1 {
                self.hist.record(self.last_count);
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
        self.hist.record(delta as u64);
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
    pub time: ResponseTime,
    pub throughput: Throughput,
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

    fn run_comp<T, F>(&mut self, id: Id, fun: &mut F) -> T
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

    fn dump(&self, out: impl io::Write) {
        todo!()
    }
}
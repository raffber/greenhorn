use std::collections::HashMap;
use crate::Id;
use std::io;
use serde::Serialize;
use std::time::Instant;
use hdrhistogram::Histogram;

trait Metric {
    fn histogram(&self) -> &Histogram;
}

struct ResponseTime {
}

struct Throughput {

}

impl Throughput {
    fn hit(&mut self) {

    }
}

impl ResponseTime {
    fn new() -> Self {

    }

    fn run<T, F: FnOnce() -> T>(&mut self, fun: F) -> T {
        let before = Instant::now();
        let ret = fun();
        let after = Instant::now();
        let delta = after.duration_since(before);
        delta.as_micros()
        ret
    }
}

impl Default for ResponseTime {
    fn default() -> Self {

    }
}

#[derive(Serialize, Default)]
pub struct ComponentMetric {
    pub time: ResponseTime,
    pub throughput: Throughput,
}

#[derive(Serialize, Default)]
pub struct Metrics {
    pub components: HashMap<Id, ComponentMetric>,
    pub root: ComponentMetric,
}


impl Metrics {
    fn dump(&self, out: impl io::Write) {

        todo!()
    }
}
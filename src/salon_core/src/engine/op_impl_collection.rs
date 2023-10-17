use super::ops::{exposure::AdjustExposureImpl, saturation::AdjustSaturationImpl, histogram::ComputeHistogramImpl, collect_statistics::CollectStatisticsImpl};

#[derive(Default)]
pub struct OpImplCollection {
    pub exposure: Option<AdjustExposureImpl>,
    pub saturation: Option<AdjustSaturationImpl>,
    pub histogram: Option<ComputeHistogramImpl>,
    pub collect_statistics: Option<CollectStatisticsImpl>
}

impl OpImplCollection {
    pub fn new() -> Self {
        OpImplCollection {
            ..Default::default()
        }
    }
}

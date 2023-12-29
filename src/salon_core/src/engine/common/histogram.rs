pub struct ImageHistogram {
    pub r: Vec<u32>,
    pub g: Vec<u32>,
    pub b: Vec<u32>,
    pub luma: Vec<u32>,
    pub num_bins: u32,
}

impl ImageHistogram {
    pub fn max_bins() -> usize {
        256
    }
    pub fn num_bins_for(dimensions: (u32, u32)) -> usize {
        // need more sophisticated logic here?
        100
    }
    pub fn max_value(&self) -> u32 {
        let mut max = 0u32;
        for r in self.r.iter() {
            max = max.max(*r);
        }
        for g in self.g.iter() {
            max = max.max(*g);
        }
        for b in self.b.iter() {
            max = max.max(*b);
        }
        for luma in self.luma.iter() {
            max = max.max(*luma);
        }
        max
    }
}

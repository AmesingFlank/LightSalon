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
    pub fn num_bins_for(_dimensions: (u32, u32)) -> usize {
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

    pub fn from_u32_slice(data: &[u32]) -> Self {
        let r = data[ImageHistogram::max_bins() * 0..ImageHistogram::max_bins() * 1].to_vec();
        let g = data[ImageHistogram::max_bins() * 1..ImageHistogram::max_bins() * 2].to_vec();
        let b = data[ImageHistogram::max_bins() * 2..ImageHistogram::max_bins() * 3].to_vec();
        let luma = data[ImageHistogram::max_bins() * 3..ImageHistogram::max_bins() * 4].to_vec();

        let num_bins = data[ImageHistogram::max_bins() * 4];

        ImageHistogram {
            r,
            g,
            b,
            luma,
            num_bins,
        }
    }
}

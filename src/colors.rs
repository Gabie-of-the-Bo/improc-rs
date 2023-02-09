pub struct Gradient {
    colors: Vec<((u8, u8, u8), u32)>,
    min_pos: u32,
    max_pos: u32
}

impl Gradient {
    pub fn new() -> Gradient {
        return Gradient { colors: vec!(), min_pos: 0, max_pos: 0 };
    }

    pub fn add(&mut self, color: (u8, u8, u8), position: u32) {
        self.colors.push((color, position));

        self.colors.sort_by_key(|(_, p)| *p);

        self.min_pos = self.colors.first().unwrap().1;
        self.max_pos = self.colors.last().unwrap().1;
    }

    pub fn get_color(&self, mut frac: f32) -> (u8, u8, u8) {
        assert!(frac >= 0.0 && frac <= 1.0);
        
        frac *= (self.max_pos - self.min_pos) as f32;
        frac += self.min_pos as f32;

        let mut color_idx = self.colors.len() - 1;

        // Scan accumulated weights
        for (i, (_, p)) in self.colors.iter().enumerate() {
            if *p as f32 > frac {
                color_idx = i;
                break;
            }
        }

        color_idx = color_idx.max(1);

        // Interpolate colors
        let c1 = &self.colors[color_idx - 1];
        let c2 = &self.colors[color_idx];
        let c_frac = (frac - c1.1 as f32) / (c2.1 - c1.1) as f32;

        return (
            (c1.0.0 as f32 * (1.0 - c_frac) + c2.0.0 as f32 * c_frac) as u8,
            (c1.0.1 as f32 * (1.0 - c_frac) + c2.0.1 as f32 * c_frac) as u8,
            (c1.0.2 as f32 * (1.0 - c_frac) + c2.0.2 as f32 * c_frac) as u8
        );
    }

    pub fn simple(colors: Vec<(u8, u8, u8)>) -> Gradient {
        let mut res = Gradient::new();

        for (i, c) in colors.into_iter().enumerate() {
            res.add(c, i as u32);
        }

        return res;
    }
}
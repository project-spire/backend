pub struct Grid<T> {
    pub width: usize,
    pub height: usize,
    data: Vec<T>,
}

impl<T> Grid<T> {
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.data.get(y * self.width + x)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(y * self.width + x)
    }

    /// Returns cardinal neighbors **within** the bound.
    pub fn neighbors_cardinal(&self, x: usize, y: usize) -> impl Iterator<Item = ((usize, usize), &T)> {
        let width = self.width as isize;
        let height = self.height as isize;
        let offsets = [
                      (0, -1),
            (-1,  0),          (1,  0),
                      (0,  1),
        ];

        offsets.into_iter().filter_map(move |(dx, dy)| {
            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if nx >= 0 && nx < width && ny >= 0 && ny < height {
                Some(((nx as usize, ny as usize), self.get(nx as usize, ny as usize).unwrap()))
            } else {
                None
            }
        })
    }

    /// Returns diagonal neighbors **within** the bound.
    pub fn neighbors_diagonal(&self, x: usize, y: usize) -> impl Iterator<Item = ((usize, usize), &T)> {
        let width = self.width as isize;
        let height = self.height as isize;
        let offsets = [
            (-1, -1), (0, -1), (1, -1),
            (-1,  0),          (1,  0),
            (-1,  1), (0,  1), (1,  1),
        ];

        offsets.into_iter().filter_map(move |(dx, dy)| {
            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if nx >= 0 && nx < width && ny >= 0 && ny < height {
                Some(((nx as usize, ny as usize), self.get(nx as usize, ny as usize).unwrap()))
            } else {
                None
            }
        })
    }

    pub fn row(&self, y: usize) -> Option<&[T]> {
        if y >= self.height {
            return None;
        }

        let start = y * self.width;
        let end = start + self.width;
        Some(&self.data[start..end])
    }

    pub fn row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        if y >= self.height {
            return None;
        }
        let start = y * self.width;
        let end = start + self.width;
        Some(&mut self.data[start..end])
    }

    pub fn iter_rows(&self) -> std::slice::Chunks<'_, T> {
        self.data.chunks(self.width)
    }

    pub fn iter_rows_mut(&mut self) -> std::slice::ChunksMut<'_, T> {
        self.data.chunks_mut(self.width)
    }
}

impl<T> Grid<T>
where
    T: Default + Clone,
{
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![T::default(); width * height]
        }
    }
}

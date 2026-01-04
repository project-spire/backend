use std::ops::{Index, IndexMut};

pub struct Grid<T> {
    pub width: usize,
    pub height: usize,
    data: Vec<T>,
}

impl<T> Grid<T> {
    pub const fn offsets_cardinal() -> &'static [(isize, isize); 4] {
        static OFFSETS: [(isize, isize); 4] = [
                      (0, -1),
            (-1,  0),          (1,  0),
                      (0,  1),
        ];
        &OFFSETS
    }

    pub const fn offsets_diagonal() -> &'static [(isize, isize); 8] {
        static OFFSETS: [(isize, isize); 8] = [
            (-1, -1), (0, -1), (1, -1),
            (-1,  0),          (1,  0),
            (-1,  1), (0,  1), (1,  1),
        ];
        &OFFSETS
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.data.get(y * self.width + x)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(y * self.width + x)
    }

    /// Returns cardinal neighbors **within** the bound.
    pub fn neighbors_cardinal(&self, x: usize, y: usize) -> impl Iterator<Item = ((usize, usize), &T)> {
        self.neighbors_inner(x, y, Self::offsets_cardinal())
    }

    /// Returns diagonal neighbors **within** the bound.
    pub fn neighbors_diagonal(&self, x: usize, y: usize) -> impl Iterator<Item = ((usize, usize), &T)> {
        self.neighbors_inner(x, y, Self::offsets_diagonal())
    }

    fn neighbors_inner(
        &self,
        x: usize,
        y: usize,
        offsets: &[(isize, isize)],
    ) -> impl Iterator<Item = ((usize, usize), &T)> {
        offsets.into_iter().filter_map(move |(dx, dy)| {
            let nx = x as isize + dx;
            let ny = y as isize + dy;

            self.data
                .get(ny as usize * self.width + nx as usize)
                .map(|v| ((nx as usize, ny as usize), v))
        })
    }

    pub fn is_within(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize
    }

    pub fn is_border(&self, x: isize, y: isize) -> bool {
        x == 0 || x == self.width as isize - 1 || y == 0 || y == self.height as isize - 1
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

impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.get(x, y).expect("Index out of bounds")
    }
}

impl<T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        self.get_mut(x, y).expect("Index out of bounds")
    }
}

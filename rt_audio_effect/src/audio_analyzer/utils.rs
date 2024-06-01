#[derive(Clone)]
pub struct TimeSeries<T> {
    data: Vec<T>,
    length: usize,
    width: usize,
    total_size: usize,
}

impl<T> TimeSeries<T>
where
    T: Clone,
{
    pub fn new(length: usize, width: usize, default_value: T) -> TimeSeries<T> {
        TimeSeries {
            data: vec![default_value; length * width],
            length: length,
            width: width,
            total_size: length * width,
        }
    }

    pub fn push(&mut self, data: Vec<T>) {
        self.data.extend(data);

        if self.data.len() >= self.length * self.width {
            self.data.drain(0..self.data.len() - self.total_size);
        }
    }

    pub fn get_total_len(&self) -> usize {
        self.total_size
    }

    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_last(&self) -> &[T] {
        &self.data[self.data.len() - self.width..]
    }
    pub fn get_data(&self) -> &[T] {
        self.data.as_slice()
    }
}

#[derive(Clone)]
pub struct MultiChannel<T> {
    pub channels: Vec<T>,
}

impl<T> MultiChannel<T>
where
    T: Clone,
{
    pub fn new(channel_count: usize, default_value: T) -> MultiChannel<T> {
        MultiChannel {
            channels: vec![default_value; channel_count],
        }
    }

    pub fn len(&self) -> usize {
        self.channels.len()
    }

    pub fn get_channel(&self, channel: usize) -> &T {
        &self.channels[channel]
    }

    pub fn get_channel_mut(&mut self, channel: usize) -> &mut T {
        &mut self.channels[channel]
    }

    pub fn inner_mut(&mut self) -> &mut Vec<T> {
        &mut self.channels
    }
}

impl<T> From<Vec<T>> for MultiChannel<T>
where
    T: Clone,
{
    fn from(data: Vec<T>) -> MultiChannel<T> {
        MultiChannel { channels: data }
    }
}

impl<T> IntoIterator for MultiChannel<T>
where
    T: Clone,
{
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.channels.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a MultiChannel<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.channels.iter()
    }
}

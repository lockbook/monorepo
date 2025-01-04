use crate::svg;

/// represents a 3 step clone
/// step 1: calculate memory foot print (requires lock)
/// step 2: actually allocate the memory
/// step 3: do the copy (requires lock)
pub trait MultiClone {
    type Footprint;

    fn footprint(&self) -> Self::Footprint;
    fn allocate(fp: Self::Footprint) -> Self;
    fn copy_into(&self, new: &mut Self);
}

#[derive(Debug)]
pub struct SvgFootprint {
    opened_content: usize,
    elements: usize,
    weak_images: usize,
    id_map: usize,
}

impl MultiClone for svg::Buffer {
    type Footprint = SvgFootprint;

    #[instrument(skip(self))]
    fn footprint(&self) -> Self::Footprint {
        Self::Footprint {
            opened_content: self.opened_content.len(),
            elements: self.elements.len(),
            weak_images: self.weak_images.len(),
            id_map: self.id_map.len(),
        }
    }

    #[instrument]
    fn allocate(fp: Self::Footprint) -> Self {
        let mut new = Self::default();

        new.opened_content.reserve(fp.opened_content);
        new.elements.reserve(fp.elements);
        new.weak_images.reserve(fp.weak_images);
        new.id_map.reserve(fp.id_map);

        new
    }

    #[instrument(skip(self, new))]
    fn copy_into(&self, new: &mut Self) {
        new.open_file_hmac = self.open_file_hmac;
        // new.opened_content.insert_str

        for (k, v) in &self.elements {
            new.elements.insert(*k, v.clone());
        }

        for (k, v) in &self.weak_images.0 {
            new.weak_images.0.insert(*k, *v);
        }

        new.master_transform = self.master_transform;
        for (k, v) in &self.id_map {
            new.id_map.insert(*k, v.clone());
        }
    }
}

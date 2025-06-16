use std::sync::Arc;

use plain::DeserializablePlainTexture;
use types::LDRColor;

use crate::{ImageCache, ImageLoader};

pub mod plain;

pub trait Texture {
    fn get(&self, u: f64, v: f64) -> LDRColor;
}

#[derive(Clone, Debug)]
pub enum DeserializableTexture {
    Plain(DeserializablePlainTexture),
}

impl DeserializableTexture {
    pub fn into_texture<T: ImageLoader>(
        self,
        image_cache: &mut ImageCache<T>,
    ) -> Arc<dyn Texture + Send + Sync> {
        match self {
            DeserializableTexture::Plain(t) => t.into_texture(image_cache),
        }
    }
}

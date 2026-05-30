use std::sync::atomic::{AtomicUsize, Ordering};


// ========== Texture's UUID generator
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct TextureUUID { id: usize }

impl TextureUUID {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        Self { id: COUNTER.fetch_add(1, Ordering::SeqCst) }
    }
}

// ========== Texture ==========
pub struct Texture {
    pub uuid: usize,
    pub image: image::DynamicImage
}

impl Texture {
    pub fn new(bytes: &[u8]) -> Result<Self, image::ImageError> {
        let uuid = TextureUUID::new().id;
        let image = image::load_from_memory(bytes)?;

        Ok(Self { uuid, image })
    }   
}

use super::thumbnail_generator::ThumbnailGeneratorService;

pub struct Services {
    #[cfg(not(target_arch = "wasm32"))]
    pub thumbnail_generator: ThumbnailGeneratorService,
}

impl Services {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            thumbnail_generator: ThumbnailGeneratorService::new(),
        }
    }
}

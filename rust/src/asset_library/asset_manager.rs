use crate::asset_library::asset::Asset;

pub struct AssetManager {
    assets: Vec<Asset>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self { assets: Vec::new() }
    }

    pub fn add_asset(&mut self, asset: Asset) {
        self.assets.push(asset);
    }

    pub fn get_assets(&self) -> Vec<Asset> {
        self.assets.clone()
    }
}
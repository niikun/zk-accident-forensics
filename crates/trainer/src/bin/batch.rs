use burn::data::dataloader::{batcher::Batcher, DataLoaderBuilder};
use burn::prelude::*;
use burn::tensor::{Device, Tensor};
use policy::RANGE_MAX;
use std::path::Path;
use trainer::{Batch, DataBatcher, Item};

pub fn main() {
    let path = Path::new("crates/trainer/tests/data/item_test.csv");
    let items = Item::read_csv(&path);
    let batcher = DataBatcher::default();
    let device = Device::wgpu(Default::default());
    let batch = batcher.batch(items, &device);
    println!("{:?}", batch.inputs.to_data());
    println!("{:?}", batch.classes.to_data());
    println!("{:?}", batch.inputs.dims());
    println!("{:?}", batch.classes.dims());
}

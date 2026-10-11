use std::path::Path;
use burn::tensor::{Device, Tensor};
use burn::data::dataloader::{batcher::Batcher, DataLoaderBuilder};
use burn::prelude::*;
use trainer::{Item, DataBatcher, Batch};
use policy::RANGE_MAX;



pub fn main() {
    let path = Path::new("crates/trainer/tests/data/item_test.csv");
    let items = Item::read_csv(&path);
    let batcher = DataBatcher::default();
    let device = Device::wgpu(Default::default());
    let batch = batcher.batch(items,&device);
    println!("{:?}",batch.inputs.to_data());
    println!("{:?}",batch.targets.to_data());
    println!("{:?}",batch.inputs.dims());
    println!("{:?}",batch.targets.dims());
}
use burn::data::dataloader::batcher::Batcher;
use burn::nn::{Dropout, DropoutConfig, Linear, LinearConfig, Relu};
use burn::prelude::*;
use burn::tensor::Device;
use std::path::{Path, PathBuf};
use trainer::{Batch, DataBatcher, Item, Model, ModelConfig};

fn main() {
    let batcher = DataBatcher::default();
    let test_path = Path::new("datas/test.csv");
    let test_items: Vec<Item> = Item::read_csv(test_path);
    let device = Device::wgpu(Default::default());
    let batch = Batcher::batch(&batcher, test_items.clone(), &device);
    let model_config = ModelConfig::new(9, 3, 128);
    let model = model_config.init(&device);
    let model_path = Path::new("datas/artifact/checkpoint/model-49.bpk");
    let model = model.load_file(model_path);
    let outputs = model.forward(batch);

    let mode_pred_tensor = outputs.1.argmax(1).into_data();
    let mode_pred = mode_pred_tensor.into_vec::<i32>().unwrap();
    let linear_pred_tensor = outputs.2.into_data();
    let linear_pred = linear_pred_tensor.into_vec::<f32>().unwrap();
    let modes: Vec<usize> = test_items.iter().map(|item| item.mode).collect();
    let linears: Vec<f32> = test_items.iter().map(|item| item.linear as f32).collect();
    let mut confusion_mode = [[0usize; 3]; 3];
    for (mode_pred, y) in mode_pred.iter().zip(modes.iter()) {
        let mode_pred = *mode_pred as usize;
        confusion_mode[*y][mode_pred] += 1;
    }
    let mut delta: f32 = 0.0;
    let mut counter = 0.0;
    for ((linear_pred, y), mode_pred) in linear_pred.iter().zip(linears.iter()).zip(mode_pred) {
        if mode_pred == 1 {
            delta += (*linear_pred - *y).powf(2.0);
            counter += 1.0;
        }
    }
    println!("{:?} ", confusion_mode);
    println!("{:?} ", delta / counter);
}

use std::path::{Path, PathBuf};
use burn::nn::loss::{CrossEntropyLoss, CrossEntropyLossConfig,MseLoss, Reduction};
use burn::nn::{Linear, LinearConfig, Relu, Dropout,DropoutConfig};
use burn::tensor::Device;
use burn::train::{ClassificationOutput, InferenceStep, Learner, SupervisedTraining, TrainStep, TrainOutput};
use burn::train::metric::{AccuracyMetric, LossMetric};
use burn::optim::AdamConfig;
use burn::data::dataloader::{batcher::Batcher, DataLoaderBuilder};
use burn::data::dataset::InMemDataset;
use burn::prelude::*;

use trainer::{Item, Batch, DataBatcher, ModelConfig, Model};

#[derive(Config, Debug)]
pub struct TrainingConfig {
    pub model: ModelConfig,
    pub optimizer: AdamConfig,
    #[config(default = 10)]
    pub num_epochs: usize,
    #[config(default = 64)]
    pub batch_size: usize,
    #[config(default = 4)]
    pub num_workers: usize,
    #[config(default = 42)]
    pub seed: u64,
    #[config(default = 1.0e-4)]
    pub learning_rate: f64,
}

fn create_artifact_dir(artifact_dir: &str) {
    std::fs::remove_file(PathBuf::from(artifact_dir).join("experiment.log")).ok();
    std::fs::create_dir_all(artifact_dir).ok();
}

pub fn train(artifact_dir: &str, config: TrainingConfig, train_items:Vec<Item>, test_items:Vec<Item>, device:Device) {
    create_artifact_dir(artifact_dir);
    config
        .save(format!("{artifact_dir}/config.json"))
        .expect("Config should be saved successfully");


    device.seed(config.seed);
    let autodiff_device = device.autodiff();

    let batcher = DataBatcher::default();

    let train_dataset = InMemDataset::new(train_items);
    let test_dataset =InMemDataset::new(test_items);

    let dataloader_train = DataLoaderBuilder::new(batcher.clone())
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(train_dataset);

    let dataloader_test = DataLoaderBuilder::new(batcher)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(test_dataset);

    let training = SupervisedTraining::new(artifact_dir, dataloader_train, dataloader_test)
        .metrics((AccuracyMetric::new(), LossMetric::new()))
        .with_default_checkpointers()
        .num_epochs(config.num_epochs)
        .summary();

    let model = config.model.init(&autodiff_device);
    let result = training.launch(Learner::new(
        model,
        config.optimizer.init(),
        config.learning_rate,
    ));

    result
        .model
        .into_record()
        .save(format!("{artifact_dir}/model"))
        .expect("Trained model should be saved successfully");
}

fn main() {
    let train_path = Path::new("datas/train.csv");
    let test_path = Path::new("datas/test.csv");
    let train_items = Item::read_csv(train_path);
    let test_items = Item::read_csv(test_path);
    let device = Device::wgpu(Default::default());
    let artifact_dir = "datas/artifact";
    let model_config = ModelConfig::new(
        9,
        3,
        128
    );
    let model = model_config.init(&device);
    let optimizer = AdamConfig::new();
    let train_config = TrainingConfig{
        model: model_config,
        optimizer: optimizer,
        num_epochs: 50,
        batch_size:64,
        num_workers: 4,
        seed: 42,
        learning_rate: 0.001
    };

    train(artifact_dir, train_config,train_items,test_items,device);

}
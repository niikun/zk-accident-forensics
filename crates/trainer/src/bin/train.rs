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

use trainer::{Item, Batch, DataBatcher};


#[derive(Module, Debug)]
struct Model {
    linear1: Linear,
    linear2: Linear,
    activation: Relu,
    dropout: Dropout,
    out1:Linear,
    out2:Linear,
    out3:Linear
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    num_classes: usize,
    num_modes:usize,
    hidden_size: usize,
    #[config(default = "0.1")]
    dropout: f64
}

impl ModelConfig {
    pub fn init(&self, device: &Device) -> Model {
        Model {
            linear1: LinearConfig::new(24,self.hidden_size).init(device),
            linear2: LinearConfig::new(self.hidden_size, self.hidden_size).init(device),
            activation: Relu::new(),
            dropout: DropoutConfig::new(self.dropout).init(),
            out1: LinearConfig::new(self.hidden_size, self.num_classes).init(device),
            out2: LinearConfig::new(self.hidden_size, self.num_modes).init(device),
            out3: LinearConfig::new(self.hidden_size, 1).init(device),
        }
    }
}

impl Model {
    fn forward(&self, batch:Batch) -> (Tensor<2>,Tensor<2>,Tensor<2>){
        let x  = self.activation.forward(self.linear1.forward(batch.inputs));
        let x = self.dropout.forward(self.activation.forward(self.linear2.forward(x)));
        let output1 = self.out1.forward(x.clone());
        let output2 = self.out2.forward(x.clone());
        let output3 = self.out3.forward(x);
        (output1, output2, output3)   
    }

    fn forward_classification(&self, batch:Batch) ->ClassificationOutput {
        let (output1,output2,output3) = self.forward(batch.clone());
        let loss1 = CrossEntropyLossConfig::new()
                            .init(&output1.device())
                            .forward(output1.clone(),batch.classes.clone());
        let loss2 = CrossEntropyLossConfig::new()
                            .init(&output2.device())
                            .forward(output2.clone(), batch.modes.clone());
        let loss3 = MseLoss::new()
                            .forward(output3.clone().reshape([output3.dims()[0]]), batch.linears,Reduction::Auto);
        let loss = loss1 + loss2 + loss3 * 10;
        let output = output1;
        let targets = batch.classes ;
        ClassificationOutput { loss, output, targets}
    }
}

impl TrainStep for Model {
    type Input = Batch;
    type Output = ClassificationOutput;

    fn step(&self, batch: Batch) -> TrainOutput<ClassificationOutput> {
        let item = self.forward_classification(batch);

        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl InferenceStep for Model {
    type Input = Batch;
    type Output = ClassificationOutput;

    fn step(&self, batch: Batch) -> ClassificationOutput {
        self.forward_classification(batch)
    }
}

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
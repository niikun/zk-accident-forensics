use burn::nn::loss::{CrossEntropyLoss, CrossEntropyLossConfig};
use burn::nn::{Linear, LinearConfig, Relu, Dropout,DropoutConfig};
use burn::tensor::Device;
use burn::data::{dataloader::batcher::Batcher};
use burn::train::ClassificationOutput;
use burn::prelude::*;

use trainer::Item;


#[derive(Module, Debug)]
struct Model {
    linear1: Linear,
    linear2: Linear,
    activation: Relu,
    dropout: Dropout,
    linear3:Linear,
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    num_classes: usize,
    hidden_size: usize,
    #[config(default = "0.5")]
    dropout: f64
}
#[derive(Clone, Default)]
pub struct DataBatcher {}

#[derive(Clone, Debug)]
pub struct Batch {
    pub input: Tensor<2>,
    pub targets: Tensor<1, Int>,
}


impl ModelConfig {
    pub fn init(&self, device: &Device) -> Model {
        Model {
            linear1: LinearConfig::new(24,self.hidden_size).init(device),
            linear2: LinearConfig::new(self.hidden_size, self.hidden_size).init(device),
            activation: Relu::new(),
            dropout: DropoutConfig::new(self.dropout).init(),
            linear3: LinearConfig::new(self.hidden_size, self.num_classes).init(device)
        }
    }
}

impl Batcher<Item, Batch> for DataBatcher {
    fn batch(&self, items:Vec<Item>, device: &Device) -> Batch {
       let inputs = items
            .iter()
            .map(|item| TensorData::from( item.beams))
            .map(|data| Tensor::<2>::from_data(data, device))
            .collect();

        let targets = items.targets
            .iter()
            .map(|item| Tensor::<1, Int>::from_data([item as i64], device))
            .collect();

        Batch { input, targets }
    }
}

impl Model {
    fn forward(&self, batch:Batch) -> Tensor<2>{
        let x  = self.activation.forward(self.linear1.forward(batch.input));
        let x = self.dropout.forward(self.activation.forward(self.linear2.forward(x)));
        let output = self.linear3.forward(x);
        output   
    }

    fn forward_classification(&self, batch:Batch) ->ClassificationOutput{
        let output = self.forward(batch.input);
        let loss = CrossEntropyLossConfig::new()
                            .init(&output.device())
                            .forward(output.clone(),batch.targets.clone());
        let targets = batch.targets;
        ClassificationOutput { loss, output, targets }
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
    type Input = MnistBatch;
    type Output = ClassificationOutput;

    fn step(&self, batch: Batch) -> ClassificationOutput {
        self.forward_classification(batch.input, batch.targets)
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

pub fn train(artifact_dir: &str, config: TrainingConfig, device: impl Into<Device>,train_data:Vec<String>, test_data:Vec<String>) {
    create_artifact_dir(artifact_dir);
    config
        .save(format!("{artifact_dir}/config.json"))
        .expect("Config should be saved successfully");

    let device = device.into();
    device.seed(config.seed);
    let autodiff_device = device.autodiff();

    let batcher = Batcher::default();
    let items = Item::read_csv();

    let dataloader_train = DataLoaderBuilder::new(batcher.clone())
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(train_data);

    let dataloader_test = DataLoaderBuilder::new(batcher)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(test_data);

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
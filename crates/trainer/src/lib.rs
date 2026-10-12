use burn::data::dataloader::{batcher::Batcher, DataLoaderBuilder};
use burn::nn::loss::{CrossEntropyLoss, CrossEntropyLossConfig, MseLoss, Reduction};
use burn::nn::{Dropout, DropoutConfig, Linear, LinearConfig, Relu};
use burn::optim::AdamConfig;
use burn::prelude::*;
use burn::store::ModuleRecord;
use burn::tensor::{Device, Tensor};
use burn::train::metric::{AccuracyMetric, LossMetric};
use burn::train::{
    ClassificationOutput, InferenceStep, Learner, SupervisedTraining, TrainOutput, TrainStep,
};
use std::path::Path;

use policy::normalize_beam;

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub beams: Vec<f32>,
    pub class: usize,
    pub mode: usize,
    pub linear: f32,
}

impl Item {
    pub fn read_csv(path: &Path) -> Vec<Item> {
        let mut reader =
            csv::Reader::from_path(&path).unwrap_or_else(|e| panic!("file can not open :{}", e));
        let mut items: Vec<Item> = Vec::new();
        for result in reader.records() {
            let record = result.unwrap();
            let mut beams: Vec<f32> = Vec::new();
            for i in 1..25 {
                let b: f32 = record[i].parse().unwrap();
                beams.push(b);
            }
            let class = record[25].parse().unwrap();
            let mode = record[26].parse().unwrap();
            let linear = record[27].parse().unwrap();
            let item = Item {
                beams,
                class,
                mode,
                linear,
            };
            items.push(item)
        }
        items
    }
}

#[derive(Clone, Default)]
pub struct DataBatcher {}

#[derive(Clone, Debug)]
pub struct Batch {
    pub inputs: Tensor<2>,
    pub classes: Tensor<1, Int>,
    pub modes: Tensor<1, Int>,
    pub linears: Tensor<1, Float>,
}

impl Batcher<Item, Batch> for DataBatcher {
    fn batch(&self, items: Vec<Item>, device: &Device) -> Batch {
        let mut beams_flatten: Vec<f32> = Vec::new();
        let mut samples: usize = 0;
        for item in &items {
            for b in &item.beams {
                beams_flatten.push(normalize_beam(b));
            }
            samples += 1;
        }
        let inputs_data = TensorData::new(beams_flatten, [samples, 24usize]);
        let inputs = Tensor::<2>::from_data(inputs_data, device);
        let mut classes_flatten: Vec<i64> = Vec::new();
        let mut modes_flatten: Vec<i64> = Vec::new();
        let mut linears_flatten: Vec<f64> = Vec::new();
        for item in items {
            classes_flatten.push(item.class as i64);
            modes_flatten.push(item.mode as i64);
            linears_flatten.push(item.linear as f64);
        }
        let classes_data = TensorData::new(classes_flatten, [samples]);
        let classes = Tensor::<1, Int>::from_data(classes_data, device);
        let modes_data = TensorData::new(modes_flatten, [samples]);
        let modes = Tensor::<1, Int>::from_data(modes_data, device);
        let linears_data = TensorData::new(linears_flatten, [samples]);
        let linears = Tensor::<1, Float>::from_data(linears_data, device);

        Batch {
            inputs,
            classes,
            modes,
            linears,
        }
    }
}

#[derive(Module, Debug)]
pub struct Model {
    pub linear1: Linear,
    pub linear2: Linear,
    pub activation: Relu,
    pub dropout: Dropout,
    pub out1: Linear,
    pub out2: Linear,
    pub out3: Linear,
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    pub num_classes: usize,
    pub num_modes: usize,
    pub hidden_size: usize,
    #[config(default = "0.1")]
    pub dropout: f64,
}

impl ModelConfig {
    pub fn init(&self, device: &Device) -> Model {
        Model {
            linear1: LinearConfig::new(24, self.hidden_size).init(device),
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
    pub fn forward(&self, batch: Batch) -> (Tensor<2>, Tensor<2>, Tensor<2>) {
        let x = self.activation.forward(self.linear1.forward(batch.inputs));
        let x = self
            .dropout
            .forward(self.activation.forward(self.linear2.forward(x)));
        let output1 = self.out1.forward(x.clone());
        let output2 = self.out2.forward(x.clone());
        let output3 = self.out3.forward(x);
        (output1, output2, output3)
    }

    pub fn forward_classification(&self, batch: Batch) -> ClassificationOutput {
        let (output1, output2, output3) = self.forward(batch.clone());
        let loss1 = CrossEntropyLossConfig::new()
            .init(&output1.device())
            .forward(output1.clone(), batch.classes.clone());
        let loss2 = CrossEntropyLossConfig::new()
            .init(&output2.device())
            .forward(output2.clone(), batch.modes.clone());
        let loss3 = MseLoss::new().forward(
            output3.clone().reshape([output3.dims()[0]]),
            batch.linears,
            Reduction::Auto,
        );
        let loss = loss1 + loss2 + loss3 * 10;
        let output = output1;
        let targets = batch.classes;
        ClassificationOutput {
            loss,
            output,
            targets,
        }
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

pub fn extract() -> [Vec<f32>; 10] {
    let model_path = Path::new("datas/artifact/checkpoint/model-49.bpk");
    let record = ModuleRecord::load(model_path)
        .expect("Should be able to load the model weights from the provided file");
    let device = Device::wgpu(Default::default());
    let model_config = ModelConfig::new(9, 3, 128);
    let model = model_config.init(&device).load_record(record);
    let linear1_weights: Vec<f32> = model.linear1.weight.val().to_data().to_vec().unwrap();
    let linear1_bias: Vec<f32> = model
        .linear1
        .bias
        .unwrap()
        .val()
        .to_data()
        .to_vec()
        .unwrap();
    let linear2_weights: Vec<f32> = model.linear2.weight.val().to_data().to_vec().unwrap();
    let linear2_bias: Vec<f32> = model
        .linear2
        .bias
        .unwrap()
        .val()
        .to_data()
        .to_vec()
        .unwrap();
    let out1_weights: Vec<f32> = model.out1.weight.val().to_data().to_vec().unwrap();
    let out1_bias: Vec<f32> = model.out1.bias.unwrap().val().to_data().to_vec().unwrap();
    let out2_weights: Vec<f32> = model.out2.weight.val().to_data().to_vec().unwrap();
    let out2_bias: Vec<f32> = model.out2.bias.unwrap().val().to_data().to_vec().unwrap();

    let out3_weights: Vec<f32> = model.out3.weight.val().to_data().to_vec().unwrap();
    let out3_bias: Vec<f32> = model.out3.bias.unwrap().val().to_data().to_vec().unwrap();
    [
        linear1_weights,
        linear1_bias,
        linear2_weights,
        linear2_bias,
        out1_weights,
        out1_bias,
        out2_weights,
        out2_bias,
        out3_weights,
        out3_bias,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv() {
        let path = Path::new("tests/data/item_test.csv");
        let items = Item::read_csv(&path);
        let beams = vec![
            1.1593302, 1.1572905, 0.8348803, 0.571826, 0.4590087, 0.48808533, 1.3824172, 1.3799846,
            1.4172432, 1.5656986, 1.8795558, 2.7733724, 1.7838885, 2.6428282, 1.1427711, 0.7827062,
            0.62250215, 0.54576385, 0.5209911, 0.5200744, 0.534116, 1.6593945, 1.385217, 1.2144555,
        ];
        let class: usize = 2;
        let mode: usize = 0;
        let linear: f32 = 0.2;
        let item_test = Item {
            beams,
            class,
            mode,
            linear,
        };
        assert_eq!(items[0], item_test);
        assert_eq!(items.len(), 3usize);
    }
}

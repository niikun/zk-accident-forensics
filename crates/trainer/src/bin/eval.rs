use std::path::{Path, PathBuf};
use burn::nn::{Linear, LinearConfig, Relu, Dropout,DropoutConfig};
use burn::tensor::Device;
use burn::data::dataloader::{batcher::Batcher};
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

#[derive(Config, Debug, Default)]
pub struct ModelConfig {
    num_classes: usize,
    num_modes:usize,
    hidden_size: usize,
    #[config(default = "0.1")]
    dropout: f64
}

impl ModelConfig {
    fn init(&self, device: &Device) -> Model {
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

    // fn forward_classification(&self, batch:Batch) ->ClassificationOutput {
    //     let (output1,output2,output3) = self.forward(batch.clone());
    //     let loss1 = CrossEntropyLossConfig::new()
    //                         .init(&output1.device())
    //                         .forward(output1.clone(),batch.classes.clone());
    //     let loss2 = CrossEntropyLossConfig::new()
    //                         .init(&output2.device())
    //                         .forward(output2.clone(), batch.modes.clone());
    //     let loss3 = MseLoss::new()
    //                         .forward(output3.clone().reshape([output3.dims()[0]]), batch.linears,Reduction::Auto);
    //     let loss = loss1 + loss2 + loss3;
    //     let output = output1;
    //     let targets = batch.classes ;
    //     ClassificationOutput { loss, output, targets}
    // }
}

// impl InferenceStep for Model {
//     type Input = Batch;
//     type Output = ClassificationOutput;

//     fn step(&self, batch: Batch) -> ClassificationOutput {
//         self.forward_classification(batch)
//     }
// }


fn main() {
    let batcher =DataBatcher::default();
    let test_path = Path::new("datas/test.csv");
    let test_items:Vec<Item> = Item::read_csv(test_path);
    let device = Device::wgpu(Default::default()); 
    let batch = Batcher::batch(&batcher,test_items.clone(),&device);
    let model_config = ModelConfig::new(
        9,
        3,
        128
    );
    let model = model_config.init(&device);
    let model_path = Path::new("datas/artifact/checkpoint/model-49.bpk");
    let model = model.load_file(model_path);
    let outputs = model.forward(batch);

    let mode_pred_tensor = outputs.1.argmax(1).into_data();
    let mode_pred = mode_pred_tensor.into_vec::<i32>().unwrap();
    let linear_pred_tensor = outputs.2.into_data();
    let linear_pred = linear_pred_tensor.into_vec::<f32>().unwrap();
    let modes:Vec<usize> = test_items
                    .iter()
                    .map(|item| item.mode)
                    .collect();
    let linears:Vec<f32> = test_items
                    .iter()
                    .map(|item| item.linear as f32)
                    .collect();
    let mut confusion_mode = [[0usize;3];3];
    for (mode_pred, y) in mode_pred.iter().zip(modes.iter()){
        let mode_pred = *mode_pred as usize;
        confusion_mode[*y][mode_pred] += 1;       
    }
    let mut delta:f32 = 0.0;
    let mut counter = 0.0;
    for ((linear_pred, y), mode_pred) in linear_pred.iter().zip(linears.iter()).zip(mode_pred){
        if mode_pred == 1{
            delta += (*linear_pred - *y).powf(2.0);
            counter += 1.0;
        }
    }
    println!("{:?} ", confusion_mode);
    println!("{:?} ", delta/ counter);

}
    




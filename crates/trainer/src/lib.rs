use std::path::Path;
use burn::tensor::{Device, Tensor};
use burn::data::dataloader::{batcher::Batcher, DataLoaderBuilder};
use burn::prelude::*;

use policy::RANGE_MAX;

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub beams:Vec<f32>,
    pub class:usize,
    pub mode:usize,
    pub linear:f32
}

impl Item {
    pub fn read_csv(path:&Path) -> Vec<Item>{        
        let mut reader = csv::Reader::from_path(&path).unwrap_or_else(|e| panic!("file can not open :{}",e));
        let mut items:Vec<Item> = Vec::new();
        for result in reader.records() {
            let record = result.unwrap();
            let mut beams:Vec<f32> = Vec::new();
            for i in 1..25{
                let b:f32 = record[i].parse().unwrap();
                beams.push(b);
            }
            let class = record[25].parse().unwrap();
            let mode = record[26].parse().unwrap();
            let linear = record[27].parse().unwrap();     
            let item = Item{beams, class, mode, linear};
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
    pub targets: Tensor<1, Int>,
}

impl Batcher<Item, Batch> for DataBatcher {
    fn batch(&self, items:Vec<Item>, device: &Device) -> Batch {
        let mut beams_flatten:Vec<f32> = Vec::new();
        let mut samples:usize = 0;
        for item in &items{
            for b in &item.beams{
                beams_flatten.push(*b / RANGE_MAX);
            }
            samples += 1;
        }
        let inputs_data = TensorData::new(beams_flatten,[samples, 24usize]);
        let inputs =  Tensor::<2>::from_data(inputs_data, device);
        let mut classes_flatten:Vec<i64> = Vec::new();
        for item in items{
            classes_flatten.push(item.class as i64)
        }
        let targets_data = TensorData::new(classes_flatten,[samples]);
        let targets = Tensor::<1, Int>::from_data(targets_data, device);

        Batch { inputs, targets }
    }
}



#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_read_csv(){
        let path = Path::new("tests/data/item_test.csv");
        let items = Item::read_csv(&path);
        let beams = vec![1.1593302,1.1572905,0.8348803,0.571826,0.4590087,0.48808533,1.3824172,1.3799846,1.4172432,1.5656986,1.8795558,2.7733724,1.7838885,2.6428282,1.1427711,0.7827062,0.62250215,0.54576385,0.5209911,0.5200744,0.534116,1.6593945,1.385217,1.2144555];
        let class:usize = 2;
        let mode:usize = 0;
        let linear:f32 = 0.2;
        let item_test = Item{beams, class, mode, linear};
        assert_eq!(items[0], item_test);
        assert_eq!(items.len(), 3usize);
    }
}
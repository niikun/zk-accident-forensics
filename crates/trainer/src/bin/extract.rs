use std::error::Error;
use std::fs;
use trainer::extract;


fn main() -> Result<(),Box<dyn Error>> {
    let values = extract();
    let labels = ["L1_W","L1_B","L2_W","L2_B","O1_W","O1_B","O2_W","O2_B","O3_W","O3_B"];
    let mut out = String::new();
    for i in 0..10 {
        let str1 = format!("pub const {}:[f32;{}] = {:#?};\n",labels[i], values[i].len() as usize, values[i].as_slice());  
        out.push_str(&str1);
    }
    fs::write("crates/policy/src/weights.rs", out)?; 
    Ok(())
}
    
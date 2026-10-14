
#[test]    
fn read_csv(){
    let path = "../../datas/test.csv";
    let mut reader = csv::Reader::from_path(&path)
        .unwrap_or_else(|e| panic!("file can not open :{}", e));
    for record in reader.records(){
        println!("{:?}",record.unwrap());
    }
}


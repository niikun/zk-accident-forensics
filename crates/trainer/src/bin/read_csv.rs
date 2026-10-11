use std::env;
use std::path::Path;
use trainer::Item;

fn main() {
    let args:Vec<String> = env::args().collect();
        if args.len() < 2{
            eprintln!("file名を指定してください");
            std::process::exit(1);
        }
    let dir = "datas";
    let path = Path::new(dir).join(&args[1]);
    let items = Item::read_csv(&path);
    println!("{:?}",items);
}

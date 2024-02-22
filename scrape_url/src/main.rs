use std::fs;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if let [_path, url, ..] = args.as_slice() {
        let output = "rust.md";
        println!("Fetching url: {}", url);
        let body = reqwest::blocking::get(url).unwrap().text().unwrap();
    
        println!("Converting html to markdown...");
        let md = html2md::parse_html(&body);
    
        fs::write(output, md.as_bytes()).unwrap();
        println!("Converted markdown has been saved in {}.", output);
    }
    // let url = std::env::args();
}

use trn_rust::*;

fn main() {
    println!("=== Testing URL conversion ===");
    
    let trn_str = "trn:aiplatform:model:bert:base-model:v1.0";
    println!("Original TRN string: {}", trn_str);
    
    match trn_str.parse::<Trn>() {
        Ok(trn) => {
            println!("Parsed TRN: {}", trn);
            println!("Components:");
            println!("  platform: {}", trn.platform());
            println!("  scope: {:?}", trn.scope());
            println!("  resource_type: {}", trn.resource_type());
            println!("  type_: {}", trn.type_());
            println!("  subtype: {:?}", trn.subtype());
            println!("  instance_id: {}", trn.instance_id());
            println!("  version: {}", trn.version());
            println!("  tag: {:?}", trn.tag());
            println!("  hash: {:?}", trn.hash());
            
            match trn.to_url() {
                Ok(url) => {
                    println!("\nGenerated URL: {}", url);
                    println!("Expected URL:  trn://aiplatform/model/bert/base-model/v1.0");
                    println!("Matches expected: {}", url == "trn://aiplatform/model/bert/base-model/v1.0");
                }
                Err(e) => {
                    println!("URL generation failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
        }
    }
} 
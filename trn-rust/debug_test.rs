use trn_rust::*;

fn main() {
    let result = TrnBuilder::new()
        .platform("user")
        .scope("alice")
        .resource_type("tool")
        .type_("openapi")
        .subtype("rest")
        .instance_id("github-api")
        .version("v1.0")
        .tag("stable")
        .hash("sha256:a1b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789ab")
        .build();
    
    match result {
        Ok(trn) => {
            println!("Successfully built TRN: {}", trn);
            println!("Validation result: {:?}", trn.validate());
        }
        Err(e) => {
            println!("Build failed: {:?}", e);
        }
    }
} 
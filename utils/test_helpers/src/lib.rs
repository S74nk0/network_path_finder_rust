use std::path::PathBuf;
use json_comments::StripComments;

#[macro_export]
macro_rules! impl_resources_test_helpers
{
    () =>
    {
        use std::path::PathBuf;
        const CARGO_MANIFEST_DIR: Option<&'static str> = option_env!("CARGO_MANIFEST_DIR");
        const TEST_RESOURCES: &'static str = "resources/test";

        fn resource_path(sub_path: &str) -> PathBuf {
            test_helpers::resource_path(CARGO_MANIFEST_DIR, TEST_RESOURCES, sub_path)
        }
        
        fn read_file_as_string(file: &str) -> String {
            std::fs::read_to_string(resource_path(file)).unwrap()
        }
        
        fn deserialize_json_file<T: serde::de::DeserializeOwned>(file: &str) -> T {
            let p = resource_path(file);
            test_helpers::deserialize_json_file(p)
        }
    };
}


pub fn resource_path(cargo_manifest_dir: Option<&'static str>, test_resources: &'static str, sub_path: &str) -> PathBuf {
    let mut p = PathBuf::from(cargo_manifest_dir.expect("CARGO_MANIFEST_DIR not found"));
    p.push(test_resources);
    p.push(sub_path);
    p
}

pub fn deserialize_json_file<T: serde::de::DeserializeOwned>(file: PathBuf) -> T {
    let f = std::fs::File::open(file).unwrap();
    let reader = std::io::BufReader::new(f);
    let stripped = StripComments::new(reader);
    serde_json::from_reader(stripped).unwrap()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_jsonc() {
        let jsonc = r#"
        {
            /* comments */
            "name": "John Doe", // comment 
            "age": 43,
            "address": {
                "street": "10 Downing Street",
                "city": "London"
            },
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }
        "#;
        let value: serde_jsonc::Value = serde_jsonc::from_str(&jsonc).unwrap();
        println!("{:?}", value);
        if let serde_jsonc::Value::Object(o) = value {
            for (k, v) in o {
                println!("{:?}", k);
            }
        }
    }

}


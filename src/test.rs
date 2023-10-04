#[cfg(test)]
mod tests {

    use regex::Regex;
    use sha256::digest;

    use crate::{fill_hash_map, replace_event_names_in_files_with_hashes};
    use std::{collections::HashMap, fs, path::PathBuf};

    #[test]
    fn test_replace_event_names_in_files_with_hashes() {
        let mut event_hashes: HashMap<String, String> = HashMap::new();
        event_hashes.insert("client:event1".to_string(), "hash1".to_string());
        event_hashes.insert("server:event2".to_string(), "hash2".to_string());

        let test_data = r#"
            Some code with "client:event1" and "server:event2"
        "#;

        let test_client_path = "test_data/client_index_test.js";
        let test_server_path = "test_data/server_index_test.js";

        // Create test file
        fs::write(test_client_path, test_data).expect("Unable to write the test file");
        fs::write(test_server_path, test_data).expect("Unable to write the test file");

        let paths: Vec<PathBuf> = vec![
            PathBuf::from(test_client_path),
            PathBuf::from(test_server_path),
        ];

        for path in paths {
            let result = replace_event_names_in_files_with_hashes(&event_hashes, path.as_os_str());
            assert!(result.is_ok());

            // Read the file and check if the replacements are correct
            let content = fs::read_to_string(&path).expect("Unable to read the file");
            assert!(content.contains("hash1"));
            assert!(content.contains("hash2"));
        }

        // Clean up
        fs::remove_file(test_client_path).expect("Unable to remove the test file");
        fs::remove_file(test_server_path).expect("Unable to remove the test file");
    }

    #[test]
    fn test_fill_hash_map() {
        let client_content = r#"
            Some code with "client:event1:hejsan" and "client:event2:tjosan"
        "#;
        let server_content = r#"
            Some code with "server:event3" and "server:event4:hejsan"
        "#;
        let cef_content = r#"
            Some code with "server:event5" and "server:event6:hejsan"
        "#;

        let mut event_hashes: HashMap<String, String> = HashMap::new();
        let re =
            Regex::new(r#"("server:[a-zA-Z-0-9:-_]*\")|(\"client:[a-zA-Z-0-9:-_]*\")"#).unwrap();

        fill_hash_map(
            client_content.to_string(),
            server_content.to_string(),
            cef_content.to_string(),
            &mut event_hashes,
            &re,
        );

        assert_eq!(event_hashes.len(), 6);
        assert_eq!(
            event_hashes.get("client:event1:hejsan"),
            Some(&digest("client:event1:hejsan"))
        );
        assert_eq!(
            event_hashes.get("server:event4:hejsan"),
            Some(&digest("server:event4:hejsan"))
        );
        assert_eq!(
            event_hashes.get("server:event6:hejsan"),
            Some(&digest("server:event6:hejsan"))
        );
    }
}

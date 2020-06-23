pub mod layer1;
pub mod layer2;
pub mod layer3;

pub fn find_input(haystack: &str) -> Option<Vec<u8>> {
    let needle = "==[ Payload ]===============================================";
    haystack.find(needle).map(|idx| haystack[idx..].trim().as_bytes().to_vec())
}
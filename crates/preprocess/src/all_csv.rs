#[cfg(feature = "compressed")]
pub fn load_csv() -> String {
    String::from_utf8(
        zstd::decode_all(include_bytes!("../../../data/all.csv.zstd").as_ref()).unwrap(),
    )
    .unwrap()
}

#[cfg(not(feature = "compressed"))]
pub fn load_csv() -> String {
    include_str!("../../../data/all.csv").to_string()
}

#[test]fn invalid_bytes_are_preserved_for_recovery() {
    let db=sled::Config::new().temporary(true).open().unwrap();
    db.insert("course-ui-v1",b"not-json").unwrap();db.flush().unwrap();
    assert!(desktop_core::Model::open(db.clone()).is_err());
    assert_eq!(db.get("course-ui-v1").unwrap().unwrap().as_ref(),b"not-json");
}
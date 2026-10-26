mod common;

use common::tmp_root;
use slot_store::{read_language, write_language, Lang};
use tempfile::TempDir;

fn card() -> TempDir {
    let d = tmp_root();
    std::fs::create_dir_all(d.path().join("Config")).expect("config dir");
    d
}

#[test]
fn a_card_that_never_said_is_in_english() {
    let d = card();
    assert_eq!(
        read_language(d.path()),
        Lang::En,
        "unslot spoke English before there was a choice"
    );
}

#[test]
fn a_picked_language_survives_a_restart() {
    let d = card();
    write_language(d.path(), Lang::ZhCn).expect("write");
    assert_eq!(read_language(d.path()), Lang::ZhCn);

    write_language(d.path(), Lang::En).expect("write");
    assert_eq!(read_language(d.path()), Lang::En);
}

#[test]
fn a_language_file_nobody_can_read_falls_back_to_english() {
    let d = card();
    std::fs::write(d.path().join("Config/language.txt"), "klingon").expect("write");
    assert_eq!(read_language(d.path()), Lang::En);
}

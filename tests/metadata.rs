use rusty_dlp::engine::metadata::parse_preview;
use serde_json::json;

#[test]
fn flat_entries_preserve_indexes_and_unavailable_slots() {
    let data = json!({"_type":"playlist","title":"日本語 playlist","extractor":"youtube:tab","playlist_count":3,
        "entries":[{"id":"one","url":"https://www.youtube.com/watch?v=one","title":"First","playlist_index":1},null,
        {"id":"three","url":"https://www.youtube.com/watch?v=three","title":"Third","playlist_index":3}]});
    let preview = parse_preview("https://example.com/list", &data, 1).unwrap();
    assert_eq!(preview.entries.len(), 3);
    assert_eq!(preview.entries[2].index, 3);
    assert!(preview.entries[1].unavailable.is_some());
    assert!(!preview.entries[1].selected);
    assert!(preview.next_page.is_none());
}

#[test]
fn missing_optional_metadata_has_a_usable_preview() {
    let preview = parse_preview("https://example.com/a", &json!({"id":"one", "formats":[{"format_id":"140","ext":"m4a","acodec":"mp4a","vcodec":"none"}]}), 1).unwrap();
    assert_eq!(preview.title, "Untitled media");
    assert!(preview.duration.is_none());
    assert!(preview.formats[0].has_audio());
    assert!(!preview.formats[0].has_video());
}

#[test]
fn page_entry_count_is_not_mistaken_for_total_playlist_count() {
    let items = (0..50)
        .map(|i| json!({"id":format!("id{i}"),"url":format!("https://example.com/{i}")}))
        .collect::<Vec<_>>();
    let preview = parse_preview(
        "https://example.com/list",
        &json!({"_type":"playlist", "n_entries":50, "entries":items}),
        1,
    )
    .unwrap();
    assert_eq!(preview.next_page, Some(51));
}

#[test]
fn source_indexes_survive_filtered_playlist_entries() {
    let value = json!({"_type":"playlist", "requested_entries":[3,7], "entries":[{"url":"https://example.com/3"},{"url":"https://example.com/7"}]});
    let preview = parse_preview("https://example.com/list", &value, 1).unwrap();
    assert_eq!(preview.entries[0].index, 3);
    assert_eq!(preview.entries[1].index, 7);
}

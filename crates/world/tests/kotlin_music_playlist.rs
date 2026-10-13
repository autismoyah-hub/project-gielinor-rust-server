use gielinor_world::music_playlist::Playlist;
#[test]
fn original_kotlin_playlist_shuffle_and_cursor() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/music-playlist.expected"
    ))
    .expect("Run scripts/music-playlist-parity.ps1");
    assert_eq!(fixture.lines().count(), 129);
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('|').collect();
        let parse = |s: &str| {
            s.split(',')
                .map(|n| n.parse::<u16>().unwrap())
                .collect::<Vec<_>>()
        };
        if fields[0] == "D" {
            assert_eq!(
                Playlist(fields[1].parse().unwrap())
                    .shuffled(&[1, 2, 1, 3, 2])
                    .unwrap(),
                parse(fields[2]),
                "{line}"
            );
            continue;
        }
        let playlist = Playlist(fields[0].parse().unwrap());
        let count = fields[1].parse::<u16>().unwrap();
        assert_eq!(
            playlist.shuffled(&(1..=count).collect::<Vec<_>>()).unwrap(),
            parse(fields[2]),
            "{line}"
        );
        let next = playlist
            .next(count as usize)
            .map(|p| p.0.to_string())
            .unwrap_or("ERROR".into());
        assert_eq!(next, fields[3], "{line}");
    }
}

use simpalt::print::{Color, Div, Printer, Symbol};

const LENGTH: usize = 64;

pub fn render<P>(printer: &mut P) -> crate::Result
where
    P: Printer,
{
    render_track(printer, get_track())
}

#[cfg(target_os = "macos")]
fn get_track() -> Option<(bool, String)> {
    let info = media_remote::get_info()?;
    let playing = info.is_playing?;
    let track = format!(
        "{s_title} {title} {s_artist} {artist}",
        s_title = Symbol::Song,
        title = info.title?,
        s_artist = Symbol::Artist,
        artist = info.artist.unwrap_or_default()
    );
    Some((playing, track))
}

#[cfg(target_os = "linux")]
fn get_track() -> Option<(bool, String)> {
    let player_finder = mpris::PlayerFinder::new().ok()?;
    let mut paused = None;
    for player in player_finder.iter_players().ok()? {
        let Ok(player) = player else { continue };
        let Ok(metadata) = player.get_metadata() else {
            continue;
        };
        let Some(title) = metadata.title().map(str::trim).filter(|s| !s.is_empty()) else {
            continue;
        };
        let Ok(status) = player.get_playback_status() else {
            continue;
        };
        match status {
            mpris::PlaybackStatus::Playing => {
                return Some((
                    true,
                    make_track_name(title, &metadata.artists().unwrap_or_default()),
                ));
            }
            mpris::PlaybackStatus::Paused if paused.is_none() => {
                paused = Some(make_track_name(
                    title,
                    &metadata.artists().unwrap_or_default(),
                ));
            }
            _ => {}
        }
    }
    paused.map(|t| (false, t))
}

#[cfg(any(test, target_os = "linux"))]
fn make_track_name(title: &str, artists: &[&str]) -> String {
    [Symbol::Song.str(), " ", title.trim()]
        .iter()
        .copied()
        .chain(
            artists
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .flat_map(|s| [" ", Symbol::Artist.str(), " ", s]),
        )
        .fold(String::with_capacity(256), |mut acc, curr| {
            acc.push_str(curr);
            acc
        })
}

// Needs a pre-allocated String for track so that we can calculate the scroll
fn render_track<P>(printer: &mut P, track: Option<(bool, String)>) -> crate::Result
where
    P: Printer,
{
    if let Some((playing, track)) = track {
        let track = scroll(
            &track,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .ok(),
        );

        printer
            .div(Div::SlantTopRight, Color::Vga(234))?
            .fg(Color::Vga(37))
            .txt_gap(format_args!(
                "{icon} {track}",
                icon = if playing { Symbol::Play } else { Symbol::Pause }.str(),
            ))?;
    }

    Ok(())
}

fn scroll(track: &str, tick: Option<u64>) -> &str {
    let track_len = unicode_segmentation::UnicodeSegmentation::graphemes(track, true).count();

    if let Some(overflow) = track_len.checked_sub(LENGTH) // If it is smaller than LENGTH, return
                                                          // the whole track
        && let Ok(scroll_len) = u64::try_from(2 * overflow + 2) // The scroll goes forwards and
                                                                // backwards, so we need it to be twice
                                                                // the overflow size plus two extra
                                                                // steps (one extra step for each
                                                                // extremity)
        && let Some(tick) = tick.map(|t| t % scroll_len) // Each tick moves along the scroll space
        && let Ok(start) = usize::try_from(tick)
    {
        let start = start.saturating_sub(1); // Step 0 should count twice, so that the beginning stays for longer

        let start = if start <= overflow {
            start
        } else {
            2 * overflow - (start - 1)
        };

        let mut graphemes =
            unicode_segmentation::UnicodeSegmentation::grapheme_indices(track, true).skip(start);

        let fisrt_byte = graphemes.next().map_or(0, |(b, _)| b);
        let last_byte = graphemes.nth(LENGTH - 1).map_or(track.len(), |(b, _)| b);

        &track[fisrt_byte..last_byte]
    } else {
        track
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::expect;
    use super::*;

    fn test(track: Option<(bool, String)>) -> String {
        {
            let mut buffer = String::new();
            let mut printer = unsafe { simpalt::print::Ansi::new(buffer.as_mut_vec()) };
            render_track(&mut printer, track.clone()).unwrap();
            println!("{buffer}[m");
        }
        let mut buffer = String::new();
        let mut printer = unsafe { simpalt::print::Tmux::new(buffer.as_mut_vec()) };
        render_track(&mut printer, track).unwrap();
        buffer
    }

    #[test]
    fn render_no_track() {
        let result = test(None);
        let expected = expect(&result, []);
        assert_eq!(result, expected);
    }

    #[test]
    fn render_short_paused() {
        let result = test(Some((false, "A".repeat(16))));
        let expected = expect(
            &result,
            [
                "#[fg=colour234]",
                Div::SlantTopRight.str(),
                "#[fg=colour37,bg=colour234]",
                " ",
                Symbol::Pause.str(),
                " ",
                "A".repeat(16).as_str(),
                " ",
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn render_short_playing() {
        let result = test(Some((true, "A".repeat(16))));
        let expected = expect(
            &result,
            [
                "#[fg=colour234]",
                Div::SlantTopRight.str(),
                "#[fg=colour37,bg=colour234]",
                " ",
                Symbol::Play.str(),
                " ",
                "A".repeat(16).as_str(),
                " ",
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn render_long_paused() {
        let result = test(Some((false, "A".repeat(LENGTH * 2))));
        let expected = expect(
            &result,
            [
                "#[fg=colour234]",
                Div::SlantTopRight.str(),
                "#[fg=colour37,bg=colour234]",
                " ",
                Symbol::Pause.str(),
                " ",
                "A".repeat(LENGTH).as_str(),
                " ",
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn render_long_playing() {
        let result = test(Some((true, "A".repeat(LENGTH * 2))));
        let expected = expect(
            &result,
            [
                "#[fg=colour234]",
                Div::SlantTopRight.str(),
                "#[fg=colour37,bg=colour234]",
                " ",
                Symbol::Play.str(),
                " ",
                "A".repeat(LENGTH).as_str(),
                " ",
            ],
        );
        assert_eq!(result, expected);
    }

    mod scroll {
        use super::*;

        #[test]
        fn empty() {
            let track = scroll("", None);
            assert_eq!(track, "");
            for i in 0..16 {
                let track = scroll("", Some(i * 24));
                assert_eq!(track, "");
            }
        }

        #[test]
        fn less_than_scroll() {
            for i in 0..LENGTH {
                let original = ('0'..='9')
                    .chain('a'..='z')
                    .chain('A'..='Z')
                    .cycle()
                    .take(i)
                    .collect::<String>();
                let track = scroll(&original, None);
                assert_eq!(track, original);
                for i in 0..16 {
                    let track = scroll(&original, Some(i * 24));
                    assert_eq!(track, original);
                }
            }
        }

        #[test]
        fn edge_no_tick() {
            let original = ('0'..='9')
                .chain('a'..='z')
                .chain('A'..='Z')
                .cycle()
                .take(LENGTH + 1)
                .collect::<String>();

            let track = scroll(&original, None);
            assert_eq!(track, original);
        }

        #[test]
        fn edge_ticked() {
            let original = ('0'..='9')
                .chain('a'..='z')
                .chain('A'..='Z')
                .cycle()
                .take(LENGTH + 1)
                .collect::<String>();

            let track = scroll(&original, Some(0));
            assert_eq!(track, &original[..LENGTH]);

            let track = scroll(&original, Some(1));
            assert_eq!(track, &original[..LENGTH]);

            let track = scroll(&original, Some(2));
            assert_eq!(track, &original[1..]);

            let track = scroll(&original, Some(3));
            assert_eq!(track, &original[1..]);

            let track = scroll(&original, Some(4));
            assert_eq!(track, &original[..LENGTH]);

            let track = scroll(&original, Some(5));
            assert_eq!(track, &original[..LENGTH]);

            let track = scroll(&original, Some(6));
            assert_eq!(track, &original[1..]);

            let track = scroll(&original, Some(7));
            assert_eq!(track, &original[1..]);
        }

        #[test]
        fn bounce() {
            const OVERFLOW: usize = 16;

            let original = ('0'..='9')
                .chain('a'..='z')
                .chain('A'..='Z')
                .cycle()
                .take(LENGTH + OVERFLOW)
                .collect::<String>();

            let indices = [
                0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 16, 15, 14, 13, 12,
                11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
            ];

            for i in 0..OVERFLOW * 4 {
                let track = scroll(&original, u64::try_from(i).ok());
                let index = indices[i % indices.len()];
                assert_eq!(track, &original[index..index + LENGTH]);
            }
        }
    }

    mod track_name {
        use super::*;

        #[test]
        fn empty() {
            let track = make_track_name("bloink", &[]);
            assert_eq!(track, format!("{} bloink", Symbol::Song.str()));
        }

        #[test]
        fn blank() {
            let track = make_track_name("bloink", &["", " "]);
            assert_eq!(track, format!("{} bloink", Symbol::Song.str()));
        }

        #[test]
        fn one_artist() {
            let track = make_track_name("bloink", &["", " yoink	"]);
            assert_eq!(
                track,
                format!(
                    "{} bloink {} yoink",
                    Symbol::Song.str(),
                    Symbol::Artist.str()
                )
            );
        }

        #[test]
        fn artists() {
            let track = make_track_name("bloink", &["", " yoink	", "boom", "", "yo"]);
            assert_eq!(
                track,
                format!(
                    "{} bloink {} yoink {} boom {} yo",
                    Symbol::Song.str(),
                    Symbol::Artist.str(),
                    Symbol::Artist.str(),
                    Symbol::Artist.str(),
                )
            );
        }
    }
}

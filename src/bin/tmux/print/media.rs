use simpalt::print::{Color, Div, Printer, Symbol};

pub fn render<P>(printer: &mut P) -> crate::Result
where
    P: Printer,
{
    let Some((playing, track)) = get_mpris_track() else {
        return Ok(());
    };

    render_track(printer, playing, &track)
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
        let Some(title) = metadata.title().filter(|s| !s.is_empty()) else {
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

fn make_track_name(title: &str, artists: &[&str]) -> String {
    [Symbol::Song.str(), " ", title]
        .iter()
        .copied()
        .chain(
            artists
                .iter()
                .flat_map(|s| [" ", Symbol::Artist.str(), " ", s]),
        )
        .fold(String::with_capacity(256), |mut acc, curr| {
            acc.push_str(curr);
            acc
        })
}

fn render_track<P>(printer: &mut P, playing: bool, track: &str) -> crate::Result
where
    P: Printer,
{
    let track = if track.len() > 64
        && let Ok(len) = u64::try_from(2 * (track.len() - 64))
        && let Ok(secs) = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() % len)
        && let Ok(start) = usize::try_from(secs)
    {
        let start = if start < track.len() - 64 {
            start
        } else {
            2 * (track.len() - 64) - start
        };
        let mut graphemes =
            unicode_segmentation::UnicodeSegmentation::grapheme_indices(track, true).skip(start);

        let fisrt_byte = graphemes.next().map_or(0, |(b, _)| b);
        let last_byte = graphemes.nth(63).map_or(track.len(), |(b, _)| b);

        &track[fisrt_byte..last_byte]
    } else {
        track
    };

    printer
        .div(Div::SlantTopRight, Color::Vga(234))?
        .fg(Color::Vga(37))
        .txt_gap(format_args!(
            "{icon} {track}",
            icon = if playing { Symbol::Play } else { Symbol::Pause }.str(),
        ))?;

    Ok(())
}

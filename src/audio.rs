use crate::*;

fn make_audio_command(soundfile: PathBuf) -> Command {
    let mut c = Command::new("/usr/bin/mpv");
    c.arg("--no-terminal")
        .arg("--no-video")
        .arg(soundfile);
    c
}

pub async fn start_audio_task(mut event_rx: broadcast::Receiver<AppEvent>) {
    let mut child: Option<Child> = None;
    while let Some(ev) = event_rx.next().await {
        match ev {
            AppEvent::Ring(Alarm { soundfile: Some(soundfile), .. }) => {
                if child.is_none() {
                    let mut cmd = make_audio_command(soundfile.into());
                    child = cmd.spawn().ok();

                }
            },
            AppEvent::Ack => {
                if let Some(mut child) = child.take() {
                    child.kill();
                    child.status().await;
                }
            },
            _ => { continue; }
        }
    }
    unreachable!()
}

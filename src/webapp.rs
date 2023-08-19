use maud::{html, Markup, Render, DOCTYPE};
use tide::{
    http::{Mime, StatusCode},
    sse, Request, Response,
};
use std::net::SocketAddr;
use std::str::FromStr;
use crate::*;

#[derive(Clone)]
pub struct WebState {
    cmd_tx: Sender<AppCommand>,
    alarm_list: Arc<RwLock<Vec<Alarm>>>,
    tz: TimeZone,
    ltt: String,
    // update_tx: Sender<
}

impl WebState {
    pub fn new(cmd_tx: Sender<AppCommand>, 
               alarm_list: Arc<RwLock<Vec<Alarm>>>,
               tz: TimeZone,
               ) -> Self {
        let ltt = tz.as_ref().local_time_types().last().map(|l| l.time_zone_designation()).unwrap_or("").to_owned();
        Self {
            cmd_tx,
            alarm_list,
            tz,
            ltt,
        }
    }
}

pub async fn server(state: WebState, port: u16) {
    info!("Starting web server");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut app = tide::with_state(state);
    // app.at("/").get(main_page).post(main_page_post);
    // app.at("/sse")
    //     .get(sse::endpoint(|r, s| sse_handler(r, s)));
    app.listen(addr).await;
}


//
// cut and paste
//

// async fn sse_handler(state: Request<WebState>, sender: sse::Sender) -> tide::Result<()> {
//     let mut chan = state.state().subscribe();
//     while let Ok(msg) = chan.recv().await {
//         match msg {
//             UpdateType::Mode => {
//                 let lock = state.state().appstate.read().await;
//                 let partial = mode_partial(&lock.mode).into_string();
//                 sender.send("mode", partial, None).await?;
//             },
//             UpdateType::Queue => {
//                 let lock = state.state().appstate.read().await;
//                 let q = &lock.queue;
//                 let partial = queue_partial(q.iter()).into_string();
//                 sender.send("queue", partial, None).await?;
//                 drop(lock);
//             },
//         }
//     }
//     Ok(())
// }

// async fn main_page_post(mut req: Request<WebState>) -> tide::Result<Response> {
//     if let Ok(x) = req.body_string().await {
//         if let Ok(cmd) = serde_json::from_str(&x) {
//             trace!("Command received via POST: {cmd:?}");
//             req.state().cmd_tx.send(cmd).await;
//         } else {
//             error!("Bad command received via POST");
//             return Ok(Response::new(200));
//         }
//         // HACK: wasteful double broadcast…
//         req.state().update_tx.broadcast(UpdateType::Queue).await;
//         req.state().update_tx.broadcast(UpdateType::Mode).await;
//     }
//     Ok(Response::new(200))
// }

async fn main_page(state: Request<WebState>) -> tide::Result<Response> {
    // let script = maud::PreEscaped(
    //     r#"
    //         let eventSource = new EventSource("sse");
    //         eventSource.addEventListener("queue", (event) => {
    //             document.getElementById("queue").innerHTML = event.data;
    //         });
    //         eventSource.addEventListener("mode", (event) => {
    //             document.getElementById("mode").innerHTML = event.data;
    //         });

    //         function kick() {
    //             fetch("sse", { method: "POST" });
    //         }
    //         kick();

    //         function pauseresume() {
    //             let cmd = JSON.stringify("PauseResume");
    //             fetch("/", { method: "POST", body: cmd});
    //         }
    //         function pause() {
    //             let cmd = JSON.stringify("Pause");
    //             fetch("/", { method: "POST", body: cmd });
    //         }
    //         function resume() {
    //             let cmd = JSON.stringify("Resume");
    //             fetch("/", { method: "POST", body: cmd });
    //         }
    //         function cancel() {
    //             let cmd = JSON.stringify("Cancel");
    //             fetch("/", { method: "POST", body: cmd });
    //         }
    //         function addurl() {
    //             let url = document.getElementById("url").value;
    //             document.getElementById("url").value = "";
    //             let cmd = JSON.stringify({AddUrl: url});
    //             fetch("/", { method: "POST", body: cmd });
    //         }
    // "#,
    // );

    // let lock = state.state().appstate.read().await;
    // let mp = mode_partial(&lock.mode);
    // let qp = queue_partial(lock.queue.iter());
    // drop(lock);

    let tzref = state.state().tz.as_ref();
    let now = UnixMoment::now();
    let dt = DateTime::now(tzref).unwrap();
    let current_time = format!("{} {}", humanize_datetime(dt), state.state().ltt);

    let markup = html! {
        (DOCTYPE)
        html {
            head { title { "clockd" } }
            body {
                p { strong { (current_time) } }
                // div #mode { (mp) }
                // h2 { "Queue" }
                // button onclick="pauseresume()" { strong { "PAUSE/RESUME" } }
                // // button onclick="pause()" { strong { "PAUSE" } }
                // // button onclick="resume()" { strong { "RESUME" } }
                // button onclick="cancel()" { strong { "CANCEL" } }
                // form {
                //     input #url for="url" type="text" name="url" {}
                // }
                // button onclick="addurl()" { "Add URL" }
                // div #queue { (qp) }
            }
        }
        // script { (script) }
    };
    let body = markup.render().into_string();
    let mime = Mime::from_str("text/html;charset=utf-8").unwrap();
    let mut resp = Response::new(StatusCode::Ok);
    resp.set_body(body);
    resp.set_content_type(mime);
    Ok(resp)
}

// fn mode_partial(m: &Mode) -> Markup {
//     html! {
//         @match m {
//             Mode::Idle => h1 {"Idle" } p {},
//             Mode::Starting { title } => h1 {"Starting"} (title_partial(title.as_ref())),
//             Mode::Download { title, progress, rater, eta, .. } =>
//                 h1 { "Downloading" }
//                 (title_partial(title.as_ref()))
//                 div { (progress_partial(*progress, *eta, rater)) }
//             ,
//             Mode::Stuck { title } => h1 {"Stuck"} (title_partial(title.as_ref())),
//             Mode::Hold { reason, title } => h1{"Hold: " (reason)} (title_partial(title.as_ref())),
//             _ => h1 { "Unknown state" },
//         }
//     }
//     .render()
// }

// fn title_partial(title: Option<&String>) -> Markup {
//     html! {
//         p { "Current title: " 
//         strong { @if let Some(title) = title { (title) } @else { "None" } }
//         }
//     }
//     .render()
// }

// fn progress_partial(progress: Option<f64>, eta: Option<u64>, rater: &RollingRate) -> Markup {
//     // let percent = progress.map(|x| (x * 100.0) as i64);
//     let eta = eta.map(humanize_seconds);
//     let rate = rater.rate().map(humanize_rate);
//     html! {
//         @if let Some(percent) = progress {
//             progress value=(percent) {}
//         } @else {
//             progress value="0" {}
//         }
//         @if let Some(eta) = eta {
//             span { " " (eta) }
//         }
//         @if let Some(rate) = rate {
//             span { " @ " (rate) }
//         }
//     }
//     .render()
// }

// fn queue_partial<'a>(q: impl std::iter::Iterator<Item = &'a Url>) -> Markup {
//     html! {
//         ul {
//             @for url in q {
//                 li { (url) }
//             }
//         }
//     }
//     .render()
// }

fn current_time_partial(dt: DateTime, ltt: String) {
    // let tzref = state.state().tz.as_ref();
    // let now = UnixMoment::now();
    // let dt = DateTime::now(tzref).unwrap();
    // let current_time = format!("{} {}", humanize_datetime(dt), state.state().ltt);
    // html! {
    //     p { strong { (current_time) } }
    // }
    // .render()
}

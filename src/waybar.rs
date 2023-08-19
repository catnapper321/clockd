use crate::*;
use serde::Serialize;
use tide::http::server::allow;
use std::fmt::Write;
use std::sync::RwLockReadGuard;

const TOOLTIP_HRULE: &str = "--------------------";

#[derive(Serialize)]
struct WaybarUpdate<'a> {
    text: String,
    class: String,
    tooltip: String,
    #[serde(skip)]
    time_display: String,
    #[serde(skip)]
    time_display_full: String,
    #[serde(skip)]
    now: UnixMoment,
    #[serde(skip)]
    tzref: tz::TimeZoneRef<'a>,
}

impl<'a> WaybarUpdate<'a> {
    fn new(tzref: TimeZoneRef<'a>) -> Self {
        Self {
            text: String::new(),
            class: String::new(),
            tooltip: String::new(),
            time_display: String::new(),
            time_display_full: String::new(),
            now: UnixMoment::now(),
            tzref,
        }
    }
    fn update_time(&mut self) {
        self.now = UnixMoment::now();
        let datetime = self.now.as_datetime(self.tzref);
        self.time_display = datetime.map(humanize_datetime).unwrap_or(String::from("unknown"));
        self.time_display_full = datetime.map(humanize_datetime_full).unwrap_or(String::from("unknown"));
    }
}

#[derive(Debug, Clone, Copy)]
enum WaybarDisplayMode {
    Clock,
    NextPending,
    LeadAlarm,
}

fn tooltip_section(x: &mut String, heading: &str) {
    writeln!(x, "{heading}:\n{TOOLTIP_HRULE}");
}

fn update_tooltip<'a, 'b>(
    alarm_list: RwLockReadGuard<AlarmList>,
    update: &mut WaybarUpdate,
) {
    update.tooltip.clear();
    tooltip_section(&mut update.tooltip, "Pending Alarms");
    for a in alarm_list.pending() {
        let s = update.now.seconds_until(a.end_t);
        writeln!(update.tooltip, "{} {}", a.name, humanize_seconds(s));
    }
    tooltip_section(&mut update.tooltip, "Current Alarms");
    for a in alarm_list.alarming() {
        let s = update.now.seconds_until(a.end_t);
        writeln!(update.tooltip, "{} {}", a.name, humanize_seconds(s));
    }
    writeln!(update.tooltip, "{TOOLTIP_HRULE}");
    writeln!(update.tooltip, "{}", update.time_display_full);
}

fn update_text_clock(update: &mut WaybarUpdate, pending_len: usize) {
    update.text = format!("{:>2}⏲  {}", pending_len, update.time_display);
}

fn update_text_alarm<'a>(update: &mut WaybarUpdate, pending_len: usize, alarm: Option<&Alarm>, now: UnixMoment) {
    let (name, s) = if let Some(a) = alarm {
        let s = now.seconds_until(a.end_t);
        (a.name.as_str(), s)
    } else {
        ("NO ALARM", 0)
    };
    update.text = format!("{:>2}⏲  {} {}", pending_len, name, humanize_seconds(s));
}

fn update_display(
    update: &mut WaybarUpdate,
    alarm_list: RwLockReadGuard<AlarmList>,
    display_mode: WaybarDisplayMode,
    ) {
    let len = alarm_list.pending_len();
    match display_mode {
        WaybarDisplayMode::Clock => update_text_clock(update, len),
        WaybarDisplayMode::NextPending => update_text_alarm(update, len, alarm_list.next_alarm(), update.now),
        WaybarDisplayMode::LeadAlarm => update_text_alarm(update, len, alarm_list.lead_alarming(), update.now),
    }
    update_tooltip(alarm_list, update);
}

pub async fn waybar_display_server(
    alarm_list: Arc<RwLock<AlarmList>>,
    mut event_rx: broadcast::Receiver<AppEvent>,
    timezone: tz::TimeZone,
    ) {
    let mut update = WaybarUpdate::new(timezone.as_ref());
    let mut auto_switch_mode = true;
    let mut display_mode = WaybarDisplayMode::Clock;
    let mut now = UnixMoment::now();
    let tzref = timezone.as_ref();

    // initial update
    {
        update.update_time();
        let x = alarm_list.read().unwrap();
        update_display(&mut update, x, display_mode);
    }

    while let Some(ev) = event_rx.next().await {

        // TEST:
        // println!("{}", serde_json::to_string(&update).unwrap());
        fdrecv::print_json_to_fds(&update);

        match ev {
            AppEvent::Ring(_) => {
                display_mode = WaybarDisplayMode::LeadAlarm;
                auto_switch_mode = false;
                update.class = String::from("ringing");
                update.update_time();
                let x = alarm_list.read().unwrap();
                update_display(&mut update, x, display_mode);
            },
            AppEvent::Minute(_) => {
                update.update_time();
                let x = alarm_list.read().unwrap();
                update_display(&mut update, x, display_mode)
            },
            AppEvent::Tick => {
                if auto_switch_mode {
                    display_mode = WaybarDisplayMode::NextPending;
                    trace!("display switched to NextPending");
                }
                update.update_time();
                let x = alarm_list.read().unwrap();
                update_display(&mut update, x, display_mode);
            },
            AppEvent::AlarmListUpdate => {
                let x = alarm_list.read().unwrap();
                update_tooltip(x, &mut update);
            },
            AppEvent::Ack => {
                auto_switch_mode = true;
                display_mode = WaybarDisplayMode::Clock;
                update.class.clear();
                let x = alarm_list.read().unwrap();
                update_display(&mut update, x, display_mode);
            },
            AppEvent::SwitchDisplay => {
                auto_switch_mode = false;
                let x = alarm_list.read().unwrap();
                match display_mode {
                    WaybarDisplayMode::Clock => {
                        // do not switch if no alarm is pending
                        if x.next_alarm().is_some() {
                            display_mode = WaybarDisplayMode::NextPending;
                        }
                    },
                    WaybarDisplayMode::NextPending => {
                        display_mode = WaybarDisplayMode::Clock;
                    },
                    WaybarDisplayMode::LeadAlarm => {},
                }
                update_display(&mut update, x, display_mode);
            },
            AppEvent::NewListener => {},
        }

    }
    unreachable!()
}

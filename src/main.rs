use crate::time_param::TimeParam;
use log::{LevelFilter, debug};
mod my_lib;
mod time_param;
use chrono::Local;
use fern;
use std::fs;

fn main() {
    let path = "app.log";
    let _ = fs::remove_file(path);
    // Инициализация логгера fern с записью в файл app.log
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Debug) // Установите LevelFilter::Info для менее детальных логов
        .chain(std::io::stdout()) // Вывод в консоль
        .chain(fern::log_file("app.log").expect("Не удалось открыть файл логов")) // Запись в файл
        .apply()
        .expect("Не удалось инициализировать логгер");
    let time_param = TimeParam::new();
    let trs = &time_param.threshold;
    let r_pos = &time_param.r_pos;
    // debug!("rpos: {:.?}", &r_pos[..10]);
}

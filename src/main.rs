#![windows_subsystem = "windows"]
use crate::disp::get_coef_disp;
use crate::fibr::Fibr;
use crate::my_lib::{Leads, get_coef_fibr, get_coef_p};
use crate::pacient::Pacient;
use crate::time_param::TimeParam;
use chrono::Local;
use fern;
use fltk::enums::Color;
use fltk::{app, button::Button, draw, frame::Frame, misc::Progress, prelude::*, window::Window};
use fltk_theme::{ThemeType, WidgetTheme, widget_themes};
use log::{LevelFilter, debug};
use npy_writer::NumpyWriter;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::process;
use std::thread;

mod disp;
mod fibr;
mod my_lib;
mod pacient;
mod time_param;
mod zub_p;

fn main() {
    let path = "app_f.log";
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
        .chain(fern::log_file("app_f.log").expect("Не удалось открыть файл логов")) // Запись в файл
        .apply()
        .expect("Не удалось инициализировать логгер");

    thread::spawn(move || {
        let a = app::App::default().with_scheme(app::Scheme::Gtk);
        app::background(240, 240, 240);
        // let widget_theme = WidgetTheme::new(ThemeType::Metro);
        // widget_theme.apply();
        let mut wind = Window::default().with_size(450, 90);
        wind.set_border(false);
        wind.make_modal(true);
        wind.draw(|w| {
            // Сначала заставляем окно отрисовать свой стандартный фон и виджеты
            w.draw_children();
            // Устанавливаем темно-серый цвет (или черный: draw::Color::Black)
            draw::set_draw_rgb_color(40, 40, 40);
            // Задаем толщину линии в 1 пиксель
            draw::set_line_style(draw::LineStyle::Solid, 1);
            // Рисуем прямоугольник по внутреннему контуру окна
            // Координаты: x=0, y=0, ширина, высота
            draw::draw_rect(0, 0, w.w(), w.h());
        });

        let mut progress_bar = Progress::new(10, 20, 350, 20, None); // 10, 20, 430, 20
        progress_bar.set_color(Color::from_rgb(240, 240, 240));
        progress_bar.set_selection_color(Color::from_rgb(0, 78, 215));
        progress_bar.set_maximum(1000.0);
        let mut timer = Frame::new(360, 20, 80, 20, "0.0 сек");
        let mut label = Frame::new(65, 50, 200, 26, "Поиск эпизодов ФП");

        let mut btn_cancel = Button::new(335, 50, 100, 26, "Прервать");
        btn_cancel.set_color(Color::from_rgb(240, 240, 240));
        btn_cancel.set_callback(move |_| {
            process::exit(0);
        });

        wind.end();
        wind.show();
        wind.center_screen();

        thread::spawn({
            let mut timer = timer.clone();
            let mut counter = 0.0;
            let step = 0.1;
            move || {
                loop {
                    counter += step;
                    timer.set_label(&format!("{:.1} сек", counter));
                    app::sleep(0.1);
                    app::awake();
                    timer.parent().unwrap().redraw();
                }
            }
        });

        thread::spawn({
            let mut progress_bar = progress_bar.clone();
            let mut counter = 0.0;
            let step = 1.0;
            move || {
                loop {
                    counter += step;
                    if counter > progress_bar.maximum() {
                        counter = 0.0;
                    }
                    progress_bar.set_value(counter);
                    app::sleep(0.012);
                    app::awake();
                    progress_bar.parent().unwrap().redraw();
                }
            }
        });
        a.run().unwrap();
    });

    let leads = Leads::new();
    debug!("Leads::new()");
    // let lead1 = &leads.lead1;
    // let mut file = File::create("lead1.npy").unwrap();
    // lead1.write_npy(&mut file).unwrap();
    // let lead2 = &leads.lead2;
    // let mut file = File::create("lead2.npy").unwrap();
    // lead2.write_npy(&mut file).unwrap();
    // let lead3 = &leads.lead3;
    // let mut file = File::create("lead3.npy").unwrap();
    // lead3.write_npy(&mut file).unwrap();

    let time_param = TimeParam::new();
    debug!("TimeParam::new()");
    let intervals = &time_param.intervals;
    let mut file = File::create("intervals.npy").unwrap();
    intervals.write_npy(&mut file).unwrap();
    let clear_intervals = &time_param.clear_intervals;
    let mut file = File::create("clear_intervals.npy").unwrap();
    clear_intervals.write_npy(&mut file).unwrap();
    let trs = &time_param.threshold;
    let mut file = File::create("trs.npy").unwrap();
    trs.write_npy(&mut file).unwrap();

    let r_pos = &time_param.r_pos;
    let coef_p = get_coef_p(&leads, &time_param);
    let mut file = File::create("coef_p.npy").unwrap();
    coef_p.clone().write_npy(&mut file).unwrap();
    debug!("get_coef_p");
    let coef_disp = get_coef_disp(&time_param);
    let mut file = File::create("coef_disp.npy").unwrap();
    coef_disp.clone().write_npy(&mut file).unwrap();
    debug!("get_coef_disp");
    let coef_fibr = get_coef_fibr(&coef_p, &coef_disp, &time_param);
    let mut file = File::create("coef_fibr.npy").unwrap();
    coef_fibr.clone().write_npy(&mut file).unwrap();
    debug!("get_coef_fibr");

    let mask: Vec<usize> = vec![1; coef_fibr.len()];
    let pacient = Pacient::new();
    let start_time_in_samples = pacient.start_time_in_samples;
    let fibr = Fibr::new(&coef_fibr, &trs, &r_pos, start_time_in_samples, &mask);

    let mut text = pacient.file_name.clone();
    text.push_str("\n\n");
    // let (h, m, s) = pacient.start_time.clone();
    // let start_time = format!("Начало записи {}:{}:{}\n\n", h, m, s);
    // text.push_str(&start_time);

    for i in 0..fibr.start_ind_arr.len() {
        let (d1, h1, m1, s1) = pacient.get_time_from_samples(fibr.start_ind_arr[i] as u32);
        let s1 = s1 % 60;
        let m1 = m1 % 60;
        let h1 = h1 % 24;
        let (d2, h2, m2, s2) = pacient.get_time_from_samples(fibr.stop_ind_arr[i] as u32);
        let s2 = s2 % 60;
        let m2 = m2 % 60;
        let h2 = h2 % 24;
        let (_d3, h3, m3, s3) = pacient.get_time_from_samples(fibr.diff_stop_start[i] as u32);
        let s3 = s3 % 60;
        let m3 = m3 % 60;

        let episod = format!(
            "{}, {}, {}     ({}д {}:{}:{} по {}д {}:{}:{}  длит. {}:{}:{})\n",
            fibr.start_ind_arr[i],
            fibr.stop_ind_arr[i],
            fibr.diff_stop_start[i],
            d1 + 1,
            h1,
            m1,
            s1,
            d2 + 1,
            h2,
            m2,
            s2,
            h3,
            m3,
            s3
        );
        text.push_str(&episod);
    }

    let num_of_episodes = fibr.start_ind_arr.len();
    let num_of_samples_in_fibr: usize = fibr.diff_stop_start.iter().sum();
    let (d, h, m, s) = pacient.get_time_from_samples(num_of_samples_in_fibr as u32);
    let s = s % 60;
    let m = m % 60;

    let sum_time_fibr = format!(
        "\nВсего эпизодов фибриляции: {}\n\n{} (Общее время эпизодов: {}:{}:{})\n\n",
        num_of_episodes, num_of_samples_in_fibr, h, m, s
    );
    text.push_str(&sum_time_fibr);

    // let total_time = format!(
    //     "Общее время записи: {}:{}:{}",
    //     pacient.total_time.0, pacient.total_time.1, pacient.total_time.2
    // );
    // text.push_str(&total_time);
    let file = File::create("c:\\EcgVar\\F.txt").expect("Не удалось создать файл");
    debug!("File::create c:\\EcgVar\\F.txt");
    let mut writer = BufWriter::new(file);
    let text = text.as_bytes();
    writer.write_all(text).expect("Не удалось записать в файл");
}

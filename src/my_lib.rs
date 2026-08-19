use crate::multi_thread_func;
use log::debug;
use native_dialog::{DialogBuilder, MessageLevel};
use std::fs::File;
use std::io::Read;
use std::thread;

pub struct Leads {
    pub lead1: Vec<f32>,
    pub lead2: Vec<f32>,
    pub lead3: Vec<f32>,
}
impl Leads {
    pub fn new() -> Leads {
        let files = glob::glob("*.ecg").expect("Failed to read files");
        let fname = files.filter_map(Result::ok).next();
        let fname = match fname {
            Some(fname) => fname,
            None => {
                DialogBuilder::message()
                    .set_level(MessageLevel::Error)
                    .set_title("Ошибка")
                    .set_text("В текущей директории не найден файл с расширением .ecg")
                    .alert()
                    .show()
                    .unwrap();
                fname.unwrap()
            }
        };
        debug!("{:?}", fname);
        let mut file = File::open(&fname).expect("Failed to open file");

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Failed to read file");
        let new_len = buffer.len() / 2 * 2; // Округление вниз до ближайшего четного
        buffer.truncate(new_len);

        // let ecg_data: Vec<i16> = buffer[1024..1024 + 3_000_000 * 6]
        let ecg_data: Vec<i16> = buffer[1024..]
            .chunks(2)
            .map(|chunk| ((chunk[1] as i32) << 8 | (chunk[0] as i32)) as i16)
            .collect();

        let mut ch_1: Vec<f32> = ecg_data
            .iter()
            .step_by(3)
            .map(|&val| (-val as f32 + 1024.0) / 100.0)
            .collect();

        let mut ch_2: Vec<f32> = ecg_data
            .iter()
            .skip(1)
            .step_by(3)
            .map(|&val| (-val as f32 + 1024.0) / 100.0)
            .collect();

        let mut ch_3: Vec<f32> = ecg_data
            .iter()
            .skip(2)
            .step_by(3)
            .map(|&val| (-val as f32 + 1024.0) / 100.0)
            .collect();

        let len1 = ch_1.len();
        let len2 = ch_2.len();
        let len3 = ch_3.len();
        let min_len = len1.min(len2).min(len3);
        ch_1.truncate(min_len);
        ch_2.truncate(min_len);
        ch_3.truncate(min_len);

        let t_1 = thread::spawn(|| {
            ch_1 = multi_thread_func!(cut_impuls, &ch_1);
            ch_1 = multi_thread_func!(cut_impuls, &ch_1);
            ch_1 = multi_thread_func!(del_isoline, &ch_1);
            ch_1
        });

        let t_2 = thread::spawn(|| {
            ch_2 = multi_thread_func!(cut_impuls, &ch_2);
            ch_2 = multi_thread_func!(cut_impuls, &ch_2);
            ch_2 = multi_thread_func!(del_isoline, &ch_2);
            ch_2
        });

        let t_3 = thread::spawn(|| {
            ch_3 = multi_thread_func!(cut_impuls, &ch_3);
            ch_3 = multi_thread_func!(cut_impuls, &ch_3);
            ch_3 = multi_thread_func!(del_isoline, &ch_3);
            ch_3
        });
        ch_1 = t_1.join().unwrap();
        ch_2 = t_2.join().unwrap();
        ch_3 = t_3.join().unwrap();

        let leads = Leads {
            lead1: ch_1,
            lead2: ch_2,
            lead3: ch_3,
        };
        leads
    }
}

/// Макрос для удобного вызова multi_thread_func с обратной совместимостью.
///
/// Примеры использования:
/// ```
/// // Старый стиль (один параметр) - работает без изменений
/// let result = multi_thread_func!(filter_lp_35d0, &signal);
///
/// // С дополнительными параметрами через замыкание
/// let coef: f32 = 1.5;
/// let result = multi_thread_func!(|ch| filter_with_coef(ch, coef), &signal);
///
/// // Несколько дополнительных параметров
/// let result = multi_thread_func!(|ch| my_func(ch, arg1, arg2), &signal);
/// ```
#[macro_export]
macro_rules! multi_thread_func {
    ($f:ident, $ch:expr) => {
        $crate::my_lib::multi_thread_func_impl($f, $ch)
    };
    ($f:expr, $ch:expr) => {
        $crate::src::multi_thread_func_closure($f, $ch)
    };
}

/// Базовая реализация для функций типа fn(&Vec<f32>) -> Vec<f32> (обратная совместимость)
pub fn multi_thread_func_impl<F, R>(f: F, ch: &Vec<f32>) -> Vec<f32>
where
    F: Fn(&Vec<f32>) -> R + Send + Sync + Copy + 'static,
    R: Into<Vec<f32>> + Send + 'static,
{
    let len_part = ch.len() / 8;
    if len_part > 2000 {
        let chunks = [
            (0usize, len_part + 100),
            (len_part - 100, len_part * 2 + 100),
            (len_part * 2 - 100, len_part * 3 + 100),
            (len_part * 3 - 100, len_part * 4 + 100),
            (len_part * 4 - 100, len_part * 5 + 100),
            (len_part * 5 - 100, len_part * 6 + 100),
            (len_part * 6 - 100, len_part * 7 + 100),
            (len_part * 7 - 100, ch.len()),
        ];

        let mut handles = Vec::with_capacity(8);

        for (i, (start, end)) in chunks.iter().enumerate() {
            let part = ch[*start..*end].to_vec();
            let handle = thread::spawn(move || {
                let fpart: Vec<f32> = f(&part).into();
                let len_part_len = part.len();
                if i == 0 {
                    fpart[..len_part_len - 100].to_vec()
                } else if i == 7 {
                    fpart[100..].to_vec()
                } else {
                    fpart[100..len_part_len - 100].to_vec()
                }
            });
            handles.push(handle);
        }

        let results: Vec<Vec<f32>> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        results.into_iter().flatten().collect()
    } else {
        f(ch).into()
    }
}

/// Реализация для замыканий с произвольными захваченными переменными
/// Позволяет передавать дополнительные параметры через замыкание
pub fn multi_thread_func_closure<F, R>(f: F, ch: &Vec<f32>) -> Vec<f32>
where
    F: Fn(&Vec<f32>) -> R + Send + Sync + Clone + 'static,
    R: Into<Vec<f32>> + Send + 'static,
{
    let len_part = ch.len() / 8;
    if len_part > 2000 {
        let chunks = [
            (0usize, len_part + 100),
            (len_part - 100, len_part * 2 + 100),
            (len_part * 2 - 100, len_part * 3 + 100),
            (len_part * 3 - 100, len_part * 4 + 100),
            (len_part * 4 - 100, len_part * 5 + 100),
            (len_part * 5 - 100, len_part * 6 + 100),
            (len_part * 6 - 100, len_part * 7 + 100),
            (len_part * 7 - 100, ch.len()),
        ];

        let mut handles = Vec::with_capacity(8);

        for (i, (start, end)) in chunks.iter().enumerate() {
            let part = ch[*start..*end].to_vec();
            let f_clone = f.clone();
            let handle = thread::spawn(move || {
                let fpart: Vec<f32> = f_clone(&part).into();
                let len_part_len = part.len();
                if i == 0 {
                    fpart[..len_part_len - 100].to_vec()
                } else if i == 7 {
                    fpart[100..].to_vec()
                } else {
                    fpart[100..len_part_len - 100].to_vec()
                }
            });
            handles.push(handle);
        }

        let results: Vec<Vec<f32>> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        results.into_iter().flatten().collect()
    } else {
        f(ch).into()
    }
}

pub fn my_diff_abs(ch: &Vec<f32>) -> Vec<f32> {
    let mut ch_copy = ch.to_owned();
    ch_copy.remove(0);
    ch_copy.push(*ch_copy.last().unwrap());
    let mut diff_ch: Vec<f32> = ch
        .iter()
        .zip(ch_copy.iter())
        .map(|(val1, val2)| (val1 - val2).abs())
        .collect();
    let sum: f32 = diff_ch.iter().sum();
    let count: f32 = diff_ch.len() as f32;
    let mean: f32 = sum / count;
    diff_ch[0] = mean;
    diff_ch[1] = mean;
    diff_ch[2] = mean;
    diff_ch
}

fn get_mean_diff(ch: &Vec<f32>) -> f32 {
    let d_ch = my_diff_abs(&ch);
    let sum_d_ch: f32 = d_ch.iter().sum();
    let mean_d_ch = sum_d_ch / d_ch.len() as f32;
    let (count, sum): (usize, f32) = d_ch
        .iter()
        .filter(|&x| *x > mean_d_ch)
        .fold((0, 0.0), |(count, sum), &x| (count + 1, sum + x));
    let mean_d_ch1 = sum / count as f32;
    let (count, sum): (usize, f32) = d_ch
        .iter()
        .filter(|&x| *x > mean_d_ch1)
        .fold((0, 0.0), |(count, sum), &x| (count + 1, sum + x));
    let mean_d_ch2 = sum / count as f32;
    let out = mean_d_ch + mean_d_ch1 + mean_d_ch2;
    out
}

pub fn cut_impuls(ch: &Vec<f32>) -> Vec<f32> {
    let mut out = ch.clone();
    let mean_d = get_mean_diff(&ch);
    for i in 5..ch.len() - 6 {
        let d_out0 = (&out[i] - &out[i - 1]) / &mean_d;
        let d_out1 = (&out[i + 1] - &out[i]) / &mean_d;
        let d_out2 = (&out[i + 2] - &out[i + 1]) / &mean_d;
        let sign0 = d_out0.signum();
        let sign1 = d_out1.signum();
        let sign2 = d_out2.signum();
        let abs0 = d_out0.abs();
        let abs1 = d_out1.abs();
        let abs2 = d_out2.abs();
        if (sign0 != sign1)
            && (sign1 != sign2)
            && (abs1 > 1.4)
            && (((abs0 - abs1).abs() < abs1 * 0.88) || ((abs1 - abs2).abs() < abs1 * 0.88))
        {
            let win: Vec<&f32> = ch.iter().skip(i - 5).take(11).collect();
            let mut sort_win = win.to_owned();
            sort_win.sort_by(|a, b| a.partial_cmp(b).unwrap());
            out[i - 2] = *sort_win[5];
            out[i - 1] = *sort_win[5];
            out[i] = *sort_win[5];
            out[i + 1] = *sort_win[5];
            out[i + 2] = *sort_win[5];
            out[i + 3] = *sort_win[5];
            if ((&out[i + 4] - &out[i + 3]) / &mean_d).abs() > 0.15 {
                out[i + 4] = *sort_win[5];
                out[i + 5] = *sort_win[5];
            }
        }
        if (sign0 != sign1)
            && (abs0 > 1.0)
            && (abs1 > 1.0)
            && ((abs0 - abs1).abs() < (abs0 + abs1) * 0.5)
        {
            let win: Vec<&f32> = ch.iter().skip(i - 5).take(11).collect();
            let mut sort_win = win.to_owned();
            sort_win.sort_by(|a, b| a.partial_cmp(b).unwrap());
            out[i - 2] = *sort_win[5];
            out[i - 1] = *sort_win[5];
            out[i] = *sort_win[5];
            out[i + 1] = *sort_win[5];
            out[i + 2] = *sort_win[5];
            if ((&out[i + 2] - &out[i + 1]) / &mean_d).abs() > 0.05 {
                out[i + 2] = *sort_win[5];
                out[i + 3] = *sort_win[5];
            }
        }
    }
    out
}

pub fn del_isoline(ch: &Vec<f32>) -> Vec<f32> {
    let mut out = ch.to_owned();
    let len_ch = out.len();
    let len_win: usize = 120; // = 120
    let half_win: usize = len_win / 2;
    for i in (half_win..len_ch - half_win).step_by(3) {
        let mut win: Vec<f32> = ch[i - half_win..i + half_win].to_vec();
        let med_win: f32 = median_f32(&mut win);
        out[i - 2] = ch[i - 2] - med_win;
        out[i - 1] = ch[i - 1] - med_win;
        out[i] = ch[i] - med_win;
        out[i + 1] = ch[i + 1] - med_win;
    }
    let value_hw = out[half_win];
    out[..half_win].fill(value_hw);
    out
}

pub fn median_f32(vec: &mut Vec<f32>) -> f32 {
    let len = vec.len();
    let mut out: f32 = 1.0;
    if len == 1 {
        out = vec[0];
    } else if len == 2 {
        out = (vec[0] + vec[1]) / 2.0;
    } else if len >= 300 {
        let mut temp_vec: Vec<f32> = vec![];
        for i in (0..len).step_by(4) {
            temp_vec.push(vec[i]);
        }
        temp_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let small_len = temp_vec.len();
        if small_len % 2 == 0 {
            // Если четное количество элементов
            let mid1 = temp_vec[small_len / 2 - 1];
            let mid2 = temp_vec[small_len / 2];
            out = (mid1 + mid2) / 2.0;
        } else {
            // Если нечетное количество элементов
            out = temp_vec[small_len / 2];
        }
    } else if len > 2 && len < 300 {
        // Сортируем вектор
        vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if len % 2 == 0 {
            // Если четное количество элементов
            let mid1 = vec[len / 2 - 1];
            let mid2 = vec[len / 2];
            out = (mid1 + mid2) / 2.0;
        } else {
            // Если нечетное количество элементов
            out = vec[len / 2];
        }
    }
    out
}

pub fn step_moving_average(data: &Vec<f32>, window_size: usize) -> Vec<f32> {
    let mut out: Vec<f32> = vec![];
    for i in (0..data.len() - window_size).step_by(window_size) {
        let buff = &data[i..i + window_size];
        let mean_buff = buff.iter().sum::<f32>() / buff.len() as f32;
        for _i in 0..window_size {
            out.push(mean_buff);
        }
    }
    let len_data = data.len();
    let len_out = out.len();
    let diff_len = len_data - len_out;
    let last_out = *out.last().unwrap();
    for _i in 0..diff_len {
        out.push(last_out);
    }
    out
}

pub fn moving_average(data: &Vec<f32>, window_size: usize) -> Vec<f32> {
    let half_win = window_size / 2;
    let begin = &data[0..window_size];
    let mean_begin = begin.iter().sum::<f32>() / begin.len() as f32;
    let mut out: Vec<f32> = vec![mean_begin; half_win];
    for i in half_win..data.len() - half_win {
        let buff = &data[i - half_win..i + half_win];
        let mean_buff = buff.iter().sum::<f32>() / buff.len() as f32;
        out.push(mean_buff);
    }
    let last_out = *out.last().unwrap();
    for _i in 0..half_win {
        out.push(last_out);
    }
    out
}

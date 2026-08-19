use crate::my_lib::{moving_average, step_moving_average};
use native_dialog::{DialogBuilder, MessageLevel};
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};

pub struct TimeParam {
    pub r_pos: Vec<i32>,
    pub intervals: Vec<f32>,
    pub inds_min_diff: Vec<usize>,
    pub clear_intervals: Vec<f32>,
    pub threshold: Vec<f32>,
    pub chars: Vec<char>,
    pub forms: Vec<usize>,
}

impl TimeParam {
    pub fn new() -> TimeParam {
        let mut time_param = TimeParam {
            r_pos: vec![],
            intervals: vec![],
            inds_min_diff: vec![],
            clear_intervals: vec![],
            threshold: vec![],
            chars: vec![],
            forms: vec![],
        };
        time_param.parse_b_txt();
        time_param.get_inds_min_diff();
        time_param.get_clear_intervals();
        time_param.get_threshold();
        time_param
    }

    fn parse_b_txt(&mut self) {
        let path_b = "c:\\EcgVar\\B.txt";
        let file = File::open(&path_b);
        let file = match file {
            Ok(file) => file,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    DialogBuilder::message()
                        .set_level(MessageLevel::Error)
                        .set_title("Ошибка")
                        .set_text("В директории c:\\EcgVar\\ отсутствует файл B.txt ")
                        .alert()
                        .show()
                        .unwrap();
                    return;
                }
                other_error => panic!("{:?}", other_error),
            },
        };
        let reader = BufReader::new(&file);
        for (_i, res_line) in reader.lines().enumerate() {
            let line: String = match res_line {
                Ok(val) => val,
                Err(_err) => continue,
            };
            if line.contains(';') {
                let split_line: Vec<&str> = line.split(';').collect();
                if split_line.len() == 3 {
                    let end_line: Vec<&str> = split_line[2].split(':').collect();
                    if end_line.len() == 2 {
                        self.r_pos.push(split_line[0].parse::<i32>().unwrap());
                        let mut interval = split_line[1].parse::<f32>().unwrap();
                        if interval > 450.0 {
                            interval = 200.0;
                        }
                        self.intervals.push(interval);
                        self.forms.push(end_line[1].parse::<usize>().unwrap());
                        let char_in_line = end_line[0].chars().nth(0);
                        let char_end = char_in_line.unwrap_or_else(|| "A".chars().next().unwrap());
                        self.chars.push(char_end);
                    }
                }
            }
        }
    }

    fn get_clear_intervals(&mut self) {
        let mut intervals = self.intervals.clone();
        let mut step: usize = 1;
        for i in 0..self.chars.len() {
            if step == 2 {
                step = 1;
                continue;
            }
            if (i > 3) && (i < intervals.len() - 4) {
                let mean_intervals = (intervals[i - 3]
                    + intervals[i - 2]
                    + intervals[i - 1]
                    + intervals[i + 2]
                    + intervals[i + 3])
                    / 5.0;
                let mut diff_mean = intervals[i] - mean_intervals;
                if diff_mean == 0.0 {
                    diff_mean = 0.1;
                }
                let sign_diff_mean = diff_mean / diff_mean.abs();
                let dev_mean = sign_diff_mean * (diff_mean.abs().sqrt()) * 0.4;
                if (self.chars[i] == 'V') && (self.chars[i + 1] == 'V') {
                    self.clear_intervals.push(mean_intervals + dev_mean);
                    intervals[i] = mean_intervals + dev_mean;
                    step = 1;
                } else if (self.chars[i] == 'V') && (self.chars[i + 1] != 'V') {
                    self.clear_intervals.push(mean_intervals + dev_mean);
                    self.clear_intervals.push(mean_intervals - dev_mean);
                    intervals[i] = mean_intervals + dev_mean;
                    intervals[i + 1] = mean_intervals - dev_mean;
                    step = 2;
                } else if (self.chars[i] != 'V') && (self.forms[i] == 1) {
                    let tf = intervals[i - 1..i + 4].to_vec();
                    let sum_interval = intervals[i] + intervals[i + 1];
                    let diff21 = tf[2] - tf[1];
                    let diff23 = tf[2] - tf[3];
                    let diff24 = (tf[2] - tf[4]).abs();
                    let diff13 = (tf[1] - tf[3]).abs();

                    if (tf[1] < intervals[i - 1])
                        && (tf[2] > intervals[i - 1])
                        && (diff21 > 65.0)
                        && (diff23 > 10.0)
                    {
                        self.clear_intervals.push(mean_intervals + dev_mean);
                        self.clear_intervals.push(mean_intervals - dev_mean);
                        intervals[i] = mean_intervals + dev_mean;
                        intervals[i + 1] = mean_intervals - dev_mean;
                        step = 2;
                    } else if sum_interval < intervals[i - 1] * 1.1 {
                        self.clear_intervals.push(mean_intervals + dev_mean);
                        self.clear_intervals.push(mean_intervals + dev_mean);
                        intervals[i] = mean_intervals + dev_mean;
                        intervals[i + 1] = mean_intervals + dev_mean;
                        step = 2;
                    } else if (diff24 < 10.0) && (diff13 < 10.0) && (diff21 > 70.0) {
                        self.clear_intervals.push(mean_intervals + dev_mean);
                        self.clear_intervals.push(mean_intervals - dev_mean);
                        intervals[i] = mean_intervals + dev_mean;
                        intervals[i + 1] = mean_intervals - dev_mean;
                        step = 2;
                    } else {
                        self.clear_intervals.push(intervals[i]);
                        step = 1;
                    }
                } else {
                    self.clear_intervals.push(intervals[i]);
                    step = 1;
                }
            } else {
                self.clear_intervals.push(intervals[i]);
                step = 1;
            }
        }
    }

    fn get_threshold(&mut self) {
        self.threshold = step_moving_average(&self.clear_intervals, 2);
        self.threshold = step_moving_average(&self.threshold, 3);
        self.threshold = step_moving_average(&self.threshold, 2);
        self.threshold = moving_average(&self.threshold, 12);
    }

    fn get_inds_min_diff(&mut self) {
        /*
        Параметры:
        intervals: Vec<f32>.
        Возвращает:
        массив индексов Vec<usize>, где абсолютная разница
        между последовательными интервалами меньше или равна 3.
        */
        for i in 1..self.intervals.len() {
            let min_diff = &self.intervals[i] * 0.02;
            if (self.chars[i] == 'N')
                && (self.chars[i - 1] == 'N')
                && (self.forms[i] != 100)
                && (self.forms[i] == self.forms[i - 1])
            {
                let diff = (self.intervals[i] - self.intervals[i - 1]).abs();
                if diff <= min_diff {
                    self.inds_min_diff.push(i);
                }
            }
        }
    }
}

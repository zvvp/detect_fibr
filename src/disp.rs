use crate::my_lib::{moving_average, step_moving_average, truncate_win2};
use crate::time_param::TimeParam;

fn get_diff_intervals(intervals: &Vec<f32>, step: usize) -> Vec<f32> {
    let mut diff_intervals: Vec<f32> = intervals.clone();
    let diff_length = diff_intervals.len();
    for i in step..diff_length {
        diff_intervals[i] = (&intervals[i] - &intervals[i - step]).abs();
    }
    for i in 0..step {
        diff_intervals[i] = diff_intervals[step];
    }
    for i in diff_length - step..diff_length {
        diff_intervals[i] = diff_intervals[diff_length - step - 1];
    }
    diff_intervals
}
pub fn get_coef_disp(time_param: &TimeParam) -> Vec<f32> {
    let diff1 = get_diff_intervals(&time_param.clear_intervals, 1);
    let diff2 = get_diff_intervals(&time_param.clear_intervals, 2);
    let diff3 = get_diff_intervals(&time_param.clear_intervals, 3);
    let diff4 = get_diff_intervals(&time_param.clear_intervals, 4);
    let len_diff = diff1.len();
    let mut out: Vec<f32> = vec![0.0; len_diff];
    for i in 10..len_diff - 10 {
        let win_diff1 = diff1[i - 10..i + 11].to_vec();
        let win_diff2 = diff2[i - 10..i + 11].to_vec();
        let win_diff3 = diff3[i - 10..i + 11].to_vec();
        let win_diff4 = diff4[i - 10..i + 11].to_vec();
        let mul12: Vec<f32> = win_diff1
            .iter()
            .zip(win_diff2.iter())
            .map(|(&x, &y)| x * y)
            .collect();
        let mul13: Vec<f32> = win_diff1
            .iter()
            .zip(win_diff3.iter())
            .map(|(&x, &y)| x * y)
            .collect();
        let mul14: Vec<f32> = win_diff1
            .iter()
            .zip(win_diff4.iter())
            .map(|(&x, &y)| x * y)
            .collect();
        let mul23: Vec<f32> = win_diff2
            .iter()
            .zip(win_diff3.iter())
            .map(|(&x, &y)| x * y)
            .collect();
        let mul24: Vec<f32> = win_diff2
            .iter()
            .zip(win_diff4.iter())
            .map(|(&x, &y)| x * y)
            .collect();
        let mul34: Vec<f32> = win_diff3
            .iter()
            .zip(win_diff4.iter())
            .map(|(&x, &y)| x * y)
            .collect();
        let sum_diff: Vec<f32> = mul12
            .iter()
            .zip(mul13.iter())
            .zip(mul14.iter())
            .zip(mul23.iter())
            .zip(mul24.iter())
            .zip(mul34.iter())
            .map(|(((((a, b), c), d), e), f)| a + b + c + d + e + f)
            .collect();
        let mut mul_diff: Vec<f32> = sum_diff.iter().map(|&x| x.sqrt()).collect();
        mul_diff.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let sort_diff: Vec<f32> = mul_diff[5..mul_diff.len() - 5].to_vec();
        let sum_sort_diff: f32 = sort_diff.iter().sum();
        let count = sort_diff.len();
        let mean_sort_diff = if count > 0 {
            sum_sort_diff / count as f32
        } else {
            0.0
        };
        out[i] = mean_sort_diff * 0.35; // 0.4
    }
    for i in 0..30 {
        out[i] = out[30];
    }
    for i in len_diff - 30..len_diff {
        out[i] = out[len_diff - 31];
    }
    out = out.iter().map(|&x| x * 1.0 + 0.0).collect();
    out = truncate_win2(&out, 0.7, 100);
    out = step_moving_average(&out, 25);
    out = moving_average(&out, 40);
    out
}

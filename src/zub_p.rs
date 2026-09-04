use crate::my_lib::{find_local_extrema, find_max, median_filter, my_filtfilt};
use crate::time_param::TimeParam;
use log::{debug, error};

pub struct Zubp {
    pub mean_amp_p: f32,
    pub presence_pr: Vec<i32>,
    pub intervals_pr: Vec<i32>,
    pub inds_pr: Vec<usize>,
}
impl Zubp {
    pub fn new(lead: &Vec<f32>, r_pos_len: usize) -> Zubp {
        // let lead = lead;
        Zubp {
            mean_amp_p: 0.0,
            presence_pr: vec![0; r_pos_len],
            intervals_pr: vec![],
            inds_pr: vec![],
        }
    }
    pub fn get_mean_amp_pos(&mut self, lead: &Vec<f32>, time_param: &TimeParam) {
        /*
           Заполняет структуру Zubp
        */
        let lead = lead.clone();
        let mut vec_amp_p: Vec<f32> = vec![];

        for ind in &time_param.inds_min_diff {
            let coef = 0.31 + time_param.intervals[*ind] / 2000.0; // 0.31 +
            let len_pr: i32 = (time_param.intervals[*ind] * coef) as i32; // *0.45
            let start = (time_param.r_pos[*ind] - len_pr) as usize;
            let stop = (time_param.r_pos[*ind] - 10) as usize;
            // debug!("start, stop: {}, {}", start, stop);

            let fragment = lead[start..stop].to_vec();
            let (amp_p, ind_p) = self.get_amp_ind_p(&fragment);
            let pr = len_pr - ind_p as i32;
            if pr > 0 {
                self.intervals_pr.push(pr);
                self.inds_pr.push(*ind);
            }
            if (amp_p < 1.0) && (amp_p > 0.001) {
                vec_amp_p.push(amp_p);
            }
        }
        debug!("for vec_amp_p ok");
        if vec_amp_p.is_empty() {
            self.mean_amp_p = 0.0;
        } else {
            let sum: f32 = vec_amp_p.iter().sum();
            let count = vec_amp_p.len() as f32;
            self.mean_amp_p = sum / count;
        }
        debug!("mean_amp_p ok");
        debug!("self.intervals_pr_len = {}", &self.intervals_pr.len());
        self.intervals_pr = median_filter(&self.intervals_pr, 59); // 19
        debug!("median_filter");
        self.interp_pr(time_param);
        debug!("interp_pr");
        self.clean_presence_pr();
        debug!("clean_presence_pr");
    }

    fn get_amp_ind_p(&mut self, fragment: &Vec<f32>) -> (f32, usize) {
        /*
           Возвращает амп. P и его индекс в фрагменте(PR)
           для вычисления mean_amp_p и mean_PR
        */
        let b: Vec<f32> = vec![0.02785977, 0.05571953, 0.02785977];
        let a: Vec<f32> = vec![1.0, -1.47548044, 0.58691951];
        let bi: Vec<f32> = vec![0.03634612, 0.03634612];
        let ai: Vec<f32> = vec![1.0, -0.92730777];
        let mut amp_p: f32 = 0.0;
        let mut ind_p: usize = 0;
        let mut fragment = my_filtfilt(&b, &a, &fragment);
        let isoline = my_filtfilt(&bi, &ai, &fragment);
        for i in 0..fragment.len() {
            fragment[i] = fragment[i] - isoline[i];
        }
        let (vec_ind_extrema, vec_val_extrema) = find_local_extrema(&fragment);
        if vec_val_extrema.len() == 1 {
            let (val_max, ind_max) = find_max(&fragment);
            if ind_max == 0 || ind_max == fragment.len() - 1 {
                amp_p = 0.0;
            } else {
                amp_p = val_max;
            }
        } else if vec_val_extrema.len() == 2 {
            amp_p = (vec_val_extrema[0] - vec_val_extrema[1]).abs();
        } else if vec_val_extrema.len() > 2 {
            let (val_max_extrema, ind_max_extrema) = find_max(&vec_val_extrema);
            ind_p = vec_ind_extrema[ind_max_extrema];
            if ind_max_extrema == 0 {
                amp_p = val_max_extrema;
            } else if ind_max_extrema == vec_ind_extrema.len() - 1 {
                amp_p = val_max_extrema;
            } else {
                let side1 = vec_val_extrema[ind_max_extrema] - vec_val_extrema[ind_max_extrema - 1];
                let side2 = vec_val_extrema[ind_max_extrema] - vec_val_extrema[ind_max_extrema + 1];
                if side1 < side2 {
                    amp_p = side1;
                } else {
                    amp_p = side2;
                }
            }
        }
        (amp_p, ind_p)
    }

    pub fn get_p_in_lead(&mut self, lead: &Vec<f32>, time_param: &TimeParam) -> Vec<f32> {
        let lead = lead.clone();
        let mut p: Vec<f32> = vec![0.0; time_param.r_pos.len()];
        let w = (self.mean_amp_p * 30.0) as i32 + 12; // +10
        for i in 0..self.presence_pr.len() {
            if time_param.r_pos[i] < lead.len() as i32 {
                let start = (time_param.r_pos[i] - self.presence_pr[i] - w) as usize; // -15
                let stop = if (self.presence_pr[i] - w) > 9 {
                    (time_param.r_pos[i] - self.presence_pr[i] + w) as usize
                } else {
                    (time_param.r_pos[i] - 9) as usize
                };
                let fragment = lead[start..stop].to_vec();
                if time_param.chars[i] != 'N' {
                    p[i] = 1.0;
                } else {
                    let (amp_p, _ind_p) = self.get_amp_ind_p(&fragment);
                    if amp_p > self.mean_amp_p * 0.45 {
                        //0.3
                        p[i] = 1.0;
                    }
                }
            }
        }
        p
    }

    fn interp_line(&mut self, ind_start: usize, ind_stop: usize, val_start: i32, val_stop: i32) {
        let diff_val = val_stop - val_start;
        let count_step = ind_stop - ind_start;
        if count_step > 1 {
            let step_val = diff_val / count_step as i32;
            for i in ind_start..ind_stop {
                let j = i - ind_start;
                self.presence_pr[i] = val_start + step_val * j as i32;
            }
        } else {
            self.presence_pr[ind_start] = val_start;
        }
    }
    fn interp_pr(&mut self, time_param: &TimeParam) {
        for i in 1..self.inds_pr.len() - 1 {
            let ind_start = self.inds_pr[i];
            let ind_stop = self.inds_pr[i + 1];
            let val_start = self.intervals_pr[i];
            let val_stop = self.intervals_pr[i + 1];
            self.interp_line(ind_start, ind_stop, val_start, val_stop);
        }
        self.interp_line(
            0,
            self.inds_pr[0],
            self.intervals_pr[0],
            self.intervals_pr[0],
        );
        self.interp_line(
            self.inds_pr[self.inds_pr.len() - 1],
            time_param.r_pos.len(),
            self.intervals_pr[self.intervals_pr.len() - 1],
            self.intervals_pr[self.intervals_pr.len() - 1],
        );
    }
    fn clean_presence_pr(&mut self) {
        for i in 1..self.presence_pr.len() - 1 {
            if self.presence_pr[i] == 0 {
                self.presence_pr[i] = self.presence_pr[i - 1];
            }
        }
    }
}

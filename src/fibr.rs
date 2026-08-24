// #[derive(Debug)]
pub struct Fibr {
    pub start_ind_arr: Vec<usize>,
    pub stop_ind_arr: Vec<usize>,
    pub diff_stop_start: Vec<usize>,
}
impl Fibr {
    pub fn new(coef_fibr: &Vec<f32>, trs: &Vec<f32>, r_pos: &Vec<i32>, start_time_in_samples: usize, mask: &Vec<usize>) -> Fibr {
        let mut fibr = Fibr {
            start_ind_arr: vec![],
            stop_ind_arr: vec![],
            diff_stop_start: vec![],
        };
        fibr.get_start_stop(coef_fibr, trs, r_pos, start_time_in_samples, mask);
        fibr
    }

    fn get_start_stop(&mut self, coef_fibr: &Vec<f32>, trs: &Vec<f32>, r_pos: &Vec<i32>, start_time_in_samples: usize, mask: &Vec<usize>) {
        let mut start_ind: usize = 0;
        let mut stop_ind: usize = 0;
        let min_size: usize = 20;
        let mut flag: bool = false;
        if coef_fibr[0] > trs[0] {
            self.start_ind_arr.push(r_pos[start_ind] as usize + start_time_in_samples);
            flag = true;
        } 
        for i in 0..coef_fibr.len() - 1 {
            if (flag == false) && (coef_fibr[i] < trs[i]) && (coef_fibr[i + 1] > trs[i + 1]) && (mask[i] == 1) {
                if i < min_size {
                    start_ind = i;
                    self.start_ind_arr.push(r_pos[start_ind] as usize + start_time_in_samples);
                
                } else if i - stop_ind > min_size {
                    start_ind = i;
                    self.start_ind_arr.push(r_pos[start_ind] as usize + start_time_in_samples);
                } else {
                    if self.stop_ind_arr.len() > 0 {
                        self.stop_ind_arr.pop().unwrap();
                    }
                }
                flag = true;
            } else if (flag == true) && (coef_fibr[i] > trs[i]) && (coef_fibr[i + 1] < trs[i + 1]) && (mask[i] == 1) {
                if i - start_ind > min_size {
                    stop_ind = i;
                    self.stop_ind_arr.push(r_pos[stop_ind] as usize + start_time_in_samples);
                } else {
                    if self.start_ind_arr.len() > 0 {
                        self.start_ind_arr.pop().unwrap();
                    }
                }
                flag = false;
            } else if (flag == true) && (i == coef_fibr.len() - 2) {
                if i - start_ind > min_size {
                    stop_ind = i + 1;
                    self.stop_ind_arr.push(r_pos[stop_ind] as usize + start_time_in_samples);
                } else {
                    if self.start_ind_arr.len() > 0 {
                        self.start_ind_arr.pop().unwrap();
                    }
                }
            }
        }
        if (self.start_ind_arr.len() > 0) && (self.stop_ind_arr.len() > 0) {
            for i in 0..self.start_ind_arr.len() {
                self.diff_stop_start.push(self.stop_ind_arr[i] - self.start_ind_arr[i]);
            }
        }
        
    }
}
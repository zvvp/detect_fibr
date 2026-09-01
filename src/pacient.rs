use native_dialog::{DialogBuilder, MessageLevel};
use std::fs::{File, metadata};
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct Pacient {
    pub file_name: String,
    pub start_time: (u32, u32, u32),
    pub total_time: (u32, u32, u32),
    pub start_time_in_samples: usize,
}

impl Pacient {
    pub fn new() -> Pacient {
        let mut pacient = Pacient {
            file_name: String::new(),
            start_time: (0, 0, 0),
            total_time: (0, 0, 0),
            start_time_in_samples: 0,
        };
        pacient.parse_pacient_card();
        pacient.get_start_time();
        pacient
    }

    fn parse_pacient_card(&mut self) {
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
                return;
            }
        };
        self.file_name = fname.to_str().unwrap().to_string();

        let len_file: u32 = metadata(fname).unwrap().len() as u32;
        let num_of_samples = (len_file - 1024) / 6;
        let (d, h, m, s) = self.get_time_from_samples(num_of_samples);
        let s = s % 60;
        let m = m % 60;
        self.total_time = (h, m, s);
    }

    pub fn get_time_from_samples(&self, num_of_samples: u32) -> (u32, u32, u32, u32) {
        let s = num_of_samples / 250;
        let m = s / 60;
        // s = s % 60;
        let h = m / 60;
        // m = m % 60;
        let d = h / 24;
        // h = h % 24;
        (d, h, m, s)
    }

    fn get_start_time(&mut self) {
        let mut file = File::open(&self.file_name).expect("Failed to open file");
        file.seek(SeekFrom::Start(151)).unwrap();
        let mut dlmt = [0u8; 1];
        file.read_exact(&mut dlmt).unwrap();
        if dlmt[0] == b':' {
            file.seek(SeekFrom::Start(150)).unwrap();
            let mut start_h = [0u8; 1];
            file.read_exact(&mut start_h).unwrap();
            let start_h = start_h[0] as char;
            let start_h = start_h.to_digit(10).unwrap();

            file.seek(SeekFrom::Start(152)).unwrap();
            let mut start_m = [0u8; 2];
            file.read_exact(&mut start_m).unwrap();
            let start_m0 = start_m[0] as char;
            let start_m0 = start_m0.to_digit(10).unwrap();
            let start_m1 = start_m[1] as char;
            let start_m1 = start_m1.to_digit(10).unwrap();
            let start_m = start_m0 * 10 + start_m1;

            file.seek(SeekFrom::Start(155)).unwrap();
            let mut start_s = [0u8; 2];
            file.read_exact(&mut start_s).unwrap();
            let start_s0 = start_s[0] as char;
            let start_s0 = start_s0.to_digit(10).unwrap();
            let start_s1 = start_s[1] as char;
            let start_s1 = start_s1.to_digit(10).unwrap();
            let start_s = start_s0 * 10 + start_s1;

            self.start_time = (start_h, start_m, start_s);
            self.start_time_in_samples = ((start_h * 3600 + start_m * 60 + start_s) * 250) as usize;
        } else {
            file.seek(SeekFrom::Start(150)).unwrap();
            let mut start_h = [0u8; 2];
            file.read_exact(&mut start_h).unwrap();
            let start_h0 = start_h[0] as char;
            let start_h0 = start_h0.to_digit(10).unwrap();
            let start_h1 = start_h[1] as char;
            let start_h1 = start_h1.to_digit(10).unwrap();
            let start_h = start_h0 * 10 + start_h1;

            file.seek(SeekFrom::Start(153)).unwrap();
            let mut start_m = [0u8; 2];
            file.read_exact(&mut start_m).unwrap();
            let start_m0 = start_m[0] as char;
            let start_m0 = start_m0.to_digit(10).unwrap();
            let start_m1 = start_m[1] as char;
            let start_m1 = start_m1.to_digit(10).unwrap();
            let start_m = start_m0 * 10 + start_m1;

            file.seek(SeekFrom::Start(156)).unwrap();
            let mut start_s = [0u8; 2];
            file.read_exact(&mut start_s).unwrap();
            let start_s0 = start_s[0] as char;
            let start_s0 = start_s0.to_digit(10).unwrap();
            let start_s1 = start_s[1] as char;
            let start_s1 = start_s1.to_digit(10).unwrap();
            let start_s = start_s0 * 10 + start_s1;

            self.start_time = (start_h, start_m, start_s);
            self.start_time_in_samples = ((start_h * 3600 + start_m * 60 + start_s) * 250) as usize;
        }
    }
}

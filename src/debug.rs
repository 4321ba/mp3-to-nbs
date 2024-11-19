#[allow(dead_code)]
pub fn debug_save_as_wav(wf: &babycat::Waveform, filename: &str) {
    wf.to_wav_file(filename).unwrap();
}
#[allow(dead_code)]
pub fn debug_save_as_image(array2d: &[Vec<f32>], filename: &str) {

    let mut bytes: Vec<u8> = Vec::new();
    // Write a &str in the file (ignoring the result).
    let width = array2d.len();
    let height = array2d[0].len();
    for y in (0..height/8).rev() { // 0 instead of height*7/8 to print the whole thing
        for _ in 0..4 {
            for x in 0..width {
                for _ in 0..4 {
                    let strength = (array2d[x][y] * 256.0*16.0) as i8;
                    if strength >= 0 {
                        bytes.push(strength as u8);
                        bytes.push(strength as u8);
                        bytes.push(strength as u8);
                    } else {
                        bytes.push((-(strength as i16)) as u8);
                        bytes.push(0);
                        bytes.push(0);
                    }
                }
            }

        }
    }
    image::save_buffer(filename, &bytes, 4*width as u32, 4*(height/8) as u32, image::ColorType::Rgb8).unwrap()
}

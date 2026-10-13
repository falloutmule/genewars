//! Small port-owned status lettering, separate from commercial game artwork.
use genewars_render::RgbImage;

fn glyph(c: u8) -> [u8;5] {
    match c.to_ascii_uppercase() {
        b'A'=>[126,9,9,9,126],b'B'=>[127,73,73,73,54],b'C'=>[62,65,65,65,34],
        b'D'=>[127,65,65,34,28],b'E'=>[127,73,73,73,65],b'F'=>[127,9,9,9,1],
        b'G'=>[62,65,73,73,58],b'H'=>[127,8,8,8,127],b'I'=>[65,65,127,65,65],
        b'J'=>[32,64,65,63,1],b'K'=>[127,8,20,34,65],b'L'=>[127,64,64,64,64],
        b'M'=>[127,2,4,2,127],b'N'=>[127,2,4,8,127],b'O'=>[62,65,65,65,62],
        b'P'=>[127,9,9,9,6],b'Q'=>[62,65,81,33,94],b'R'=>[127,9,25,41,70],
        b'S'=>[38,73,73,73,50],b'T'=>[1,1,127,1,1],b'U'=>[63,64,64,64,63],
        b'V'=>[31,32,64,32,31],b'W'=>[63,64,48,64,63],b'X'=>[99,20,8,20,99],
        b'Y'=>[3,4,120,4,3],b'Z'=>[97,81,73,69,67],
        b'0'=>[62,81,73,69,62],b'1'=>[0,66,127,64,0],b'2'=>[98,81,73,73,70],
        b'3'=>[34,65,73,73,54],b'4'=>[24,20,18,127,16],b'5'=>[39,69,69,69,57],
        b'6'=>[60,74,73,73,48],b'7'=>[1,113,9,5,3],b'8'=>[54,73,73,73,54],
        b'9'=>[6,73,73,41,30],b'-'=>[8,8,8,8,8],b'_'=>[64,64,64,64,64],
        b'.'=>[0,96,96,0,0],b':'=>[0,54,54,0,0],b'/'=>[32,16,8,4,2],
        b'('=>[0,28,34,65,0],b')'=>[0,65,34,28,0],b'='=>[20,20,20,20,20],
        b','=>[0,64,32,0,0],b' '=>[0;5],_=>[2,1,81,9,6],
    }
}

pub fn overlay(image: &RgbImage, lines: &[String]) -> Result<RgbImage,String> {
    let mut wrapped = Vec::new();
    for line in lines {
        let mut current = String::new();
        for word in line.split_whitespace() {
            if !current.is_empty() && current.len() + word.len() + 1 > 51 {
                wrapped.push(current); current = String::new();
            }
            if !current.is_empty() { current.push(' '); }
            current.push_str(word);
        }
        wrapped.push(current);
    }
    let mut rgb = image.rgb_bytes().to_vec();
    let height = (wrapped.len() * 18 + 12).min(480);
    let top = 480 - height;
    for y in top..480 { for x in 0..640 {
        rgb[(y*640+x)*3..(y*640+x)*3+3].copy_from_slice(&[12,18,25]);
    }}
    for (line,text) in wrapped.iter().enumerate() {
        for (col,c) in text.bytes().take(51).enumerate() {
            for (dx,bits) in glyph(c).iter().enumerate() { for dy in 0..7 {
                for sy in 0..2 { for sx in 0..2 {
                    let x = 10 + col*12 + dx*2 + sx;
                    let y = top + 6 + line*18 + dy*2 + sy;
                    if y<480 && bits&(1<<dy)!=0 {
                        rgb[(y*640+x)*3..(y*640+x)*3+3].copy_from_slice(&[229,239,218]);
                    }
                }}
            }}
        }
    }
    RgbImage::from_rgb24(640,480,rgb).map_err(|e|e.to_string())
}

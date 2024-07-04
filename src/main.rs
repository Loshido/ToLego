use image::{imageops::FilterType, io::Reader as ImageReader, GenericImageView, ImageFormat, Pixel, RgbaImage};
use std::{fs, path::Path};
use clap::Parser;
use oxipng::{optimize, Options, OutFile};

fn legofy(file: &Path, brick_size: u32) {
    let img = ImageReader::open(file)
        .unwrap().decode().unwrap();

    let width = img.width();
    let height = img.height();

    // removing extra spaces (in case of half brick...)
    let brick_dimension = [
        width - width % brick_size, 
        height - height % brick_size
    ];

    // legofied image buffer
    let mut lego = RgbaImage::new(brick_dimension[0], brick_dimension[1]);

    let brick = match ImageReader::open("brick.jpg") {
        Err(_) => panic!("brick.jpg needs to be in the same directory as the ToLego.exe!"),
        Ok(result) => result
            .decode().unwrap()
            .resize(brick_size, brick_size, FilterType::Triangle)
    };

    // loop through each brick
    // 8.div_euclid(3) == 2 <=> // in python
    for i in 0 .. width.div_euclid(brick_size) {
        for j in 0 .. height.div_euclid(brick_size) {
            let x = i * brick_size;
            let y = j * brick_size;

            // average of colors from the brick
            let mut channels: [u64; 3] = [0, 0, 0]; // sum rgb
            let mut compteur = 0; // count

            // loop through each pixel of the brick
            // except the first & last one (gap between bricks)
            for xi in x + 1 .. x + brick_size {
                for yj in y + 1 .. y + brick_size {
                    let pixel = img.get_pixel(xi, yj).0;
                    for i in 0..3 {
                        channels[i] += u64::from(pixel[i]);
                    };
                    compteur += 1;
                }
            }
            // compute the average of each channel
            for i in 0..3 {
                channels[i] = channels[i].div_euclid(compteur);
            };
            
            // loop through each pixels to assign the average rgb
            // + multiply the color by the brick image colors
            for xi in x + 1 .. x + brick_size {
                for yj in y + 1 .. y + brick_size {
                    // xi is the real index in the width of the picture
                    // xi - x => relative index in the loop (0, 1, 2...)

                    let brick_pixel = brick.get_pixel(xi - x, yj - y).0;
                    let diff: [u64; 3] = [
                        u64::from(brick_pixel[0]),
                        u64::from(brick_pixel[1]),
                        u64::from(brick_pixel[2])
                    ];

                    // 1 >= diff[i] / 255 >= 0
                    // <=> if the brick color tends to white then it tends to 1
                    // <=> otherwise if the brick color tends to black it tends to 0
                    let new_pixel: [u8; 4] = [
                        u8::try_from(channels[0] * diff[0] / 255).unwrap(),
                        u8::try_from(channels[1] * diff[1] / 255).unwrap(),
                        u8::try_from(channels[2] * diff[2] / 255).unwrap(),
                        255
                    ];

                    let pixel = Pixel::from_slice(&new_pixel);
                    lego.put_pixel(xi, yj, *pixel);
                }
            }
        }
    }
    // recovering the filename
    let filename = file.file_stem().unwrap();
    // in the right format
    let filename_str = filename.to_str().unwrap();
    // path to the default legofy output
    let path = format!("{}.lego", filename_str);
    match lego.save_with_format(&path, ImageFormat::Png) {
        Ok(_) => println!("Legofication successfully!"),
        Err(err) => eprintln!("Legofication failed: {}", err)
    };
    
    // optimizing the output
    let optimized_filename = format!("{}.lego.png", filename_str);
    let output_path = Path::new(&optimized_filename);
    let output = OutFile::from_path(output_path.to_path_buf());
    let options = Options::default();

    // Optimize the image
    let unoptimized_path = path.clone();
    match optimize(&path.into(), &output, &options) {
        Ok(_) => {
            println!("Optimization successful!");
            fs::remove_file(unoptimized_path).unwrap();
        },
        Err(err) => {
            eprintln!("Optimization failed: {}", err);

            fs::rename(unoptimized_path, optimized_filename).unwrap();
        },
    }
}

#[derive(Parser, Debug)]
struct Args {
    // Path of the picture to legofy
    #[arg(short, long)]
    file: String,

    // Brick size
    #[arg(short, long, default_value_t = 50)]
    brick_size: u32
}

fn main() {
    let args = Args::parse();
    let filepath = Path::new(&args.file);

    if Path::exists(filepath) {
        legofy(filepath, args.brick_size)
    } else {
        println!("There is not file!")
    }
}
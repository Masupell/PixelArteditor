#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::PathBuf;
use image::error::ImageFormatHint;
use image::{DynamicImage, ImageBuffer, Rgba};
use egui::Pos2;
use egui_file::FileDialog;
use macroquad::prelude::*;

fn window_conf() -> Conf
{
    Conf 
    {
        window_title: "editor".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() 
{
    let mut rgba: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    let mut width_str = String::new();
    let mut height_str = String::new();

    let mut clicked: bool = false;

    let mut width: i32 = 0;
    let mut height: i32 = 0;

    let mut x = 0.0;
    let mut y = 0.0;
    let mut canvas_width = 0.0;
    let mut canvas_height = 0.0;
    let original_grid_size = 20.0;
    let mut grid_size = original_grid_size;
    
    let mut show_grid: bool = false;

    let mut zoom: f32 = 1.0;
    let mut camera: (f32, f32) = (0.0, 0.0);

    let mut mouse_pos1: (f32, f32);
    let mut mouse_middle = false;

    let mut pixels: Vec<Vec<(u8, u8, u8, u8)>> = Vec::new();

    let mut save_path = String::new();
    let mut load_path = String::new();

    let mut color_picker = false;
    let mut eraser = false;
    let mut fill_tool = false;
    let mut brush_size = 0;

    loop 
    {
        clear_background(Color::from_rgba(8, 16, 41, 255));
        // 17, 26, 56, 255
        // 20.0, 20.0, 20.0, 255.0

        //Update

        //Zoom
        if mouse_wheel().1 > 0.0
        {
            let z = zoom_in(zoom, camera);
            zoom = z.0;
            camera = z.1;
            grid_size *= 2.0;
        }
        else if mouse_wheel().1 < 0.0
        {
            let z = zoom_out(zoom, camera);
            zoom = z.0;
            camera = z.1;
            grid_size *= 0.5;
        }

        //Movement

        //Keyboard
        if is_key_down(KeyCode::Left)
        {
            camera.0 -= 5.0;
        }
        if is_key_down(KeyCode::Up)
        {
            camera.1 -= 5.0;
        }
        if is_key_down(KeyCode::Right)
        {
            camera.0 += 5.0;
        }
        if is_key_down(KeyCode::Down)
        {
            camera.1 += 5.0;
        }

        //Mouse
        // if is_mouse_button_pressed(MouseButton::Middle)
        // {
        //     mouse_pos1 = mouse_position();
        //     mouse_middle = true;
        // }

        // if is_mouse_button_down(MouseButton::Middle) && mouse_middle
        // {

        // }

        // if is_mouse_button_released(MouseButton::Middle)
        // {
        //     mouse_middle = false;
        // }


        //GUI
        egui_macroquad::ui(|egui_ctx| 
        {
            egui::Window::new("File").anchor(egui::Align2::LEFT_CENTER, egui::vec2(50.0, -250.0)).show(egui_ctx, |ui|
            {
                ui.label("Path: (without ending)");
                ui.text_edit_singleline(&mut save_path);
                if ui.button("Save").clicked()
                {
                    let width = width as u16;
                    let height = height as u16;
                    let pixels = two_to_one(pixels.to_vec());
                    let image = Image::new(width, height, pixels);

                    save(&image, &save_path);
                }
                ui.spacing();
                ui.separator();
                ui.spacing();
                ui.label("Path: (without ending)");
                ui.text_edit_singleline(&mut load_path);
                if ui.button("Load").clicked()
                {
                    let image = load(&mut load_path);

                    if image.is_ok()
                    {
                        let image = image.unwrap();
                        
                        width = image.width as i32;
                        height = image.height as i32;

                        canvas_width = width as f32 * original_grid_size;
                        canvas_height = height as f32 * original_grid_size;
                        x = screen_width()/2.0-canvas_width/2.0;
                        y = screen_height()/2.0-canvas_height/2.0;
                        pixels.clear();

                        pixels = one_to_two(image.pixels, height as usize, width as usize);
                        clicked = true;
                    }
                    else 
                    {
                        println!("Was not able to load file");
                    }
                }
                ui.spacing();
                ui.separator();
                ui.spacing();
                ui.label("Path: (without ending)");
                ui.text_edit_singleline(&mut save_path);
                if ui.button("Export").clicked()
                {
                    let width = width as u16;
                    let height = height as u16;
                    let pixels = two_to_one(pixels.to_vec());
                    let image = Image::new(width, height, pixels);

                    let png = export_as_png(&image, &save_path);
                    if png.is_ok()
                    {
                        png.unwrap();
                    }
                    else
                    {
                        println!("Could not export");
                    }
                }
                ui.spacing();
                ui.separator();
                ui.spacing();
                ui.label("Path: (without ending)");
                ui.text_edit_singleline(&mut load_path);
                if ui.button("Import").clicked()
                {
                    let image = import_from_png(&load_path);

                    if image.is_ok()
                    {
                        let image = image.unwrap();
                        
                        width = image.width as i32;
                        height = image.height as i32;

                        canvas_width = width as f32 * original_grid_size;
                        canvas_height = height as f32 * original_grid_size;
                        x = screen_width()/2.0-canvas_width/2.0;
                        y = screen_height()/2.0-canvas_height/2.0;
                        pixels.clear();

                        pixels = one_to_two(image.pixels, height as usize, width as usize);
                        clicked = true;
                    }
                    else 
                    {
                        println!("Was not able to import image");
                    }
                }
            });
            
            egui::Window::new("New Image").anchor(egui::Align2::LEFT_CENTER, egui::vec2(50.0, -25.0)).show(egui_ctx, |ui|
            {
                ui.label("Width:");
                ui.text_edit_singleline(&mut width_str);
                ui.label("Height");
                ui.text_edit_singleline(&mut height_str);
                if ui.button("Create").clicked()
                {
                    width = width_str.parse().unwrap();
                    height = height_str.parse().unwrap();

                    canvas_width = width as f32 * original_grid_size;
                    canvas_height = height as f32 * original_grid_size;
                    x = screen_width()/2.0-canvas_width/2.0;
                    y = screen_height()/2.0-canvas_height/2.0;
                    pixels.clear();

                    for i in 0..height
                    {
                        let mut temp_vec: Vec<(u8, u8, u8, u8)> = Vec::new();
                        for j in 0..width
                        {
                            temp_vec.push((0, 0, 0, 0));
                        }
                        pixels.push(temp_vec);
                    }
                    clicked = true;
                }
            });
            
            egui::Window::new("Tools").anchor(egui::Align2::LEFT_CENTER, egui::vec2(50.0, 150.0)).show(egui_ctx, |ui|
            {
                ui.color_edit_button_rgba_unmultiplied(&mut rgba);
                ui.spacing();
                let response = ui.add(egui::Slider::new(&mut brush_size, 1..=50));
                response.on_hover_text("BrushSize");
                ui.spacing();
                if ui.button("ColorPicker").clicked()
                {
                    color_picker = !color_picker;
                }
                ui.spacing();
                if ui.button("Eraser").clicked()
                {
                    eraser = !eraser;
                }
                ui.spacing();
                if ui.button("Filltool").clicked()
                {
                    fill_tool = !fill_tool;
                }
                ui.separator();
                ui.spacing();
                if ui.button("Show Grid").clicked()
                {
                    show_grid = !show_grid;
                }
                ui.spacing();
                if ui.button("Reset Zoom").clicked()
                {
                    zoom = 1.0;
                    camera = (0.0, 0.0);
                }
                ui.spacing();
            });
        });


        if clicked
        {
            let p = (x-camera.0, y-camera.1);
            let draw_size = (canvas_width*zoom, canvas_height*zoom);

            if is_mouse_button_down(MouseButton::Left)
            {
                let grid = to_grid(mouse_position(), x, y, camera, zoom, grid_size);
                if !(grid.0 < 0 || grid.0 > width-1 || grid.1 < 0 || grid.1 > height-1)
                {
                    if color_picker
                    {
                        rgba = u8_to_rgba(pixels[grid.1 as usize][grid.0 as usize]);
                        color_picker = false;
                    }
                    else if eraser
                    {
                        let half_brush_size = brush_size / 2;
                        for y in (grid.1 as isize - half_brush_size as isize)..(grid.1 as isize + half_brush_size as isize + 1) 
                        {
                            for x in (grid.0 as isize - half_brush_size as isize)..(grid.0 as isize + half_brush_size as isize + 1) 
                            {
                                if x >= 0 && x < width as isize && y >= 0 && y < height as isize 
                                {
                                    let x = x as usize;
                                    let y = y as usize;
                                    pixels[y][x] = (0, 0, 0, 0);
                                }
                            }
                        }
                    }
                    else if fill_tool 
                    {
                        let target_color = u8_to_rgba(pixels[grid.1 as usize][grid.0 as usize]);
                        
                        let mut stack = Vec::new();
                        stack.push(grid);

                        while let Some((x, y)) = stack.pop()
                        {
                            if pixels[y as usize][x as usize] != rgba_to_u8(target_color)
                            {
                                continue;
                            }

                            pixels[y as usize][x as usize] = rgba_to_u8(rgba);

                            if x > 0 
                            {
                                stack.push((x - 1, y));
                            }
                            if x < width - 1 
                            {
                                stack.push((x + 1, y));
                            }
                            if y > 0 
                            {
                                stack.push((x, y - 1));
                            }
                            if y < height - 1 
                            {
                                stack.push((x, y + 1));
                            }
                        }
                        fill_tool = false;
                    }
                    else 
                    {
                        // pixels[grid.1 as usize][grid.0 as usize] = rgba_to_u8(rgba);
                        let half_brush_size = brush_size / 2;
                        for y in (grid.1 as isize - half_brush_size as isize)..(grid.1 as isize + half_brush_size as isize + 1) 
                        {
                            for x in (grid.0 as isize - half_brush_size as isize)..(grid.0 as isize + half_brush_size as isize + 1) 
                            {
                                if x >= 0 && x < width as isize && y >= 0 && y < height as isize 
                                {
                                    let x = x as usize;
                                    let y = y as usize;
                                    pixels[y][x] = rgba_to_u8(rgba);
                                }
                            }
                        }
                    }
                }
            }

            //canvas
            draw_rectangle(p.0, p.1, draw_size.0, draw_size.1, Color::new(1.0, 1.0, 1.0, 0.25));
            draw_rectangle_lines(p.0, p.1, draw_size.0, draw_size.1, 1.0, BLACK);

            for i in 0..height
            {
                for j in 0..width
                {
                    let c = pixels[i as usize][j as usize];
                    if c.3 > 0
                    {
                        draw_rectangle(p.0+j as f32 * grid_size, p.1+i as f32 * grid_size, grid_size, grid_size, Color::from_rgba(c.0, c.1, c.2, c.3));
                    }
                }
            }

            //grid
            if show_grid
            {
                for i in 0..width
                {
                    draw_line(p.0+i as f32 * grid_size, p.1, p.0+i as f32 * grid_size, p.1+draw_size.1, 0.25, BLACK);
                }

                for i in 0..height
                {
                    draw_line(p.0, p.1+i as f32 * grid_size, p.0+draw_size.0, p.1+i as f32 * grid_size, 0.25, BLACK);
                }
            }
        }


        egui_macroquad::draw();

        next_frame().await
    }
}

pub fn zoom_in(zoom: f32, minus_pos: (f32, f32)) -> (f32, (f32, f32))
{
    zoom_mul(2.0, zoom, minus_pos)
}

pub fn zoom_out(zoom: f32, minus_pos: (f32, f32)) -> (f32, (f32, f32))
{
    zoom_mul(0.5, zoom, minus_pos)
}

pub fn zoom_mul(val: f32, zoom: f32, minus_pos: (f32, f32)) -> (f32, (f32, f32))
{
    change_zoom(|zoom| zoom * val, zoom, minus_pos)
}

pub fn change_zoom<F: Fn(f32) -> f32>(op: F, zoom: f32, minus_pos: (f32, f32)) -> (f32, (f32, f32))
{
    let new_zoom = (op)(zoom).clamp(0.125, 1500.0);
    let fac = new_zoom / zoom;
    let pos = (minus_pos.0 * fac, minus_pos.1 * fac);
    (new_zoom, pos)
}

fn to_grid(pos: (f32, f32), offset_x: f32, offset_y: f32, camera: (f32, f32), zoom: f32, grid_size: f32/*, width: f32, height: f32, grid_size: i32*/) -> (i32, i32)
{
    let canvas_x = offset_x - camera.0;
    let canvas_y = offset_y - camera.1;

    let canvas = ((pos.0 - canvas_x) as i32, (pos.1 - canvas_y) as i32);

    (canvas.0/grid_size as i32, canvas.1/grid_size as i32)
}

fn two_to_one(two_d: Vec<Vec<(u8, u8, u8, u8)>>) -> Vec<(u8, u8, u8, u8)>
{
    let mut result = Vec::new();

    for row in two_d
    {
        result.extend(row);
    }

    result
}

fn one_to_two(one_d: Vec<(u8, u8, u8, u8)>, rows: usize, cols: usize) -> Vec<Vec<(u8, u8, u8, u8)>>//row = height; cols = width
{
    let mut result = Vec::new();

    for i in 0..rows
    {
        let row = one_d[i * cols..(i+1) * cols].to_vec();
        result.push(row);
    }

    result
}

fn rgba_to_u8(rgba: [f32; 4]) -> (u8, u8, u8, u8)
{
    let one = (rgba[0]*255.0) as u8;
    let two = (rgba[1]*255.0) as u8;
    let three = (rgba[2]*255.0) as u8;
    let four = (rgba[3]*255.0) as u8;
    (one, two, three, four)
}

fn u8_to_rgba(u: (u8, u8, u8, u8)) -> [f32; 4]
{
    let one = u.0 as f32 / 255.0;
    let two = u.1 as f32 / 255.0;
    let three = u.2 as f32 / 255.0;
    let four = u.3 as f32 / 255.0;
    [one, two, three, four]
}

struct ByteBuffer
{
    /*
    uint8_t* data;
    uint32_t position;
    uint32_t size;
    uint32_t capacity;
     */
}

struct Texture
{
    /*
    uint32_t width;
    uint32_t height;
    uint32_t num_comps;
    void* pixels;//Array of all pixels
     */
}

pub struct Image
{
    width: u16,
    height: u16,
    pixels: Vec<(u8, u8, u8, u8)>
}

impl Image
{
    pub fn new(width: u16, height: u16, pixels: Vec<(u8, u8, u8, u8)>) -> Image
    {
        Image
        {
            width,
            height,
            pixels
        }
    }
}

pub fn save(image: &Image, path: &str)
{
    let p = path.to_string() + ".pix";
    let mut file = File::create(&p).unwrap();
    file.write_all(&image.width.to_le_bytes()).unwrap();
    file.write_all(&image.height.to_le_bytes()).unwrap();

    for (r, g, b, a) in image.pixels.to_vec()
    {
        file.write_all(&[r, g, b, a]).unwrap();
    }
}

pub fn load(path: &str) -> Result<Image, std::io::Error>
{
    let p = path.to_string() + ".pix";
    
    let mut file = File::open(p)?;

    let mut width_bytes = [0; 2];
    let mut height_bytes = [0; 2];
    file.read_exact(&mut width_bytes)?;
    file.read_exact(&mut height_bytes)?;

    let width = u16::from_le_bytes(width_bytes);
    let height = u16::from_le_bytes(height_bytes);

    let mut pixels = vec![(0, 0, 0, 0); (width * height) as usize];
    for pixel in pixels.iter_mut()
    {
        let mut pixel_bytes = [0; 4];
        file.read_exact(&mut pixel_bytes)?;
        *pixel = (pixel_bytes[0], pixel_bytes[1], pixel_bytes[2], pixel_bytes[3]);
    }

    let image = Image::new(width, height, pixels);

    Ok(image)
}

pub fn export_as_png(image: &Image, path: &str) -> Result<(), image::ImageError>
{
    let p = path.to_string() + ".png";
    
    let mut imgbuf = ImageBuffer::new(image.width.into(), image.height.into());

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut()
    {
        let (r, g, b, a) = image.pixels[x as usize + y as usize * image.width as usize];
        *pixel = Rgba([r, g, b, a]);
    }

    let dynamic_image: DynamicImage = DynamicImage::ImageRgba8(imgbuf);

    dynamic_image.save_with_format(p, image::ImageFormat::Png)
}

pub fn import_from_png(path: &str) -> Result<Image, image::ImageError>
{
    let p = path.to_string() + ".png";
    
    let img = image::open(p)?;

    if let DynamicImage::ImageRgba8(imgbuf) = img 
    {
        let (width, height) = imgbuf.dimensions();
        let mut pixels = Vec::new();

        for pixel in imgbuf.pixels() 
        {
            let rgba = pixel.0;
            pixels.push((rgba[0], rgba[1], rgba[2], rgba[3]));
        }

        Ok(Image::new(width as u16, height as u16, pixels))
    }
    else 
    {
        Err(image::ImageError::Unsupported(image::error::UnsupportedError::from_format_and_kind(ImageFormatHint::Unknown, image::error::UnsupportedErrorKind::Format(ImageFormatHint::Unknown))))
    }
}

/*
image = "0.23.14"

use image::{DynamicImage, ImageBuffer, Rgba};
use std::path::Path;

fn encode_as_png(width: u32, height: u32, pixel_data: &Vec<(u8, u8, u8, u8)>, output_path: &str) -> Result<(), image::ImageError> {
    // Create a new ImageBuffer with RGBA format.
    let mut imgbuf = ImageBuffer::new(width, height);

    // Copy pixel data into the image buffer.
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let (r, g, b, a) = pixel_data[(x as usize + y as usize * width as usize)];
        *pixel = Rgba([r, g, b, a]);
    }

    // Convert the ImageBuffer to a DynamicImage.
    let dynamic_image: DynamicImage = DynamicImage::ImageRgba8(imgbuf);

    // Save the image as a PNG file.
    dynamic_image.save_with_format(output_path, image::ImageFormat::Png)
}




use std::fs::File;
use std::io::{Result, Write};

fn create_png_image(width: u32, height: u32, pixel_data: &Vec<(u8, u8, u8, u8)>, output_path: &str) -> Result<()> {
    let mut file = File::create(output_path)?;

    // PNG signature
    file.write_all(&[137, 80, 78, 71, 13, 10, 26, 10])?;

    // IHDR chunk (Image Header)
    let ihdr_chunk = [
        0, 0, 0, 13, // Chunk length
        73, 72, 68, 82, // "IHDR" signature
        (width >> 24) as u8, (width >> 16) as u8, (width >> 8) as u8, width as u8, // Width
        (height >> 24) as u8, (height >> 16) as u8, (height >> 8) as u8, height as u8, // Height
        8, 6, 0, 0, 0, // Bit depth, color type, compression method, filter method, interlace method
    ];
    file.write_all(&ihdr_chunk)?;

    // IDAT chunk (Image Data)
    // You would need to compress and write the pixel_data here, which is quite complex.

    // IEND chunk (End)
    let iend_chunk = [0, 0, 0, 0, 73, 69, 78, 68];
    file.write_all(&iend_chunk)?;

    Ok(())
}

fn main() {
    let width = 32;
    let height = 32;
    let pixel_data: Vec<(u8, u8, u8, u8)> = vec![(255, 0, 0, 255); (width * height) as usize]; // Example data

    if let Err(err) = create_png_image(width, height, &pixel_data, "output.png") {
        eprintln!("Error: {}", err);
    } else {
        println!("PNG image created successfully.");
    }
}
*/
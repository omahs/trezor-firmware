#[cfg(any(feature = "model_tt", feature = "model_tr"))]
pub mod loader;

use super::constant;
use crate::{
    error::Error,
    time::Duration,
    trezorhal::{display, qr, time},
    ui::lerp::Lerp,
};
use core::slice::from_raw_parts;

use super::geometry::{Offset, Point, Rect};
use crate::trezorhal::buffers::{get_buffer_16bpp, get_buffer_4bpp, get_text_buffer};
#[cfg(feature = "dma2d")]
use crate::trezorhal::dma2d::{
    dma2d_setup_4bpp_over_16bpp, dma2d_setup_4bpp_over_4bpp, dma2d_start_blend,
    dma2d_wait_for_transfer,
};

use crate::trezorhal::{
    display::ToifFormat,
    uzlib::{UzlibContext, UZLIB_WINDOW_SIZE},
};

#[cfg(any(feature = "model_tt", feature = "model_tr"))]
pub use loader::{loader, loader_indeterminate, LOADER_MAX, LOADER_MIN};

pub fn backlight() -> i32 {
    display::backlight(-1)
}

pub fn set_backlight(val: i32) {
    display::backlight(val);
}

pub fn fade_backlight(target: i32) {
    const BACKLIGHT_DELAY: Duration = Duration::from_millis(14);
    const BACKLIGHT_STEP: usize = 15;

    let current = backlight();
    if current < target {
        for val in (current..target).step_by(BACKLIGHT_STEP) {
            set_backlight(val);
            time::sleep(BACKLIGHT_DELAY);
        }
    } else {
        for val in (target..current).rev().step_by(BACKLIGHT_STEP) {
            set_backlight(val);
            time::sleep(BACKLIGHT_DELAY);
        }
    }
}

pub fn rect_fill(r: Rect, fg_color: Color) {
    display::bar(r.x0, r.y0, r.width(), r.height(), fg_color.into());
}

pub fn rect_stroke(r: Rect, fg_color: Color) {
    display::bar(r.x0, r.y0, r.width(), 1, fg_color.into());
    display::bar(r.x0, r.y0 + r.height() - 1, r.width(), 1, fg_color.into());
    display::bar(r.x0, r.y0, 1, r.height(), fg_color.into());
    display::bar(r.x0 + r.width() - 1, r.y0, 1, r.height(), fg_color.into());
}

pub fn rect_fill_rounded(r: Rect, fg_color: Color, bg_color: Color, radius: u8) {
    assert!([2, 4, 8, 16].iter().any(|allowed| radius == *allowed));
    display::bar_radius(
        r.x0,
        r.y0,
        r.width(),
        r.height(),
        fg_color.into(),
        bg_color.into(),
        radius,
    );
}

/// NOTE: Cannot start at odd x-coordinate. In this case icon is shifted 1px
/// left.
pub fn icon_top_left(top_left: Point, data: &[u8], fg_color: Color, bg_color: Color) {
    let toif_info = unwrap!(display::toif_info(data), "Invalid TOIF data");
    assert_eq!(toif_info.format, ToifFormat::GrayScaleEH);
    display::icon(
        top_left.x,
        top_left.y,
        toif_info.width.into(),
        toif_info.height.into(),
        &data[12..], // Skip TOIF header.
        fg_color.into(),
        bg_color.into(),
    );
}

pub fn icon(center: Point, data: &[u8], fg_color: Color, bg_color: Color) {
    let toif_info = unwrap!(display::toif_info(data), "Invalid TOIF data");
    assert_eq!(toif_info.format, ToifFormat::GrayScaleEH);

    let r = Rect::from_center_and_size(
        center,
        Offset::new(toif_info.width.into(), toif_info.height.into()),
    );
    display::icon(
        r.x0,
        r.y0,
        r.width(),
        r.height(),
        &data[12..], // Skip TOIF header.
        fg_color.into(),
        bg_color.into(),
    );
}

pub fn icon_rust(center: Point, data: &[u8], fg_color: Color, bg_color: Color) {
    let toif_info = unwrap!(display::toif_info(data), "Invalid TOIF data");
    assert_eq!(toif_info.format, ToifFormat::GrayScaleEH);

    let r = Rect::from_center_and_size(
        center,
        Offset::new(toif_info.width.into(), toif_info.height.into()),
    );

    let area = r.translate(get_offset());
    let clamped = area.clamp(constant::screen());
    let colortable = get_color_table(fg_color, bg_color);

    set_window(clamped);

    let mut dest = [0_u8; 1];

    let mut window = [0; UZLIB_WINDOW_SIZE];
    let mut ctx = UzlibContext::new(&data[12..], Some(&mut window));

    for py in area.y0..area.y1 {
        for px in area.x0..area.x1 {
            let p = Point::new(px, py);
            let x = p.x - area.x0;

            if clamped.contains(p) {
                if x % 2 == 0 {
                    unwrap!(ctx.uncompress(&mut dest), "Decompression failed");
                    pixeldata(colortable[(dest[0] & 0xF) as usize]);
                } else {
                    pixeldata(colortable[(dest[0] >> 4) as usize]);
                }
            } else if x % 2 == 0 {
                //continue unzipping but dont write to display
                unwrap!(ctx.uncompress(&mut dest), "Decompression failed");
            }
        }
    }

    pixeldata_dirty();
}

pub fn image(center: Point, data: &[u8]) {
    let toif_info = unwrap!(display::toif_info(data), "Invalid TOIF data");
    assert_eq!(toif_info.format, ToifFormat::FullColorLE);

    let r = Rect::from_center_and_size(
        center,
        Offset::new(toif_info.width.into(), toif_info.height.into()),
    );
    display::image(
        r.x0,
        r.y0,
        r.width(),
        r.height(),
        &data[12..], // Skip TOIF header.
    );
}

pub fn toif_info(data: &[u8]) -> Option<(Offset, ToifFormat)> {
    if let Ok(info) = display::toif_info(data) {
        Some((
            Offset::new(info.width.into(), info.height.into()),
            info.format,
        ))
    } else {
        None
    }
}

// Used on T1 only.
pub fn rect_fill_rounded1(r: Rect, fg_color: Color, bg_color: Color) {
    display::bar(r.x0, r.y0, r.width(), r.height(), fg_color.into());
    let corners = [
        r.top_left(),
        r.top_right() - Offset::x(1),
        r.bottom_right() - Offset::uniform(1),
        r.bottom_left() - Offset::y(1),
    ];
    for p in corners.iter() {
        display::bar(p.x, p.y, 1, 1, bg_color.into());
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TextOverlay<'a> {
    area: Rect,
    text: &'a str,
    font: Font,
}

impl<'a> TextOverlay<'a> {
    pub fn new(text: &'a str, font: Font) -> Self {
        let area = Rect::zero();
        Self { area, text, font }
    }

    pub fn place(&mut self, baseline: Point) {
        let text_width = self.font.text_width(self.text);
        let text_height = self.font.text_height();

        let text_area_start = baseline + Offset::new(-(text_width / 2), -text_height);
        let text_area_end = baseline + Offset::new(text_width / 2, 0);
        let area = Rect::new(text_area_start, text_area_end);

        self.area = area;
    }

    pub fn get_pixel(&self, underlying: Color, fg: Color, p: Point) -> Color {
        if !self.area.contains(p) {
            return underlying;
        }

        let mut tot_adv = 0;

        let p_rel = Point::new(p.x - self.area.x0, p.y - self.area.y0);

        for g in self.text.bytes().filter_map(|c| self.font.get_glyph(c)) {
            let char_area = Rect::new(
                Point::new(tot_adv + g.bearing_x, g.height - g.bearing_y),
                Point::new(tot_adv + g.bearing_x + g.width, g.bearing_y),
            );

            tot_adv += g.adv;

            if !char_area.contains(p_rel) {
                continue;
            }

            let p_inner = p_rel - char_area.top_left();
            let overlay_data = g.get_pixel_data(p_inner);
            return Color::lerp(underlying, fg, overlay_data as f32 / 15_f32);
        }

        underlying
    }
}

/// Performs a conversion from `angle` (in degrees) to a vector (`Point`)
/// (polar to cartesian transformation)
/// Suitable for cases where we don't care about distance, it is assumed 1000
///
/// The implementation could be replaced by (cos(`angle`), sin(`angle`)),
/// if we allow trigonometric functions.
/// In the meantime, approximate this with predefined octagon
fn get_vector(angle: i32) -> Point {
    //octagon vertices
    let v = [
        Point::new(0, 1000),
        Point::new(707, 707),
        Point::new(1000, 0),
        Point::new(707, -707),
        Point::new(0, -1000),
        Point::new(-707, -707),
        Point::new(-1000, 0),
        Point::new(-707, 707),
    ];

    let angle = angle % 360;
    let vertices = v.len() as i32;
    let sector_length = 360 / vertices; // only works if 360 is divisible by vertices
    let sector = angle / sector_length;
    let sector_angle = (angle % sector_length) as f32;
    let v1 = v[sector as usize];
    let v2 = v[((sector + 1) % vertices) as usize];
    Point::lerp(v1, v2, sector_angle / sector_length as f32)
}

/// Find whether vector `v2` is clockwise to another vector v1
/// `n_v1` is counter clockwise normal vector to v1
/// ( if v1=(x1,y1), then the counter-clockwise normal is n_v1=(-y1,x1)
#[inline(always)]
fn is_clockwise_or_equal(n_v1: Point, v2: Point) -> bool {
    let psize = v2.x * n_v1.x + v2.y * n_v1.y;
    psize < 0
}

/// Find whether vector v2 is clockwise or equal to another vector v1
/// `n_v1` is counter clockwise normal vector to v1
/// ( if v1=(x1,y1), then the counter-clockwise normal is n_v1=(-y1,x1)
#[inline(always)]
fn is_clockwise_or_equal_inc(n_v1: Point, v2: Point) -> bool {
    let psize = v2.x * n_v1.x + v2.y * n_v1.y;
    psize <= 0
}

/// Draw a rounded rectangle with corner radius 2
/// Draws only a part (sector of a corresponding circe)
/// of the rectangle according to `show_percent` argument,
/// and optionally draw an `icon` inside
pub fn rect_rounded2_partial(
    area: Rect,
    fg_color: Color,
    bg_color: Color,
    show_percent: i32,
    icon: Option<(&[u8], Color)>,
) {
    const MAX_ICON_SIZE: u16 = 64;

    let r = area.translate(get_offset());
    let clamped = r.clamp(constant::screen());

    set_window(clamped);

    let center = r.center();
    let colortable = get_color_table(fg_color, bg_color);
    let mut icon_colortable = colortable;

    let mut use_icon = false;
    let mut icon_area = Rect::zero();
    let mut icon_area_clamped = Rect::zero();
    let mut icon_data = [0_u8; ((MAX_ICON_SIZE * MAX_ICON_SIZE) / 2) as usize];
    let mut icon_width = 0;

    if let Some((icon_bytes, icon_color)) = icon {
        let toif_info = unwrap!(display::toif_info(icon_bytes), "Invalid TOIF data");
        assert_eq!(toif_info.format, ToifFormat::GrayScaleEH);

        if toif_info.width <= MAX_ICON_SIZE && toif_info.height <= MAX_ICON_SIZE {
            icon_area = Rect::from_center_and_size(
                center,
                Offset::new(toif_info.width.into(), toif_info.height.into()),
            );
            icon_area_clamped = icon_area.clamp(constant::screen());

            let mut ctx = UzlibContext::new(&icon_bytes[12..], None);
            unwrap!(ctx.uncompress(&mut icon_data), "Decompression failed");
            icon_colortable = get_color_table(icon_color, bg_color);
            icon_width = toif_info.width.into();
            use_icon = true;
        }
    }

    let start = 0;
    let end = (start + ((360 * show_percent) / 100)) % 360;

    let start_vector;
    let end_vector;

    let mut show_all = false;
    let mut inverted = false;

    if show_percent >= 100 {
        show_all = true;
        start_vector = Point::zero();
        end_vector = Point::zero();
    } else if show_percent > 50 {
        inverted = true;
        start_vector = get_vector(end);
        end_vector = get_vector(start);
    } else {
        start_vector = get_vector(start);
        end_vector = get_vector(end);
    }

    let n_start = Point::new(-start_vector.y, start_vector.x);

    for y_c in r.y0..r.y1 {
        for x_c in r.x0..r.x1 {
            let p = Point::new(x_c, y_c);

            let mut icon_pixel = false;
            if use_icon && icon_area_clamped.contains(p) {
                let x_i = p.x - icon_area.x0;
                let y_i = p.y - icon_area.y0;

                let data = icon_data[(((x_i & 0xFE) + (y_i * icon_width)) / 2) as usize];
                if (x_i & 0x01) == 0 {
                    pixeldata(icon_colortable[(data & 0xF) as usize]);
                } else {
                    pixeldata(icon_colortable[(data > 4) as usize]);
                }
                icon_pixel = true;
            }

            if !clamped.contains(p) || icon_pixel {
                continue;
            }

            let y_p = -(p.y - center.y);
            let x_p = p.x - center.x;

            let vx = Point::new(x_p, y_p);
            let n_vx = Point::new(-y_p, x_p);

            let is_past_start = is_clockwise_or_equal(n_start, vx);
            let is_before_end = is_clockwise_or_equal_inc(n_vx, end_vector);

            if show_all
                || (!inverted && (is_past_start && is_before_end))
                || (inverted && !(is_past_start && is_before_end))
            {
                let p_b = p - r.top_left();
                let c = rect_rounded2_get_pixel(p_b, r.size(), colortable, false, 2);
                pixeldata(c);
            } else {
                pixeldata(bg_color);
            }
        }
    }

    pixeldata_dirty();
}

fn position_buffer(
    dest_buffer: &mut [u8],
    src_buffer: &[u8],
    buffer_bpp: usize,
    offset_x: i32,
    data_width: i32,
) {
    // sets the pixel position inside dest buffer
    // end of data will be cut in the DMA transfer
    let start: usize = (offset_x).clamp(0, constant::WIDTH) as usize;
    let end: usize = (offset_x + data_width).clamp(0, constant::WIDTH) as usize;
    let width = end - start;
    // if the offset is negative, need to skip beginning of uncompressed data
    let x_sh = if offset_x < 0 {
        (-offset_x).clamp(0, constant::WIDTH - width as i32) as usize
    } else {
        0
    };
    dest_buffer[((start * buffer_bpp) / 8)..((start + width) * buffer_bpp) / 8].copy_from_slice(
        &src_buffer[((x_sh * buffer_bpp) / 8) as usize..((x_sh as usize + width) * buffer_bpp) / 8],
    );
}

fn process_buffer<const BUFFER_BPP: usize, const BUFFER_SIZE: usize>(
    y: i32,
    r: Rect,
    offset: Offset,
    ctx: &mut UzlibContext,
    buffer: &mut [u8],
    i: &mut i32,
) -> bool {
    let mut not_empty = false;
    let mut uncomp_buffer = [0u8; BUFFER_SIZE];

    if y >= r.y0 && y < r.y1 {
        let line_idx = y - r.y0;

        while *i < line_idx {
            //compensate uncompressed unused lines
            unwrap!(
                ctx.uncompress(
                    &mut uncomp_buffer[0..((r.width() * BUFFER_BPP as i32) / 8) as usize]
                ),
                "Decompression failed"
            );

            (*i) += 1;
        }
        // decompress whole line
        unwrap!(
            ctx.uncompress(&mut uncomp_buffer[0..((r.width() * BUFFER_BPP as i32) / 8) as usize]),
            "Decompression failed"
        );

        (*i) += 1;

        position_buffer(buffer, &uncomp_buffer, BUFFER_BPP, offset.x, r.width());

        not_empty = true;
    }

    not_empty
}

#[cfg(feature = "dma2d")]
pub fn text_over_image(
    bg_area: Option<(Rect, Color)>,
    data: &[u8],
    text: &str,
    font: Font,
    offset_img: Offset,
    offset_text: Offset,
    text_color: Color,
) {
    let text_buffer = get_text_buffer(0, true);
    let img1 = get_buffer_16bpp(0, true);
    let img2 = get_buffer_16bpp(1, true);
    let empty_img = get_buffer_16bpp(2, true);
    let t1 = get_buffer_4bpp(0, true);
    let t2 = get_buffer_4bpp(1, true);
    let empty_t = get_buffer_4bpp(2, true);

    let toif_info = unwrap!(display::toif_info(data), "Invalid TOIF data");
    assert_eq!(toif_info.format, ToifFormat::FullColorLE);
    assert!(toif_info.width <= constant::WIDTH as u16);

    let r_img;
    let area;
    if let Some((a, color)) = bg_area {
        let hi = color.hi_byte();
        let lo = color.lo_byte();
        //prefill image/bg buffers with the bg color
        for i in 0..(constant::WIDTH) as usize {
            img1[2 * i] = lo;
            img1[2 * i + 1] = hi;
            img2[2 * i] = lo;
            img2[2 * i + 1] = hi;
            empty_img[2 * i] = lo;
            empty_img[2 * i + 1] = hi;
        }
        area = a;
        r_img = Rect::from_top_left_and_size(
            a.top_left() + offset_img,
            Offset::new(toif_info.width.into(), toif_info.height.into()),
        );
    } else {
        area = Rect::from_top_left_and_size(
            Point::new(offset_img.x, offset_img.y),
            Offset::new(toif_info.width.into(), toif_info.height.into()),
        );
        r_img = area;
    }
    let clamped = area.clamp(constant::screen());

    let text_width = display::text_width(text, font.0);
    let font_max_height = display::text_max_height(font.0);
    let font_baseline = display::text_baseline(font.0);
    let text_width_clamped = text_width.clamp(0, clamped.width());

    let text_top = offset_text.y - font_max_height + font_baseline;
    let text_bottom = offset_text.y + font_baseline;
    let text_left = offset_text.x;
    let text_right = offset_text.x + text_width_clamped;

    let text_area = Rect::new(
        Point::new(text_left, text_top),
        Point::new(text_right, text_bottom),
    );

    display::text_into_buffer(text, font.0, text_buffer, text_area.x0);

    set_window(clamped);

    let mut window = [0; UZLIB_WINDOW_SIZE];
    let mut ctx = UzlibContext::new(&data[12..], Some(&mut window));

    dma2d_setup_4bpp_over_16bpp(text_color.into());

    let mut i = 0;

    for y in clamped.y0..clamped.y1 {
        let mut img_buffer = &mut *empty_img;
        let mut t_buffer = &mut *empty_t;
        let img_buffer_used;
        let t_buffer_used;

        if y % 2 == 0 {
            t_buffer_used = &mut *t1;
            img_buffer_used = &mut *img1;
        } else {
            t_buffer_used = &mut *t2;
            img_buffer_used = &mut *img2;
        }

        const BUFFER_BPP: usize = 16;

        let using_img = process_buffer::<
            BUFFER_BPP,
            { ((constant::WIDTH as usize) * BUFFER_BPP) / 8 },
        >(y, r_img, offset_img, &mut ctx, img_buffer_used, &mut i);

        if y >= text_area.y0 && y < text_area.y1 {
            let y_pos = y - text_area.y0;
            position_buffer(
                t_buffer_used,
                &text_buffer[(y_pos * constant::WIDTH / 2) as usize
                    ..((y_pos + 1) * constant::WIDTH / 2) as usize],
                4,
                0,
                text_width,
            );
            t_buffer = t_buffer_used;
        }

        if using_img {
            img_buffer = img_buffer_used;
        }

        dma2d_wait_for_transfer();
        dma2d_start_blend(t_buffer, img_buffer, clamped.width());
    }

    dma2d_wait_for_transfer();
}

#[cfg(feature = "dma2d")]
pub fn icon_over_icon(
    bg_area: Option<Rect>,
    bg: (&[u8], Offset, Color),
    fg: (&[u8], Offset, Color),
    bg_color: Color,
) {
    let bg1 = get_buffer_16bpp(0, true);
    let bg2 = get_buffer_16bpp(1, true);
    let empty1 = get_buffer_16bpp(2, true);
    let fg1 = get_buffer_4bpp(0, true);
    let fg2 = get_buffer_4bpp(1, true);
    let empty2 = get_buffer_4bpp(2, true);

    let (data_bg, offset_bg, color_icon_bg) = bg;
    let (data_fg, offset_fg, color_icon_fg) = fg;

    let toif_info_bg = unwrap!(display::toif_info(data_bg), "Invalid TOIF data");
    assert_eq!(toif_info_bg.format, ToifFormat::GrayScaleEH);
    assert!(toif_info_bg.width <= constant::WIDTH as u16);
    assert_eq!(toif_info_bg.width % 2, 0);

    let toif_info_fg = unwrap!(display::toif_info(data_fg), "Invalid TOIF data");
    assert_eq!(toif_info_fg.format, ToifFormat::GrayScaleEH);
    assert!(toif_info_fg.width <= constant::WIDTH as u16);
    assert_eq!(toif_info_fg.width % 2, 0);

    let area;
    let r_bg;
    if let Some(a) = bg_area {
        area = a;
        r_bg = Rect::from_top_left_and_size(
            a.top_left() + offset_bg,
            Offset::new(toif_info_bg.width.into(), toif_info_bg.height.into()),
        );
    } else {
        r_bg = Rect::from_top_left_and_size(
            Point::new(offset_bg.x, offset_bg.y),
            Offset::new(toif_info_bg.width.into(), toif_info_bg.height.into()),
        );
        area = r_bg;
    }

    let r_fg = Rect::from_top_left_and_size(
        area.top_left() + offset_fg,
        Offset::new(toif_info_fg.width.into(), toif_info_fg.height.into()),
    );

    let clamped = area.clamp(constant::screen());

    set_window(clamped);

    let mut window_bg = [0; UZLIB_WINDOW_SIZE];
    let mut ctx_bg = UzlibContext::new(&data_bg[12..], Some(&mut window_bg));

    let mut window_fg = [0; UZLIB_WINDOW_SIZE];
    let mut ctx_fg = UzlibContext::new(&data_fg[12..], Some(&mut window_fg));

    dma2d_setup_4bpp_over_4bpp(color_icon_bg.into(), bg_color.into(), color_icon_fg.into());

    let mut fg_i = 0;
    let mut bg_i = 0;

    for y in clamped.y0..clamped.y1 {
        let mut fg_buffer = &mut *empty2;
        let mut bg_buffer = &mut *empty1;
        let fg_buffer_used;
        let bg_buffer_used;

        if y % 2 == 0 {
            fg_buffer_used = &mut *fg1;
            bg_buffer_used = &mut *bg1;
        } else {
            fg_buffer_used = &mut *fg2;
            bg_buffer_used = &mut *bg2;
        }

        const BUFFER_BPP: usize = 4;

        let using_fg = process_buffer::<BUFFER_BPP, { (constant::WIDTH as usize * BUFFER_BPP) / 8 }>(
            y,
            r_fg,
            offset_fg,
            &mut ctx_fg,
            fg_buffer_used,
            &mut fg_i,
        );
        let using_bg = process_buffer::<BUFFER_BPP, { (constant::WIDTH as usize * BUFFER_BPP) / 8 }>(
            y,
            r_bg,
            offset_bg,
            &mut ctx_bg,
            bg_buffer_used,
            &mut bg_i,
        );

        if using_fg {
            fg_buffer = fg_buffer_used;
        }
        if using_bg {
            bg_buffer = bg_buffer_used;
        }

        dma2d_wait_for_transfer();
        dma2d_start_blend(fg_buffer, bg_buffer, clamped.width());
    }

    dma2d_wait_for_transfer();
}

/// Gets a color of a pixel on `p` coordinates of rounded rectangle with corner
/// radius 2
fn rect_rounded2_get_pixel(
    p: Offset,
    size: Offset,
    colortable: [Color; 16],
    fill: bool,
    line_width: i32,
) -> Color {
    let border = (p.x >= 0 && p.x < line_width)
        || ((p.x >= size.x - line_width) && p.x <= (size.x - 1))
        || (p.y >= 0 && p.y < line_width)
        || ((p.y >= size.y - line_width) && p.y <= (size.y - 1));

    let corner_lim = 2 * line_width;
    let corner_inner = line_width;

    let corner_all = ((p.x > size.x - (corner_lim + 1)) || p.x < corner_lim)
        && (p.y < corner_lim || p.y > size.y - (corner_lim + 1));

    let corner = corner_all
        && (p.y >= corner_inner)
        && (p.x >= corner_inner)
        && (p.y <= size.y - (corner_inner + 1))
        && (p.x <= size.x - (corner_inner + 1));

    let corner_out = corner_all && !corner;

    if (border || corner || fill) && !corner_out {
        colortable[15]
    } else {
        colortable[0]
    }
}

/// Draws a rounded rectangle with corner radius 2, partially filled
/// according to `fill_from` and `fill_to` arguments.
/// Optionally draws a text inside the rectangle and adjusts its color to match
/// the fill. The coordinates of the text are specified in the TextOverlay
/// struct.
pub fn bar_with_text_and_fill(
    area: Rect,
    overlay: Option<TextOverlay>,
    fg_color: Color,
    bg_color: Color,
    fill_from: i32,
    fill_to: i32,
) {
    let r = area.translate(get_offset());
    let clamped = r.clamp(constant::screen());
    let colortable = get_color_table(fg_color, bg_color);

    set_window(clamped);

    for y_c in clamped.y0..clamped.y1 {
        for x_c in clamped.x0..clamped.x1 {
            let p = Point::new(x_c, y_c);
            let r_offset = p - r.top_left();

            let filled = (r_offset.x >= fill_from
                && fill_from >= 0
                && (r_offset.x <= fill_to || fill_to < fill_from))
                || (r_offset.x < fill_to && fill_to >= 0);

            let underlying_color =
                rect_rounded2_get_pixel(r_offset, r.size(), colortable, filled, 1);

            let final_color = overlay.map_or(underlying_color, |o| {
                let text_color = if filled { bg_color } else { fg_color };
                o.get_pixel(underlying_color, text_color, p)
            });

            pixeldata(final_color);
        }
    }
    pixeldata_dirty();
}

// Used on T1 only.
pub fn dotted_line(start: Point, width: i32, color: Color) {
    for x in (start.x..width).step_by(2) {
        display::bar(x, start.y, 1, 1, color.into());
    }
}

pub fn qrcode(center: Point, data: &str, max_size: u32, case_sensitive: bool) -> Result<(), Error> {
    qr::render_qrcode(center.x, center.y, data, max_size, case_sensitive)
}

pub fn text(baseline: Point, text: &str, font: Font, fg_color: Color, bg_color: Color) {
    display::text(
        baseline.x,
        baseline.y,
        text,
        font.0,
        fg_color.into(),
        bg_color.into(),
    );
}

pub fn text_center(baseline: Point, text: &str, font: Font, fg_color: Color, bg_color: Color) {
    let w = font.text_width(text);
    display::text(
        baseline.x - w / 2,
        baseline.y,
        text,
        font.0,
        fg_color.into(),
        bg_color.into(),
    );
}

pub fn text_right(baseline: Point, text: &str, font: Font, fg_color: Color, bg_color: Color) {
    let w = font.text_width(text);
    display::text(
        baseline.x - w,
        baseline.y,
        text,
        font.0,
        fg_color.into(),
        bg_color.into(),
    );
}

#[inline(always)]
pub fn pixeldata(color: Color) {
    display::pixeldata(color.into());
}

pub fn pixeldata_dirty() {
    display::pixeldata_dirty();
}

pub fn get_offset() -> Offset {
    let offset = display::get_offset();
    Offset::new(offset.0, offset.1)
}

pub fn set_window(window: Rect) {
    display::set_window(
        window.x0 as u16,
        window.y0 as u16,
        window.x1 as u16 - 1,
        window.y1 as u16 - 1,
    );
}

pub fn get_color_table(fg_color: Color, bg_color: Color) -> [Color; 16] {
    let mut table: [Color; 16] = [Color::from_u16(0); 16];

    for (i, item) in table.iter_mut().enumerate() {
        *item = Color::lerp(bg_color, fg_color, i as f32 / 15_f32);
    }

    table
}

pub struct Glyph {
    pub width: i32,
    pub height: i32,
    pub adv: i32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    data: &'static [u8],
}

impl Glyph {
    /// Construct a `Glyph` from a raw pointer.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller has to guarantee that `data`
    /// is pointing to a memory containing a valid glyph data, that is:
    /// - contains valid glyph metadata
    /// - data has appropriate size
    /// - data must have static lifetime
    pub unsafe fn load(data: *const u8) -> Self {
        unsafe {
            let width = *data.offset(0) as i32;
            let height = *data.offset(1) as i32;

            let data_bits = constant::FONT_BPP * width * height;

            let data_bytes = if data_bits % 8 == 0 {
                data_bits / 8
            } else {
                (data_bits / 8) + 1
            };

            Glyph {
                width,
                height,
                adv: *data.offset(2) as i32,
                bearing_x: *data.offset(3) as i32,
                bearing_y: *data.offset(4) as i32,
                data: from_raw_parts(data.offset(5), data_bytes as usize),
            }
        }
    }

    pub fn print(&self, pos: Point, colortable: [Color; 16]) -> i32 {
        let bearing = Offset::new(self.bearing_x, -self.bearing_y);
        let size = Offset::new(self.width, self.height);
        let pos_adj = pos + bearing;
        let r = Rect::from_top_left_and_size(pos_adj, size);

        let area = r.translate(get_offset());
        let window = area.clamp(constant::screen());

        set_window(window);

        for y in window.y0..window.y1 {
            for x in window.x0..window.x1 {
                let p = Point::new(x, y);
                let r = p - pos_adj;
                let c = self.get_pixel_data(r);
                pixeldata(colortable[c as usize]);
            }
        }
        self.adv
    }

    pub fn unpack_bpp1(&self, a: i32) -> u8 {
        let c_data = self.data[(a / 8) as usize];
        ((c_data >> (7 - (a % 8))) & 0x01) * 15
    }

    pub fn unpack_bpp2(&self, a: i32) -> u8 {
        let c_data = self.data[(a / 4) as usize];
        ((c_data >> (6 - (a % 4) * 2)) & 0x03) * 5
    }

    pub fn unpack_bpp4(&self, a: i32) -> u8 {
        let c_data = self.data[(a / 2) as usize];
        (c_data >> (4 - (a % 2) * 4)) & 0x0F
    }

    pub fn unpack_bpp8(&self, a: i32) -> u8 {
        let c_data = self.data[a as usize];
        c_data >> 4
    }

    pub fn get_pixel_data(&self, p: Offset) -> u8 {
        let a = p.x + p.y * self.width;

        match constant::FONT_BPP {
            1 => self.unpack_bpp1(a),
            2 => self.unpack_bpp2(a),
            4 => self.unpack_bpp4(a),
            8 => self.unpack_bpp8(a),
            _ => 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Font(i32);

impl Font {
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    pub fn text_width(self, text: &str) -> i32 {
        display::text_width(text, self.0)
    }

    pub fn char_width(self, ch: char) -> i32 {
        display::char_width(ch, self.0)
    }

    pub fn text_height(self) -> i32 {
        display::text_height(self.0)
    }

    pub fn line_height(self) -> i32 {
        constant::LINE_SPACE + self.text_height()
    }

    pub fn get_glyph(self, char_byte: u8) -> Option<Glyph> {
        let gl_data = display::get_char_glyph(char_byte, self.0);

        if gl_data.is_null() {
            return None;
        }
        unsafe { Some(Glyph::load(gl_data)) }
    }

    pub fn display_text(self, text: &str, baseline: Point, fg_color: Color, bg_color: Color) {
        let colortable = get_color_table(fg_color, bg_color);
        let mut adv_total = 0;
        for c in text.bytes() {
            let g = self.get_glyph(c);
            if let Some(gly) = g {
                let adv = gly.print(baseline + Offset::new(adv_total, 0), colortable);
                adv_total += adv;
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Color(u16);

impl Color {
    pub const fn from_u16(val: u16) -> Self {
        Self(val)
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        let r = (r as u16 & 0xF8) << 8;
        let g = (g as u16 & 0xFC) << 3;
        let b = (b as u16 & 0xF8) >> 3;
        Self(r | g | b)
    }

    pub const fn luminance(self) -> u32 {
        ((self.r() as u32 * 299) / 1000)
            + (self.g() as u32 * 587) / 1000
            + (self.b() as u32 * 114) / 1000
    }

    pub const fn r(self) -> u8 {
        (self.0 >> 8) as u8 & 0xF8
    }

    pub const fn g(self) -> u8 {
        (self.0 >> 3) as u8 & 0xFC
    }

    pub const fn b(self) -> u8 {
        (self.0 << 3) as u8 & 0xF8
    }

    pub fn to_u16(self) -> u16 {
        self.0
    }

    pub fn hi_byte(self) -> u8 {
        (self.to_u16() >> 8) as u8
    }

    pub fn lo_byte(self) -> u8 {
        (self.to_u16() & 0xFF) as u8
    }

    pub fn negate(self) -> Self {
        Self(!self.0)
    }
}

impl Lerp for Color {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        let r = u8::lerp(a.r(), b.r(), t);
        let g = u8::lerp(a.g(), b.g(), t);
        let b = u8::lerp(a.b(), b.b(), t);
        Color::rgb(r, g, b)
    }
}

impl From<u16> for Color {
    fn from(val: u16) -> Self {
        Self(val)
    }
}

impl From<Color> for u16 {
    fn from(val: Color) -> Self {
        val.to_u16()
    }
}
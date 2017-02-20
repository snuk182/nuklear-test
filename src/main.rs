#[macro_use]
extern crate nuklear_rust;
extern crate nuklear_backend_gfx;

extern crate image;

extern crate gfx;
extern crate gfx_window_dxgi;

extern crate winit;

use nuklear_rust::*;
use nuklear_backend_gfx::{Drawer, GfxBackend};

use gfx::Device as Gd;

use std::fs::*;
use std::io::BufReader;

pub type RenderFormat = gfx::format::Rgba8;

const MAX_VERTEX_MEMORY: usize = 512 * 1024;
const MAX_ELEMENT_MEMORY: usize = 128 * 1024;
const MAX_COMMANDS_MEMORY: usize = 64 * 1024;

struct BasicState {
    image_active: bool,
    check0: bool,
    check1: bool,
    prog: usize,
    selected_item: usize,
    selected_image: usize,
    selected_icon: usize,
    items: [&'static str; 3],
    piemenu_active: bool,
    piemenu_pos: NkVec2,
}

struct ButtonState {
    option: i32,
    toggle0: bool,
    toggle1: bool,
    toggle2: bool,
}

struct GridState {
    text: [[u8; 64]; 4],
    text_len: [i32; 4],
    items: [&'static str; 4],
    selected_item: usize,
    check: bool,
}

#[allow(dead_code)]
struct Media {
    font_14: NkFont,
    font_18: NkFont,
    font_20: NkFont,
    font_22: NkFont,

    font_tex: NkHandle,

    unchecked: NkImage,
    checked: NkImage,
    rocket: NkImage,
    cloud: NkImage,
    pen: NkImage,
    play: NkImage,
    pause: NkImage,
    stop: NkImage,
    prev: NkImage,
    next: NkImage,
    tools: NkImage,
    dir: NkImage,
    copy: NkImage,
    convert: NkImage,
    del: NkImage,
    edit: NkImage,
    images: [NkImage; 9],
    menu: [NkImage; 6],
}

fn icon_load<F, R: gfx::Resources>(factory: &mut F, drawer: &mut Drawer<R>, filename: &str) -> NkImage
    where F: gfx::Factory<R>
{

    let img = image::load(BufReader::new(File::open(filename).unwrap()), image::PNG).unwrap().to_rgba();

    let (w, h) = img.dimensions();
    let mut hnd = drawer.add_texture(factory, &img, w, h);

    NkImage::with_id(hnd.id().unwrap())
}

fn main() {
	let mut fw = 1280;
	let mut fh = 800;
    let (mut window, mut device, mut factory, render_target) = gfx_window_dxgi::init::<RenderFormat>(
    	winit::WindowBuilder::new()
    	    .with_dimensions(fw, fh)
    	    .with_title("dx11 window"))
    .unwrap();
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let mut cfg = NkFontConfig::new(0.0);
    cfg.set_oversample_h(3);
    cfg.set_oversample_v(2);
    cfg.set_glyph_range(nuklear_rust::font_cyrillic_glyph_ranges());
    cfg.set_ttf(include_bytes!("../res/fonts/Roboto-Regular.ttf"));
    cfg.set_ttf_data_owned_by_atlas(true);

    let mut allo = NkAllocator::new_vec();

    let mut drawer = Drawer::new(&mut factory,
                                 &render_target,
                                 36,
                                 MAX_VERTEX_MEMORY,
                                 MAX_ELEMENT_MEMORY,
                                 NkBuffer::with_size(&mut allo, MAX_COMMANDS_MEMORY),
                                 GfxBackend::DX11Hlsl
    );
    
    let mut atlas = NkFontAtlas::new(&mut allo);

    cfg.set_size(14f32);
    let mut font_14 = atlas.add_font_with_config(&cfg).unwrap();
    cfg.set_size(18f32);
    let font_18 = atlas.add_font_with_config(&cfg).unwrap();
    cfg.set_size(20f32);
    let font_20 = atlas.add_font_with_config(&cfg).unwrap();
    cfg.set_size(22f32);
    let font_22 = atlas.add_font_with_config(&cfg).unwrap();

    let font_tex = {
        let (b, w, h) = atlas.bake(NkFontAtlasFormat::NK_FONT_ATLAS_RGBA32);
        drawer.add_texture(&mut factory, b, w, h)
    };

    let mut null = NkDrawNullTexture::default();

    atlas.end(font_tex, Some(&mut null));

    let mut ctx = NkContext::new(&mut allo, &font_14.handle());

    let mut media = Media {
        font_14: font_14,
        font_18: font_18,
        font_20: font_20,
        font_22: font_22,

        font_tex: font_tex,

        unchecked: icon_load(&mut factory, &mut drawer, "res/icon/unchecked.png"),
        checked: icon_load(&mut factory, &mut drawer, "res/icon/checked.png"),
        rocket: icon_load(&mut factory, &mut drawer, "res/icon/rocket.png"),
        cloud: icon_load(&mut factory, &mut drawer, "res/icon/cloud.png"),
        pen: icon_load(&mut factory, &mut drawer, "res/icon/pen.png"),
        play: icon_load(&mut factory, &mut drawer, "res/icon/play.png"),
        pause: icon_load(&mut factory, &mut drawer, "res/icon/pause.png"),
        stop: icon_load(&mut factory, &mut drawer, "res/icon/stop.png"),
        prev: icon_load(&mut factory, &mut drawer, "res/icon/prev.png"),
        next: icon_load(&mut factory, &mut drawer, "res/icon/next.png"),
        tools: icon_load(&mut factory, &mut drawer, "res/icon/tools.png"),
        dir: icon_load(&mut factory, &mut drawer, "res/icon/directory.png"),
        copy: icon_load(&mut factory, &mut drawer, "res/icon/copy.png"),
        convert: icon_load(&mut factory, &mut drawer, "res/icon/export.png"),
        del: icon_load(&mut factory, &mut drawer, "res/icon/delete.png"),
        edit: icon_load(&mut factory, &mut drawer, "res/icon/edit.png"),
        images: [icon_load(&mut factory, &mut drawer, "res/images/image1.png"),
                 icon_load(&mut factory, &mut drawer, "res/images/image2.png"),
                 icon_load(&mut factory, &mut drawer, "res/images/image3.png"),
                 icon_load(&mut factory, &mut drawer, "res/images/image4.png"),
                 icon_load(&mut factory, &mut drawer, "res/images/image5.png"),
                 icon_load(&mut factory, &mut drawer, "res/images/image6.png"),
                 icon_load(&mut factory, &mut drawer, "res/images/image7.png"),
                 icon_load(&mut factory, &mut drawer, "res/images/image8.png"),
                 icon_load(&mut factory, &mut drawer, "res/images/image9.png")],
        menu: [icon_load(&mut factory, &mut drawer, "res/icon/home.png"),
               icon_load(&mut factory, &mut drawer, "res/icon/phone.png"),
               icon_load(&mut factory, &mut drawer, "res/icon/plane.png"),
               icon_load(&mut factory, &mut drawer, "res/icon/wifi.png"),
               icon_load(&mut factory, &mut drawer, "res/icon/settings.png"),
               icon_load(&mut factory, &mut drawer, "res/icon/volume.png")],
    };

    let mut basic_state = BasicState {
        image_active: false,
        check0: true,
        check1: false,
        prog: 80,
        selected_item: 0,
        selected_image: 3,
        selected_icon: 0,
        items: ["Item 0", "item 1", "item 2"],
        piemenu_active: false,
        piemenu_pos: NkVec2::default(),
    };

    let mut button_state = ButtonState {
        option: 1,
        toggle0: true,
        toggle1: false,
        toggle2: true,
    };

    let mut grid_state = GridState {
        text: [[0; 64]; 4],
        text_len: [0; 4],
        items: ["Item 0", "item 1", "item 2", "Item 4"],
        selected_item: 2,
        check: true,
    };

    let mut mx = 0;
    let mut my = 0;

    let mut config = NkConvertConfig::default();
    config.set_null(null.clone());
    config.set_circle_segment_count(22);
    config.set_curve_segment_count(22);
    config.set_arc_segment_count(22);
    config.set_global_alpha(1.0f32);
    config.set_shape_aa(NkAntiAliasing::NK_ANTI_ALIASING_ON);
    config.set_line_aa(NkAntiAliasing::NK_ANTI_ALIASING_ON);
    
    let mut resize = None;

    'main: loop {
    	//not yet published
    	/*if let Some((width, height)) = resize {
    		drawer.set_render_target(None);
    		
    		let new_rtv = gfx_window_dxgi::update_views::<RenderFormat>(&mut window, &mut factory, &mut device, width, height).unwrap();
    		drawer.set_render_target(Some(new_rtv));
    		fw = width as u32;
    		fh = height as u32;

            resize = None;
        }*/

        ctx.input_begin();
        for event in window.poll_events() {
            // println!("{:?}", event);

            match event {
                winit::Event::Closed => break 'main,
                winit::Event::ReceivedCharacter(c) => {
                    ctx.input_unicode(c);
                }
                winit::Event::KeyboardInput(s, _, k) => {
                    if let Some(k) = k {
                        let key = match k {
                            winit::VirtualKeyCode::Back => NkKey::NK_KEY_BACKSPACE,
                            winit::VirtualKeyCode::Delete => NkKey::NK_KEY_DEL,
                            winit::VirtualKeyCode::Up => NkKey::NK_KEY_UP,
                            winit::VirtualKeyCode::Down => NkKey::NK_KEY_DOWN,
                            winit::VirtualKeyCode::Left => NkKey::NK_KEY_LEFT,
                            winit::VirtualKeyCode::Right => NkKey::NK_KEY_RIGHT,
                            _ => NkKey::NK_KEY_NONE,
                        };

                        ctx.input_key(key, s == winit::ElementState::Pressed);
                    }
                }
                winit::Event::MouseMoved(x, y) => {
                    mx = x;
                    my = y;
                    ctx.input_motion(x, y);
                }
                winit::Event::MouseInput(s, b) => {
                    let button = match b {
                        winit::MouseButton::Left => NkButton::NK_BUTTON_LEFT,
                        winit::MouseButton::Middle => NkButton::NK_BUTTON_MIDDLE,
                        winit::MouseButton::Right => NkButton::NK_BUTTON_RIGHT,
                        _ => NkButton::NK_BUTTON_MAX,
                    };

                    ctx.input_button(button, mx, my, s == winit::ElementState::Pressed)
                }
                winit::Event::MouseWheel(d, _) => {
                    if let winit::MouseScrollDelta::LineDelta(_, y) = d {
                        ctx.input_scroll(y * 22f32);
                    }
                }
                winit::Event::Resized(w, h) => {
                	resize = Some((w as u16, h as u16));
                }
                _ => (),
            }
        }
        ctx.input_end();
        
        // println!("{:?}", event);
        //let (w, h) = window.get_inner_size_pixels().unwrap();
        //let (fw, fh) = window.get_inner_size().unwrap();
        let scale = NkVec2 {
            x: 1.0,
            y: 1.0,
        };

        basic_demo(&mut ctx, &mut media, &mut basic_state);
        button_demo(&mut ctx, &mut media, &mut button_state);
        grid_demo(&mut ctx, &mut media, &mut grid_state);

        encoder.clear(&render_target, [0.1f32, 0.2f32, 0.3f32, 1.0f32]);
        drawer.draw(&mut ctx,
                    &mut config,
                    &mut encoder,
                    &mut factory,
                    fw as u32,
                    fh as u32,
                    scale);
        encoder.flush(&mut device);
        window.swap_buffers(0);
        device.cleanup();

        ::std::thread::sleep(::std::time::Duration::from_millis(20));

        ctx.clear();
    }

    atlas.clear();
    ctx.free();
}

fn ui_header(ctx: &mut NkContext, media: &mut Media, title: &str) {
    ctx.style_set_font(&media.font_18.handle());
    ctx.layout_row_dynamic(20f32, 1);
    ctx.text(title, NkTextAlignment::NK_TEXT_LEFT as NkFlags);
}

const RATIO_W: [f32; 2] = [0.15f32, 0.85f32];
fn ui_widget(ctx: &mut NkContext, media: &mut Media, height: f32) {
    ctx.style_set_font(&media.font_22.handle());
    ctx.layout_row(NkLayoutFormat::NK_DYNAMIC, height, &RATIO_W);
    // ctx.layout_row_dynamic(height, 1);
    ctx.spacing(1);
}

const RATIO_WC: [f32; 3] = [0.15f32, 0.50f32, 0.35f32];
fn ui_widget_centered(ctx: &mut NkContext, media: &mut Media, height: f32) {
    ctx.style_set_font(&media.font_22.handle());
    ctx.layout_row(NkLayoutFormat::NK_DYNAMIC, height, &RATIO_WC);
    ctx.spacing(1);
}

fn free_type(_: NkTextEdit, c: char) -> bool {
    (c > '\u{0030}')
}

fn grid_demo(ctx: &mut NkContext, media: &mut Media, state: &mut GridState) {
    ctx.style_set_font(&media.font_20.handle());
    if ctx.begin(nk_string!("Grid Nuklear Rust!"),
                 NkRect {
                     x: 600f32,
                     y: 350f32,
                     w: 275f32,
                     h: 250f32,
                 },
                 NkPanelFlags::NK_WINDOW_BORDER as NkFlags | NkPanelFlags::NK_WINDOW_MOVABLE as NkFlags | NkPanelFlags::NK_WINDOW_TITLE as NkFlags | NkPanelFlags::NK_WINDOW_NO_SCROLLBAR as NkFlags) {
        ctx.style_set_font(&media.font_18.handle());
        ctx.layout_row_dynamic(30f32, 2);
        ctx.text("Free type:", NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
        ctx.edit_string_custom_filter(NkEditType::NK_EDIT_FIELD as NkFlags,
                                      &mut state.text[3],
                                      &mut state.text_len[3],
                                      free_type);
        ctx.text("Floating point:", NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
        ctx.edit_string(NkEditType::NK_EDIT_FIELD as NkFlags,
                        &mut state.text[0],
                        &mut state.text_len[0],
                        NK_FILTER_FLOAT);
        ctx.text("Hexadecimal:", NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
        ctx.edit_string(NkEditType::NK_EDIT_FIELD as NkFlags,
                        &mut state.text[1],
                        &mut state.text_len[1],
                        NK_FILTER_HEX);
        ctx.text("Binary:", NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
        ctx.edit_string(NkEditType::NK_EDIT_FIELD as NkFlags,
                        &mut state.text[2],
                        &mut state.text_len[2],
                        NK_FILTER_BINARY);
        ctx.text("Checkbox:", NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
        ctx.checkbox_text("Check me", &mut state.check);
        ctx.text("Combobox:", NkTextAlignment::NK_TEXT_RIGHT as NkFlags);

        let widget_width = ctx.widget_width();
        if ctx.combo_begin_text(state.items[state.selected_item],
                                NkVec2 {
                                    x: widget_width,
                                    y: 200f32,
                                }) {
            ctx.layout_row_dynamic(25f32, 1);
            for i in 0..state.items.len() {
                if ctx.combo_item_text(state.items[i], NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
                    state.selected_item = i;
                }
            }
            ctx.combo_end();
        }
    }
    ctx.end();
    ctx.style_set_font(&media.font_14.handle());
}

fn button_demo(ctx: &mut NkContext, media: &mut Media, state: &mut ButtonState) {
    ctx.style_set_font(&media.font_20.handle());

    ctx.begin(nk_string!("Button Nuklear Rust!"),
              NkRect {
                  x: 50f32,
                  y: 50f32,
                  w: 255f32,
                  h: 610f32,
              },
              NkPanelFlags::NK_WINDOW_BORDER as NkFlags | NkPanelFlags::NK_WINDOW_MOVABLE as NkFlags | NkPanelFlags::NK_WINDOW_TITLE as NkFlags);

    // ------------------------------------------------
    //                  MENU
    // ------------------------------------------------
    ctx.menubar_begin();
    {
        // toolbar
        ctx.layout_row_static(40f32, 40, 4);
        if ctx.menu_begin_image(nk_string!("Music"),
                                media.play.clone(),
                                NkVec2 {
                                    x: 110f32,
                                    y: 120f32,
                                }) {
            // settings
            ctx.layout_row_dynamic(25f32, 1);
            ctx.menu_item_image_text(media.play.clone(),
                                     "Play",
                                     NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
            ctx.menu_item_image_text(media.stop.clone(),
                                     "Stop",
                                     NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
            ctx.menu_item_image_text(media.pause.clone(),
                                     "Pause",
                                     NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
            ctx.menu_item_image_text(media.next.clone(),
                                     "Next",
                                     NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
            ctx.menu_item_image_text(media.prev.clone(),
                                     "Prev",
                                     NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
            ctx.menu_end();
        }
        ctx.button_image(media.tools.clone());
        ctx.button_image(media.cloud.clone());
        ctx.button_image(media.pen.clone());
    }
    ctx.menubar_end();

    // ------------------------------------------------
    //                  BUTTON
    // ------------------------------------------------
    ui_header(ctx, media, "Push buttons");
    ui_widget(ctx, media, 35f32);
    if ctx.button_text("Push me") {
        println!("pushed!");
    }
    ui_widget(ctx, media, 35f32);
    if ctx.button_image_text(media.rocket.clone(),
                             "Styled",
                             NkTextAlignment::NK_TEXT_CENTERED as NkFlags) {
        println!("rocket!");
    }

    // ------------------------------------------------
    //                  REPEATER
    // ------------------------------------------------
    ui_header(ctx, media, "Repeater");
    ui_widget(ctx, media, 35f32);
    if ctx.button_text("Press me") {
        println!("pressed!");
    }

    // ------------------------------------------------
    //                  TOGGLE
    // ------------------------------------------------
    ui_header(ctx, media, "Toggle buttons");
    ui_widget(ctx, media, 35f32);
    if ctx.button_image_text(if state.toggle0 {
                                 media.checked.clone()
                             } else {
                                 media.unchecked.clone()
                             },
                             "Toggle",
                             NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.toggle0 = !state.toggle0;
    }

    ui_widget(ctx, media, 35f32);
    if ctx.button_image_text(if state.toggle1 {
                                 media.checked.clone()
                             } else {
                                 media.unchecked.clone()
                             },
                             "Toggle",
                             NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.toggle1 = !state.toggle1;
    }

    ui_widget(ctx, media, 35f32);
    if ctx.button_image_text(if state.toggle2 {
                                 media.checked.clone()
                             } else {
                                 media.unchecked.clone()
                             },
                             "Toggle",
                             NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.toggle2 = !state.toggle2;
    }

    // ------------------------------------------------
    //                  RADIO
    // ------------------------------------------------
    ui_header(ctx, media, "Radio buttons");
    ui_widget(ctx, media, 35f32);
    if ctx.button_symbol_text(if state.option == 0 {
                                  NkSymbolType::NK_SYMBOL_CIRCLE_OUTLINE
                              } else {
                                  NkSymbolType::NK_SYMBOL_CIRCLE_SOLID
                              },
                              "Select 1",
                              NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.option = 0;
    }
    ui_widget(ctx, media, 35f32);
    if ctx.button_symbol_text(if state.option == 1 {
                                  NkSymbolType::NK_SYMBOL_CIRCLE_OUTLINE
                              } else {
                                  NkSymbolType::NK_SYMBOL_CIRCLE_SOLID
                              },
                              "Select 2",
                              NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.option = 1;
    }
    ui_widget(ctx, media, 35f32);
    if ctx.button_symbol_text(if state.option == 2 {
                                  NkSymbolType::NK_SYMBOL_CIRCLE_OUTLINE
                              } else {
                                  NkSymbolType::NK_SYMBOL_CIRCLE_SOLID
                              },
                              "Select 3",
                              NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.option = 2;
    }

    // ------------------------------------------------
    //                  CONTEXTUAL
    // ------------------------------------------------
    ctx.style_set_font(&media.font_18.handle());
    let bounds = ctx.window_get_bounds();
    if ctx.contextual_begin(NkPanelFlags::NK_WINDOW_NO_SCROLLBAR as NkFlags,
                            NkVec2 {
                                x: 150f32,
                                y: 300f32,
                            },
                            bounds) {
        ctx.layout_row_dynamic(30f32, 1);
        if ctx.contextual_item_image_text(media.copy.clone(),
                                          "Clone",
                                          NkTextAlignment::NK_TEXT_RIGHT as NkFlags) {
            println!("pressed clone!");
        }
        if ctx.contextual_item_image_text(media.del.clone(),
                                          "Delete",
                                          NkTextAlignment::NK_TEXT_RIGHT as NkFlags) {
            println!("pressed delete!");
        }
        if ctx.contextual_item_image_text(media.convert.clone(),
                                          "Convert",
                                          NkTextAlignment::NK_TEXT_RIGHT as NkFlags) {
            println!("pressed convert!");
        }
        if ctx.contextual_item_image_text(media.edit.clone(),
                                          "Edit",
                                          NkTextAlignment::NK_TEXT_RIGHT as NkFlags) {
            println!("pressed edit!");
        }
        ctx.contextual_end();
    }
    ctx.style_set_font(&media.font_14.handle());
    ctx.end();
}

fn basic_demo(ctx: &mut NkContext, media: &mut Media, state: &mut BasicState) {
    ctx.style_set_font(&media.font_20.handle());
    ctx.begin(nk_string!("Basic Nuklear Rust!"),
              NkRect {
                  x: 320f32,
                  y: 50f32,
                  w: 275f32,
                  h: 610f32,
              },
              NkPanelFlags::NK_WINDOW_BORDER as NkFlags | NkPanelFlags::NK_WINDOW_MOVABLE as NkFlags | NkPanelFlags::NK_WINDOW_TITLE as NkFlags);

    // ------------------------------------------------
    //                  POPUP BUTTON
    // ------------------------------------------------


    ui_header(ctx, media, "Popup & Scrollbar & Images");
    ui_widget(ctx, media, 35f32);
    if ctx.button_image_text(media.dir.clone(),
                             "Images",
                             NkTextAlignment::NK_TEXT_CENTERED as NkFlags) {
        state.image_active = !state.image_active;
    }

    // ------------------------------------------------
    //                  SELECTED IMAGE
    // ------------------------------------------------
    ui_header(ctx, media, "Selected Image");
    ui_widget_centered(ctx, media, 100f32);
    ctx.image(media.images[state.selected_image].clone());

    // ------------------------------------------------
    //                  IMAGE POPUP
    // ------------------------------------------------
    if state.image_active {
        if ctx.popup_begin(NkPopupType::NK_POPUP_STATIC,
                           nk_string!("Image Popup"),
                           0,
                           NkRect {
                               x: 265f32,
                               y: 0f32,
                               w: 320f32,
                               h: 220f32,
                           }) {
            ctx.layout_row_static(82f32, 82, 3);
            for i in 0..9 {
                if ctx.button_image(media.images[i].clone()) {
                    state.selected_image = i;
                    state.image_active = false;
                    ctx.popup_close();
                }
            }
            ctx.popup_end();
        }
    }
    // ------------------------------------------------
    //                  COMBOBOX
    // ------------------------------------------------
    ui_header(ctx, media, "Combo box");
    ui_widget(ctx, media, 40f32);
    let widget_width = ctx.widget_width();
    if ctx.combo_begin_text(state.items[state.selected_item],
                            NkVec2 {
                                x: widget_width,
                                y: 200f32,
                            }) {
        ctx.layout_row_dynamic(35f32, 1);
        for i in 0..3 {
            if ctx.combo_item_text(state.items[i], NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
                state.selected_item = i;
            }
        }
        ctx.combo_end();
    }

    ui_widget(ctx, media, 40f32);
    let widget_width = ctx.widget_width();
    if ctx.combo_begin_image_text(state.items[state.selected_icon],
                                  media.images[state.selected_icon].clone(),
                                  NkVec2 {
                                      x: widget_width,
                                      y: 200f32,
                                  }) {
        ctx.layout_row_dynamic(35f32, 1);
        for i in 0..3 {
            if ctx.combo_item_image_text(media.images[i].clone(),
                                         state.items[i],
                                         NkTextAlignment::NK_TEXT_RIGHT as NkFlags) {
                state.selected_icon = i;
            }
        }
        ctx.combo_end();
    }

    // ------------------------------------------------
    //                  CHECKBOX
    // ------------------------------------------------
    ui_header(ctx, media, "Checkbox");
    ui_widget(ctx, media, 30f32);
    ctx.checkbox_text("Flag 1", &mut state.check0);
    ui_widget(ctx, media, 30f32);
    ctx.checkbox_text("Flag 2", &mut state.check1);

    // ------------------------------------------------
    //                  PROGRESSBAR
    // ------------------------------------------------
    ui_header(ctx, media, "Progressbar");
    ui_widget(ctx, media, 35f32);
    ctx.progress(&mut state.prog, 100, true);

    // ------------------------------------------------
    //                  PIEMENU
    // ------------------------------------------------
    if ctx.input().is_mouse_click_down_in_rect(NkButton::NK_BUTTON_RIGHT, ctx.window_get_bounds(), true) {
        state.piemenu_pos = ctx.input().mouse().pos();
        state.piemenu_active = true;
    }

    if state.piemenu_active {
        let ret = ui_piemenu(ctx, state.piemenu_pos, 140f32, &media.menu);
        if ret == -2 {
            state.piemenu_active = false;
        }
        if ret != -1 {
            println!("piemenu selected: {}\n", ret);
            state.piemenu_active = false;
        }
    }
    ctx.style_set_font(&media.font_14.handle());
    ctx.end();
}

// ===============================================================
//
//                          CUSTOM WIDGET
//
// ===============================================================
#[allow(unused_assignments)]
fn ui_piemenu(ctx: &mut NkContext, pos: NkVec2, radius: f32, icons: &[NkImage]) -> i32 {
    let mut ret = -1i32;
    let mut total_space = NkRect::default();
    let mut bounds = NkRect::default();
    let mut active_item = 0;

    // pie menu popup
    let border = ctx.style().window().border_color();
    let background = ctx.style().window().fixed_background();
    ctx.style().window().set_fixed_background(NkStyleItem::hide());
    ctx.style().window().set_border_color(color_rgba(0, 0, 0, 0));

    total_space = ctx.window_get_content_region();
    ctx.style().window().set_spacing(NkVec2 { x: 0f32, y: 0f32 });
    ctx.style().window().set_padding(NkVec2 { x: 0f32, y: 0f32 });

    if ctx.popup_begin(NkPopupType::NK_POPUP_STATIC,
                       nk_string!("piemenu"),
                       NkPanelFlags::NK_WINDOW_NO_SCROLLBAR as NkFlags,
                       NkRect {
                           x: pos.x - total_space.x - radius,
                           y: pos.y - radius - total_space.y,
                           w: 2f32 * radius,
                           h: 2f32 * radius,
                       }) {
        let mut out = ctx.window_get_canvas().unwrap();
        let inp = ctx.input();

        total_space = ctx.window_get_content_region();
        ctx.style().window().set_spacing(NkVec2 { x: 4f32, y: 4f32 });
        ctx.style().window().set_padding(NkVec2 { x: 8f32, y: 8f32 });
        ctx.layout_row_dynamic(total_space.h, 1);
        ctx.widget(&mut bounds);

        // outer circle
        out.fill_circle(bounds, nuklear_rust::color_rgb(50, 50, 50));
        {
            // circle buttons
            let step = (2f32 * ::std::f32::consts::PI) / (::std::cmp::max(1, icons.len()) as f32);
            let mut a_min = 0f32;
            let mut a_max = step;

            let center = NkVec2 {
                x: bounds.x + bounds.w / 2.0f32,
                y: bounds.y + bounds.h / 2.0f32,
            };
            let drag = NkVec2 {
                x: inp.mouse().pos().x - center.x,
                y: inp.mouse().pos().y - center.y,
            };
            let mut angle = drag.y.atan2(drag.x);
            if angle < -0.0f32 {
                angle += 2.0f32 * 3.141592654f32;
            }
            active_item = (angle / step) as usize;

            for i in 0..icons.len() {
                let mut content = NkRect::default();
                out.fill_arc(center.x,
                             center.y,
                             (bounds.w / 2.0f32),
                             a_min,
                             a_max,
                             if active_item == i {
                                 nuklear_rust::color_rgb(45, 100, 255)
                             } else {
                                 nuklear_rust::color_rgb(60, 60, 60)
                             });

                // separator line
                let mut rx = bounds.w / 2.0f32;
                let mut ry = 0f32;
                let dx = rx * a_min.cos() - ry * a_min.sin();
                let dy = rx * a_min.sin() + ry * a_min.cos();
                out.stroke_line(center.x,
                                center.y,
                                center.x + dx,
                                center.y + dy,
                                1.0f32,
                                nuklear_rust::color_rgb(50, 50, 50));

                // button content
                let a = a_min + (a_max - a_min) / 2.0f32;
                rx = bounds.w / 2.5f32;
                ry = 0f32;
                content.w = 30f32;
                content.h = 30f32;
                content.x = center.x + ((rx * a.cos() - ry * a.sin()) - content.w / 2.0f32);
                content.y = center.y + (rx * a.sin() + ry * a.cos() - content.h / 2.0f32);
                out.draw_image(content, &icons[i], nuklear_rust::color_rgb(255, 255, 255));
                a_min = a_max;
                a_max += step;
            }
        }
        {
            // inner circle
            let mut inner = NkRect::default();
            inner.x = bounds.x + bounds.w / 2f32 - bounds.w / 4f32;
            inner.y = bounds.y + bounds.h / 2f32 - bounds.h / 4f32;
            inner.w = bounds.w / 2f32;
            inner.h = bounds.h / 2f32;
            out.fill_circle(inner, nuklear_rust::color_rgb(45, 45, 45));

            // active icon content
            bounds.w = inner.w / 2.0f32;
            bounds.h = inner.h / 2.0f32;
            bounds.x = inner.x + inner.w / 2f32 - bounds.w / 2f32;
            bounds.y = inner.y + inner.h / 2f32 - bounds.h / 2f32;
            out.draw_image(bounds,
                           &icons[active_item],
                           nuklear_rust::color_rgb(255, 255, 255));
        }
        ctx.layout_space_end();
        if !ctx.input().is_mouse_down(NkButton::NK_BUTTON_RIGHT) {
            ctx.popup_close();
            ret = active_item as i32;
        }
    } else {
        ret = -2;
    }
    ctx.style().window().set_spacing(NkVec2 { x: 4f32, y: 4f32 });
    ctx.style().window().set_padding(NkVec2 { x: 8f32, y: 8f32 });
    ctx.popup_end();

    ctx.style().window().set_fixed_background(background);
    ctx.style().window().set_border_color(border);
    ret
}

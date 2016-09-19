#[macro_use]
extern crate nuklear_rust;

#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

use nuklear_rust::*;

use image::png::PNGDecoder;
use image::{ColorType, ImageDecoder, ImageFormat};

use glutin::GlRequest;
use gfx::{Factory, Resources, Encoder};
use gfx::tex::{Kind, AaMode};
use gfx::pso::{PipelineData, PipelineState};
use gfx_core::handle::{ShaderResourceView};
use gfx::traits::FactoryExt;
use gfx::Device as Gd;

use std::fs::*;
use std::io::{BufRead, BufReader};
use std::error::Error;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

const MAX_VERTEX_MEMORY: usize = 8 * 1024;
const MAX_ELEMENT_MEMORY: usize = 64 * 1024;

struct BasicState {
	image_active: bool,
    check0: bool,// = 1;
    check1: bool,// = 0;
    prog: usize,// = 80;
    selected_item: usize,// = 0;
    selected_image: usize,// = 3;
    selected_icon: usize,// = 0;
    items: [&'static str; 3],// = {"Item 0","item 1","item 2"};
    piemenu_active: bool,// = 0;
    piemenu_pos: NkVec2,
}

struct ButtonState {
	option: i32,
    toggle0: bool,
    toggle1: bool,
    toggle2: bool,
}

struct GridState {
	text: [[u8;64];3],
    text_len: [i32; 3],
    items: [&'static str; 3],
    selected_item: usize,
    check: bool,
}

struct ResourceHandle<T, R: gfx::Resources> {
	res: ShaderResourceView<R, [f32; 4]>,
	hnd: T,
}

struct Device<'a, R: Resources> {
    cmds: NkBuffer,
    null: NkDrawNullTexture,
    pso: gfx::PipelineState<R, pipe::Meta>,
    			
	vbuf: &'a mut [Vertex],
	ebuf: &'a mut [u16],
	col: gfx_core::handle::RenderTargetView<R, (gfx_core::format::R8_G8_B8_A8, gfx_core::format::Unorm)>,
	dep: gfx_core::handle::DepthStencilView<R, (gfx_core::format::D24_S8, gfx_core::format::Unorm)>,
	font_tex: ResourceHandle<NkHandle, R>,
}

struct Media<R: gfx::Resources> {
    font_14: NkFont,
    font_18: NkFont,
    font_20: NkFont,
    font_22: NkFont,

    unchecked: ResourceHandle<NkImage, R>,
    checked: ResourceHandle<NkImage, R>,
    rocket: ResourceHandle<NkImage, R>,
    cloud: ResourceHandle<NkImage, R>,
    pen: ResourceHandle<NkImage, R>,
    play: ResourceHandle<NkImage, R>,
    pause: ResourceHandle<NkImage, R>,
    stop: ResourceHandle<NkImage, R>,
    prev: ResourceHandle<NkImage, R>,
    next: ResourceHandle<NkImage, R>,
    tools: ResourceHandle<NkImage, R>,
    dir: ResourceHandle<NkImage, R>,
    copy: ResourceHandle<NkImage, R>,
    convert: ResourceHandle<NkImage, R>,
    del: ResourceHandle<NkImage, R>,
    edit: ResourceHandle<NkImage, R>,
    images: [ResourceHandle<NkImage, R>; 9],
    menu: [ResourceHandle<NkImage, R>; 6],
}

impl <R: gfx::Resources> Media<R> {
	fn find(&mut self, id: i32) -> Option<ShaderResourceView<R, [f32; 4]>> {
		if id < 1 {
			return None;
		} else if self.unchecked.hnd.id() == id as i32 {
			return Some(self.unchecked.res.clone());
		} else if self.checked.hnd.id() == id as i32 {
			return Some(self.checked.res.clone());
		} else if self.rocket.hnd.id() == id as i32 {
			return Some(self.rocket.res.clone());
		} else if self.cloud.hnd.id() == id as i32 {
			return Some(self.cloud.res.clone());
		} else if self.pen.hnd.id() == id as i32 {
			return Some(self.pen.res.clone());
		} else if self.play.hnd.id() == id as i32 {
			return Some(self.play.res.clone());
		} else if self.pause.hnd.id() == id as i32 {
			return Some(self.pause.res.clone());
		} else if self.stop.hnd.id() == id as i32 {
			return Some(self.stop.res.clone());
		} else if self.prev.hnd.id() == id as i32 {
			return Some(self.prev.res.clone());
		} else if self.next.hnd.id() == id as i32 {
			return Some(self.next.res.clone());
		} else if self.tools.hnd.id() == id as i32 {
			return Some(self.tools.res.clone());
		} else if self.dir.hnd.id() == id as i32 {
			return Some(self.dir.res.clone());
		} else if self.copy.hnd.id() == id as i32 {
			return Some(self.copy.res.clone());
		} else if self.convert.hnd.id() == id as i32 {
			return Some(self.convert.res.clone());
		} else if self.del.hnd.id() == id as i32 {
			return Some(self.edit.res.clone());
		}
		
		for mut i in &mut self.images {
			if i.hnd.id() == id as i32 {
				return Some(i.res.clone());
			}
		}
		
		for mut i in &mut self.menu {
			if i.hnd.id() == id as i32 {
				return Some(i.res.clone());
			}
		}
		
		None
	}
}

gfx_defines!{
    vertex Vertex {
	    pos: [f32; 2] = "Position",
	    tex: [f32; 2] = "TexCoord",
	    col: [gfx::format::U8Norm; 4] = "Color",
	}

    pipeline pipe {
	    vbuf: gfx::VertexBuffer<Vertex> = (),
	    proj: gfx::Global<[[f32; 4]; 4]> = "ProjMtx",
	    tex: gfx::TextureSampler<[f32; 4]> = "Texture",
	    output: gfx::BlendTarget<super::ColorFormat> = ("Out_Color", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
	    //scissors: gfx::Scissor = (),
	}
}

impl Default for Vertex {
	fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

const VS: &'static [u8] =
        b"#version 150
        uniform mat4 ProjMtx;
        in vec2 Position;
        in vec2 TexCoord;
        in vec4 Color;
        out vec2 Frag_UV;
        out vec4 Frag_Color;
        void main() {
           Frag_UV = TexCoord;
           Frag_Color = Color;
           gl_Position = ProjMtx * vec4(Position.xy, 0, 1);
        }";
const FS: &'static [u8] =
        b"#version 150\n
        precision mediump float;
	    uniform sampler2D Texture;
        in vec2 Frag_UV;
        in vec4 Frag_Color;
        out vec4 Out_Color;
        void main(){
           Out_Color = Frag_Color * texture(Texture, Frag_UV.st);
		}";

fn main() {
	let (w,h) = glutin::get_primary_monitor().get_dimensions();
	
	let gl_version = GlRequest::GlThenGles {
            opengles_version: (2, 0),
            opengl_version: (3, 3),
	};
		
	let builder = glutin::WindowBuilder::new()
				.with_depth_buffer(24)
				.with_dimensions(w / 4 * 3, h / 4 * 3)
				.with_gl(gl_version);
				
	let (window, mut device, mut factory, main_color, main_depth) = gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
	let mut encoder: Encoder<_, _> = factory.create_command_buffer().into();
	
	let mut cfg = NkFontConfig::new(0.0);
	cfg.set_oversample_h(3);
	cfg.set_oversample_v(2);
	
	let mut allo = NkAllocator::new_heap();
	
	let mut vbuf = [Vertex::default(); MAX_VERTEX_MEMORY];
	let mut ebuf = [0u16; MAX_ELEMENT_MEMORY];
	
	let mut atlas = NkFontAtlas::new(&mut allo);
	
	let font = include_bytes!("../res/fonts/Roboto-Regular.ttf");
	
	let mut font_14 = atlas.add_font_with_bytes(font, 14.0).unwrap();	
	let font_18 = atlas.add_font_with_bytes(font, 18.0).unwrap();
	let font_20 = atlas.add_font_with_bytes(font, 20.0).unwrap();
	let font_22 = atlas.add_font_with_bytes(font, 22.0).unwrap();
	
	let (b, fw, fh) = atlas.bake(NkFontAtlasFormat::NK_FONT_ATLAS_RGBA32);
	let font_tex = upload_atlas(&mut factory, &b, fw as usize, fh as usize);
	
	let mut dev = Device {
		cmds: NkBuffer::with_size(&mut allo, 16192),
		null: NkDrawNullTexture::default(),
		pso: factory.create_pipeline_simple(VS,FS, pipe::new()).unwrap(),
		vbuf: &mut vbuf,
		ebuf: &mut ebuf,
		col: main_color,
		dep: main_depth,
		font_tex: font_tex,
	};
	
	atlas.end(dev.font_tex.hnd, Some(&mut dev.null));
	
	let mut ctx = NkContext::new(&mut allo, &font_14.handle());
	
	let mut id = 0;
	
	let mut media = Media {
		font_14: font_14,
	    font_18: font_18,
	    font_20: font_20,
	    font_22: font_22,
	
	    unchecked: icon_load(&mut factory, &mut id, "res/icon/unchecked.png"),
	    checked: icon_load(&mut factory, &mut id, "res/icon/checked.png"),
	    rocket: icon_load(&mut factory, &mut id, "res/icon/rocket.png"),
	    cloud: icon_load(&mut factory, &mut id, "res/icon/cloud.png"),
	    pen: icon_load(&mut factory, &mut id, "res/icon/pen.png"),
	    play: icon_load(&mut factory, &mut id, "res/icon/play.png"),
	    pause: icon_load(&mut factory, &mut id, "res/icon/pause.png"),
	    stop: icon_load(&mut factory, &mut id, "res/icon/stop.png"),
	    prev: icon_load(&mut factory, &mut id, "res/icon/prev.png"),
	    next: icon_load(&mut factory, &mut id, "res/icon/next.png"),
	    tools: icon_load(&mut factory, &mut id, "res/icon/tools.png"),
	    dir: icon_load(&mut factory, &mut id, "res/icon/directory.png"),
	    copy: icon_load(&mut factory, &mut id, "res/icon/copy.png"),
	    convert: icon_load(&mut factory, &mut id, "res/icon/export.png"),
	    del: icon_load(&mut factory, &mut id, "res/icon/delete.png"),
	    edit: icon_load(&mut factory, &mut id, "res/icon/edit.png"),
	    images: [
			icon_load(&mut factory, &mut id, "res/images/image1.png"),
		    icon_load(&mut factory, &mut id, "res/images/image2.png"),
		    icon_load(&mut factory, &mut id, "res/images/image3.png"),
		    icon_load(&mut factory, &mut id, "res/images/image4.png"),
		    icon_load(&mut factory, &mut id, "res/images/image5.png"),
		    icon_load(&mut factory, &mut id, "res/images/image6.png"),
		    icon_load(&mut factory, &mut id, "res/images/image7.png"),
		    icon_load(&mut factory, &mut id, "res/images/image8.png"),
		    icon_load(&mut factory, &mut id, "res/images/image9.png"),
		],
	    menu: [
		    icon_load(&mut factory, &mut id, "res/icon/home.png"),
		    icon_load(&mut factory, &mut id, "res/icon/phone.png"),
		    icon_load(&mut factory, &mut id, "res/icon/plane.png"),
		    icon_load(&mut factory, &mut id, "res/icon/wifi.png"),
		    icon_load(&mut factory, &mut id, "res/icon/settings.png"),
		    icon_load(&mut factory, &mut id, "res/icon/volume.png")
		],
	};
	
	let mut basic_state = BasicState {
		image_active: false,
	    check0: true,
	    check1: false,
	    prog: 80,
	    selected_item: 0,
	    selected_image: 3,
	    selected_icon: 0,
	    items: ["Item 0","item 1","item 2"],
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
		text: [[0;64];3],
	    text_len: [0; 3],
	    items: ["Item 0","item 1","item 2"],
	    selected_item: 0,
	    check: true,
	};
	
	let sampler = factory.create_sampler_linear();
	
	let mut mx = 0;
    let mut my = 0;	
    
	let vbuf = factory.create_buffer_dynamic::<Vertex>(MAX_VERTEX_MEMORY, ::gfx::BufferRole::Vertex, ::gfx::Bind::empty()).unwrap();
	let ebuf = factory.create_buffer_dynamic::<u16>(MAX_ELEMENT_MEMORY, ::gfx::BufferRole::Index, ::gfx::Bind::empty()).unwrap();
	
	let mut tmp = [0u16; MAX_ELEMENT_MEMORY];
		
	'main: loop {
        
        //println!("{:?}", event);
        
        let (fw,fh) = window.get_inner_size_pixels().unwrap();
        let scale = NkVec2 {x: fw as f32 / w as f32, y: fh as f32 / h as f32};
        
        ctx.input_begin();
        for event in window.poll_events() {
        	match event {
	            glutin::Event::Closed => break 'main,
	            glutin::Event::KeyboardInput(s, _, k) => {
	            	if let Some(k) = k {
	            		let key = match k {
	            			glutin::VirtualKeyCode::Up => NkKey::NK_KEY_UP,
	            			glutin::VirtualKeyCode::Down => NkKey::NK_KEY_DOWN,
	            			glutin::VirtualKeyCode::Left => NkKey::NK_KEY_LEFT,
	            			glutin::VirtualKeyCode::Right => NkKey::NK_KEY_RIGHT,
	            			_ => NkKey::NK_KEY_NONE,
	            		};
	            		
	            		ctx.input_key(key, s == glutin::ElementState::Pressed);
	            	}
	            	
	            }
	            glutin::Event::MouseMoved(x,y) => {
	            	mx = x;
	            	my = y;
		            ctx.input_motion(x, y);
	            }
	            glutin::Event::MouseInput(s, b) => {
	            	let button = match b {
	            		glutin::MouseButton::Left => NkButton::NK_BUTTON_LEFT,
	            		glutin::MouseButton::Middle => NkButton::NK_BUTTON_MIDDLE,
	            		glutin::MouseButton::Right => NkButton::NK_BUTTON_RIGHT,
	            		_ => NkButton::NK_BUTTON_MAX,
	            	};
	            	
	            	ctx.input_button(button, mx, my, s == glutin::ElementState::Pressed)
	            }
	            _ => ()
	        }
        }
        ctx.input_end();
        
        basic_demo(&mut ctx, &mut media, &mut basic_state);
        button_demo(&mut ctx, &mut media, &mut button_state);
        grid_demo(&mut ctx, &mut media, &mut grid_state);
        
        encoder.clear(&dev.col, [0.1f32, 0.2f32, 0.3f32, 1.0f32]);
        device_draw(&mut dev, &mut ctx, &mut media, &mut encoder, &mut factory, &sampler, &vbuf, &ebuf, &mut tmp, w, h, scale, NkAntiAliasing::NK_ANTI_ALIASING_ON);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
		device.cleanup();
		
		::std::thread::sleep(::std::time::Duration::from_millis(10));

        ctx.clear();
	}
}

fn upload_atlas<F, R: gfx::Resources>(factory: &mut F, image: &Vec<u8>, width: usize, height: usize) -> ResourceHandle<NkHandle, R> where F: gfx::Factory<R> {
	
	let (_, view) = factory.create_texture_const_u8::<ColorFormat>(Kind::D2(width as u16, height as u16, AaMode::Single), &[image.as_slice()]).unwrap();
    /*glGenTextures(1, &dev->font_tex);
    glBindTexture(GL_TEXTURE_2D, dev->font_tex);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, (GLsizei)width, (GLsizei)height, 0,
                GL_RGBA, GL_UNSIGNED_BYTE, image);*/
    ResourceHandle {
    	res: view,
    	hnd: NkHandle::from_id(0),
    }
}

fn icon_load<F, R: gfx::Resources>(factory: &mut F, id: &mut usize, filename: &str) -> ResourceHandle<NkImage, R> where F: gfx::Factory<R> {
	
	let img = image::load(BufReader::new(File::open(filename).unwrap()), image::PNG).unwrap().to_rgba();
	
	let (x,y) = img.dimensions();
	let data = img.into_vec();
    
    let (_, view) = factory.create_texture_const_u8::<ColorFormat>(Kind::D2(x as u16, y as u16, AaMode::Single), &[data.as_slice()]).unwrap();
    
    *id += 1;
    
    ResourceHandle {
    	res: view,
    	hnd: NkImage::with_id(*id as i32),
    }
}

fn ui_header<R: gfx::Resources>(ctx: &mut NkContext, media: &mut Media<R>, title: &str) {
	ctx.style_set_font(&media.font_18.handle());
	ctx.layout_row_dynamic(20f32, 1);
    ctx.label(NkString::from(title), NkTextAlignment::NK_TEXT_LEFT as NkFlags);
}

fn ui_widget<R: gfx::Resources>(ctx: &mut NkContext, media: &mut Media<R>, height: f32) {
    let ratio = [0.15f32, 0.85f32];
    ctx.style_set_font(&media.font_22.handle());
	ctx.layout_row(NkLayoutFormat::NK_DYNAMIC, height, &ratio);
	ctx.spacing(1);
}

fn ui_widget_centered<R: gfx::Resources>(ctx: &mut NkContext, media: &mut Media<R>, height: f32) {
    let ratio = [0.15f32, 0.50f32, 0.35f32];
    ctx.style_set_font(&media.font_22.handle());
	ctx.layout_row(NkLayoutFormat::NK_DYNAMIC, height, &ratio);
	ctx.spacing(1);
}

fn grid_demo<R: gfx::Resources>(ctx: &mut NkContext, media: &mut Media<R>, state: &mut GridState) {
    let mut layout = NkPanel::default();
    let mut combo = NkPanel::default();
    
    ctx.style_set_font(&media.font_20.handle());
    if ctx.begin(&mut layout, 
	    	nk_string!("Grid Demo"), 
	    	NkRect{x:600f32, y:350f32, w:275f32, h:250f32}, 
	    	NkPanelFlags::NK_WINDOW_BORDER as NkFlags|NkPanelFlags::NK_WINDOW_MOVABLE as NkFlags|NkPanelFlags::NK_WINDOW_TITLE as NkFlags|NkPanelFlags::NK_WINDOW_NO_SCROLLBAR as NkFlags) {
        ctx.style_set_font(&media.font_18.handle());
        ctx.layout_row_dynamic(30f32, 2);
        ctx.label(nk_string!("Floating point:"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
        ctx.edit_string(NkEditType::NK_EDIT_FIELD as NkFlags, &mut state.text[0], &mut state.text_len[0], NK_FILTER_FLOAT);
        ctx.label(nk_string!("Hexadecimal:"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
        ctx.edit_string(NkEditType::NK_EDIT_FIELD as NkFlags, &mut state.text[1], &mut state.text_len[1], NK_FILTER_HEX);
        ctx.label(nk_string!("Binary:"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
        ctx.edit_string(NkEditType::NK_EDIT_FIELD as NkFlags, &mut state.text[2], &mut state.text_len[2], NK_FILTER_BINARY);
        ctx.label(nk_string!("Checkbox:"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
        ctx.checkbox_label(nk_string!("Check me"), &mut state.check);
        ctx.label(nk_string!("Combobox:"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
        
        let widget_width = ctx.widget_width();
        if ctx.combo_begin_label(&mut combo, NkString::from(state.items[state.selected_item]), NkVec2{x:widget_width, y:200f32}) {
            ctx.layout_row_dynamic(25f32, 1);
            for i in 0..state.text.len() {
                if ctx.combo_item_label(NkString::from(state.items[i]), NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
                    state.selected_item = i;
                }    
            }        
            ctx.combo_end();
        }
    }
    ctx.end();
    ctx.style_set_font(&media.font_14.handle());
}

fn button_demo<R: gfx::Resources>(ctx: &mut NkContext, media: &mut Media<R>, state: &mut ButtonState) {
    let mut layout = NkPanel::default();
    let mut menu = NkPanel::default();
    
    ctx.style_set_font(&media.font_20.handle());

    ctx.begin(&mut layout, nk_string!("Button Demo"), NkRect {x:50f32, y:50f32, w:255f32, h:610f32}, NkPanelFlags::NK_WINDOW_BORDER as NkFlags|NkPanelFlags::NK_WINDOW_MOVABLE as NkFlags|NkPanelFlags::NK_WINDOW_TITLE as NkFlags);

    /*------------------------------------------------
     *                  MENU
     *------------------------------------------------*/
    ctx.menubar_begin();
    {
        /* toolbar */
        ctx.layout_row_static(40f32, 40, 4);
        if ctx.menu_begin_image(&mut menu, nk_string!("Music"), media.play.hnd.clone(), NkVec2{x:110f32, y:120f32}) {
            /* settings */
            ctx.layout_row_dynamic(25f32, 1);
            ctx.menu_item_image_label(media.play.hnd.clone(), nk_string!("Play"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
            ctx.menu_item_image_label(media.stop.hnd.clone(), nk_string!("Stop"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
            ctx.menu_item_image_label(media.pause.hnd.clone(), nk_string!("Pause"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
            ctx.menu_item_image_label(media.next.hnd.clone(), nk_string!("Next"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
            ctx.menu_item_image_label(media.prev.hnd.clone(), nk_string!("Prev"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags);
            ctx.menu_end();
        }
        ctx.button_image(media.tools.hnd.clone());
        ctx.button_image(media.cloud.hnd.clone());
        ctx.button_image(media.pen.hnd.clone());
    }
    ctx.menubar_end();

    /*------------------------------------------------
     *                  BUTTON
     *------------------------------------------------*/
    ui_header(ctx, media, "Push buttons");
    ui_widget(ctx, media, 35f32);
    if ctx.button_label(nk_string!("Push me")) {
        println!("pushed!");
    }    
    ui_widget(ctx, media, 35f32);
    if ctx.button_image_label(media.rocket.hnd.clone(), nk_string!("Styled"), NkTextAlignment::NK_TEXT_CENTERED as NkFlags) {
        println!("rocket!");
    }    

    /*------------------------------------------------
     *                  REPEATER
     *------------------------------------------------*/
    ui_header(ctx, media, "Repeater");
    ui_widget(ctx, media, 35f32);
    if ctx.button_label(nk_string!("Press me")) {
        println!("pressed!");
    }    

    /*------------------------------------------------
     *                  TOGGLE
     *------------------------------------------------*/
    ui_header(ctx, media, "Toggle buttons");
    ui_widget(ctx, media, 35f32);
    if ctx.button_image_label(if state.toggle0 { media.checked.hnd.clone() } else { media.unchecked.hnd.clone() }, nk_string!("Toggle"), NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.toggle0 = !state.toggle0;
    }    

    ui_widget(ctx, media, 35f32);
    if ctx.button_image_label(if state.toggle1 { media.checked.hnd.clone() } else { media.unchecked.hnd.clone() }, nk_string!("Toggle"), NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.toggle1 = !state.toggle1;
    }    

    ui_widget(ctx, media, 35f32);
    if ctx.button_image_label(if state.toggle2 { media.checked.hnd.clone() } else { media.unchecked.hnd.clone() }, nk_string!("Toggle"), NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.toggle2 = !state.toggle2;
    }    

    /*------------------------------------------------
     *                  RADIO
     *------------------------------------------------*/
    ui_header(ctx, media, "Radio buttons");
    ui_widget(ctx, media, 35f32);
    if ctx.button_symbol_label( if state.option == 0 { NkSymbolType::NK_SYMBOL_CIRCLE_OUTLINE } else { NkSymbolType::NK_SYMBOL_CIRCLE_SOLID }, nk_string!("Select 1"), NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.option = 0;
    }    
    ui_widget(ctx, media, 35f32);
    if ctx.button_symbol_label( if state.option == 1 { NkSymbolType::NK_SYMBOL_CIRCLE_OUTLINE } else { NkSymbolType::NK_SYMBOL_CIRCLE_SOLID }, nk_string!("Select 2"), NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.option = 1;
    }    
    ui_widget(ctx, media, 35f32);
    if ctx.button_symbol_label( if state.option == 2 { NkSymbolType::NK_SYMBOL_CIRCLE_OUTLINE } else { NkSymbolType::NK_SYMBOL_CIRCLE_SOLID }, nk_string!("Select 3"), NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
        state.option = 2;
    }    

    /*------------------------------------------------
     *                  CONTEXTUAL
     *------------------------------------------------*/
    ctx.style_set_font(&media.font_18.handle());
    let bounds = ctx.window_get_bounds();
    if ctx.contextual_begin(&mut menu, NkPanelFlags::NK_WINDOW_NO_SCROLLBAR as NkFlags, NkVec2{ x:150f32, y:300f32 }, bounds) {
        ctx.layout_row_dynamic(30f32, 1);
        if ctx.contextual_item_image_label(media.copy.hnd.clone(), nk_string!("Clone"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags) {
            println!("pressed clone!");
        }    
        if ctx.contextual_item_image_label(media.del.hnd.clone(), nk_string!("Delete"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags) {
            println!("pressed delete!");
        }    
        if ctx.contextual_item_image_label(media.convert.hnd.clone(), nk_string!("Convert"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags) {
            println!("pressed convert!");
        }    
        if ctx.contextual_item_image_label(media.edit.hnd.clone(), nk_string!("Edit"), NkTextAlignment::NK_TEXT_RIGHT as NkFlags) {
            println!("pressed edit!");
        }    
        ctx.contextual_end();
    }
    ctx.style_set_font(&media.font_14.handle());
    ctx.end();
}

fn basic_demo<R: gfx::Resources>(ctx: &mut NkContext, media: &mut Media<R>, state: &mut BasicState) {
	let mut layout = NkPanel::default();
    let mut combo = NkPanel::default();
    
    ctx.style_set_font(&media.font_20.handle());
    ctx.begin(&mut layout, nk_string!("Basic Demo"), NkRect {x:320f32, y:50f32, w:275f32, h:610f32}, NkPanelFlags::NK_WINDOW_BORDER as NkFlags|NkPanelFlags::NK_WINDOW_MOVABLE as NkFlags|NkPanelFlags::NK_WINDOW_TITLE as NkFlags);
    
    /*------------------------------------------------
     *                  POPUP BUTTON
     *------------------------------------------------*/
    
    
    ui_header(ctx, media, "Popup & Scrollbar & Images");
    ui_widget(ctx, media, 35f32);
    if ctx.button_image_label(media.dir.hnd.clone(), nk_string!("Images"), NkTextAlignment::NK_TEXT_CENTERED as NkFlags) {
        state.image_active = !state.image_active;
    }    

    /*------------------------------------------------
     *                  SELECTED IMAGE
     *------------------------------------------------*/
    ui_header(ctx, media, "Selected Image");
    ui_widget_centered(ctx, media, 100f32);
    ctx.image(media.images[state.selected_image].hnd.clone());
    
    /*------------------------------------------------
     *                  IMAGE POPUP
     *------------------------------------------------*/
    if state.image_active {
        let mut popup = NkPanel::default();
        if ctx.popup_begin(&mut popup, NkPopupType::NK_POPUP_STATIC, nk_string!("Image Popup"), 0, NkRect {x:265f32, y:0f32, w:320f32, h:220f32}) {
            ctx.layout_row_static(82f32, 82, 3);
            for i in 0..9 {
                if ctx.button_image(media.images[i].hnd.clone()) {
                    state.selected_image = i;
                    state.image_active = false;
                    ctx.popup_close();
                }
            }
            ctx.popup_end();
        }
    }
    /*------------------------------------------------
     *                  COMBOBOX
     *------------------------------------------------*/
    ui_header(ctx, media, "Combo box");
    ui_widget(ctx, media, 40f32);
    let widget_width = ctx.widget_width();
    if ctx.combo_begin_label(&mut combo, NkString::from(state.items[state.selected_item]), NkVec2{ x:widget_width, y:200f32}) {
        ctx.layout_row_dynamic(35f32, 1);
        for i in 0..3 {
            if ctx.combo_item_label(NkString::from(state.items[i]), NkTextAlignment::NK_TEXT_LEFT as NkFlags) {
                state.selected_item = i;
            }    
        }        
        ctx.combo_end();
    }

    ui_widget(ctx, media, 40f32);
    let widget_width = ctx.widget_width();
    if ctx.combo_begin_image_label(&mut combo, NkString::from(state.items[state.selected_icon]), media.images[state.selected_icon].hnd.clone(), NkVec2{ x:widget_width, y:200f32}) {
        ctx.layout_row_dynamic(35f32, 1);
        for i in 0..3 {
            if ctx.combo_item_image_label(media.images[i].hnd.clone(), NkString::from(state.items[i]), NkTextAlignment::NK_TEXT_RIGHT as NkFlags) {
                state.selected_icon = i;
            }    
        }        
        ctx.combo_end();
    }

    /*------------------------------------------------
     *                  CHECKBOX
     *------------------------------------------------*/
    ui_header(ctx, media, "Checkbox");
    ui_widget(ctx, media, 30f32);
    ctx.checkbox_label(nk_string!("Flag 1"), &mut state.check0);
    ui_widget(ctx, media, 30f32);
    ctx.checkbox_label(nk_string!("Flag 2"), &mut state.check1);

    /*------------------------------------------------
     *                  PROGRESSBAR
     *------------------------------------------------*/
    ui_header(ctx, media, "Progressbar");
    ui_widget(ctx, media, 35f32);
    ctx.progress(&mut state.prog, 100, true);

    /*------------------------------------------------
     *                  PIEMENU
     *------------------------------------------------*/
    if ctx.input().is_mouse_click_down_in_rect(NkButton::NK_BUTTON_RIGHT, layout.bounds(), true) {
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

/* ===============================================================
 *
 *                          CUSTOM WIDGET
 *
 * ===============================================================*/
fn ui_piemenu<R: gfx::Resources>(ctx: &mut NkContext, pos: NkVec2, radius: f32, icons: &[ResourceHandle<NkImage, R>]) -> i32 {
    let mut ret = -1i32;
    let mut total_space = NkRect::default();
    let mut popup = NkPanel::default();
    let mut bounds = NkRect::default();
    let mut active_item = 0; 

    /* pie menu popup */
    let border = ctx.style().window().border_color();
    let background = ctx.style().window().fixed_background();
    ctx.style().window().set_fixed_background(NkStyleItem::hide());
    ctx.style().window().set_border_color(color_rgba(0,0,0,0));

    total_space  = ctx.window_get_content_region();
    ctx.style().window().set_spacing(NkVec2{x:0f32, y:0f32});
    ctx.style().window().set_padding(NkVec2{x:0f32, y:0f32});

    if ctx.popup_begin(
	    	&mut popup, 
	    	NkPopupType::NK_POPUP_STATIC, 
	    	nk_string!("piemenu"), 
	    	NkPanelFlags::NK_WINDOW_NO_SCROLLBAR as NkFlags,
	        NkRect{x: pos.x - total_space.x - radius, y: pos.y - radius - total_space.y, w: 2f32*radius, h: 2f32*radius}) {
        let mut out = ctx.window_get_canvas().unwrap();
        let inp = ctx.input();

        total_space = ctx.window_get_content_region();
        ctx.style().window().set_spacing(NkVec2{x:4f32, y:4f32});
        ctx.style().window().set_padding(NkVec2{x:8f32, y:8f32});
        ctx.layout_row_dynamic(total_space.h, 1);
        ctx.widget(&mut bounds);

        /* outer circle */
        out.fill_circle(bounds, nuklear_rust::color_rgb(50,50,50));
        {
            /* circle buttons */
            let step = (2f32 * ::std::f32::consts::PI) / (::std::cmp::max(1, icons.len()) as f32);
            let mut a_min = 0f32; 
            let mut a_max = step;

            let center = NkVec2{x: bounds.x + bounds.w / 2.0f32, y: bounds.y + bounds.h / 2.0f32};
            let drag = NkVec2{x: inp.mouse().pos().x - center.x, y: inp.mouse().pos().y - center.y};
            let mut angle = drag.y.atan2(drag.x);
            if angle < -0.0f32 {
            	angle += 2.0f32 * 3.141592654f32;
            }	
            active_item = (angle/step) as usize;

            for i in 0..icons.len() {
                let mut content = NkRect::default();
                out.fill_arc(center.x, center.y, (bounds.w/2.0f32), a_min, a_max, if active_item == i { nuklear_rust::color_rgb(45,100,255) } else { nuklear_rust::color_rgb(60,60,60) });

                /* separator line */
                let mut rx = bounds.w/2.0f32; 
                let mut ry = 0f32;
                let dx = rx * a_min.cos() - ry * a_min.sin();
                let dy = rx * a_min.sin() + ry * a_min.cos();
                out.stroke_line(center.x, center.y,
                    center.x + dx, center.y + dy, 1.0f32, nuklear_rust::color_rgb(50,50,50));

                /* button content */
                let a = a_min + (a_max - a_min)/2.0f32;
                rx = bounds.w/2.5f32; 
                ry = 0f32;
                content.w = 30f32; content.h = 30f32;
                content.x = center.x + ((rx * a.cos() - ry * a.sin()) - content.w/2.0f32);
                content.y = center.y + (rx * a.sin() + ry * a.cos() - content.h/2.0f32);
                out.draw_image(content, &icons[i].hnd, nuklear_rust::color_rgb(255,255,255));
                a_min = a_max; a_max += step;
            }
        }
        {
            /* inner circle */
            let mut inner = NkRect::default();
            inner.x = bounds.x + bounds.w/2f32 - bounds.w/4f32;
            inner.y = bounds.y + bounds.h/2f32 - bounds.h/4f32;
            inner.w = bounds.w/2f32; 
            inner.h = bounds.h/2f32;
            out.fill_circle(inner, nuklear_rust::color_rgb(45,45,45));

            /* active icon content */
            bounds.w = inner.w / 2.0f32;
            bounds.h = inner.h / 2.0f32;
            bounds.x = inner.x + inner.w/2f32 - bounds.w/2f32;
            bounds.y = inner.y + inner.h/2f32 - bounds.h/2f32;
            out.draw_image(bounds, &icons[active_item].hnd, nuklear_rust::color_rgb(255,255,255));
        }
        ctx.layout_space_end();
        if !ctx.input().is_mouse_down(NkButton::NK_BUTTON_RIGHT) {
            ctx.popup_close();
            ret = active_item as i32;
        }
    } else {
    	ret = -2;
    }
    ctx.style().window().set_spacing(NkVec2{x:4f32, y:4f32});
    ctx.style().window().set_padding(NkVec2{x:8f32, y:8f32});
    ctx.popup_end();

    ctx.style().window().set_fixed_background(background);
    ctx.style().window().set_border_color(border);
    ret
}

fn device_draw<F, R: gfx_core::Resources,B: gfx_core::draw::CommandBuffer<R>>(dev: &mut Device<R>, ctx: &mut NkContext, media: &mut Media<R>, encoder: &mut Encoder<R,B>, factory: &mut F, sampler: &gfx_core::handle::Sampler<R>, vbuf: &gfx_core::handle::Buffer<R, Vertex>, ebuf: &gfx_core::handle::Buffer<R, u16>, tmp: &mut [u16],width: u32, height: u32, scale: NkVec2, aa: NkAntiAliasing) 
		where R: gfx_core::Resources,
		F: gfx::Factory<R> {
	use gfx::pso::buffer::Structure;
	use gfx::IntoIndexBuffer;
	
	let ortho = [[2.0f32 / width as f32, 0.0f32, 				 0.0f32, 0.0f32],
				[0.0f32,				-2.0f32 / height as f32, 0.0f32, 0.0f32],
				[0.0f32, 				 0.0f32,				-1.0f32, 0.0f32],
				[-1.0f32,				 1.0f32, 				 0.0f32, 1.0f32]];
	
	let vertex_layout = [
                (NkDrawVertexLayoutAttribute::NK_VERTEX_POSITION, NkDrawVertexLayoutFormat::NK_FORMAT_FLOAT, Vertex::query("Position").unwrap().offset),
                (NkDrawVertexLayoutAttribute::NK_VERTEX_TEXCOORD, NkDrawVertexLayoutFormat::NK_FORMAT_FLOAT, Vertex::query("TexCoord").unwrap().offset),
                (NkDrawVertexLayoutAttribute::NK_VERTEX_COLOR, NkDrawVertexLayoutFormat::NK_FORMAT_R8G8B8A8, Vertex::query("Color").unwrap().offset),
                (NkDrawVertexLayoutAttribute::NK_VERTEX_ATTRIBUTE_COUNT, NkDrawVertexLayoutFormat::NK_FORMAT_COUNT, 0u32)
            ];
            
    let vertex_layout_elements = NkDrawVertexLayoutElements::new(&vertex_layout);        
	
	let mut config = NkConvertConfig::default();
	config.set_vertex_layout(&vertex_layout_elements);
	config.set_vertex_size(::std::mem::size_of::<Vertex>());
    //config.vertex_alignment = NK_ALIGNOF(struct nk_glfw_vertex);
    config.set_null(dev.null.clone());
    config.set_circle_segment_count(22);
    config.set_curve_segment_count(22);
    config.set_arc_segment_count(22);
    config.set_global_alpha(1.0f32);
    config.set_shape_aa(aa);
    config.set_line_aa(aa);

	{
		let mut rwv = factory.map_buffer_rw(vbuf);
		let mut rvbuf = unsafe { ::std::slice::from_raw_parts_mut(&mut *rwv as *mut [Vertex] as *mut u8, ::std::mem::size_of::<Vertex>() * dev.vbuf.len()) }; 
		let mut vbuf = NkBuffer::with_fixed(&mut rvbuf);
		
		let mut rebuf = unsafe { ::std::slice::from_raw_parts_mut(tmp as *mut [u16] as *mut u8, ::std::mem::size_of::<u16>() * dev.ebuf.len()) }; 
		let mut ebuf = NkBuffer::with_fixed(&mut rebuf);
		
		ctx.convert(&mut dev.cmds, &mut vbuf, &mut ebuf, &config);	
	}
	
	{
		let mut rwe = factory.map_buffer_rw(ebuf);
		(&mut *rwe).clone_from_slice(tmp);
	}
	
	let mut slice = ::gfx::Slice {
	    start: 0,
	    end: 0,
	    base_vertex: 0,
	    instances: None,
	    buffer: ebuf.clone().into_index_buffer(factory),
	};
	
	for cmd in ctx.draw_command_iterator(&mut dev.cmds) {
		
		if cmd.elem_count() < 1 { 
			continue; 
		}
		
		//println!("{:?}", cmd);
		
		slice.start = slice.end;
		slice.end += cmd.elem_count();
		
		let id = cmd.texture().id().unwrap();
		let ptr: ShaderResourceView<R, [f32; 4]> = if id == 0 {dev.font_tex.res.clone()} else {media.find(id).unwrap()}; 
		
		let data = pipe::Data {
			vbuf: vbuf.clone(),
		    proj: ortho,
		    tex: (ptr, sampler.clone()), //&mut r.res as *mut _ as *mut ::std::os::raw::c_void
		    output: dev.col.clone(),
		    /*scissors: gfx::Rect{
		    	x: (cmd.clip_rect().x * scale.x) as u16,
		    	y: ((height as f32 - cmd.clip_rect().y + cmd.clip_rect().h) * scale.y) as u16,
		    	w: (cmd.clip_rect().w * scale.x) as u16,
		    	h: (cmd.clip_rect().h * scale.y) as u16,
		    },*/
		};
		
		encoder.draw(&slice, &dev.pso, &data);
	}	
}
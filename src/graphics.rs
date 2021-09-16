use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

#[wasm_bindgen]
pub struct WebGlCanvas {
  canvas: web_sys::HtmlCanvasElement,
  context: WebGl2RenderingContext,
  vert_shader: WebGlShader,
  frag_shader: WebGlShader,
  program: WebGlProgram,
  vertices: [f32; 6],
}

// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl WebGlCanvas {

  pub fn new(canvas_id: &str) -> Result<WebGlCanvas, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(&canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es
 
        in vec2 position;
    
        out vec2 Position;
    
        void main()
        {
          gl_Position = vec4(position, 0.0, 1.0);
        }
        "##,
    )?;

    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
        precision highp float;

        uniform float u_time;
        
        in vec2 Position;
    
        out vec4 outColor;
    
        void main()
        {
          float x = Position.x;
          float y = Position.y;
          vec3 modColour = (0.5*sin(u_time + x*y)+0.5)*vec3(1.0, 1.0, 1.0);
          outColor = vec4(modColour, 1.0);
        }
        "##,
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let vertices: [f32; 6] = [0.0,  0.5,
             0.5, -0.5,
            -0.5, -0.5 ];

    let position_attribute_location = context.get_attrib_location(&program, "position");
//    let colour_attribute_location = context.get_attrib_location(&program, "colour");
    let buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    let vao = context
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    context.bind_vertex_array(Some(&vao));

    context.vertex_attrib_pointer_with_i32(position_attribute_location as u32, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(position_attribute_location as u32);

//    context.vertex_attrib_pointer_with_i32(colour_attribute_location as u32, 3, WebGl2RenderingContext::FLOAT, false, 5*4, 0);
//    context.enable_vertex_attrib_array(colour_attribute_location as u32);
  
    context.bind_vertex_array(Some(&vao));

  Ok(WebGlCanvas {
    canvas,
    context,
    vert_shader,
    frag_shader,
    program,
    vertices
  })
}
  
  pub fn render(&self, time: f32) {
    let time_location = self.context.get_uniform_location(
      &self.program,
      "u_time"
    ).expect("WebGL program should have `u_time` uniform.");

    let vert_count = (self.vertices.len() / 2) as i32;

    self.context.clear_color(0.0, 0.0, 0.0, 1.0);
    self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
  
    self.context.uniform1f(Some(&time_location), (time/1000.0) as f32);
  
    self.context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vert_count);
  }
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
  let shader = context
    .create_shader(shader_type)
    .ok_or_else(|| String::from("Unable to create shader object"))?;
  context.shader_source(&shader, source);
  context.compile_shader(&shader);
    
  if context
    .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
    .as_bool()
    .unwrap_or(false)
  {
    Ok(shader)
  } else {
    Err(context
        .get_shader_info_log(&shader)
        .unwrap_or_else(|| String::from("Unknown error creating shader")))
  }
}

fn link_program(
  context: &WebGl2RenderingContext,
  vert_shader: &WebGlShader,
  frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
  let program = context
    .create_program()
    .ok_or_else(|| String::from("Unable to create shader object"))?;
    
  context.attach_shader(&program, vert_shader);
  context.attach_shader(&program, frag_shader);
  context.link_program(&program);
  
  if context
    .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
    .as_bool()
    .unwrap_or(false)
  {
    Ok(program)
  } else {
    Err(context
      .get_program_info_log(&program)
      .unwrap_or_else(|| String::from("Unknown error creating program object")))
  }
}


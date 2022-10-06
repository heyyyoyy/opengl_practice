use std::{ffi::CString, path::Path};
use std::mem;
use std::os::raw::c_void;
use std::sync::mpsc::Receiver;
use std::ptr;

use glfw::{Action, Context, Key};
use gl::types::*;
use image;


const HEIGHT: u32 = 800;
const WIDTH: u32 = 800;


fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        if let glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                window.set_should_close(true)
        }
    }
}


fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "opengl", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");
    window.set_key_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s) as *const _);
    gl::Viewport::load_with(|s| window.get_proc_address(s) as *const _);

    let vao = unsafe {
        const VERTEX_SHADER_SOURCE: &str = r#"
            #version 330 core
            layout (location = 0) in vec3 position;
            layout (location = 1) in vec3 color;
            layout (location = 2) in vec2 TexCoord;

            out vec3 our_color;
            out vec2 tex_coord;

            void main()
            {
                gl_Position = vec4(position.x, position.y, position.z, 1.0);
                our_color = color;
                tex_coord = TexCoord;
            }
        "#;
        const VERTEX_SHADER_FRAGMENT: &str = r#"
            #version 330 core
            
            in vec3 our_color;
            in vec2 tex_coord;

            out vec4 color;

            uniform sampler2D our_texture1;
            uniform sampler2D our_texture2;


            void main()
            {
                color = mix(
                    texture(our_texture1, tex_coord), 
                    texture(our_texture2, vec2(tex_coord.x, 1.0f - tex_coord.y)) * vec4(our_color, 1.0f), 
                    0.5
                );
            }
        "#;
        let c_vertex_shader_source = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        let c_vertex_shader_fragment = CString::new(VERTEX_SHADER_FRAGMENT.as_bytes()).unwrap();
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            vertex_shader,
            1,
            &c_vertex_shader_source.as_ptr(),
            ptr::null()
        );
        gl::CompileShader(vertex_shader);

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut log_info = Vec::with_capacity(512);
            gl::GetShaderInfoLog(
                vertex_shader,
                512,
                ptr::null_mut(),
                log_info.as_mut_ptr() as *mut GLchar
            );
            println!("Vertex compilation failed\n{}", std::str::from_utf8(&log_info).unwrap());
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            fragment_shader,
            1,
            &c_vertex_shader_fragment.as_ptr(),
            ptr::null()
        );
        gl::CompileShader(fragment_shader);
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut log_info = Vec::with_capacity(512);
            gl::GetShaderInfoLog(
                fragment_shader,
                512,
                ptr::null_mut(),
                log_info.as_mut_ptr() as *mut GLchar
            );
            println!("Fragment compilation failed\n{}", std::str::from_utf8(&log_info).unwrap());
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut log_info = Vec::with_capacity(512);
            gl::GetShaderInfoLog(
                shader_program,
                512,
                ptr::null_mut(),
                log_info.as_mut_ptr() as *mut GLchar
            );
            println!("Program compilation failed\n{}", std::str::from_utf8(&log_info).unwrap());
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        type Vertex = [GLfloat; 8];
        type Indexes = [u32; 3];

        let vertices: [Vertex; 4] = [
             // Позициии       // Цвета         // Текстуры
            [0.5,  0.5, 0.0,   1.0, 0.0, 0.0,   1.0, 1.0],  // Верхний правый угол
            [0.5, -0.5, 0.0,   0.0, 1.0, 0.0,   1.0, 0.0],  // Нижний правый угол
            [-0.5, -0.5, 0.0,  0.0, 0.0, 1.0,   0.0, 0.0],  // Нижний левый угол
            [-0.5,  0.5, 0.0,  1.0, 1.0, 0.0,   0.0, 1.0 ]   // Верхний левый угол
        ];
        let indices: [Indexes; 2] = [
          [0, 1, 3],
          [1, 2, 3]
        ];
        let (mut vbo, mut vao, mut ibo) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ibo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(&vertices) as GLsizeiptr,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            mem::size_of_val(&indices) as GLsizeiptr,
            indices.as_ptr() as *const c_void,
            gl::STATIC_DRAW
        );

        let (mut texture1, mut texture2) = (0, 0);

        // our_texture1
        gl::GenTextures(1, &mut texture1);
        gl::BindTexture(gl::TEXTURE_2D, texture1);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let img = image::open(Path::new("assets/1.jpg")).unwrap();
        let data = img.to_rgb8().into_raw();
        gl::TexImage2D(
            gl::TEXTURE_2D, 
            0, 
            gl::RGB as i32, 
            img.width() as i32, 
            img.height() as i32, 
            0, 
            gl::RGB, 
            gl::UNSIGNED_BYTE, 
            data.as_ptr() as *const c_void
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        // our_texture2
        gl::GenTextures(1, &mut texture2);
        gl::BindTexture(gl::TEXTURE_2D, texture2);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let img = image::open(Path::new("assets/2.png")).unwrap();
        let data = img.to_rgb8().into_raw();
        gl::TexImage2D(
            gl::TEXTURE_2D, 
            0, 
            gl::RGB as i32, 
            img.width() as i32, 
            img.height() as i32, 
            0, 
            gl::RGB, 
            gl::UNSIGNED_BYTE, 
            data.as_ptr() as *const c_void
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        // Позиции
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>().try_into().unwrap(),
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);
        // Цвета
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>().try_into().unwrap(),
            mem::size_of::<[GLfloat; 3]>() as *const c_void
        );
        gl::EnableVertexAttribArray(1);
        // Текстуры
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>().try_into().unwrap(),
            mem::size_of::<[GLfloat; 6]>() as *const c_void
        );
        gl::EnableVertexAttribArray(2);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        gl::UseProgram(shader_program);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture1);
        let our_texture1 = CString::new("our_texture1".as_bytes()).unwrap();
        gl::Uniform1i(gl::GetUniformLocation(shader_program, our_texture1.as_ptr()), 0);
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, texture2);
        let our_texture2 = CString::new("our_texture2".as_bytes()).unwrap();
        gl::Uniform1i(gl::GetUniformLocation(shader_program, our_texture2.as_ptr()), 1);  

        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        vao
    };


    while !window.should_close() {
        glfw.poll_events();
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(vao);
            // gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }

        window.swap_buffers();
    }
}


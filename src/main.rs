use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::sync::mpsc::Receiver;
use glfw::{Action, Context, Key};
use gl::types::*;
use std::ptr;


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

    let (shader_program, vao) = unsafe {
        const VERTEX_SHADER_SOURCE: &str = r#"
            #version 330 core
            layout (location = 0) in vec3 position;
            layout (location = 1) in vec3 color;

            out vec3 our_color;

            void main()
            {
                gl_Position = vec4(position, 1.0);
                our_color = color;
            }
        "#;
        const VERTEX_SHADER_FRAGMENT: &str = r#"
            #version 330 core

            in vec3 our_color;
            out vec4 color;

            void main()
            {
                color = vec4(our_color, 1.0f);
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

        type Vertex = [GLfloat; 6];

        let vertices: [Vertex; 3] = [
            // Позиции          // Цвета
            [0.5,  -0.5, 0.0,   1.0, 0.0, 0.0],   // Нижний правый угол
            [-0.5, -0.5, 0.0,   0.0, 1.0, 0.0],   // Нижний левый угол
            [0.0,  0.5,  0.0,   0.0, 0.0, 1.0]    // Верхний угол
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

        // Атрибуты с координатами
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>().try_into().unwrap(),
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);

        // Атрибуты с цветом
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>().try_into().unwrap(),
            mem::size_of::<[GLfloat; 3]>() as *const _
        );
        gl::EnableVertexAttribArray(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        (shader_program, vao)
    };


    while !window.should_close() {
        glfw.poll_events();
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program);

            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }

        window.swap_buffers();

    }
}

